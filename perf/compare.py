#!/usr/bin/env python3
"""Compare sequential vs caller-owned Rayon pools, using one release binary.

--fixture is ONLY for disposable perf-setup fixtures: run mutates/restores them.
--pair is strictly read-only and never invokes fixture scenarios.
"""

import argparse
import csv
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, action="append", default=[])
    parser.add_argument("--pair", nargs=3, metavar=("LABEL", "ORIGINAL", "TARGET"), action="append", default=[])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("-n", type=int, default=3)
    args = parser.parse_args()
    if args.n < 3:
        parser.error("at least three observations are required")
    args.output.mkdir(parents=True, exist_ok=False)
    tasks = [(p.name, ["run", str(p), str(args.n)]) for p in args.fixture]
    tasks += [(label, ["repeat", a, b, str(args.n)]) for label, a, b in args.pair]
    for policy, prefix in [("sequential", [])] + [(f"parallel-{n}", ["parallel", str(n)]) for n in [1, 2, 4, 8]]:
        for label, command in tasks:
            filename = args.output / f"{label}-{policy}"
            print(f"measuring {filename}", flush=True)
            with filename.with_suffix(".csv").open("w") as out, filename.with_suffix(".stderr").open("w") as err:
                subprocess.run([str(args.binary), *prefix, *command], stdout=out, stderr=err, check=True)
    combine(args.output, args.output / "combined.csv")


def combine(directory, output):
    """Generate one raw-data artifact without rerunning any measurements."""
    with output.open("w", newline="") as target:
        writer = None
        for filename in sorted(directory.glob("*.csv")):
            if filename == output or filename.name == "combined.csv":
                continue
            with filename.open(newline="") as source:
                reader = csv.DictReader(source)
                if writer is None:
                    writer = csv.DictWriter(target, fieldnames=["sample", *reader.fieldnames])
                    writer.writeheader()
                for row in reader:
                    writer.writerow({"sample": filename.stem, **row})


if __name__ == "__main__":
    main()
