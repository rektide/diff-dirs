"""Build/import immutable artifacts; never time cargo or a mutable target binary."""
import os
from pathlib import Path
import re
import shutil

from .common import command, digest, environment, load, require_success, sha256, write


def register(manifest, store, names, build=False):
    store = Path(store).resolve()
    store.mkdir(parents=True, exist_ok=True)
    registry_path = store / "registry.json"
    registry = load(registry_path) if registry_path.exists() else {"schema": 1, "variants": {}}
    for name in names:
        if not re.fullmatch(r"[a-zA-Z0-9_-]+", name):
            raise ValueError("variant names must be simple identifiers")
        if name in registry["variants"]:
            raise ValueError(f"{name} already registered: use a NEW store or variant name, never overwrite")
        spec = manifest["variants"][name]
        source = spec["source"]
        if not source.get("revision") or not source.get("worktree"):
            raise ValueError("source requires exact revision and worktree (use honest unknown for legacy imports)")
        cwd = Path(source["worktree"]).resolve()
        work = store / f"{name}-{digest(spec)[:16]}"
        work.mkdir()
        env = environment(spec.get("env"))
        identity = {"spec": spec, "environment": env, "built_here": build}
        compiler = command(spec.get("compiler", ["rustc", "--version", "--verbose"]), cwd, env,
                           work / "compiler")
        require_success(compiler)
        identity["compiler"] = Path(compiler["stdout"]).read_text()
        if build:
            # Separate target dirs belong in each explicit build argv, not changed release profiles.
            if "identity_command" not in source:
                raise ValueError("builds require source.identity_command verifying source.revision")
            revision = command(source["identity_command"], cwd, env, work / "source")
            require_success(revision)
            actual = Path(revision["stdout"]).read_text().strip()
            if actual != source["revision"]:
                raise ValueError(f"source identity mismatch: expected {source['revision']}, observed {actual}")
            if "state_command" not in source:
                raise ValueError("builds require source.state_command (e.g. git status --porcelain --untracked-files=no)")
            state = command(source["state_command"], cwd, env, work / "source-state")
            require_success(state)
            identity["source_state"] = Path(state["stdout"]).read_text()
            if identity["source_state"].strip() and not (source.get("allow_dirty") and spec.get("inputs")):
                raise ValueError("dirty source: checkpoint it, or explicitly allow_dirty AND retain hashed inputs")
            require_success(command(spec["build"], cwd, env, work / "build", spec.get("build_timeout", 1800)))
        artifact = Path(spec["artifact"])
        if not artifact.is_absolute():
            artifact = cwd / artifact
        artifact = artifact.resolve()
        if not artifact.is_file() or not os.access(artifact, os.X_OK):
            raise ValueError(f"not an executable artifact: {artifact}")
        identity["inputs"] = {str(Path(p).resolve()): sha256(p) for p in spec.get("inputs", [])}
        identity["original_artifact"] = str(artifact)
        identity["sha256"] = sha256(artifact)
        if spec.get("expected_sha256") and spec["expected_sha256"] != identity["sha256"]:
            raise ValueError("import artifact differs from declared expected_sha256")
        destination = work / identity["sha256"] / artifact.name
        destination.parent.mkdir()
        shutil.copy2(artifact, destination)
        destination.chmod(0o555)
        if sha256(destination) != identity["sha256"]:
            raise ValueError("artifact changed while copying")
        identity["artifact"] = str(destination)
        identity["fingerprint"] = digest(identity)
        write(work / "identity.json", identity)
        registry["variants"][name] = identity
        write(registry_path, registry)
    return registry_path


def verify(identity):
    if sha256(identity["artifact"]) != identity["sha256"]:
        raise ValueError("registered artifact changed: refuse mislabeled baseline")
    copy = dict(identity)
    fingerprint = copy.pop("fingerprint")
    if digest(copy) != fingerprint:
        raise ValueError("variant identity fingerprint mismatch")
