#!/usr/bin/env python3
"""Untimed content/link/mode parity, excluding directory mtimes and admin metadata."""
import hashlib
import os
from pathlib import Path
import stat
import sys


def inventory(root):
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = sorted(name for name in dirs if name not in (".git", ".jj"))
        for name in dirs + sorted(files):
            if name in (".git", ".jj"):
                continue
            path = Path(directory) / name
            info = path.lstat()
            if path.is_symlink():
                value = ("link", os.readlink(path))
            elif path.is_dir():
                value = ("dir", stat.S_IMODE(info.st_mode))
            else:
                digest = hashlib.sha256()
                with path.open("rb") as stream:
                    for block in iter(lambda: stream.read(1024 * 1024), b""):
                        digest.update(block)
                value = ("file", stat.S_IMODE(info.st_mode), info.st_size, digest.hexdigest())
            result[str(path.relative_to(root))] = value
    return result


def main():
    expected = inventory(sys.argv[1])
    for target in sys.argv[2:]:
        actual = inventory(target)
        if actual != expected:
            differences = sorted(key for key in expected.keys() | actual.keys() if expected.get(key) != actual.get(key))
            raise ValueError(f"tree parity failed for {target}: {differences[:20]}")
    print("content, link and mode parity passed (directory mtimes excluded)")


if __name__ == "__main__":
    main()
