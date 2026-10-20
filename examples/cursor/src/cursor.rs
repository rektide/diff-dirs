use std::collections::BTreeMap;
use std::fs;
use std::num::NonZeroUsize;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use diff_dirs::{DirectoryPlan, NoVisitor, Presence, Purpose, inspect_directory};
use facet::Facet;
use serde::Serialize;

use crate::{Error, increment, require};

/// Explicit Unix native encoding, also rendered losslessly as a JSON byte array.
#[derive(Clone, Debug, Default, Eq, PartialEq, Facet, Serialize)]
#[serde(transparent)]
pub struct NativePath {
    pub bytes: Vec<u8>,
}

impl NativePath {
    pub fn new(path: &Path) -> Self {
        Self {
            bytes: path.as_os_str().as_bytes().to_vec(),
        }
    }

    pub fn to_path_buf(&self) -> PathBuf {
        std::ffi::OsString::from_vec(self.bytes.clone()).into()
    }

    fn validate(&self, absolute: bool) -> Result<(), Error> {
        require(!self.bytes.contains(&0), "NUL in native path")?;
        require(self.bytes.starts_with(b"/") == absolute, "wrong native path kind")?;
        let bytes = if absolute { &self.bytes[1..] } else { &self.bytes[..] };
        if !bytes.is_empty() {
            require(
                bytes.split(|b| *b == b'/').all(|part| !part.is_empty() && part != b"." && part != b".."),
                "path must have only normal components",
            )?;
        }
        Ok(())
    }
}

/// Counts of observed immediate slots, accumulated over the assessed subtree.
/// Both-sided directory slots count as equal here, independently of descendants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Facet, Serialize)]
pub struct Counts {
    pub source_only: u64,
    pub target_only: u64,
    pub changed: u64,
    pub equal: u64,
}

impl Counts {
    fn add(&mut self, other: Self) -> Result<(), Error> {
        increment(&mut self.source_only, other.source_only)?;
        increment(&mut self.target_only, other.target_only)?;
        increment(&mut self.changed, other.changed)?;
        increment(&mut self.equal, other.equal)
    }
}

/// Persisted data selects a compiled handler, never executable code.
#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct Descriptor {
    pub kind: String,
    pub parameters: BTreeMap<String, String>,
}

impl Descriptor {
    pub fn directory_summary(label: String) -> Self {
        Self {
            kind: "directory-summary/v1".into(),
            parameters: BTreeMap::from([("label".into(), label)]),
        }
    }

    fn validate(&self) -> Result<(), Error> {
        require(self.kind == "directory-summary/v1", "unknown generator kind")?;
        require(
            self.parameters.len() == 1 && self.parameters.contains_key("label"),
            "expected only generator parameter 'label'",
        )
    }

    fn generate(&self, logical: &NativePath, counts: Counts, incomplete: bool) -> Result<VirtualFile, Error> {
        self.validate()?;
        let contents = format!(
            "label: {}\ndirectory_bytes: {}\nsource_only: {}\ntarget_only: {}\nchanged: {}\nequal: {}\nincomplete: {}\n",
            serde_json::to_string(&self.parameters["label"])?,
            serde_json::to_string(logical)?,
            counts.source_only,
            counts.target_only,
            counts.changed,
            counts.equal,
            incomplete,
        )
        .into_bytes();
        Ok(VirtualFile {
            logical_name: "summary.txt".into(),
            contents,
        })
    }
}

/// All present coordinates coincide in this demo; a missing side stays absent.
#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct Tip {
    pub logical: NativePath,
    pub source: Option<NativePath>,
    pub target: Option<NativePath>,
}

impl Tip {
    fn validate(&self) -> Result<(), Error> {
        self.logical.validate(false)?;
        require(self.source.is_some() || self.target.is_some(), "tip has no present side")?;
        for side in [&self.source, &self.target].into_iter().flatten() {
            side.validate(false)?;
            require(side == &self.logical, "mapped coordinates are not supported")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Facet, Serialize)]
#[repr(C)]
pub enum Phase {
    Pending,
    Assessed,
}

#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct Frame {
    pub tip: Tip,
    pub phase: Phase,
    pub children: Vec<Tip>,
    pub next_child: u64,
    pub totals: Counts,
    pub incomplete: bool,
}

