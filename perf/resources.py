#!/usr/bin/env python3
"""Linux process CPU + sampled /proc VmHWM for one read-only benchmark command.

Prints command stdout unchanged, then resource JSON. Child user/system time is
exact getrusage accounting (startup included). RSS is the largest child VmHWM
observed at 5ms intervals: not parent RSS, and not a strict bound on a peak in
the final sampling gap. Use backend-once, not a multi-backend parity collector.
"""

import json
import resource
import subprocess
import sys
import time
from pathlib import Path


def main():
    if len(sys.argv) < 3 or sys.argv[1] != "--":
        raise SystemExit("usage: resources.py -- COMMAND ARGUMENTS ...")
    start = time.perf_counter()
    child = subprocess.Popen(sys.argv[2:])
    high_water = 0
    samples = 0
    while child.poll() is None:
        try:
            status = Path(f"/proc/{child.pid}/status").read_text()
        except FileNotFoundError:
            break
        for line in status.splitlines():
            if line.startswith("VmHWM:"):
                high_water = max(high_water, int(line.split()[1]))
                samples += 1
        time.sleep(0.005)
    code = child.wait()
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    print(json.dumps({
        "exit_code": code,
        "process_wall_seconds": time.perf_counter() - start,
        "user_seconds": usage.ru_utime,
        "system_seconds": usage.ru_stime,
        "sampled_child_vmhwm_kib": high_water,
        "rss_samples": samples,
        "sampling_seconds": 0.005,
    }, sort_keys=True))
    raise SystemExit(code)


if __name__ == "__main__":
    main()
