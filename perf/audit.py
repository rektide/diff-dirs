#!/usr/bin/env python3
"""Check an fd-annotated strace for fixture payload opens/reads/mmaps."""

import sys
from pathlib import Path


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: audit.py IO.trace FIXTURE_PATH_SUBSTRING")
    trace, fixture = sys.argv[1:]
    opens = links = 0
    violations = []
    for line in Path(trace).read_text().splitlines():
        if fixture not in line:
            continue
        if line.startswith("openat("):
            opens += 1
            if "O_DIRECTORY" not in line:
                violations.append(line)
        elif line.startswith(("read(", "pread64(", "mmap(")):
            violations.append(line)
        elif line.startswith(("readlink(", "readlinkat(")):
            links += 1
    if opens == 0:
        raise SystemExit("no fixture directory opens matched; check substring/trace")
    if violations:
        raise SystemExit("unexpected fixture payload IO:\n" + "\n".join(violations))
    print(f"fixture_directory_opens={opens} symlink_reads={links} payload_opens_reads_mmaps=0")


if __name__ == "__main__":
    main()
