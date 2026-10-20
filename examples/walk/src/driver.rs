use diff_dirs::DirectoryPlan;

use crate::{
    Aggregate, Bindings, DesiredCopy, Error, Frame, Manifest, ManifestState, Node, Outcome, Pending, Phase, SavedWalk, Unit, Work, add, expected_unit, require,
};

pub trait Inspector {
    fn inspect(&mut self, bindings: &Bindings, node: &Node) -> Result<DirectoryPlan, Error>;
}

pub trait Executor {
    fn execute(&mut self, bindings: &Bindings, work: &Work) -> Attempt<EffectOutcome>;
}

/// Actual destination installations/creations observed by this attempt, including
/// after-visible-effect failure. Not a census of staging syscalls or a saved ledger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Writes {
    pub file_installations: u64,
    pub directories_created: u64,
}

impl Writes {
    fn fold(self, other: Self) -> Result<Self, Error> {
        Ok(Self {
            file_installations: add(self.file_installations, other.file_installations)?,
            directories_created: add(self.directories_created, other.directories_created)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectOutcome {
    NoWrite,
    Installed,
    Created,
}

#[derive(Debug)]
pub struct Attempt<T> {
    pub result: Result<T, Error>,
    pub writes: Writes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Inspect { ticket: u64, node: Node },
    Effect { ticket: u64, work: Work },
    Done(Outcome),
}

impl Request {
    fn from_pending(pending: &Pending) -> Self {
        match &pending.unit {
            Unit::Inspect(node) => Self::Inspect {
                ticket: pending.ticket,
                node: node.clone(),
            },
            Unit::Effect(work) => Self::Effect {
                ticket: pending.ticket,
                work: work.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub enum Ack {
    Inspected {
        ticket: u64,
        node: Node,
        plan: DirectoryPlan,
    },
    Effected {
        ticket: u64,
        work: Work,
        outcome: EffectOutcome,
        writes: Writes,
    },
    Failed {
        ticket: u64,
        unit: Unit,
        error: String,
        writes: Writes,
    },
}

impl Ack {
    pub fn writes(&self) -> Writes {
        match self {
            Self::Inspected { .. } => Writes::default(),
            Self::Effected { writes, .. } | Self::Failed { writes, .. } => *writes,
        }
    }
    fn identity(&self) -> (u64, Unit) {
        match self {
            Self::Inspected { ticket, node, .. } => (*ticket, Unit::Inspect(node.clone())),
            Self::Effected { ticket, work, .. } => (*ticket, Unit::Effect(work.clone())),
            Self::Failed { ticket, unit, .. } => (*ticket, unit.clone()),
        }
    }
}

pub enum StartParts {
    Fresh(Bindings),
    Restored(SavedWalk),
}

/// Construction and restoration are filesystem-free; supplied objects are moved
/// into the runtime, not substituted with defaults. Bind explicitly before I/O.
#[bon::builder(finish_fn = build)]
pub fn cursor<I: Inspector, E: Executor>(
    parts: StartParts,
    inspector: I,
    executor: E,
    program: DesiredCopy,
    manifest: Manifest,
) -> Result<Running<I, E>, Error> {
    let saved = match parts {
        StartParts::Fresh(bindings) => SavedWalk {
            version: 1,
            platform: "unix".into(),
            bindings,
            program: program.descriptor().into(),
            manifest_config: manifest.config(),
            manifest_state: ManifestState { next_ordinal: 1 },
            stack: vec![Frame::new(Node {
                relative: Default::default(),
                kind: crate::Kind::Directory,
                ensure: false,
            })],
            pending: None,
            next_ticket: 1,
            terminal: None,
        },
        StartParts::Restored(saved) => saved,
    };
    saved.validate()?;
    require(
        saved.program == program.descriptor() && saved.manifest_config == manifest.config(),
        "rebound component configuration differs",
    )?;
    Ok(Running {
        saved,
        inspector,
        executor,
        program,
        manifest,
        bound: false,
    })
}

pub struct Running<I, E> {
    saved: SavedWalk,
    inspector: I,
    executor: E,
    program: DesiredCopy,
    manifest: Manifest,
    bound: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub assessments: u64,
    pub effects: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Goals {
    pub containers: u64,
    pub leaves: u64,
    pub finishes: u64,
    pub no_write: u64,
}

impl Goals {
    fn fold(self, other: Self) -> Result<Self, Error> {
        Ok(Self {
            containers: add(self.containers, other.containers)?,
            leaves: add(self.leaves, other.leaves)?,
            finishes: add(self.finishes, other.finishes)?,
            no_write: add(self.no_write, other.no_write)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub enum Stop {
    Budget,
    Blocked,
    Done,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Progress {
    pub stop: Stop,
    pub attempted_assessments: u64,
    pub attempted_effects: u64,
    pub accepted: Goals,
    pub writes: Writes,
}

impl<I: Inspector, E: Executor> Running<I, E> {
    pub fn saved(&self) -> &SavedWalk {
        &self.saved
    }
    pub fn into_saved(self) -> Result<SavedWalk, Error> {
        self.saved.validate()?;
        Ok(self.saved)
    }

    pub fn bind_filesystem(&mut self) -> Result<(), Error> {
        if self.saved.terminal.is_none() {
            crate::bind_roots(&self.saved.bindings)?;
        }
        self.bound = true;
        Ok(())
    }

    /// No I/O. Drain only pure pushes/phase changes; repeated calls preserve ticket.
    pub fn next_request(&mut self) -> Result<Request, Error> {
        if let Some(outcome) = &self.saved.terminal {
            return Ok(Request::Done(outcome.clone()));
        }
        if let Some(pending) = &self.saved.pending {
            return Ok(Request::from_pending(pending));
        }
        let mut candidate = self.saved.clone();
        loop {
            let top = candidate.stack.last_mut().expect("validated live stack");
            if top.phase == Phase::Children {
                if let Some(child) = top.remaining.first().cloned() {
                    candidate.stack.push(Frame::new(child));
                } else {
                    top.phase = Phase::Finish;
                }
                continue;
            }
            let unit = expected_unit(top, &self.manifest, &candidate.manifest_state).expect("request phase");
            let ticket = candidate.next_ticket;
            candidate.next_ticket = add(ticket, 1)?;
            candidate.pending = Some(Pending { ticket, unit, blocked: None });
            candidate.validate()?;
            let request = Request::from_pending(candidate.pending.as_ref().expect("issued"));
            self.saved = candidate;
            return Ok(request);
        }
    }

    pub fn dispatch(&mut self, request: &Request) -> Result<Ack, Error> {
        require(self.bound, "bind_filesystem must precede dispatch")?;
        let pending = self.saved.pending.as_ref().ok_or_else(|| Error::invalid("no pending request"))?;
        require(
            pending.blocked.is_none() && Request::from_pending(pending) == *request,
            "request is blocked or does not match pending unit",
        )?;
        let ticket = pending.ticket;
        Ok(match &pending.unit {
            Unit::Inspect(node) => match self.inspector.inspect(&self.saved.bindings, node) {
                Ok(plan) => Ack::Inspected {
                    ticket,
                    node: node.clone(),
                    plan,
                },
                Err(error) => Ack::Failed {
                    ticket,
                    unit: pending.unit.clone(),
                    error: error.to_string(),
                    writes: Writes::default(),
                },
            },
            Unit::Effect(work) => {
                let attempt = self.executor.execute(&self.saved.bindings, work);
                match attempt.result {
                    Ok(outcome) => Ack::Effected {
                        ticket,
                        work: work.clone(),
                        outcome,
                        writes: attempt.writes,
                    },
                    Err(error) => Ack::Failed {
                        ticket,
                        unit: pending.unit.clone(),
                        error: error.to_string(),
                        writes: attempt.writes,
                    },
                }
            }
        })
    }

    /// External successful acknowledgments are trusted caller evidence. Identity,
    /// work kind and candidate arithmetic/shape are checked before changing state.
    pub fn acknowledge(&mut self, ack: Ack) -> Result<Goals, Error> {
        let pending = self.saved.pending.as_ref().ok_or_else(|| Error::invalid("no pending acknowledgment"))?;
        let (ticket, unit) = ack.identity();
        require(
            pending.blocked.is_none() && pending.ticket == ticket && pending.unit == unit,
            "wrong/duplicate/blocked acknowledgment",
        )?;
        let mut candidate = self.saved.clone();
        let mut accepted = Goals::default();
        match ack {
            Ack::Failed { error, .. } => candidate.pending.as_mut().expect("pending").blocked = Some(error),
            Ack::Inspected { node, plan, .. } => match self.program.lower(&node, &plan, &candidate.bindings) {
                Err(error) => candidate.pending.as_mut().expect("pending").blocked = Some(error.to_string()),
                Ok((local, children)) => {
                    let top = candidate.stack.last_mut().expect("live stack");
                    top.local = Some(local);
                    top.aggregate = Aggregate {
                        leaves: 0,
                        directories: 1,
                        physical_differences: local.physical_differences,
                        retained_entries: local.retained_entries,
                    };
                    top.remaining = children;
                    top.phase = Phase::Children;
                    candidate.pending = None;
                }
            },
            Ack::Effected { work, outcome, .. } => {
                let ensure = matches!(work, Work::EnsureDirectory(_));
                require(
                    match outcome {
                        EffectOutcome::Created => ensure,
                        EffectOutcome::Installed => !ensure,
                        EffectOutcome::NoWrite => true,
                    },
                    "effect result/work kind mismatch",
                )?;
                accepted.no_write = u64::from(outcome == EffectOutcome::NoWrite);
                candidate.pending = None;
                match work {
                    Work::EnsureDirectory(_) => {
                        accepted.containers = 1;
                        candidate.stack.last_mut().expect("live stack").phase = Phase::Assess;
                    }
                    Work::SettleSourceFile(_) => {
                        accepted.leaves = 1;
                        candidate.stack.last_mut().expect("leaf").aggregate.leaves = 1;
                        close(&mut candidate)?;
                    }
                    Work::InstallReport(_) => {
                        accepted.finishes = 1;
                        candidate.manifest_state.next_ordinal = add(candidate.manifest_state.next_ordinal, 1)?;
                        close(&mut candidate)?;
                    }
                }
            }
        }
        candidate.validate()?;
        self.saved = candidate;
        Ok(accepted)
    }

    pub fn retry_blocked_unit(&mut self) -> Result<(), Error> {
        require(self.saved.pending.as_ref().is_some_and(|p| p.blocked.is_some()), "no blocked unit")?;
        let mut candidate = self.saved.clone();
        // Reissue on the next read with a new ticket and the same semantic inputs.
        candidate.pending = None;
        candidate.validate()?;
        self.saved = candidate;
        Ok(())
    }

    pub fn skip_blocked_unit(&mut self) -> Result<(), Error> {
        require(self.saved.pending.as_ref().is_some_and(|p| p.blocked.is_some()), "no blocked unit")?;
        let mut candidate = self.saved.clone();
        for frame in &mut candidate.stack {
            frame.incomplete = true;
        }
        candidate.pending = None;
        close(&mut candidate)?;
        candidate.validate()?;
        self.saved = candidate;
        Ok(())
    }

    pub fn drive(&mut self, limits: Limits) -> Result<Progress, Error> {
        let mut progress = Progress {
            stop: Stop::Budget,
            attempted_assessments: 0,
            attempted_effects: 0,
            accepted: Goals::default(),
            writes: Writes::default(),
        };
        loop {
            let request = self.next_request()?;
            if matches!(request, Request::Done(_)) {
                progress.stop = Stop::Done;
                break;
            }
            if self.saved.pending.as_ref().is_some_and(|p| p.blocked.is_some()) {
                progress.stop = Stop::Blocked;
                break;
            }
            let attempted = match request {
                Request::Inspect { .. } if progress.attempted_assessments < limits.assessments => &mut progress.attempted_assessments,
                Request::Effect { .. } if progress.attempted_effects < limits.effects => &mut progress.attempted_effects,
                _ => break,
            };
            *attempted = add(*attempted, 1)?;
            let ack = self.dispatch(&request)?;
            progress.writes = progress.writes.fold(ack.writes())?;
            progress.accepted = progress.accepted.fold(self.acknowledge(ack)?)?;
        }
        Ok(progress)
    }
}

fn close(saved: &mut SavedWalk) -> Result<(), Error> {
    let frame = saved.stack.pop().expect("live stack");
    if let Some(parent) = saved.stack.last_mut() {
        require(parent.remaining.first() == Some(&frame.node), "closing wrong child")?;
        parent.aggregate = parent.aggregate.fold(frame.aggregate)?;
        parent.incomplete |= frame.incomplete;
        parent.remaining.remove(0);
    } else {
        saved.terminal = Some(Outcome {
            aggregate: frame.aggregate,
            incomplete: frame.incomplete,
        });
    }
    Ok(())
}
