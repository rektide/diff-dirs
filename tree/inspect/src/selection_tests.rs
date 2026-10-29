use super::*;
use crate::selection::{baseline, interior_role, mirror_install, validate_entry};

fn physical(source: Option<Kind>, target: Option<Kind>, compared: Comparison, action: Action) -> EntrySelection<u8> {
    let shape = Shape {
        source: source.map_or(SourceObservation::Absent, SourceObservation::Present),
        target,
    };
    EntrySelection {
        source: shape.source,
        target,
        comparison: compared,
        baseline: baseline(shape, compared),
        action,
        install: mirror_install(shape, action),
        removal: matches!(
            action,
            Action::RemoveLeaf
                | Action::Directory {
                    container: Container::Remove,
                    ..
                }
        )
        .then_some(RemovalAuthority::SourceAbsent),
        desired: None,
        interior: interior_role(shape, action).map(|role| Interior {
            role,
            context: 7,
            input: InteriorInput::PhysicalDirectory,
        }),
    }
}
fn parts(entry: EntrySelection<u8>) -> SelectionParts<u8> {
    SelectionParts {
        relative: "q".into(),
        purpose: Purpose::Compare,
        structural_root: false,
        domain: SelectionDomain::PhysicalUnion,
        input_context: 7,
        policy: SemanticId::new("test", 1, vec![]),
        context_schema: SemanticId::new("test.u8", 1, vec![]),
        comparator: StatKey.identity().clone(),
        scoped: false,
        root_excluded: false,
        excluded_names: vec![],
        entries: vec![EntryParts { name: "entry".into(), entry }],
    }
}

#[test]
fn complete_physical_action_table_and_grant_validation() {
    use Action::*;
    use Kind::{Directory as D, File as F, Other as O, Symlink as L};
    let create = Directory {
        container: Container::Create,
        descend: true,
    };
    let conditional = Directory {
        container: Container::CreateIfNeeded,
        descend: true,
    };
    let keep = Directory {
        container: Container::Keep,
        descend: true,
    };
    let stop = Directory {
        container: Container::Keep,
        descend: false,
    };
    let remove = Directory {
        container: Container::Remove,
        descend: true,
    };
    let replace = ReplaceWithDirectory { descend: true };
    let create_stop = Directory {
        container: Container::Create,
        descend: false,
    };
    let replace_stop = ReplaceWithDirectory { descend: false };
    let actions = [
        PreserveTree,
        KeepLeaf,
        InstallLeaf,
        RemoveLeaf,
        ConfirmAbsent,
        create,
        create_stop,
        conditional,
        Directory {
            container: Container::CreateIfNeeded,
            descend: false,
        },
        keep,
        stop,
        remove,
        Directory {
            container: Container::Remove,
            descend: false,
        },
        ReplaceWithLeaf,
        replace,
        replace_stop,
    ];
    type Row<'a> = (Option<Kind>, Option<Kind>, Comparison, &'a [Action]);
    let rows: &[Row<'_>] = &[
        (Some(F), None, Comparison::Different, &[PreserveTree, KeepLeaf, InstallLeaf]),
        (Some(L), None, Comparison::Different, &[PreserveTree, KeepLeaf, InstallLeaf]),
        (None, Some(F), Comparison::Different, &[PreserveTree, KeepLeaf, RemoveLeaf]),
        (Some(F), Some(F), Comparison::Different, &[PreserveTree, KeepLeaf, InstallLeaf]),
        (Some(F), Some(F), Comparison::Equal, &[PreserveTree, KeepLeaf]),
        (Some(F), Some(L), Comparison::Different, &[PreserveTree, KeepLeaf, InstallLeaf]),
        (Some(D), None, Comparison::Different, &[PreserveTree, create, create_stop, conditional]),
        (None, Some(D), Comparison::Different, &[PreserveTree, keep, stop, remove]),
        (Some(D), Some(D), Comparison::Equal, &[PreserveTree, keep, stop]),
        (Some(F), Some(D), Comparison::Different, &[PreserveTree, keep, stop, ReplaceWithLeaf]),
        (Some(D), Some(F), Comparison::Different, &[PreserveTree, KeepLeaf, replace, replace_stop]),
        (Some(O), Some(F), Comparison::Unsupported { code: 1 }, &[PreserveTree]),
        (Some(F), Some(O), Comparison::Unsupported { code: 1 }, &[PreserveTree]),
        (None, Some(O), Comparison::Unsupported { code: 1 }, &[PreserveTree]),
    ];
    for &(s, t, c, allowed) in rows {
        for action in actions {
            let entry = physical(s, t, c, action);
            assert_eq!(
                validate_entry(SelectionDomain::PhysicalUnion, &entry).is_ok(),
                allowed.contains(&action),
                "{s:?}/{t:?} {c:?}: {action:?}"
            );
        }
    }
    let mut entry = physical(Some(F), Some(F), Comparison::Equal, InstallLeaf);
    entry.install = Some(InstallAuthority {
        intent: InstallIntent::Renew,
        destination: DestinationGrant::ReplaceLeaf,
    });
    assert!(validate_entry(SelectionDomain::PhysicalUnion, &entry).is_ok());
    entry.install.as_mut().unwrap().destination = DestinationGrant::AbsentOnly;
    assert!(validate_entry(SelectionDomain::PhysicalUnion, &entry).is_err());
    entry.action = KeepLeaf;
    assert!(validate_entry(SelectionDomain::PhysicalUnion, &entry).is_err(), "inert grants must not survive");
}

