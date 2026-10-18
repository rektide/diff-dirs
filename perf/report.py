#!/usr/bin/env python3
"""Summarize raw perf-example CSV observations; Python stdlib only."""

import csv
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        raise SystemExit("usage: report.py RESULTS.csv [RESULTS.csv ...]")
    print("fixture,scenario,n,median_ms,min_ms,max_ms,mean_ms,population_stddev_ms")
    for filename in sys.argv[1:]:
        groups = defaultdict(list)
        with Path(filename).open(newline="") as source:
            for row in csv.DictReader(source):
                groups[row["scenario"]].append(int(row["elapsed_ns"]) / 1_000_000)
        for scenario, times in groups.items():
            values = (
                statistics.median(times), min(times), max(times),
                statistics.mean(times), statistics.pstdev(times),
            )
            print(f"{Path(filename).name},{scenario},{len(times)}," + ",".join(f"{v:.6f}" for v in values))


if __name__ == "__main__":
    main()
