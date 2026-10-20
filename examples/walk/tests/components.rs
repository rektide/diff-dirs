use std::cell::RefCell;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{MetadataExt, symlink};
use std::rc::Rc;

use diff_dirs::DirectoryPlan;
use diff_dirs_walk_example::*;

struct Fixture { _temp: tempfile::TempDir, bindings: Bindings }

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir_in("/tmp/opencode").unwrap();
        for name in ["source", "target", "reports"] { fs::create_dir(temp.path().join(name)).unwrap(); }
        let bindings = Bindings::new(temp.path().join("source"), temp.path().join("target"), temp.path().join("reports")).unwrap();
        Self { _temp: temp, bindings }
    }
    fn source(&self) -> std::path::PathBuf { self.bindings.source.to_path_buf() }
    fn target(&self) -> std::path::PathBuf { self.bindings.target.to_path_buf() }
    fn reports(&self) -> std::path::PathBuf { self.bindings.reports.to_path_buf() }
    fn pair() -> Self {
        let f = Self::new();
        fs::create_dir(f.source().join("d")).unwrap();
        fs::write(f.source().join("d/a"), b"one").unwrap();
        fs::write(f.source().join("d/b"), b"two").unwrap();
        f
    }
    fn run(&self, label: &str) -> Running<FullInspector, ByteExecutor> {
        let mut run = cursor().parts(StartParts::Fresh(self.bindings.clone())).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(Manifest::builder().label(label.to_owned()).build()).build().unwrap();
        run.bind_filesystem().unwrap();
        run
    }
}

fn limits(assessments: u64, effects: u64) -> Limits { Limits { assessments, effects } }
fn all() -> Limits { limits(100, 100) }
fn leaf_count(saved: &SavedWalk, path: &[u8]) -> u64 { saved.stack.iter().find(|f| f.node.relative.bytes == path).unwrap().aggregate.leaves }

struct CountingInspector(Rc<RefCell<Vec<Vec<u8>>>>);
impl Inspector for CountingInspector {
    fn inspect(&mut self, bindings: &Bindings, node: &Node) -> Result<DirectoryPlan, Error> {
        self.0.borrow_mut().push(node.relative.bytes.clone());
        FullInspector.inspect(bindings, node)
    }
}

