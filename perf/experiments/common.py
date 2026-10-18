import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def sha256(path):
    result = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def load(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def environment(extra=None):
    # Do not retain credentials or silently inherit performance knobs.
    keep = ("PATH", "HOME", "LANG", "LC_ALL", "TMPDIR", "SYSTEMROOT")
    return {key: os.environ[key] for key in keep if key in os.environ} | (extra or {})


def command(argv, cwd, env, stem, timeout=180):
    """Log argv boundaries and output; bound and kill ordinary descendants."""
    stem = Path(stem)
    record = {"argv": list(map(str, argv)), "cwd": str(cwd), "env": env,
              "started_ns": time.time_ns(), "stdout": str(stem) + ".stdout",
              "stderr": str(stem) + ".stderr"}
    start = time.perf_counter()
    with open(record["stdout"], "xb") as out, open(record["stderr"], "xb") as err:
        child = subprocess.Popen(record["argv"], cwd=cwd, env=env, stdout=out, stderr=err,
                                 start_new_session=True)
        try:
            record["returncode"] = child.wait(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            record.update(returncode=child.returncode, timed_out=isinstance(error, subprocess.TimeoutExpired))
            if isinstance(error, KeyboardInterrupt):
                write(str(stem) + ".json", record)
                raise
    record["wall_seconds"] = time.perf_counter() - start
    write(str(stem) + ".json", record)
    return record


def require_success(record):
    if record["returncode"]:
        raise ValueError(f"command failed ({record['returncode']}), see {record['stderr']}")


def expand(value, variables):
    """Literal token replacement, never shell evaluation or str.format braces."""
    for key, replacement in variables.items():
        value = value.replace("{" + key + "}", str(replacement))
    return value


def argv_expand(argv, variables):
    if not isinstance(argv, list) or not argv or not all(isinstance(x, str) for x in argv):
        raise ValueError("commands must be nonempty argv arrays")
    return [expand(x, variables) for x in argv]
