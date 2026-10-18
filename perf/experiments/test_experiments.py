"""Behavioral tests; only tiny privately owned fixtures, never live repositories."""
from collections import Counter
import copy
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from experiments.common import argv_expand, load, sha256, write
from experiments.fixtures import census, create, prepare, reset, safe_relative, snapshot
from experiments.frozen import plan
from experiments.registry import register, verify
from experiments.report import summarize
from experiments.runner import csv_metrics, run
from experiments.schedule import schedule


class Ordering(unittest.TestCase):
    def test_all_variants_have_mirrored_positions_and_seed_is_replayable(self):
        names = ["a", "b", "c", "d"]
        actual = schedule(names, 20, 51)
        self.assertEqual(actual, schedule(names, 20, 51))
        self.assertNotEqual(actual, schedule(names, 20, 52))
        for left, right in zip(actual[::2], actual[1::2]):
            self.assertEqual(left, right[::-1])
            for name in names:
                self.assertEqual(left.index(name) + right.index(name), len(names) - 1)
        self.assertEqual(Counter(name for block in actual for name in block), dict.fromkeys(names, 20))
        for blocks in (0, 1, 3):
            with self.assertRaises(ValueError):
                schedule(names, blocks, 0)

    def test_literal_argv_is_not_shell_or_format_syntax(self):
        path = "/a path/it's native"
        self.assertEqual(argv_expand(["{binary}", "{state}/child", "$(touch nope)", '{"json":1}'],
                                     {"binary": path, "state": "/state space"}),
                         [path, "/state space/child", "$(touch nope)", '{"json":1}'])


