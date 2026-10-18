"""Recipe adapter for exact checked-streaming frozen executables, without rebuilds."""
from pathlib import Path

from .common import load, sha256, write
from .fixtures import census, clone, safe_relative


FIELDS = ["label", "check", "threads", "iteration", "scenario", "ns", "pairs", "files_reflinked",
          "bytes_reflinked", "files_removed", "dirs_created", "dirs_removed", "links_created",
          "links_removed", "dir_permission_sets", "files_copied"]


def plan(frozen, fixture, output, relative="node_modules", jobs="clean"):
    frozen, fixture, output = map(lambda path: Path(path).resolve(), (frozen, fixture, output))
    identity = load(frozen / "identity.json")
    fixture_identity = load(fixture / "fixture.json")
    if census(fixture / "base") != fixture_identity["metadata"] or not (fixture / "OWNED").is_file():
        raise ValueError("fixture changed or is not owned")
    source = fixture / "base" / safe_relative(relative)
    if not source.is_dir() or source.is_symlink():
        raise ValueError("frozen comparison source must be an owned real directory")
    variants = {}
    for name, binary, revision, check in [
        ("baseline", "checked-live-before-trace", "fd277144c51700ca70d068fc88f3ff72a0d64293", "none"),
        ("default", "checked-live-after-trace", "243be8e2a28a796c61652161c9dd30c6b35fc5d0", "none"),
        ("admin", "checked-live-after-trace", "243be8e2a28a796c61652161c9dd30c6b35fc5d0", "admin"),
    ]:
        artifact = frozen / binary
        expected = identity["artifacts"][binary]["sha256"]
        if sha256(artifact) != expected:
            raise ValueError(f"frozen executable changed: {artifact}")
        variants[name] = {"source": {"revision": revision, "worktree": str(frozen),
                                     "provenance": str(frozen / "identity.json")},
                          "artifact": str(artifact), "expected_sha256": expected,
                          "compiler": ["rustc", "--version", "--verbose"],
                          "build_compiler_record": identity["compiler"],
                          "profile": "original default release; library parallel only; no diagnostics",
                          "parameters": {"label": name, "check": check},
                          "inputs": [str(frozen / value) for value in ("identity.json", "source-live.rs", "before-Cargo.toml",
                                      "after-Cargo.toml", "before-Cargo.lock", "after-Cargo.lock")]}
    output.mkdir(parents=True, exist_ok=False)
    source_metadata = census(source)
    cases = []
    selected = ["clean"] if jobs == "clean" else ["clean", "measure", "fill"]
    for action in selected:
        # Delta mutation changes directory mtimes, even after successful convergence.
        # Keep its case separate from the exact read-only clean control.
        case = output / f"case-{action}"
        case.mkdir()
        (case / ".owned").write_text("checked real-shape owned target v1\n")
        (case / ".source").write_text(str(source))
        if action != "fill":
            clone(source, case / "control-target", output / f"initial-target-{action}")
            target_metadata = census(case / "control-target")
            if source_metadata != target_metadata:
                raise ValueError("initial target is not metadata-equivalent")
        for threads in (0, 4):
            command = ["{binary}", action, str(case), "{label}", "{check}", str(threads)]
            if action == "measure":
                command.append("1")
            checks = {str(source): source_metadata}
            if action == "clean":
                checks[str(case / "control-target")] = target_metadata
            cases.append({"id": f"{action}-{'serial' if threads == 0 else 'p4'}", "timing": "csv",
                          "scope": "in-process API timer; pool/startup/setup/verification excluded",
                          "fixture_identity": {"fixture": fixture_identity, "source": str(source), "case": str(case),
                              "starting_state": "shared metadata-equivalent target; runner restores clean before each measure; fill uses new private target"},
                          "fixture_checks": checks, "read_only": action == "clean", "command": command,
                          "csv": {"field": "ns", "fieldnames": FIELDS, "dimensions": ["scenario"]},
                          "timeout": 1800})
    manifest = {"description": "Exact frozen checked-streaming controls: baseline/default/admin; original frozen records untouched",
                "cache_condition": "shared owned source/clean target; pre-sample metadata validation warms caches; never cache-dropped",
                "variants": variants, "jobs": cases}
    write(output / "manifest.json", manifest)
    return output / "manifest.json"
