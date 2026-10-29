use super::*;
use diff_dirs_tree_inspect::*;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.listed", 1, Vec::new()));
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Declared,
    Prepare,
    Coupled { root: NodeId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Context {
    pub intent: ListedIntent,
    pub desired_slice: NodeId,
    pub owned_node: Option<NodeId>,
    pub mode: Mode,
}
pub(super) struct Selector<'a, G: ListedGuard> {
    pub desired: &'a PreparedLeafList,
    pub target: &'a ListedTarget<'a, G>,
}
impl<G: ListedGuard> Selector<'_, G> {
    pub fn root(&self) -> Context {
        Context {
            intent: self.target.intent,
            desired_slice: NodeId(0),
            owned_node: (self.target.intent == ListedIntent::RenewOwned).then_some(NodeId(0)),
            mode: Mode::Declared,
        }
    }
    pub fn relation(&self, context: &Context, path: &Path) -> ListedRelation {
        let desired = (!matches!(context.mode, Mode::Coupled { .. }))
            .then(|| self.desired.index.find(path))
            .flatten()
            .map(|n| self.desired.index.input(n));
        let owned = context
            .owned_node
            .and_then(|root| {
                let root = self.target.owned.index.node(root)?;
                path.starts_with(&root.relative).then(|| self.target.owned.index.find(path)).flatten()
            })
            .and_then(|n| self.target.owned.index.node(n).map(|node| (n, node)));
        ListedRelation {
            desired,
            owned_leaf: owned.and_then(|(_, n)| n.leaf),
            owned_directory: owned.and_then(|(id, n)| n.leaf.is_none().then_some(id)),
        }
    }
    pub fn declarations(&self, at: &Path, context: &Context) -> Vec<(OsString, Option<DesiredInput>)> {
        let mut names = Vec::new();
        if !matches!(context.mode, Mode::Coupled { .. })
            && let Some(node) = self.desired.index.find(at).and_then(|id| self.desired.index.node(id))
        {
            for (name, child) in &node.children {
                names.push((name.clone(), Some(self.desired.index.input(*child))));
            }
        }
        if let Some(node) = context.owned_node.and_then(|id| self.target.owned.index.node(id)) {
            for (name, _) in &node.children {
                if !names.iter().any(|(n, _)| n == name) {
                    names.push((name.clone(), None));
                }
            }
        }
        names.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()));
        names
    }
    fn guard_candidate<'a>(&self, context: &Context, observed: DeclaredCandidate<'a>) -> ListedCandidate<'a> {
        let path = match observed {
            DeclaredCandidate::Route(route) => route.relative.to_owned(),
            DeclaredCandidate::Entry(entry) => entry.coordinate.to_path_buf(),
        };
        ListedCandidate {
            observed,
            intent: context.intent,
            mode: match context.mode {
                Mode::Declared => ListedMode::Declared,
                Mode::Prepare => ListedMode::DesiredPreparation,
                Mode::Coupled { .. } => ListedMode::CoupledRemoval,
            },
            relation: self.relation(context, &path),
        }
    }
    pub fn lower(&self, context: &Context, at: DeclaredEntryCandidate<'_>) -> Result<DeclaredChoice<Context>, CheckError> {
        let path = at.coordinate.to_path_buf();
        let relation = self.relation(context, &path);
        if relation.desired != at.desired {
            return Err("declared candidate does not match desired index".into());
        }
        let shape = at.shape;
        if shape.target == Some(Kind::Other) || shape.source.kind() == Some(Kind::Other) {
            return Err("unsupported declared physical kind".into());
        }
        if relation.owned_leaf.is_some() && shape.target == Some(Kind::Directory) {
            return Err("exact owned leaf is now a directory".into());
        }
        let leaf_removal = relation.owned_leaf.map(|leaf| RemovalAuthority::OwnedLeaf {
            owned: self.target.owned.id(),
            leaf,
        });
        let tree_removal = relation.owned_directory.map(|subtree| RemovalAuthority::CoupledOwned {
            owned: self.target.owned.id(),
            subtree,
        });
        let renew = |destination| {
            Some(InstallAuthority {
                intent: InstallIntent::Renew,
                destination,
            })
        };
        let mut choice = DeclaredChoice {
            action: Action::PreserveTree,
            install: None,
            removal: None,
            interior: None,
        };
        let mut child = context.clone();
        let role;
        match relation.desired {
            Some(DesiredInput::Leaf { .. }) => match shape.target {
                None => {
                    choice.action = Action::InstallLeaf;
                    choice.install = renew(DestinationGrant::AbsentOnly);
                    return Ok(choice);
                }
                Some(Kind::File | Kind::Symlink) if leaf_removal.is_some() => {
                    choice.action = Action::InstallLeaf;
                    choice.install = renew(DestinationGrant::ReplaceLeaf);
                    choice.removal = leaf_removal;
                    return Ok(choice);
                }
                Some(Kind::Directory) if tree_removal.is_some() => {
                    choice.action = Action::ReplaceWithLeaf;
                    choice.install = renew(DestinationGrant::ReplaceTree);
                    choice.removal = tree_removal;
                    child.owned_node = relation.owned_directory;
                    child.mode = Mode::Coupled {
                        root: relation.owned_directory.unwrap(),
                    };
                    role = InteriorRole::RemoveForReplacement;
                }
                _ => return Err("desired occupied leaf lacks exact owned authority".into()),
            },
            Some(DesiredInput::Directory { node }) => {
                child.desired_slice = node;
                child.owned_node = relation.owned_directory.filter(|_| shape.target == Some(Kind::Directory));
                match shape.target {
                    None => {
                        choice.action = Action::Directory {
                            container: Container::Create,
                            descend: true,
                        };
                        role = InteriorRole::SourceOnly;
                    }
                    Some(Kind::Directory) => {
                        choice.action = Action::Directory {
                            container: Container::Keep,
                            descend: true,
                        };
                        role = InteriorRole::Paired;
                    }
                    Some(Kind::File | Kind::Symlink) if leaf_removal.is_some() => {
                        choice.action = Action::ReplaceWithDirectory { descend: true };
                        choice.install = renew(DestinationGrant::ReplaceLeaf);
                        choice.removal = leaf_removal;
                        child.mode = Mode::Prepare;
                        child.owned_node = None;
                        role = InteriorRole::PrepareReplacement;
                    }
                    _ => return Err("desired routing directory is obstructed by an unowned leaf".into()),
                }
            }
            None if relation.owned_leaf.is_some() => {
                choice.action = if shape.target.is_none() {
                    Action::ConfirmAbsent
                } else if shape.target.is_some_and(Kind::is_leaf) {
                    Action::RemoveLeaf
                } else {
                    return Err("obsolete owned leaf has unsupported actual shape".into());
                };
                choice.removal = leaf_removal;
                return Ok(choice);
            }
            None if relation.owned_directory.is_some() => {
                if shape.target.is_none() && !matches!(context.mode, Mode::Coupled { .. }) {
                    return Ok(choice);
                }
                if shape.target != Some(Kind::Directory) {
                    return Err("owned structural route is not a real directory".into());
                }
                let remove = matches!(context.mode, Mode::Coupled { .. });
                choice.action = Action::Directory {
                    container: if remove { Container::Remove } else { Container::Keep },
                    descend: true,
                };
                choice.removal = if remove { tree_removal } else { None };
                child.owned_node = relation.owned_directory;
                role = InteriorRole::TargetOnly;
            }
            _ => return Err("actual census child is not owned".into()),
        }
        choice.interior = Some(Interior {
            role,
            input: InteriorInput::Listed {
                desired: self.desired.id(),
                subtree: child.desired_slice,
            },
            context: child,
        });
        Ok(choice)
    }
}
impl<G: ListedGuard> DeclaredSelection for Selector<'_, G> {
    type Context = Context;
    fn identity(&self) -> &SemanticId {
        &ID
    }
    fn validate_context(&self, at: ContextSite<'_>, context: &Context) -> Result<(), CheckError> {
        if at.structural_root || context.intent != self.target.intent {
            return Err("invalid listed context interpretation".into());
        }
        let slice = self
            .desired
            .index
            .node(context.desired_slice)
            .filter(|n| n.leaf.is_none())
            .ok_or("invalid enclosing desired slice")?;
        if !at.relative.starts_with(&slice.relative) {
            return Err("listed context escaped enclosing desired slice".into());
        }
        if context.intent == ListedIntent::CreateAbsent && (context.owned_node.is_some() || !matches!(context.mode, Mode::Declared)) {
            return Err("strict-create context has owned authority".into());
        }
        if let Some(node) = context.owned_node {
            let node = self.target.owned.index.node(node).ok_or("unknown owned node")?;
            if node.leaf.is_some() || node.relative != at.relative {
                return Err("owned scope is not this exact structural directory".into());
            }
        }
        match context.mode {
            Mode::Prepare if context.owned_node.is_some() || slice.relative != at.relative => {
                return Err("private desired preparation escaped its S-only slice".into());
            }
            Mode::Coupled { root } => {
                let root = self.target.owned.index.node(root).filter(|n| n.leaf.is_none()).ok_or("invalid coupled root")?;
                if !at.relative.starts_with(&root.relative) || context.owned_node.is_none() {
                    return Err("coupled removal escaped its owned subtree".into());
                }
                if at
                    .role
                    .is_some_and(|r| !matches!(r, InteriorRole::RemoveForReplacement | InteriorRole::TargetOnly))
                {
                    return Err("invalid coupled context role".into());
                }
            }
            _ => {}
        }
        if !matches!(context.mode, Mode::Coupled { .. })
            && let Some(node) = self.desired.index.find(at.relative).and_then(|id| self.desired.index.node(id))
            && node.leaf.is_none()
            && node.relative != slice.relative
        {
            return Err("desired directory did not narrow its input slice".into());
        }
        Ok(())
    }
    fn demand(&self, context: &Context, at: DeclaredCandidate<'_>) -> SideDemands {
        self.target.guard.demand(self.guard_candidate(context, at))
    }
    fn check(&self, context: &Context, at: DeclaredCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError> {
        self.target.guard.check(self.guard_candidate(context, at), facts)
    }
    fn select(&self, context: &Context, at: DeclaredEntryCandidate<'_>) -> Result<DeclaredChoice<Context>, CheckError> {
        self.lower(context, at)
    }
}

