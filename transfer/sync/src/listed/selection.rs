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
        let desired = if matches!(context.mode, Mode::Coupled { .. }) {
            &[][..]
        } else {
            self.desired
                .index
                .find(at)
                .and_then(|id| self.desired.index.node(id))
                .map_or(&[][..], |n| n.children.as_slice())
        };
        let owned = context
            .owned_node
            .and_then(|id| self.target.owned.index.node(id))
            .map_or(&[][..], |n| n.children.as_slice());
        let mut names = Vec::with_capacity(desired.len() + owned.len());
        let mut desired = desired.iter().peekable();
        let mut owned = owned.iter().peekable();
        while desired.peek().is_some() || owned.peek().is_some() {
            let order = match (desired.peek(), owned.peek()) {
                (Some((a, _)), Some((b, _))) => a.as_encoded_bytes().cmp(b.as_encoded_bytes()),
                (Some(_), None) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Greater,
            };
            let d = (order != std::cmp::Ordering::Greater).then(|| desired.next().unwrap());
            let o = (order != std::cmp::Ordering::Less).then(|| owned.next().unwrap());
            names.push((
                d.map_or_else(|| o.unwrap().0.clone(), |(name, _)| name.clone()),
                d.map(|(_, node)| self.desired.index.input(*node)),
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    struct Corrupt<'a> {
        selector: Selector<'a, NoListedGuard>,
        alter: u8,
    }
    impl DeclaredSelection for Corrupt<'_> {
        type Context = Context;
        fn identity(&self) -> &SemanticId {
            self.selector.identity()
        }
        fn validate_context(&self, at: ContextSite<'_>, context: &Context) -> Result<(), CheckError> {
            self.selector.validate_context(at, context)
        }
        fn demand(&self, _: &Context, _: DeclaredCandidate<'_>) -> SideDemands {
            SideDemands::default()
        }
        fn check(&self, _: &Context, _: DeclaredCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
            Ok(())
        }
        fn select(&self, context: &Context, mut at: DeclaredEntryCandidate<'_>) -> Result<DeclaredChoice<Context>, CheckError> {
            if self.alter == 1 || self.alter == 2 {
                at.desired = self.selector.relation(context, &at.coordinate.to_path_buf()).desired;
            }
            let mut choice = self.selector.lower(context, at)?;
            if self.alter == 3 {
                choice.install.as_mut().unwrap().destination = DestinationGrant::ReplaceTree;
            }
            if self.alter == 4 && choice.removal.is_some() {
                choice.removal = Some(RemovalAuthority::OwnedLeaf {
                    owned: self.selector.target.owned.id(),
                    leaf: LeafId(1),
                });
            }
            if self.alter == 5 {
                choice.removal = Some(RemovalAuthority::CoupledOwned {
                    owned: self.selector.target.owned.id(),
                    subtree: NodeId(0),
                });
            }
            if self.alter == 6
                && let Some(interior) = choice.interior.as_mut()
            {
                interior.input = InteriorInput::Listed {
                    desired: self.selector.desired.id(),
                    subtree: NodeId(0),
                };
            }
            Ok(choice)
        }
    }
    #[test]
    fn actual_bind_rejects_wrong_ids_expectations_omitted_slots_and_overbroad_grants_before_effects() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/listed-binding");
        std::fs::create_dir_all(&scratch).unwrap();
        for alter in 0..7 {
            let temp = tempfile::tempdir_in(std::fs::canonicalize(&scratch).unwrap()).unwrap();
            let source_root = temp.path().join("source");
            let target_root = temp.path().join("target");
            std::fs::create_dir(&source_root).unwrap();
            std::fs::create_dir(&target_root).unwrap();
            let paths = if alter == 5 {
                vec!["q"]
            } else if alter == 6 {
                vec!["q/a"]
            } else {
                vec!["a", "z"]
            };
            for path in &paths {
                std::fs::create_dir_all(source_root.join(path).parent().unwrap()).unwrap();
                std::fs::write(source_root.join(path), b"real").unwrap();
            }
            let desired = PreparedLeafList::new(&paths).unwrap();
            let clone = desired.clone();
            assert!(Arc::ptr_eq(&desired.index, &clone.index));
            let owned = OwnedLeafSet::new(if alter == 5 { vec!["q/a"] } else { vec!["a", "z"] }).unwrap();
            let owned_clone = owned.clone();
            assert!(Arc::ptr_eq(&owned.index, &owned_clone.index));
            if alter == 4 {
                std::fs::write(target_root.join("a"), b"owned").unwrap();
            }
            if alter == 5 {
                std::fs::create_dir(target_root.join("q")).unwrap();
                std::fs::write(target_root.join("q/a"), b"owned").unwrap();
            }
            let binding = EffectBinding::new(&source_root, &target_root, crate::CopyPolicy::Copy).unwrap();
            let target_id = TargetId("binding".into());
            let target = ListedTarget {
                id: &target_id,
                binding: &binding,
                owned: &owned,
                intent: if alter == 3 { ListedIntent::CreateAbsent } else { ListedIntent::RenewOwned },
                guard: &NoListedGuard,
            };
            let selector = Selector {
                desired: &desired,
                target: &target,
            };
            let context = selector.root();
            let mut declarations = selector.declarations(Path::new(""), &context);
            if alter == 0 {
                declarations.pop();
            }
            if alter == 1 {
                declarations[0].1 = Some(DesiredInput::Leaf {
                    leaf: LeafId(999),
                    expected: LeafExpectation::SupportedLeaf,
                });
            }
            if alter == 2 {
                declarations[0].1 = Some(DesiredInput::Leaf {
                    leaf: LeafId(0),
                    expected: LeafExpectation::Exact {
                        kind: LeafKind::RegularFile,
                        executable: None,
                    },
                });
            }
            let names: Vec<_> = declarations.iter().map(|(name, desired)| DeclaredName { name, desired: *desired }).collect();
            let evil = Corrupt { selector, alter };
            let source = SourceSession::new(&source_root);
            let request = DeclaredInspection {
                target: 0,
                logical_root: &target_root,
                physical_directory: Some(&target_root),
                desired: desired.id(),
                owned: owned.id(),
                context: &context,
                names: DeclaredNames::Exact(&names),
                selection: &evil,
            };
            let prepared = inspect_declared_group(
                &source,
                DeclaredRequest {
                    relative: Path::new(""),
                    purpose: Purpose::Copy,
                },
                &[request],
                &crate::PhysicalExecutionDemand,
                &InspectionEpoch::new(),
                Execution::Sequential,
            )
            .unwrap()
            .remove(0)
            .outcome
            .unwrap();
            let scope = Scope {
                relative: PathBuf::new(),
                context,
                observed: Some(target_root.clone()),
                output: target_root.clone(),
            };
            assert!(
                bind_listed_batch(&prepared, &source, &evil.selector, &scope, true).is_err(),
                "corruption {alter} passed actual binding"
            );
            assert_eq!(std::fs::read_dir(&target_root).unwrap().count(), usize::from(alter == 4 || alter == 5));
        }
    }
    #[test]
    fn no_execution_demand_preparation_is_not_an_executable_manifest() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/listed-binding");
        std::fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(std::fs::canonicalize(scratch).unwrap()).unwrap();
        let source_root = temp.path().join("source");
        let target_root = temp.path().join("target");
        std::fs::create_dir(&source_root).unwrap();
        std::fs::create_dir(&target_root).unwrap();
        std::fs::write(source_root.join("required"), b"real").unwrap();
        let desired = PreparedLeafList::new(["required"]).unwrap();
        let owned = OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap();
        let binding = EffectBinding::new(&source_root, &target_root, crate::CopyPolicy::Copy).unwrap();
        let id = TargetId("ready".into());
        let target = ListedTarget {
            id: &id,
            binding: &binding,
            owned: &owned,
            intent: ListedIntent::CreateAbsent,
            guard: &NoListedGuard,
        };
        let selector = Selector {
            desired: &desired,
            target: &target,
        };
        let context = selector.root();
        let declarations = selector.declarations(Path::new(""), &context);
        let names: Vec<_> = declarations.iter().map(|(name, desired)| DeclaredName { name, desired: *desired }).collect();
        let source = SourceSession::new(&source_root);
        let prepared = inspect_declared_group(
            &source,
            DeclaredRequest {
                relative: Path::new(""),
                purpose: Purpose::Copy,
            },
            &[DeclaredInspection {
                target: 0,
                logical_root: &target_root,
                physical_directory: Some(&target_root),
                desired: desired.id(),
                owned: owned.id(),
                context: &context,
                names: DeclaredNames::Exact(&names),
                selection: &selector,
            }],
            &NoExecutionDemand,
            &InspectionEpoch::new(),
            Execution::Sequential,
        )
        .unwrap()
        .remove(0)
        .outcome
        .unwrap();
        let scope = Scope {
            relative: PathBuf::new(),
            context,
            observed: Some(target_root.clone()),
            output: target_root.clone(),
        };
        assert!(bind_listed_batch(&prepared, &source, &selector, &scope, false).is_ok());
        assert!(bind_listed_batch(&prepared, &source, &selector, &scope, true).is_err());
        assert_eq!(std::fs::read_dir(target_root).unwrap().count(), 0);
    }
}
