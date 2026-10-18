use std::ffi::OsString;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

use crate::pair::Roots;
use crate::{Check, DirDiff, Error, NoCheck, ScanState, Summary, WalkOptions};

/// Validate two existing real directory roots and construct a lazy walker.
pub fn walk(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    walk_with_options(original, target, WalkOptions::default())
}

/// Construct a walk with explicit execution policy. Contents remain lazy.
pub fn walk_with_options(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions) -> Result<Walk, Error> {
    walk_with_options_checked(original, target, options, NoCheck)
}

/// Compare with a statically dispatched accept/refuse check, including equal
/// entries and roots. One-sided directory interiors remain opaque to this walk;
/// use checked sync or audit for recursive coverage.
pub fn walk_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: C) -> Result<Walk<C>, Error> {
    walk_with_options_checked(original, target, WalkOptions::default(), check)
}

pub fn walk_with_options_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions, check: C) -> Result<Walk<C>, Error> {
    Ok(Walk {
        roots: Roots::new_checked(original, target, &check)?,
        check,
        frames: Vec::new(),
        start: true,
        prune_eligible: false,
        summary: Summary::default(),
        options,
        #[cfg(feature = "parallel")]
        frontier: None,
        #[cfg(feature = "parallel")]
        use_frontier: options.execution == crate::Execution::Parallel,
    })
}

/// Deterministic pull walk. Sequential by default; optional ambient frontier
/// execution also retains count-bounded private lookahead.
#[derive(Debug)]
pub struct Walk<C = NoCheck> {
    roots: Roots,
    check: C,
    frames: Vec<Frame>,
    start: bool,
    prune_eligible: bool,
    summary: Summary,
    options: WalkOptions,
    #[cfg(feature = "parallel")]
    frontier: Option<Box<crate::frontier::Frontier>>,
    #[cfg(feature = "parallel")]
    use_frontier: bool,
}

#[derive(Debug)]
struct Frame {
    dir: PathBuf,
    children: std::vec::IntoIter<OsString>,
    direct_ancestor: bool,
    subtree_changed: bool,
}

impl<C: Check> Walk<C> {
    /// Skip pending children once immediately after a successful yielded record.
    pub fn prune_here(&mut self) -> bool {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            return self.frontier.as_mut().is_some_and(|frontier| frontier.prune_here());
        }
        if !self.prune_eligible {
            return false;
        }
        self.prune_eligible = false;
        let frame = self.frames.last_mut().expect("yielded record has a frame");
        frame.children = Vec::new().into_iter();
        self.summary.pruned.push(frame.dir.clone());
        true
    }

    /// Return the examined prefix without finishing the walk or performing IO.
    pub fn into_summary(mut self) -> Summary {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            return self.frontier.map_or_else(Summary::default, |frontier| frontier.into_summary());
        }
        while !self.frames.is_empty() {
            self.finish_frame();
        }
        self.summary
    }

    fn finish_frame(&mut self) {
        let frame = self.frames.pop().expect("nonempty frames");
        if frame.subtree_changed {
            self.summary.changed_dirs.push(frame.dir);
            if let Some(parent) = self.frames.last_mut() {
                parent.subtree_changed = true;
            }
        }
    }

    fn examine(&mut self, dir: PathBuf, direct_ancestor: bool) -> Result<Option<DirDiff>, Error> {
        let pair = self.roots.inspect_checked(dir, self.options.execution, &self.check)?;
        let changed = !pair.diff.is_empty();
        self.summary.examined_directories += 1;
        self.summary.counts.add(&pair.diff);
        if changed && !direct_ancestor {
            self.summary.topmost_divergent.push(pair.diff.dir.clone());
        }
        self.frames.push(Frame {
            dir: pair.diff.dir.clone(),
            children: pair.children.into_iter(),
            direct_ancestor: direct_ancestor || changed,
            subtree_changed: changed,
        });
        Ok(changed.then_some(pair.diff))
    }

    fn advance(&mut self) -> Result<Option<DirDiff>, Error> {
        loop {
            let (dir, direct_ancestor) = if self.start {
                self.start = false;
                (PathBuf::new(), false)
            } else if let Some(frame) = self.frames.last_mut() {
                if let Some(name) = frame.children.next() {
                    (frame.dir.join(name), frame.direct_ancestor)
                } else {
                    self.finish_frame();
                    continue;
                }
            } else {
                self.summary.state = ScanState::Complete;
                return Ok(None);
            };
            if let Some(diff) = self.examine(dir, direct_ancestor)? {
                return Ok(Some(diff));
            }
        }
    }
}

impl<C: Check> Iterator for Walk<C> {
    type Item = Result<DirDiff, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            let frontier = self
                .frontier
                .get_or_insert_with(|| Box::new(crate::frontier::Frontier::new(crate::ParallelLimits::for_threads(rayon::current_num_threads()))));
            return frontier.next_with(|dir| self.roots.inspect_checked(dir.to_path_buf(), crate::Execution::Sequential, &self.check));
        }
        self.prune_eligible = false;
        if self.summary.state != ScanState::Incomplete {
            return None;
        }
        match self.advance() {
            Ok(Some(diff)) => {
                self.prune_eligible = true;
                Some(Ok(diff))
            }
            Ok(None) => None,
            Err(error) => {
                self.summary.state = ScanState::Failed;
                Some(Err(error))
            }
        }
    }
}

impl<C: Check> FusedIterator for Walk<C> {}

/// Temporary measurement control for the predecessor per-frame backend.
/// Not part of the supported embedding API; requires `parallel-bench`.
#[doc(hidden)]
#[cfg(feature = "parallel-bench")]
pub fn walk_per_frame(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    let mut walk = walk_with_options(
        original,
        target,
        WalkOptions {
            execution: crate::Execution::Parallel,
        },
    )?;
    walk.use_frontier = false;
    Ok(walk)
}