pub(super) struct Scope {
    pub relative: PathBuf,
    pub context: Context,
    pub observed: Option<PathBuf>,
    pub output: PathBuf,
}
/// A borrowed gate, not another selection body or serializable capability.
pub(super) struct BoundBatch<'a> {
    pub batch: &'a PreparedDirectory<Context>,
}
pub(super) fn bind_listed_batch<'a, G: ListedGuard>(
    batch: &'a PreparedDirectory<Context>,
    source: &SourceSession,
    selector: &Selector<'_, G>,
    scope: &Scope,
    effects: bool,
) -> Result<BoundBatch<'a>, Failure> {
    let selected = batch.selection();
    let invalid = || {
        Failure::invalid(
            "bind listed batch",
            &scope.relative,
            "live preparation does not match exact manifest obligations, placement or authority",
        )
    };
    if batch.source_root() != source.root()
        || batch.source_root() != selector.target.binding.source()
        || batch.target_root() != selector.target.binding.target()
        || batch.source_generation() != source.generation()
        || selected.relative() != scope.relative
        || selected.input_context() != &scope.context
        || selected.domain()
            != (SelectionDomain::Listed {
                desired: selector.desired.id(),
                owned: selector.target.owned.id(),
            })
        || selected.policy() != &*ID
        || selected.context_schema() != &*ID
        || selected.is_root()
        || selected.comparator() != &SemanticId::new("diff-dirs.listed.not-assessed", 1, Vec::new())
    {
        return Err(invalid());
    }
    selector
        .validate_context(
            ContextSite {
                relative: &scope.relative,
                structural_root: false,
                role: None,
            },
            &scope.context,
        )
        .map_err(|_| invalid())?;
    // Private output mapping comes only from the runner. Ordinary/census observed
    // inputs are exact final routes; preparation deliberately observes no target.
    match scope.context.mode {
        Mode::Prepare if scope.observed.is_some() => return Err(invalid()),
        Mode::Declared | Mode::Coupled { .. }
            if scope.observed.as_ref() != Some(&selector.target.binding.target().join(&scope.relative))
                || scope.output != selector.target.binding.target().join(&scope.relative) =>
        {
            return Err(invalid());
        }
        _ => {}
    }
    if !matches!(scope.context.mode, Mode::Coupled { .. }) {
        let required = selector.declarations(&scope.relative, &scope.context);
        if required.len() != selected.len()
            || required
                .iter()
                .zip(selected.entries())
                .any(|((name, desired), entry)| name != entry.name() || *desired != entry.desired)
        {
            return Err(invalid());
        }
    }
    let info = BatchInfo {
        source_root: source.root(),
        target_root: selector.target.binding.target(),
        relative: &scope.relative,
        structural_root: false,
        purpose: selected.purpose(),
    };
    for entry in selected.entries() {
        let candidate = DeclaredEntryCandidate {
            info,
            domain: selected.domain(),
            coordinate: Coordinate {
                parent: &scope.relative,
                name: entry.name(),
                structural_root: false,
            },
            desired: entry.desired,
            shape: entry.shape(),
        };
        // One pure sync lowering owns authority; generic kind/action legality is
        // still exclusively inspect's common validator. Resolve real IDs/coords
        // and compare the weakest exact grants, not just digest equality.
        let choice = selector.lower(&scope.context, candidate).map_err(|_| invalid())?;
        if choice.action != entry.action || choice.install != entry.install || choice.removal != entry.removal || choice.interior != entry.interior {
            return Err(invalid());
        }
        let actual_desired = selector.relation(&scope.context, &scope.relative.join(entry.name())).desired;
        if entry.desired != actual_desired {
            return Err(invalid());
        }
    }
    if effects {
        crate::apply::require_execution_inputs(batch)?;
    }
    Ok(BoundBatch { batch })
}