class Experiments(unittest.TestCase):
    def setUp(self):
        scratch = Path(__file__).resolve().parents[2] / ".test-agent/experiment-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="space ' quote-", dir=scratch)
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def manifest(self):
        return {"variants": {name: {"source": {"revision": "system-cp", "worktree": str(self.root)},
                          "artifact": shutil.which("cp"), "compiler": [shutil.which("cp"), "--version"],
                          "profile": "system binary"} for name in ("before", "after")},
                "jobs": [{"id": "csv", "fixture_identity": {"kind": "tiny deterministic test"}, "timing": "csv",
                          "command": [sys.executable, "-c", "print('scenario,ns\\nclean,1000\\nclean,3000')"],
                          "csv": {"field": "ns", "dimensions": ["scenario"]}}]}

    def test_preserved_artifact_survives_original_replacement_and_detects_corruption(self):
        original = self.root / "native 'copy"
        shutil.copy2(shutil.which("cp"), original)
        manifest = self.manifest()
        manifest["variants"]["before"]["artifact"] = str(original)
        registry = load(register(manifest, self.root / "store", ["before"]))
        identity = registry["variants"]["before"]
        original.write_bytes(b"later cargo invocation")
        verify(identity)
        preserved = Path(identity["artifact"])
        preserved.chmod(0o755)
        preserved.write_bytes(b"not the saved binary")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            verify(identity)

    def test_dirty_build_or_failed_build_cannot_register_a_binary(self):
        manifest = self.manifest()
        spec = manifest["variants"]["before"]
        spec["source"].update(identity_command=[sys.executable, "-c", "print('system-cp')"],
                              state_command=[sys.executable, "-c", "print(' M source.rs')"])
        spec["build"] = [shutil.which("false")]
        with self.assertRaisesRegex(ValueError, "dirty source"):
            register(manifest, self.root / "dirty", ["before"], True)
        spec["source"]["state_command"] = [shutil.which("true")]
        with self.assertRaisesRegex(ValueError, "command failed"):
            register(manifest, self.root / "failed", ["before"], True)
        self.assertFalse((self.root / "failed/registry.json").exists())

    def test_reset_is_private_equivalent_and_rejects_mutated_template(self):
        fixture = self.root / "fixture"
        create(fixture, files=3, size=16)
        prepared = prepare(fixture, self.root / "prepared", "sparse", 2)
        first, second = self.root / "first", self.root / "second"
        first.mkdir()
        second.mkdir()
        reset(prepared, first / "state")
        reset(prepared, second / "state")
        self.assertEqual(census(first / "state"), census(second / "state"))
        file = next((first / "state/source").rglob("file-*"))
        relative = file.relative_to(first / "state")
        other = second / "state" / relative
        self.assertNotEqual(file.stat().st_ino, other.stat().st_ino)
        file.write_bytes(b"sample mutated")
        self.assertNotEqual(file.read_bytes(), other.read_bytes())
        self.assertEqual(census(second / "state"), prepared["metadata"])
        (Path(prepared["template"]) / relative).write_bytes(b"template tampered")
        third = self.root / "third"
        third.mkdir()
        with self.assertRaisesRegex(ValueError, "fingerprint mismatch"):
            reset(prepared, third / "state")

    def test_each_scenario_has_the_intended_start_not_a_cold_baseline(self):
        fixture = self.root / "fixture"
        create(fixture, files=3, size=16)
        for scenario in ("unchanged", "initialization", "sparse", "subtree"):
            prepared = prepare(fixture, self.root / scenario, scenario, 1)
            source = Path(prepared["template"]) / "source"
            target = Path(prepared["template"]) / "target-0"
            if scenario == "unchanged":
                self.assertEqual(census(source), census(target))
            elif scenario == "initialization":
                self.assertEqual(list(target.iterdir()), [])
            else:
                self.assertNotEqual(census(source), census(target))
            if scenario == "subtree":
                mutation = load(self.root / scenario / "mutation.json")
                self.assertTrue((source / mutation["added"]).is_dir())
                self.assertFalse((source / mutation["removed"]).exists())
                self.assertTrue((target / mutation["removed"]).is_dir())

    def test_snapshot_preserves_selected_nested_payload_and_excludes_admin(self):
        donor = self.root / "donor"
        (donor / "packages/a/node_modules/lib").mkdir(parents=True)
        (donor / "packages/a/node_modules/lib/data").write_bytes(b"dependency")
        (donor / ".jj").mkdir()
        (donor / ".jj/DO-NOT-COPY").write_text("admin")
        (donor / "tracked.txt").write_text("tracked")
        before = census(donor)
        identity = snapshot(donor, self.root / "snapshot", ["tracked.txt"], ["**/node_modules"], self.root / "copy")
        self.assertEqual(identity["payload_roots"], ["packages/a/node_modules"])
        self.assertEqual(census(donor), before)
        self.assertFalse((self.root / "snapshot/.jj").exists())
        self.assertEqual((self.root / "snapshot/packages/a/node_modules/lib/data").read_bytes(), b"dependency")
        for bad in ("../outside", "/absolute", "x/.jj/file", ".git/config"):
            with self.assertRaises(ValueError):
                safe_relative(bad)

    def test_frozen_delta_cases_cannot_invalidate_shared_clean_case(self):
        fixture = self.root / "fixture"
        create(fixture, files=2, size=16)
        frozen = self.root / "frozen"
        frozen.mkdir()
        names = ("checked-live-before-trace", "checked-live-after-trace")
        for name in names:
            shutil.copy2(shutil.which("cp"), frozen / name)
        write(frozen / "identity.json", {"compiler": "retained compiler", "artifacts": {
            name: {"sha256": sha256(frozen / name)} for name in names}})
        manifest = load(plan(frozen, fixture, self.root / "plan", jobs="all"))
        jobs = {job["id"]: job for job in manifest["jobs"]}
        clean = jobs["clean-p4"]["command"][2]
        delta = jobs["measure-p4"]["command"][2]
        self.assertNotEqual(clean, delta)
        before = census(Path(clean) / "control-target")
        (Path(delta) / "control-target/delta-only").write_bytes(b"mutation")
        self.assertEqual(census(Path(clean) / "control-target"), before)
        (frozen / names[0]).write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "frozen executable changed"):
            plan(frozen, fixture, self.root / "bad-plan")

    def test_csv_rows_retained_but_internal_replicates_not_independent_blocks(self):
        metrics, rows = csv_metrics("case,ns\na,1000\na,3000\nb,5000\n", {"field": "ns", "dimensions": ["case"]})
        self.assertAlmostEqual(metrics["a"], 2e-6)
        self.assertEqual(len(rows), 3)
        with self.assertRaises(ValueError):
            csv_metrics("case,ns\na,nan\n", {"field": "ns"})

    def test_failed_commands_retained_and_never_reported_as_faster(self):
        manifest = self.manifest()
        manifest["jobs"][0]["command"] = [sys.executable, "-c", "raise SystemExit(7)"]
        registry = register(manifest, self.root / "store", list(manifest["variants"]))
        path = run(manifest, registry, self.root / "run", blocks=2, warmups=0)
        result = load(path)
        self.assertEqual(result["status"], "failed")
        self.assertEqual(len(result["samples"]), 4)
        self.assertEqual(len(summarize(result)["failed_samples"]), 4)
        self.assertEqual(summarize(result)["groups"], [])
        for sample in result["samples"]:
            self.assertEqual(load(Path(sample["directory"]) / "measure.json")["returncode"], 7)

    def test_pairing_refuses_fixture_and_variant_identity_mismatch(self):
        manifest = self.manifest()
        registry = register(manifest, self.root / "store", list(manifest["variants"]))
        result = load(run(manifest, registry, self.root / "run", blocks=2, warmups=1))
        report = summarize(result)
        self.assertEqual(report["groups"][0]["relative_percent"]["n"], 2)
        self.assertEqual(report["groups"][0]["relative_percent"]["median"], 0)
        altered = copy.deepcopy(result)
        altered["samples"][-1]["fixture_fingerprint"] = "wrong"
        with self.assertRaisesRegex(ValueError, "fixture_fingerprint"):
            summarize(altered)
        altered = copy.deepcopy(result)
        altered["samples"][-1]["variant_fingerprint"] = "wrong"
        with self.assertRaisesRegex(ValueError, "variant fingerprint"):
            summarize(altered)

    @unittest.skipUnless(shutil.which("hyperfine"), "optional installed hyperfine required")
    def test_hyperfine_native_paths_and_failed_exit_codes(self):
        manifest = self.manifest()
        native = self.root / "native ' copy"
        shutil.copy2(shutil.which("cp"), native)
        for spec in manifest["variants"].values():
            spec["artifact"] = str(native)
        create(self.root / "fixture", files=1, size=16)
        job = {"id": "native", "fixture": str(self.root / "fixture"), "scenario": "initialization", "timing": "hyperfine",
               "command": ["{binary}", "-a", "--reflink=always", "{source}/.", "{target0}"]}
        manifest["jobs"] = [job]
        registry = register(manifest, self.root / "store", list(manifest["variants"]))
        result = load(run(manifest, registry, self.root / "good", blocks=2, warmups=0))
        self.assertEqual(result["status"], "passed")
        for sample in result["samples"]:
            raw = load(Path(sample["directory"]) / "hyperfine.json")
            self.assertEqual(raw["results"][0]["exit_codes"], [0])
        job["command"] = ["{binary}", "--not-a-valid-option"]
        result = load(run(manifest, registry, self.root / "bad", blocks=2, warmups=0))
        self.assertEqual(result["status"], "failed")
        self.assertTrue(all(row["status"] == "failed" for row in result["samples"]))


if __name__ == "__main__":
    unittest.main()