struct FailingExecutor { after: bool, fail: Option<&'static str>, calls: Rc<RefCell<Vec<Work>>> }
impl Executor for FailingExecutor {
    fn execute(&mut self, bindings: &Bindings, work: &Work) -> Attempt<EffectOutcome> {
        self.calls.borrow_mut().push(work.clone());
        let matches = match (self.fail, work) {
            (Some("b"), Work::SettleSourceFile(node)) => node.relative.bytes == b"d/b",
            (Some("finish"), Work::InstallReport(recipe)) => recipe.relative.bytes == b"d",
            _ => false,
        };
        if matches {
            self.fail = None;
            let writes = if self.after { ByteExecutor.execute(bindings, work).writes } else { Writes::default() };
            return Attempt { result: Err(Error::invalid("injected failure")), writes };
        }
        ByteExecutor.execute(bindings, work)
    }
}

fn instrumented(parts: StartParts, label: &str, fail: Option<&'static str>, after: bool, inspections: Rc<RefCell<Vec<Vec<u8>>>>, calls: Rc<RefCell<Vec<Work>>>) -> Running<CountingInspector, FailingExecutor> {
    let mut run = cursor().parts(parts).inspector(CountingInspector(inspections)).executor(FailingExecutor { after, fail, calls }).program(DesiredCopy::v1()).manifest(Manifest::builder().label(label.to_owned()).build()).build().unwrap();
    run.bind_filesystem().unwrap();
    run
}

#[test]
fn supplied_components_codec_free_restore_and_four_gates() {
    let f = Fixture::pair();
    let inspections = Rc::default();
    let calls = Rc::default();
    let mut run = instrumented(StartParts::Fresh(f.bindings.clone()), "custom", Some("b"), false, Rc::clone(&inspections), Rc::clone(&calls));
    let first = run.drive(all()).unwrap();
    assert_eq!(first.stop, Stop::Blocked);
    assert_eq!((first.attempted_assessments, first.attempted_effects), (2, 3));
    assert_eq!(first.accepted.leaves, 1);
    assert_eq!(*inspections.borrow(), vec![b"".to_vec(), b"d".to_vec()]);
    assert_eq!(calls.borrow().len(), 3);
    assert_eq!(leaf_count(run.saved(), b"d"), 1);
    assert_eq!(leaf_count(run.saved(), b""), 0);
    let saved = run.into_saved().unwrap();
    let restored_inspections = Rc::default();
    let mut run = instrumented(StartParts::Restored(saved), "custom", None, false, Rc::clone(&restored_inspections), Rc::default());
    run.retry_blocked_unit().unwrap();
    run.drive(limits(0, 1)).unwrap();
    assert_eq!(leaf_count(run.saved(), b"d"), 2);
    assert_eq!(run.saved().manifest_state.next_ordinal, 1);
    run.drive(limits(0, 1)).unwrap();
    assert_eq!(leaf_count(run.saved(), b""), 2);
    assert_eq!(run.saved().manifest_state.next_ordinal, 2);
    assert_eq!(run.drive(limits(0, 1)).unwrap().stop, Stop::Done);
    assert!(restored_inspections.borrow().is_empty());
    assert!(String::from_utf8(fs::read(f.reports().join("root.txt")).unwrap()).unwrap().contains("custom\nordinal: 2\nleaves: 2"));
}

#[test]
fn failures_before_and_after_visible_leaf_and_finish_retry_without_rewrite() {
    for fail in ["b", "finish"] {
        for after in [false, true] {
            let f = Fixture::pair();
            let mut run = instrumented(StartParts::Fresh(f.bindings.clone()), "A", Some(fail), after, Rc::default(), Rc::default());
            let progress = run.drive(all()).unwrap();
            assert_eq!(progress.stop, Stop::Blocked);
            assert_eq!(leaf_count(run.saved(), b"d"), if fail == "b" { 1 } else { 2 });
            assert_eq!(leaf_count(run.saved(), b""), 0);
            assert_eq!(run.saved().manifest_state.next_ordinal, 1);
            assert_eq!(progress.writes.file_installations, if fail == "b" { 1 + u64::from(after) } else { 2 + u64::from(after) });
            let path = if fail == "b" { f.target().join("d/b") } else { f.reports().join("dir-64.txt") };
            let inode = fs::metadata(&path).ok().map(|m| m.ino());
            run.retry_blocked_unit().unwrap();
            let retry = run.drive(limits(0, 1)).unwrap();
            assert_eq!(retry.writes.file_installations, u64::from(!after));
            assert_eq!(retry.accepted.no_write, u64::from(after));
            if after { assert_eq!(inode, Some(fs::metadata(path).unwrap().ino())); }
            assert_eq!(run.drive(all()).unwrap().stop, Stop::Done);
            assert_eq!(run.saved().terminal.as_ref().unwrap().aggregate.leaves, 2);
            assert_eq!(run.saved().manifest_state.next_ordinal, 3);
        }
    }
}

#[test]
fn skip_leaf_or_finish_marks_all_ancestors_incomplete_and_does_not_consume_ordinal() {
    for fail in ["b", "finish"] {
        let f = Fixture::pair();
        let mut run = instrumented(StartParts::Fresh(f.bindings.clone()), "A", Some(fail), true, Rc::default(), Rc::default());
        run.drive(all()).unwrap();
        run.skip_blocked_unit().unwrap();
        assert!(run.saved().stack.iter().all(|f| f.incomplete));
        assert_eq!(run.saved().manifest_state.next_ordinal, 1);
        assert!(f.target().join("d/b").exists());
        run.drive(all()).unwrap();
        let outcome = run.saved().terminal.as_ref().unwrap();
        assert!(outcome.incomplete);
        assert_eq!(outcome.aggregate.leaves, if fail == "b" { 1 } else { 2 });
        let root = String::from_utf8(fs::read(f.reports().join("root.txt")).unwrap()).unwrap();
        assert!(root.contains("incomplete: true"));
        assert!(root.contains(if fail == "b" { "ordinal: 2" } else { "ordinal: 1" }));
        if fail == "b" { assert!(String::from_utf8(fs::read(f.reports().join("dir-64.txt")).unwrap()).unwrap().contains("leaves: 1\nincomplete: true")); }
    }
}

#[test]
fn manual_protocol_rejects_wrong_duplicate_and_wrong_kind_and_restores_unacknowledged() {
    let f = Fixture::new();
    let mut run = f.run("A");
    let request = run.next_request().unwrap();
    assert_eq!(request, run.next_request().unwrap());
    let snapshot = run.saved().clone();
    let ack = run.dispatch(&request).unwrap();
    let (ticket, node) = match &request { Request::Inspect { ticket, node } => (*ticket, node.clone()), _ => panic!() };
    assert!(run.acknowledge(Ack::Failed { ticket: ticket + 1, unit: Unit::Inspect(node.clone()), error: "wrong".into(), writes: Writes::default() }).is_err());
    assert!(run.acknowledge(Ack::Effected { ticket, work: Work::EnsureDirectory(node.clone()), outcome: EffectOutcome::Created, writes: Writes::default() }).is_err());
    assert_eq!(run.saved(), &snapshot);
    run.acknowledge(ack).unwrap();
    assert!(run.acknowledge(Ack::Failed { ticket, unit: Unit::Inspect(node), error: "duplicate".into(), writes: Writes::default() }).is_err());
    assert_eq!(run.drive(limits(0, 0)).unwrap().stop, Stop::Budget);
    assert_eq!(run.drive(limits(0, 0)).unwrap().attempted_effects, 0);
    assert!(!f.reports().join("root.txt").exists());
    // Effect ran, but no ack was accepted: restored state must re-examine it.
    let request = run.next_request().unwrap();
    let saved = run.saved().clone();
    let _unaccepted = run.dispatch(&request).unwrap();
    let mut restored = cursor().parts(StartParts::Restored(saved)).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(Manifest::builder().label("A".into()).build()).build().unwrap();
    restored.bind_filesystem().unwrap();
    let progress = restored.drive(limits(0, 1)).unwrap();
    assert_eq!(progress.stop, Stop::Done);
    assert_eq!(progress.accepted.no_write, 1);
    assert_eq!(progress.writes.file_installations, 0);
}

#[test]
fn zero_budgets_clean_generated_work_labels_and_terminal_no_io() {
    let f = Fixture::new();
    for (label, installations) in [("A", 1), ("B", 1), ("B", 0)] {
        let mut run = f.run(label);
        assert_eq!(run.drive(limits(0, 0)).unwrap().attempted_assessments, 0);
        let assessed = run.drive(limits(1, 0)).unwrap();
        assert_eq!((assessed.attempted_assessments, assessed.attempted_effects), (1, 0));
        assert!(matches!(run.next_request().unwrap(), Request::Effect { work: Work::InstallReport(_), .. }));
        let progress = run.drive(limits(0, 1)).unwrap();
        assert_eq!(progress.stop, Stop::Done);
        assert_eq!(progress.writes.file_installations, installations);
    }
    let mut run = f.run("B");
    run.drive(all()).unwrap();
    let terminal = run.into_saved().unwrap();
    for root in f.bindings.roots() { fs::remove_dir_all(root).unwrap(); }
    let mut restored = cursor().parts(StartParts::Restored(terminal)).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(Manifest::builder().label("B".into()).build()).build().unwrap();
    restored.bind_filesystem().unwrap();
    assert_eq!(restored.drive(limits(0, 0)).unwrap().stop, Stop::Done);
    // Nonterminal construction is likewise data-only; binding is the I/O boundary.
    let fresh = cursor().parts(StartParts::Fresh(f.bindings.clone())).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(Manifest::builder().label("B".into()).build()).build();
    assert!(fresh.is_ok());
    assert!(fresh.unwrap().bind_filesystem().is_err());
}

#[test]
fn exact_bytes_ignore_stat_equality_extras_are_retained_and_non_utf8_is_lossless() {
    let f = Fixture::new();
    let name = std::ffi::OsString::from_vec(vec![b'x', 0xff]);
    fs::create_dir(f.source().join(&name)).unwrap();
    fs::write(f.source().join(&name).join("a"), b"one").unwrap();
    fs::create_dir(f.target().join(&name)).unwrap();
    let output = f.target().join(&name).join("a");
    fs::write(&output, b"two").unwrap();
    let modified = fs::metadata(f.source().join(&name).join("a")).unwrap().modified().unwrap();
    fs::File::options().write(true).open(&output).unwrap().set_modified(modified).unwrap();
    fs::create_dir(f.target().join("extra")).unwrap();
    symlink("missing", f.target().join("extra/link")).unwrap();
    let mut run = f.run("A");
    let progress = run.drive(all()).unwrap();
    assert_eq!(progress.stop, Stop::Done);
    assert_eq!(fs::read(output).unwrap(), b"one");
    assert_eq!(run.saved().terminal.as_ref().unwrap().aggregate.physical_differences, 1); // target-only extra, not the stat-equal file
    assert!(fs::symlink_metadata(f.target().join("extra/link")).unwrap().is_symlink());
    assert!(f.reports().join("dir-78ff.txt").exists());
}

#[test]
fn shape_links_and_report_route_limits_refuse_before_children_effects() {
    for case in ["shape", "link", "long"] {
        let f = Fixture::new();
        match case {
            "shape" => { fs::create_dir(f.source().join("d")).unwrap(); fs::write(f.target().join("d"), "collision").unwrap(); }
            "link" => symlink("missing", f.source().join("d")).unwrap(),
            _ => fs::create_dir(f.source().join("x".repeat(124))).unwrap(),
        }
        let mut run = f.run("A");
        let progress = run.drive(all()).unwrap();
        assert_eq!(progress.stop, Stop::Blocked);
        assert_eq!(progress.attempted_effects, 0);
        assert_eq!(progress.attempted_assessments, 1);
        run.skip_blocked_unit().unwrap();
        assert!(run.saved().terminal.as_ref().unwrap().incomplete);
    }
}

#[test]
fn malformed_saved_parts_and_changed_label_are_rejected_without_io() {
    let f = Fixture::pair();
    let mut run = f.run("A");
    run.drive(limits(2, 2)).unwrap();
    let saved = run.into_saved().unwrap();
    let changed = cursor().parts(StartParts::Restored(saved.clone())).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(Manifest::builder().label("B".into()).build()).build();
    assert!(changed.is_err());
    for case in 0..7 {
        let mut bad = saved.clone();
        match case {
            0 => bad.stack[1].node.relative.bytes = b"../d".to_vec(),
            1 => bad.pending.as_mut().unwrap().ticket = bad.next_ticket,
            2 => bad.manifest_state.next_ordinal = 0,
            3 => bad.stack[0].remaining.clear(),
            4 => bad.stack[1].local = None,
            5 => bad.stack[0].aggregate.leaves = u64::MAX,
            _ => bad.bindings.reports = bad.bindings.target.clone(),
        }
        assert!(bad.validate().is_err(), "case {case}");
    }
}
