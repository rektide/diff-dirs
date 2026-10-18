import argparse
import json
from pathlib import Path
import sys

from .common import load
from .fixtures import create
from .frozen import plan
from .registry import register
from .report import compare, summarize
from .runner import run


def main():
    parser = argparse.ArgumentParser(description="Register immutable binaries; run balanced matched blocks without concurrent benchmarks")
    commands = parser.add_subparsers(dest="action", required=True)
    registration = commands.add_parser("register", help="import existing binaries, or explicitly build then preserve")
    registration.add_argument("manifest")
    registration.add_argument("store")
    registration.add_argument("names", nargs="*")
    registration.add_argument("--build", action="store_true")
    fixture = commands.add_parser("fixture", help="create a NEW owned small or read-only opencode-derived fixture")
    fixture.add_argument("root")
    fixture.add_argument("--files", type=int, default=32)
    fixture.add_argument("--width", type=int, default=4)
    fixture.add_argument("--depth", type=int, default=2)
    fixture.add_argument("--bytes", type=int, default=4096)
    fixture.add_argument("--donor")
    fixture.add_argument("--tracked-json", help="JSON array of donor-relative tracked paths, exported read-only by operator")
    fixture.add_argument("--payload", action="append", default=None)
    execution = commands.add_parser("run")
    execution.add_argument("manifest")
    execution.add_argument("registry")
    execution.add_argument("output")
    execution.add_argument("--blocks", type=int, default=12)
    execution.add_argument("--seed", type=int, default=0)
    execution.add_argument("--warmups", type=int, default=1)
    execution.add_argument("--hyperfine", default="hyperfine")
    execution.add_argument("--lock", help="override shared advisory lock only for an intentionally separate host")
    frozen = commands.add_parser("frozen-plan", help="registerable exact frozen checked controls using an owned stable source/target")
    frozen.add_argument("frozen_directory")
    frozen.add_argument("fixture")
    frozen.add_argument("output")
    frozen.add_argument("--relative", default="node_modules")
    frozen.add_argument("--jobs", choices=["clean", "all"], default="clean")
    reporting = commands.add_parser("report")
    reporting.add_argument("results", nargs="+")
    args = parser.parse_args()
    try:
        if args.action == "register":
            manifest = load(args.manifest)
            print(register(manifest, args.store, args.names or list(manifest["variants"]), args.build))
        elif args.action == "fixture":
            if min(args.files, args.width, args.bytes) < 1 or args.depth < 0:
                raise ValueError("files, width and bytes must be positive; depth nonnegative")
            if args.donor and not args.tracked_json:
                raise ValueError("donor import requires explicit --tracked-json (use [] for payload only)")
            print(create(args.root, args.files, args.width, args.depth, args.bytes, args.donor,
                         load(args.tracked_json) if args.tracked_json else None,
                         args.payload or ["node_modules", "**/node_modules", "**/dist"]))
        elif args.action == "frozen-plan":
            print(plan(args.frozen_directory, args.fixture, args.output, args.relative, args.jobs))
        elif args.action == "run":
            path = run(load(args.manifest), args.registry, args.output, args.blocks, args.seed,
                       args.warmups, args.hyperfine, args.lock)
            print(path)
            return 0 if load(path)["status"] == "passed" else 1
        else:
            reports = compare(args.results) if len(args.results) > 1 else [summarize(load(args.results[0]))]
            print(json.dumps(reports, indent=2))
        return 0
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
