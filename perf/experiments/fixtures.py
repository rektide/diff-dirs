"""Owned metadata-preserving, private-reflink fixtures. Never reseed live donors."""
import fnmatch
import os
from pathlib import Path
import stat

from .common import command, digest, environment, load, require_success, write


def census(root):
    """Metadata contract, NOT content equivalence. No inode identity in clone fingerprint."""
    root = Path(root)
    rows = []
    def visit(directory):
        for entry in sorted(os.scandir(directory), key=lambda item: item.name):
            path = Path(entry.path)
            info = entry.stat(follow_symlinks=False)
            kind = stat.S_IFMT(info.st_mode)
            if not (stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode) or stat.S_ISLNK(info.st_mode)):
                raise ValueError(f"special file not supported in experiment fixture: {path}")
            rows.append([str(path.relative_to(root)), kind, stat.S_IMODE(info.st_mode),
                         info.st_size if not stat.S_ISDIR(info.st_mode) else None,
                         info.st_mtime_ns, os.readlink(path) if stat.S_ISLNK(info.st_mode) else None])
            if stat.S_ISDIR(info.st_mode):
                visit(path)
    visit(root)
    return {"entries": len(rows), "fingerprint": digest(rows)}


def clone(source, destination, log):
    source, destination = Path(source), Path(destination)
    if destination.exists() or destination.is_symlink():
        raise ValueError(f"clone destination must not exist: {destination}")
    if source.stat().st_dev != destination.parent.stat().st_dev:
        raise ValueError("private fixture clones must share the source filesystem")
    require_success(command(["cp", "-a", "--reflink=always", "--", str(source), str(destination)],
                            destination.parent, environment(), log, timeout=1800))


def safe_relative(value):
    path = Path(value)
    if path.is_absolute() or not path.parts or any(x in ("..", ".git", ".jj") for x in path.parts):
        raise ValueError(f"unsafe tracked/payload path: {value!r}")
    return path


def snapshot(donor, destination, tracked, patterns, log):
    """Read-only donor access. Include explicit tracked paths plus selected payload roots."""
    donor = Path(donor).resolve()
    destination.mkdir()
    selected = []
    for directory, dirs, _ in os.walk(donor, followlinks=False):
        dirs[:] = sorted(name for name in dirs if name not in (".git", ".jj"))
        for name in list(dirs):
            path = Path(directory) / name
            relative = path.relative_to(donor).as_posix()
            if any(fnmatch.fnmatchcase(relative, pattern) for pattern in patterns):
                selected.append(safe_relative(relative))
                dirs.remove(name)
    paths = sorted(set(selected + [safe_relative(value) for value in tracked]), key=lambda p: (len(p.parts), str(p)))
    copied = []
    for index, relative in enumerate(paths):
        if any(parent == relative or parent in relative.parents for parent in copied):
            continue
        source = donor / relative
        if not source.parent.resolve().is_relative_to(donor):
            raise ValueError(f"symlink parent escapes donor: {relative}")
        # A selected subtree must not smuggle administrative state into fixtures.
        if source.is_dir() and not source.is_symlink():
            for directory, dirs, files in os.walk(source, followlinks=False):
                if {".git", ".jj"}.intersection(dirs + files):
                    raise ValueError(f"administrative metadata inside selected subtree: {directory}")
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        clone(source, target, Path(str(log) + f"-{index}"))
        copied.append(relative)
    return {"donor": str(donor), "tracked_paths": list(map(str, tracked)),
            "payload_policy": patterns, "payload_roots": list(map(str, selected)),
            "snapshot_contract": "private reflinks; preserved metadata; no admin state; donor not quiesced"}


