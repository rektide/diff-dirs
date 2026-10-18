#!/usr/bin/env python3
"""Interleave saved, same-build leaf variants on exclusively owned fixtures."""
import csv
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRATCH = ROOT / ".test-agent/leaf-pipeline"
VARIANTS = ["unchanged", "cleanup", "descriptor", "shared", "handoff"]
FIELDS = ["round", "variant", "fixture", "policy", "execution", "scenario", "ns", "cpu_ns",
          "files_copied", "files_reflinked", "bytes_copied", "bytes_reflinked", "symlinks_created"]


def run(variant, fixture, policy, execution):
    result = subprocess.run([str(SCRATCH / variant), "run", fixture, policy, execution],
                            cwd=ROOT, text=True, capture_output=True, check=True)
    return list(csv.reader(result.stdout.splitlines()))


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "confirm":
        confirm()
        return
    if len(sys.argv) > 1 and sys.argv[1] == "isolate":
        isolate()
        return
    rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    output = SCRATCH / "observations.csv"
    assert not output.exists(), "preserve prior observations"
    # Warmups are explicit and not silently mixed with retained timed samples.
    for variant in VARIANTS:
        for policy in ["copy", "strict", "fallback"]:
            for execution in ["serial", "p4"]:
                run(variant, "dense1k", policy, execution)
    with output.open("x", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(FIELDS)
        for iteration in range(rounds):
            variants = VARIANTS[iteration % len(VARIANTS):] + VARIANTS[:iteration % len(VARIANTS)]
            tasks = [(fixture, policy, execution)
                     for fixture in ["single", "shallow8", "dense1k", "large"]
                     for policy in (["copy", "strict", "fallback"] if fixture == "dense1k" else ["copy", "strict"])
                     for execution in ["serial", "p4"]]
            tasks = tasks[iteration % len(tasks):] + tasks[:iteration % len(tasks)]
            for fixture, policy, execution in tasks:
                for variant in variants:
                    for row in run(variant, fixture, policy, execution):
                        writer.writerow([iteration + 1, variant, *row])
                    stream.flush()
            # Larger confirmation is bounded: endpoints, P4, Copy/strict.
            for policy in ["copy", "strict"]:
                for variant in [v for v in variants if v in ["unchanged", "handoff"]]:
                    for row in run(variant, "tree30k", policy, "p4"):
                        writer.writerow([iteration + 1, variant, *row])
                    stream.flush()
            print(f"completed interleaved round {iteration + 1}", flush=True)


def confirm():
    """Small final-build confirmation and whole-process peak RSS controls."""
    with (SCRATCH / "confirmation.csv").open("x", newline="") as timing, (SCRATCH / "resources.csv").open("x", newline="") as resource:
        times = csv.writer(timing)
        resources = csv.writer(resource)
        times.writerow(FIELDS)
        resources.writerow(["round", "variant", "fixture", "policy", "execution", "user_s", "system_s", "maxrss_kib", "process_wall_s"])
        for iteration in range(3):
            for policy in ["copy", "strict"]:
                for variant in (["unchanged", "final"] if iteration % 2 == 0 else ["final", "unchanged"]):
                    path = SCRATCH / f"resource-{iteration}-{variant}-{policy}"
                    start = time.perf_counter()
                    with path.with_suffix(".stdout").open("x") as out, path.with_suffix(".stderr").open("x") as err:
                        process = subprocess.Popen([str(SCRATCH / variant), "run", "tree30k", policy, "p4"], cwd=ROOT, stdout=out, stderr=err)
                        _, status, usage = os.wait4(process.pid, 0)
                        process.returncode = os.waitstatus_to_exitcode(status)
                    elapsed = time.perf_counter() - start
                    if process.returncode:
                        raise RuntimeError(path.with_suffix(".stderr").read_text())
                    for row in csv.reader(path.with_suffix(".stdout").read_text().splitlines()):
                        times.writerow([iteration + 1, variant, *row])
                    resources.writerow([iteration + 1, variant, "tree30k", policy, "p4", usage.ru_utime, usage.ru_stime, usage.ru_maxrss, elapsed])
                    timing.flush()
                    resource.flush()
            print(f"completed final-build resource round {iteration + 1}", flush=True)


def isolate():
    """Price held clone descriptors after staging already removed its costs."""
    for execution in ["serial", "p4"]:
        for variant in ["staging-only", "final"]:
            run(variant, "dense1k", "strict", execution)
    with (SCRATCH / "descriptor-isolation.csv").open("x", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(FIELDS)
        for iteration in range(9):
            for execution in ["serial", "p4"]:
                for variant in (["staging-only", "final"] if iteration % 2 == 0 else ["final", "staging-only"]):
                    for row in run(variant, "dense1k", "strict", execution):
                        writer.writerow([iteration + 1, variant, *row])
                    stream.flush()


if __name__ == "__main__":
    main()
