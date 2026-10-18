#!/usr/bin/env python3
"""Read-only real donor; every modified target is created under the marked root."""
import pathlib,subprocess,hashlib,json
ROOT=pathlib.Path(__file__).resolve().parent
REPO=ROOT.parent.parent
before=REPO/'target/checked-control-before/release/checked-live'
after=REPO/'target/checked-control-after/release/checked-live'
fixture=ROOT/'live-node-modules'
subprocess.run([str(after),'setup',str(fixture),'/home/rektide/a/a/opencode/node_modules'],check=True)
commands=[]
header='round,label,check,threads,iteration,scenario,ns,pairs,files_reflinked,bytes_reflinked,files_removed,dirs_created,dirs_removed,links_created,links_removed,dir_permission_sets,files_copied\n'
with (ROOT/'live.csv').open('x') as output:
    output.write(header)
    for binary,label,mode,p in [(before,'before','none','0'),(after,'optimized','admin','0'),(before,'before','none','4'),(after,'optimized','admin','4')]:
        command=[str(binary),'fill',str(fixture),label,mode,p]
        result=subprocess.run(command,check=True,text=True,capture_output=True)
        commands.append({'command':command,'stderr':result.stderr})
        for line in result.stdout.splitlines():output.write(f'fill,{line}\n')
        output.flush()
    configs=[(before,'before','none','0'),(after,'optimized','none','0'),(after,'optimized','admin','0'),(before,'before','none','4'),(after,'optimized','none','4'),(after,'optimized','admin','4')]
    for round in range(6): # first round is warmup; five retained
        for binary,label,mode,p in configs[round:]+configs[:round]:
            command=[str(binary),'measure',str(fixture),label,mode,p,'1']
            result=subprocess.run(command,check=True,text=True,capture_output=True)
            commands.append({'command':command,'stderr':result.stderr})
            for line in result.stdout.splitlines():output.write(f'{round},{line}\n')
            output.flush()
(ROOT/'live-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
(ROOT/'live-identity.json').write_text(json.dumps({'source':(fixture/'.source').read_text(),'baseline':'fd277144','candidate':'243be8e2','sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [before,after]}},indent=2)+'\n')