impl Frame {
    fn pending(tip: Tip) -> Self {
        Self {
            tip,
            phase: Phase::Pending,
            children: Vec::new(),
            next_child: 0,
            totals: Counts::default(),
            incomplete: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct Block {
    pub logical: NativePath,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct FinalState {
    pub totals: Counts,
    pub incomplete: bool,
}

/// The entire continuation: active frames and aggregate state, not prior output.
#[derive(Clone, Debug, Eq, PartialEq, Facet, Serialize)]
pub struct Cursor {
    pub platform: String,
    pub source_root: NativePath,
    pub target_root: NativePath,
    pub generator: Descriptor,
    pub stack: Vec<Frame>,
    pub attempted_total: u64,
    pub completed_total: u64,
    pub skipped_total: u64,
    pub blocked: Option<Block>,
    pub final_state: Option<FinalState>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VirtualFile {
    pub logical_name: String,
    pub contents: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Completion {
    pub sequence: u64,
    pub logical: NativePath,
    pub totals: Counts,
    pub incomplete: bool,
    pub generated: VirtualFile,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Assessment {
        logical: NativePath,
        observed: Option<Counts>,
        error: Option<String>,
    },
    Completion(Completion),
    Skipped {
        logical: NativePath,
    },
}

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub skip_blocked: bool,
    /// Invocation-local failure injection, outside the pure Visitor callbacks.
    pub fail_at: Option<NativePath>,
}

#[derive(Debug)]
pub struct Run {
    pub attempted: u64,
    pub completed: u64,
    pub skipped: u64,
    pub done: bool,
    pub events: Vec<Event>,
}

impl Cursor {
    pub fn start(source: &Path, target: &Path, generator: Descriptor) -> Result<Self, Error> {
        generator.validate()?;
        // Reject symlink roots before canonicalizing their absolute bindings.
        real_directory(source)?;
        real_directory(target)?;
        let source = fs::canonicalize(source).map_err(|e| Error::io(source, "bind root", e))?;
        let target = fs::canonicalize(target).map_err(|e| Error::io(target, "bind root", e))?;
        let root = NativePath::default();
        let cursor = Self {
            platform: "unix".into(),
            source_root: NativePath::new(&source),
            target_root: NativePath::new(&target),
            generator,
            stack: vec![Frame::pending(Tip {
                logical: root.clone(),
                source: Some(root.clone()),
                target: Some(root),
            })],
            attempted_total: 0,
            completed_total: 0,
            skipped_total: 0,
            blocked: None,
            final_state: None,
        };
        cursor.validate()?;
        Ok(cursor)
    }

    /// Validate application invariants after the backend's archive validation.
    /// No filesystem work occurs, including for a terminal cursor.
    pub fn validate(&self) -> Result<(), Error> {
        require(self.platform == "unix", "unsupported path platform")?;
        self.source_root.validate(true)?;
        self.target_root.validate(true)?;
        let source = self.source_root.to_path_buf();
        let target = self.target_root.to_path_buf();
        require(!source.starts_with(&target) && !target.starts_with(&source), "roots must not overlap")?;
        self.generator.validate()?;
        require(self.stack.is_empty() == self.final_state.is_some(), "invalid terminal state")?;
        require(
            self.completed_total <= self.attempted_total && self.skipped_total <= self.attempted_total - self.completed_total,
            "invalid progress counters",
        )?;
        if let Some(root) = self.stack.first() {
            require(
                root.tip.logical.bytes.is_empty() && root.tip.source == Some(NativePath::default()) && root.tip.target == Some(NativePath::default()),
                "invalid root tip",
            )?;
        }
        for (index, frame) in self.stack.iter().enumerate() {
            frame.tip.validate()?;
            let next = usize::try_from(frame.next_child).map_err(|_| Error::Invalid("child index exceeds host width".into()))?;
            require(next <= frame.children.len(), "child index out of bounds")?;
            if frame.phase == Phase::Pending {
                require(
                    index + 1 == self.stack.len() && frame.children.is_empty() && next == 0 && frame.totals == Counts::default() && !frame.incomplete,
                    "invalid pending frame",
                )?;
            }
            let parent = frame.tip.logical.to_path_buf();
            let mut previous: Option<&[u8]> = None;
            for child in &frame.children {
                child.validate()?;
                require(
                    child.logical.to_path_buf().parent() == Some(parent.as_path()),
                    "child is not immediately below parent",
                )?;
                require(frame.tip.source.is_some() || child.source.is_none(), "source side reappeared")?;
                require(frame.tip.target.is_some() || child.target.is_none(), "target side reappeared")?;
                require(previous.is_none_or(|p| p < child.logical.bytes.as_slice()), "children are not uniquely sorted")?;
                previous = Some(&child.logical.bytes);
            }
            if let Some(active) = self.stack.get(index + 1) {
                require(frame.phase == Phase::Assessed && next > 0, "invalid active parent phase/index")?;
                require(frame.children.get(next - 1) == Some(&active.tip), "active child does not match parent cursor")?;
            }
        }
        if let Some(block) = &self.blocked {
            block.logical.validate(false)?;
            require(
                self.stack.last().is_some_and(|f| f.phase == Phase::Pending && f.tip.logical == block.logical),
                "block does not identify pending tip",
            )?;
            require(self.attempted_total > 0, "block without an attempt")?;
        }
        Ok(())
    }

    /// Advance without root replay. Completion/push/pop transitions cost no units.
    /// A failed attempt consumes one unit and stops this invocation with a block.
    pub fn advance(&mut self, units: NonZeroUsize, options: &RunOptions) -> Result<Run, Error> {
        self.validate()?;
        if let Some(path) = &options.fail_at {
            path.validate(false)?;
        }
        let mut run = Run {
            attempted: 0,
            completed: 0,
            skipped: 0,
            done: false,
            events: Vec::new(),
        };
        if options.skip_blocked {
            require(self.blocked.is_some(), "--skip-blocked requires a blocked cursor")?;
            let frame = self.stack.pop().expect("validated blocked frame");
            self.blocked = None;
            increment(&mut self.skipped_total, 1)?;
            run.skipped = 1;
            run.events.push(Event::Skipped { logical: frame.tip.logical });
            if let Some(parent) = self.stack.last_mut() {
                parent.incomplete = true;
            } else {
                self.final_state = Some(FinalState {
                    totals: Counts::default(),
                    incomplete: true,
                });
            }
        }
        while let Some(frame) = self.stack.last() {
            if frame.phase == Phase::Pending {
                if run.attempted >= units.get() as u64 {
                    break;
                }
                self.blocked = None;
                increment(&mut self.attempted_total, 1)?;
                run.attempted += 1;
                let tip = frame.tip.clone();
                match self.assess(&tip, options) {
                    Ok((counts, children)) => {
                        let frame = self.stack.last_mut().expect("pending frame");
                        frame.phase = Phase::Assessed;
                        frame.totals = counts;
                        frame.children = children;
                        run.events.push(Event::Assessment {
                            logical: tip.logical,
                            observed: Some(counts),
                            error: None,
                        });
                    }
                    Err(error) => {
                        let message: String = error.to_string().chars().take(1024).collect();
                        self.blocked = Some(Block {
                            logical: tip.logical.clone(),
                            message: message.clone(),
                        });
                        run.events.push(Event::Assessment {
                            logical: tip.logical,
                            observed: None,
                            error: Some(message),
                        });
                        break;
                    }
                }
            } else {
                let next = frame.next_child as usize; // validated and only advanced within this Vec
                if let Some(tip) = frame.children.get(next).cloned() {
                    self.stack.last_mut().expect("parent frame").next_child += 1;
                    self.stack.push(Frame::pending(tip));
                } else {
                    // Generate before removing the state needed to finish this frame.
                    let generated = self.generator.generate(&frame.tip.logical, frame.totals, frame.incomplete)?;
                    increment(&mut self.completed_total, 1)?;
                    let frame = self.stack.pop().expect("completed frame");
                    run.completed += 1;
                    run.events.push(Event::Completion(Completion {
                        sequence: self.completed_total,
                        logical: frame.tip.logical,
                        totals: frame.totals,
                        incomplete: frame.incomplete,
                        generated,
                    }));
                    if let Some(parent) = self.stack.last_mut() {
                        parent.totals.add(frame.totals)?;
                        parent.incomplete |= frame.incomplete;
                    } else {
                        self.final_state = Some(FinalState {
                            totals: frame.totals,
                            incomplete: frame.incomplete,
                        });
                    }
                }
            }
        }
        run.done = self.stack.is_empty();
        Ok(run)
    }

    fn assess(&self, tip: &Tip, options: &RunOptions) -> Result<(Counts, Vec<Tip>), Box<dyn std::error::Error>> {
        if options.fail_at.as_ref() == Some(&tip.logical) {
            return Err("injected assessment failure".into());
        }
        let source_root = self.source_root.to_path_buf();
        let target_root = self.target_root.to_path_buf();
        let source = tip.source.as_ref().map(|p| source_root.join(p.to_path_buf()));
        let target = tip.target.as_ref().map(|p| target_root.join(p.to_path_buf()));
        for path in [&source, &target].into_iter().flatten() {
            real_directory(path)?;
        }
        let purpose = match (source.is_some(), target.is_some()) {
            (true, true) => Purpose::Compare,
            (true, false) => Purpose::Copy,
            (false, true) => Purpose::Remove,
            (false, false) => unreachable!("validated nonempty tip"),
        };
        let plan = inspect_directory(
            &source_root,
            &target_root,
            &tip.logical.to_path_buf(),
            source.as_deref(),
            target.as_deref(),
            purpose,
            &NoVisitor,
        )?;
        Ok(reduce(&plan)?)
    }
}

fn real_directory(path: &Path) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path, "inspect directory binding", e))?;
    require(metadata.is_dir(), &format!("scheduled path is not a real directory: {path:?}"))
}

fn reduce(plan: &DirectoryPlan) -> Result<(Counts, Vec<Tip>), Error> {
    let mut counts = Counts::default();
    let mut children = Vec::new();
    for op in plan.operations().entries() {
        let count = match op.presence() {
            Presence::SourceOnly => &mut counts.source_only,
            Presence::TargetOnly => &mut counts.target_only,
            Presence::Both if op.is_different() => &mut counts.changed,
            Presence::Both => &mut counts.equal,
        };
        increment(count, 1)?;
        // Assessment explicitly visits one-sided interiors, including the sole
        // directory side of a collision. This does not split a coupled action.
        let source_dir = op.source().is_some_and(|f| f.metadata().is_dir());
        let target_dir = op.target().is_some_and(|f| f.metadata().is_dir());
        if op.descends() || source_dir || target_dir {
            let logical = NativePath::new(op.relative_path());
            children.push(Tip {
                source: source_dir.then(|| logical.clone()),
                target: target_dir.then(|| logical.clone()),
                logical,
            });
        }
    }
    Ok((counts, children))
}
