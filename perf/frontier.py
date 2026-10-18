#!/usr/bin/env python3
"""Read-only, interleaved serial/per-frame/frontier comparisons on explicit pairs.

All --pair arguments (including owned static fixtures) use compare, never run
or setup. Pool creation and full record/summary parity are outside timing.
"""

import argparse
import csv
import subprocess
from pathlib import Path


def configuration(value):
    try:
        p, j, k = map(int, value.split(":"))
        if p < 1 or j < 1 or k < 0:
            raise ValueError
        return p, j, k
    except ValueError as error:
        raise argparse.ArgumentTypeError("configuration must be P:J:K (P/J positive, K nonnegative)") from error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--pair", nargs=3, metavar=("LABEL", "ORIGINAL", "TARGET"), action="append", required=True)
    parser.add_argument("--config", type=configuration, action="append")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("-n", type=int, default=5)
    args = parser.parse_args()
    if args.n < 3:
        parser.error("at least three observations are required")
    args.output.mkdir(parents=True, exist_ok=False)
    configurations = args.config or [(2, 2, 8), (4, 4, 16)]
    files = []
    for index, (p, j, k) in enumerate(configurations):
        for label, original, target in args.pair[::1 if index % 2 == 0 else -1]:
            filename = args.output / f"{label}-p{p}-j{j}-k{k}"
            print(f"measuring {filename}", flush=True)
            csv_path = filename.with_suffix(".csv")
            files.append(csv_path)
            with csv_path.open("w") as out, filename.with_suffix(".stderr").open("w") as err:
                subprocess.run(
                    [str(args.binary), "compare", original, target, str(p), str(args.n), str(j), str(k), label],
                    stdout=out, stderr=err, check=True,
                )
    combine(files, args.output / "combined.csv")


def combine(files, output):
    with output.open("w", newline="") as target:
        writer = None
        for filename in files:
            with filename.open(newline="") as source:
                reader = csv.DictReader(source)
                if writer is None:
                    writer = csv.DictWriter(target, fieldnames=reader.fieldnames)
                    writer.writeheader()
                writer.writerows(reader)


if __name__ == "__main__":
    main()
