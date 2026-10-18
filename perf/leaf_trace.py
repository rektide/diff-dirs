#!/usr/bin/env python3
"""Separate syscall census; traced runtimes are never benchmark samples."""
import collections
import csv
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRATCH = ROOT / ".test-agent/leaf-pipeline"


def main():
    output = SCRATCH / "syscalls.csv"
    assert not output.exists()
    with output.open("x", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(["variant", "policy", "scope", "syscall", "calls"])
        configs = [(v, p) for v in ["unchanged", "cleanup", "descriptor", "shared", "handoff", "final"] for p in ["copy", "strict"]]
        configs += [("staging-only", "strict"), ("final", "fallback")]
        for variant, policy in configs:
            path = SCRATCH / f"census-{variant}-{policy}.trace"
            subprocess.run(["strace", "-f", "-yy", "-e", "trace=%file,getdents64,close,copy_file_range,ioctl,fchmod", "-o", str(path),
                            str(SCRATCH / variant), "run", "tiny", policy, "serial"], cwd=ROOT, capture_output=True, text=True, check=True)
            total = collections.Counter()
            arena = collections.Counter()
            for line in path.read_text().splitlines():
                match = re.match(r"\d+\s+(\w+)\(", line)
                if match:
                    total[match[1]] += 1
                    if ".diff-dirs-sync-" in line:
                        arena[match[1]] += 1
            for scope, counts in [("whole-process", total), ("arena-path-mentioned", arena)]:
                for syscall, calls in sorted(counts.items()):
                    writer.writerow([variant, policy, scope, syscall, calls])


if __name__ == "__main__":
    main()