def create(root, files=32, width=4, depth=2, size=4096, donor=None, tracked=None, patterns=None):
    root = Path(root).resolve()
    root.mkdir(parents=True, exist_ok=False)
    (root / "OWNED").write_text("experiment fixture; cleanup only by explicit operator decision\n")
    base = root / "base"
    if donor:
        identity = snapshot(donor, base, tracked or [], patterns or [], root / "snapshot")
    else:
        base.mkdir()
        for index in range(files):
            components = [f"level-{(index // (width ** level)) % width}" for level in range(depth)]
            directory = base.joinpath("node_modules", *components)
            directory.mkdir(parents=True, exist_ok=True)
            data = directory / f"file-{index:06}"
            data.write_bytes(bytes([index % 251]) * size)
        (base / "tracked.txt").write_text("synthetic tracked content\n")
        (base / "package-link").symlink_to("node_modules")
        identity = {"synthetic": {"files": files, "width": width, "depth": depth, "bytes": size},
                    "payload_policy": ["node_modules", "**/node_modules", "**/dist"]}
    identity.update(schema=1, root=str(root), metadata=census(base))
    identity["fingerprint"] = digest(identity)
    write(root / "fixture.json", identity)
    return root / "fixture.json"


def prepare(root, output, scenario, destinations):
    """Create one immutable scenario template; retain all sample resets separately."""
    root, output = Path(root).resolve(), Path(output).resolve()
    identity = load(root / "fixture.json")
    if not (root / "OWNED").is_file() or census(root / "base") != identity["metadata"]:
        raise ValueError("fixture fingerprint mismatch or missing ownership marker")
    output.mkdir(exist_ok=False)
    template = output / "template"
    template.mkdir()
    clone(root / "base", template / "source", output / "clone-source")
    for index in range(destinations):
        target = template / f"target-{index}"
        if scenario == "initialization":
            target.mkdir()
        else:
            clone(root / "base", target, output / f"clone-target-{index}")
    source = template / "source"
    scopes = [source / path for path in identity.get("payload_roots", ["node_modules"])]
    regular = sorted(p for scope in scopes for p in scope.rglob("*") if p.is_file() and not p.is_symlink())
    if scenario == "sparse":
        if not regular:
            raise ValueError("sparse fixture requires a regular file")
        path = regular[0]
        # Size AND mtime differ; do not rely on metadata-only comparison seeing equal-size byte changes.
        with path.open("ab") as stream:
            stream.write(b"\nsparse source delta\n")
        write(output / "mutation.json", {"modified": str(path.relative_to(source)), "kind": "size+mtime"})
    elif scenario == "subtree":
        directories = sorted(p for scope in scopes for p in scope.rglob("*") if p.is_dir() and not p.is_symlink())
        if not directories:
            raise ValueError("subtree fixture requires a directory")
        removed = directories[0]
        added = removed.parent / "__experiment_added_subtree"
        if added.exists():
            raise ValueError("fixture collides with reserved added-subtree name")
        clone(removed, added, output / "subtree-add")
        # Retain, don't recursively delete, the removed subtree outside the compared root.
        removed.rename(output / "removed-subtree")
        write(output / "mutation.json", {"removed": str(removed.relative_to(source)),
                                          "added": str(added.relative_to(source))})
    elif scenario not in ("unchanged", "initialization"):
        raise ValueError(f"unknown scenario: {scenario}")
    result = {"fixture": identity, "scenario": scenario, "destinations": destinations,
              "template": str(template), "metadata": census(template)}
    result["fingerprint"] = digest(result)
    write(output / "prepared.json", result)
    return result


def reset(prepared, destination):
    template = Path(prepared["template"])
    if census(template) != prepared["metadata"]:
        raise ValueError("prepared fixture fingerprint mismatch")
    destination = Path(destination)
    clone(template, destination, destination.parent / "reset")
    observed = census(destination)
    if observed != prepared["metadata"]:
        raise ValueError("reset is not metadata-equivalent to intended starting state")
    return {"method": "cp -a --reflink=always; fresh private path; no cache dropping",
            "expected": prepared["metadata"], "observed": observed}
