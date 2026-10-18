#!/usr/bin/env python3
"""Rotate same-compiler baseline/candidate controls, only owned marked fixtures."""
import csv, hashlib, itertools, json, pathlib, resource, subprocess, time

ROOT = pathlib.Path(__file__).resolve().parent
REPO = ROOT.parent.parent
BEFORE = REPO / "target/checked-control-before/release/checked-bench"
AFTER = REPO / "target/checked-control-after/release/checked-bench"
FIXTURE = ROOT / "fixture-10k"
if not FIXTURE.exists():
    subprocess.run([str(AFTER), "setup", str(FIXTURE), "100", "100"], check=True)
configs = [(BEFORE, "before", "none"), (AFTER, "after", "none"), (AFTER, "after", "noop"), (AFTER, "after", "admin")]
configs = [(*item, str(threads), policy) for item, threads, policy in itertools.product(configs, [0,4], ["copy","reflink"])]
records = []
identity = {"baseline":"fd277144c51700ca70d068fc88f3ff72a0d64293", "candidate":"243be8e2", "fixture":str(FIXTURE), "compiler":(ROOT/"compiler.txt").read_text(), "binary_sha256":{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [BEFORE,AFTER]}}
(ROOT/"optimized-identity.json").write_text(json.dumps(identity, indent=2)+"\n")
header = "round,label,check,threads,policy,iteration,scenario,ns,files_copied,files_reflinked,files_removed,dirs_created,dirs_removed,links_created,links_removed,dir_permission_sets\n"
with (ROOT/"optimized.csv").open("x") as output:
    output.write(header)
    for round in range(6): # round zero warmup, five retained observations
        rotated = configs[round:]+configs[:round]
        for binary,label,mode,threads,policy in rotated:
            command = [str(binary),"run",str(FIXTURE),label,mode,threads,policy,"1"]
            before = resource.getrusage(resource.RUSAGE_CHILDREN)
            start=time.monotonic_ns()
            result=subprocess.run(command,check=True,text=True,capture_output=True)
            after=resource.getrusage(resource.RUSAGE_CHILDREN)
            records.append({"round":round,"command":command,"process_ns":time.monotonic_ns()-start,"user_seconds":after.ru_utime-before.ru_utime,"system_seconds":after.ru_stime-before.ru_stime,"stderr":result.stderr})
            for line in result.stdout.splitlines(): output.write(f"{round},{line}\n")
            output.flush()
(ROOT/"optimized-commands.json").write_text(json.dumps(records,indent=2)+"\n")
