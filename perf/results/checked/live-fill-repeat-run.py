#!/usr/bin/env python3
"""Repeat the negative P4 first-fill control; never modify the live donor."""
import pathlib,subprocess,json,resource,time
ROOT=pathlib.Path(__file__).resolve().parent;REPO=ROOT.parent.parent
before=REPO/'target/checked-control-before/release/checked-live'
after=REPO/'target/checked-control-after/release/checked-live'
fixture=ROOT/'live-node-modules'
commands=[]
with (ROOT/'live-fill-repeat.csv').open('x') as output:
    output.write('round,label,check,threads,iteration,scenario,ns,pairs,files_reflinked,bytes_reflinked,files_removed,dirs_created,dirs_removed,links_created,links_removed,dir_permission_sets,files_copied\n')
    for round in range(3):
        configs=[(after,'optimized','admin'),(before,'before','none')]
        if round%2:configs.reverse()
        for binary,label,mode in configs:
            command=[str(binary),'fill',str(fixture),label,mode,'4']
            before_cpu=resource.getrusage(resource.RUSAGE_CHILDREN);start=time.monotonic_ns()
            result=subprocess.run(command,check=True,text=True,capture_output=True)
            after_cpu=resource.getrusage(resource.RUSAGE_CHILDREN)
            commands.append({'command':command,'process_ns':time.monotonic_ns()-start,'user_seconds':after_cpu.ru_utime-before_cpu.ru_utime,'system_seconds':after_cpu.ru_stime-before_cpu.ru_stime,'stderr':result.stderr})
            for line in result.stdout.splitlines():output.write(f'{round},{line}\n')
            output.flush()
(ROOT/'live-fill-repeat-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
