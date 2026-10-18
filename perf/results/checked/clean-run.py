#!/usr/bin/env python3
"""Focused read-only repeated clean controls on the shared owned target."""
import pathlib, subprocess, hashlib, json
ROOT=pathlib.Path(__file__).resolve().parent
REPO=ROOT.parent.parent
before=REPO/'target/checked-control-before/release/checked-bench'
initial=ROOT/'checked-bench-initial'
after=REPO/'target/checked-control-after/release/checked-bench'
fixture=ROOT/'fixture-10k'
configs=[(before,'before','none'),(initial,'initial','none'),(initial,'initial','noop'),(initial,'initial','admin'),(after,'optimized','none'),(after,'optimized','noop'),(after,'optimized','admin')]
configs=[(*item,str(p)) for p in [0,4] for item in configs]
# Initialize outside read-only measurement and ensure every engine agrees.
subprocess.run([str(after),'clean-repeat',str(fixture),'warmup','none','0','reflink','1'],check=True,capture_output=True)
commands=[]
with (ROOT/'clean-v1.csv').open('x') as output:
    output.write('round,label,check,threads,policy,iteration,scenario,ns,files_copied,files_reflinked,files_removed,dirs_created,dirs_removed,links_created,links_removed,dir_permission_sets\n')
    for round in range(15):
        for binary,label,mode,threads in configs[round:]+configs[:round]:
            command=[str(binary),'clean-repeat',str(fixture),label,mode,threads,'reflink','1']
            result=subprocess.run(command,check=True,capture_output=True,text=True)
            commands.append(command)
            for line in result.stdout.splitlines(): output.write(f'{round},{line}\n')
            output.flush()
(ROOT/'clean-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
(ROOT/'clean-identity.json').write_text(json.dumps({str(binary):hashlib.sha256(binary.read_bytes()).hexdigest() for binary in [before,initial,after]},indent=2)+'\n')