#[test]
fn honest_comparison_coverage_renewal_and_zero_planned_opaque_work() {
    let mut renew = physical(Some(Kind::File), Some(Kind::File), Comparison::Equal, Action::InstallLeaf);
    renew.install = Some(InstallAuthority {
        intent: InstallIntent::Renew,
        destination: DestinationGrant::ReplaceLeaf,
    });
    let selected = DirectorySelection::from_parts(parts(renew)).unwrap();
    assert!(selected.diff().is_empty());
    assert!(selected.coverage().is_clean());
    assert_eq!(selected.coverage().deviated, 1);
    assert_eq!(selected.coverage().application_need, ApplicationNeed::LocalOrOpaque);
    assert_eq!(DirectorySelection::from_parts(selected.to_parts()).unwrap(), selected);

    let opaque = DirectorySelection::from_parts(parts(physical(
        None,
        Some(Kind::Directory),
        Comparison::Different,
        Action::Directory {
            container: Container::Keep,
            descend: true,
        },
    )))
    .unwrap();
    assert_eq!(opaque.coverage().planned, 0);
    assert_eq!(opaque.coverage().application_need, ApplicationNeed::LocalOrOpaque);
    let paired = DirectorySelection::from_parts(parts(physical(
        Some(Kind::Directory),
        Some(Kind::Directory),
        Comparison::Equal,
        Action::Directory {
            container: Container::Keep,
            descend: true,
        },
    )))
    .unwrap();
    assert_eq!(paired.coverage().application_need, ApplicationNeed::None);
    for compared in [
        Comparison::NotAssessed,
        Comparison::Unsupported { code: 12 },
        Comparison::Unresolved { code: 9 },
    ] {
        let selected = DirectorySelection::from_parts(parts(physical(Some(Kind::File), Some(Kind::File), compared, Action::KeepLeaf))).unwrap();
        assert!(!selected.coverage().is_clean());
        assert_eq!(selected.entry(SlotId(0)).baseline, None);
        assert!(selected.diff().is_empty());
    }
}

fn manifest(byte: u8) -> ManifestId {
    ManifestId {
        version: 1,
        platform: NativePlatform::Unix,
        digest: [byte; 32],
    }
}

#[test]
fn absent_owned_structural_proof_is_not_a_preservation_boundary() {
    let mut p = parts(EntrySelection {
        source: SourceObservation::NotRequested,
        target: None,
        comparison: Comparison::NotAssessed,
        baseline: None,
        action: Action::PreserveTree,
        install: None,
        removal: None,
        desired: None,
        interior: None,
    });
    p.domain = SelectionDomain::Listed {
        desired: manifest(1),
        owned: manifest(2),
    };
    let selection = DirectorySelection::from_parts(p).unwrap();
    assert_eq!(selection.coverage().retained, 0);
    assert_eq!(selection.coverage().deviated, 1, "no baseline remains a distinct action diagnostic");
    let source_only = DirectorySelection::from_parts(parts(physical(Some(Kind::File), None, Comparison::Different, Action::PreserveTree))).unwrap();
    assert_eq!(source_only.coverage().retained, 1);
}
#[test]
fn listed_obligations_never_fabricate_absence_or_escape_their_input() {
    let desired = manifest(1);
    let owned = manifest(2);
    let domain = SelectionDomain::Listed { desired, owned };
    let mut entry = EntrySelection::<u8> {
        source: SourceObservation::NotRequested,
        target: None,
        comparison: Comparison::NotAssessed,
        baseline: None,
        action: Action::ConfirmAbsent,
        install: None,
        removal: Some(RemovalAuthority::OwnedLeaf { owned, leaf: LeafId(5) }),
        desired: None,
        interior: None,
    };
    assert!(validate_entry(domain, &entry).is_ok());
    assert!(validate_entry(SelectionDomain::PhysicalUnion, &entry).is_err());
    entry.source = SourceObservation::Absent;
    assert!(validate_entry(domain, &entry).is_err());
    entry.source = SourceObservation::NotRequested;
    entry.target = Some(Kind::File);
    entry.action = Action::RemoveLeaf;
    assert!(validate_entry(domain, &entry).is_ok());
    entry.target = Some(Kind::Directory);
    assert!(validate_entry(domain, &entry).is_err(), "an owned leaf is not recursive directory authority");

    entry.source = SourceObservation::Present(Kind::File);
    entry.desired = Some(DesiredInput::Leaf {
        leaf: LeafId(7),
        expected: LeafExpectation::Exact {
            kind: LeafKind::RegularFile,
            executable: Some(true),
        },
    });
    entry.action = Action::ReplaceWithLeaf;
    entry.install = Some(InstallAuthority {
        intent: InstallIntent::Renew,
        destination: DestinationGrant::ReplaceTree,
    });
    entry.removal = Some(RemovalAuthority::CoupledOwned { owned, subtree: NodeId(3) });
    entry.interior = Some(Interior {
        role: InteriorRole::RemoveForReplacement,
        context: 7,
        input: InteriorInput::Listed { desired, subtree: NodeId(8) },
    });
    assert!(validate_entry(domain, &entry).is_ok());
    entry.interior.as_mut().unwrap().input = InteriorInput::PhysicalDirectory;
    assert!(validate_entry(domain, &entry).is_err());

    // Expansion must carry the declared slice, never donor physical enumeration.
    entry.source = SourceObservation::Present(Kind::Directory);
    entry.target = Some(Kind::File);
    entry.desired = Some(DesiredInput::Directory { node: NodeId(4) });
    entry.action = Action::ReplaceWithDirectory { descend: true };
    entry.removal = Some(RemovalAuthority::OwnedLeaf { owned, leaf: LeafId(1) });
    entry.interior = Some(Interior {
        role: InteriorRole::PrepareReplacement,
        context: 7,
        input: InteriorInput::Listed { desired, subtree: NodeId(4) },
    });
    assert!(validate_entry(domain, &entry).is_ok());
    entry.source = SourceObservation::Absent;
    entry.target = None;
    entry.action = Action::PreserveTree;
    entry.install = None;
    entry.removal = None;
    entry.interior = None;
    assert!(validate_entry(domain, &entry).is_err(), "both-missing desired obligations cannot disappear");
}

#[test]
fn decoded_mutations_use_the_same_entry_validator() {
    let original = physical(
        Some(Kind::Directory),
        Some(Kind::File),
        Comparison::Different,
        Action::ReplaceWithDirectory { descend: true },
    );
    let mut mutations = Vec::new();
    let mut e = original.clone();
    e.interior = None;
    mutations.push(e);
    let mut e = original.clone();
    e.interior.as_mut().unwrap().role = InteriorRole::SourceOnly;
    mutations.push(e);
    let mut e = original.clone();
    e.comparison = Comparison::Equal;
    mutations.push(e);
    let mut e = original.clone();
    e.install = None;
    mutations.push(e);
    let mut e = original.clone();
    e.baseline = None;
    mutations.push(e);
    for entry in mutations {
        assert!(validate_entry(SelectionDomain::PhysicalUnion, &entry).is_err());
        assert!(DirectorySelection::from_parts(parts(entry)).is_err());
    }
}
