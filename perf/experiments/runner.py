"""Single-host serial orchestration; setup, warmup, measurement and verification are distinct."""
import csv
import fcntl
import io
import math
import os
from pathlib import Path
import platform
import shlex
import statistics
import time
import traceback

from .common import argv_expand, command, digest, environment, expand, load, require_success, sha256, write
from .fixtures import census, prepare, reset
from .registry import verify
from .report import summarize
from .schedule import schedule


def harness_identity():
    root = Path(__file__).parent
    return {path.name: sha256(path) for path in sorted(root.glob("*.py"))}


def csv_metrics(text, specification):
    reader = csv.DictReader(io.StringIO(text), fieldnames=specification.get("fieldnames"))
    grouped = {}
    raw = []
    for row in reader:
        if not all(row.get(key) == value for key, value in specification.get("select", {}).items()):
            continue
        value = float(row[specification["field"]]) * specification.get("scale", 1e-9)
        if not math.isfinite(value) or value <= 0:
            raise ValueError("invalid CSV latency")
        key = "/".join(row[field] for field in specification.get("dimensions", [])) or "latency"
        grouped.setdefault(key, []).append(value)
        raw.append(row)
    if not grouped:
        raise ValueError("no selected in-process observations; check header/filter/field schema")
    return {key: statistics.median(values) for key, values in grouped.items()}, raw


def host_load():
    result = {"time_ns": time.time_ns(), "loadavg": os.getloadavg()}
    for name in ("io", "cpu", "memory"):
        path = Path("/proc/pressure") / name
        if path.exists():
            result[f"pressure_{name}"] = path.read_text()
    return result


def run(manifest, registry_path, output, blocks=12, seed=0, warmups=1, hyperfine="hyperfine", lock=None):
    output = Path(output).resolve()
    # One advisory lock across BOTH checkout launchers. External loads are still uncontrolled.
    lock = Path(lock or "/tmp/opencode/matched-performance.lock")
    lock.parent.mkdir(parents=True, exist_ok=True)
    with lock.open("a") as locked:
        fcntl.flock(locked, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return _run(manifest, registry_path, output, blocks, seed, warmups, hyperfine, lock)


def _run(manifest, registry_path, output, blocks, seed, warmups, hyperfine, lock):
    registry = load(registry_path)
    names = list(manifest["variants"])
    order = schedule(names, blocks, seed)
    if warmups < 0:
        raise ValueError("warmups must be nonnegative")
    identities = {name: registry["variants"][name] for name in names}
    for identity in identities.values():
        verify(identity)
    jobs = manifest["jobs"]
    if not jobs or len({job["id"] for job in jobs}) != len(jobs):
        raise ValueError("jobs require unique IDs")
    for job in jobs:
        if not job["id"].replace("-", "").replace("_", "").isalnum():
            raise ValueError("job ID must be a simple identifier")
        if job["timing"] not in ("hyperfine", "csv"):
            raise ValueError("timing is hyperfine (whole process) or csv (runner-owned timer)")
        if job.get("destinations", 1) < 1:
            raise ValueError("destinations must be positive")
        if not job.get("fixture") and not job.get("fixture_identity"):
            raise ValueError("jobs without managed fixtures require an explicit fixture_identity")
    output.mkdir(parents=True, exist_ok=False)
    env = environment(manifest.get("env"))
    result = {"schema": 1, "status": "running", "manifest": manifest, "manifest_fingerprint": digest(manifest),
              "registry": identities, "registry_fingerprint": digest(identities), "variant_order": names,
              "harness": harness_identity(), "environment": {"env": env, "host": platform.node(),
                  "platform": platform.platform(), "cpu_count": os.cpu_count(), "device": output.stat().st_dev},
              "blocks": order, "seed": seed, "warmups_per_variant_job": warmups, "prepared": {}, "samples": [],
              "lock": str(lock), "cache_condition": manifest.get("cache_condition", "setup-warmed; no cache dropping"),
              "diagnostics_are_not_latency": True,
              "job_inputs": {job["id"]: {str(Path(path).resolve()): sha256(path) for path in job.get("inputs", [])}
                             for job in jobs}}
    result["harness_fingerprint"] = digest(result["harness"])
    write(output / "manifest.json", manifest)
    write(output / "registry.json", registry)
    def checkpoint():
        write(output / "result.json", result)
    checkpoint()
    try:
        if any(job["timing"] == "hyperfine" for job in jobs):
            info = command([hyperfine, "--version"], output, env, output / "hyperfine-version")
            require_success(info)
            result["hyperfine"] = Path(info["stdout"]).read_text().strip()
        for job in jobs:
            if job.get("fixture"):
                result["prepared"][job["id"]] = prepare(job["fixture"], output / (job["id"] + "-fixture"),
                                                        job.get("scenario", "unchanged"), job.get("destinations", 1))
        # Warmup commands are full reset/measure/verify attempts but excluded from summaries.
        rounds = [(True, index, names[index % len(names):] + names[:index % len(names)])
                  for index in range(warmups)] + [(False, index, variants) for index, variants in enumerate(order)]
        for warmup, block, variants in rounds:
            # Alternate job order too. Within each job, variants stay adjacent.
            for job in (jobs if block % 2 == 0 else list(reversed(jobs))):
                for position, name in enumerate(variants):
                    identity = identities[name]
                    verify(identity)
                    for path, expected in result["job_inputs"][job["id"]].items():
                        if sha256(path) != expected:
                            raise ValueError(f"adapter/input changed during campaign: {path}")
                    index = len(result["samples"])
                    directory = output / f"sample-{index:05}-{job['id']}-{name}"
                    directory.mkdir()
                    prepared = result["prepared"].get(job["id"])
                    sample = {"sample": index, "job": job["id"], "variant": name, "warmup": warmup,
                              "block": block, "position": position, "status": "running", "directory": str(directory),
                              "fixture_fingerprint": prepared["fingerprint"] if prepared else digest(job.get("fixture_identity")),
                              "job_fingerprint": digest(job), "environment_fingerprint": digest(result["environment"]),
                              "variant_fingerprint": identity["fingerprint"], "timing_scope": job.get("scope", job["timing"]),
                              "load_before": host_load()}
                    result["samples"].append(sample)
                    variables = {"binary": identity["artifact"], "sample": directory, "state": directory / "state",
                                 "engine": Path(__file__).resolve().parents[1], "variant": name,
                                 "source": directory / "state/source", "destinations": job.get("destinations", 1)}
                    for target in range(job.get("destinations", 1)):
                        variables[f"target{target}"] = directory / f"state/target-{target}"
                    parameters = identity["spec"].get("parameters", {})
                    if variables.keys() & parameters.keys():
                        raise ValueError("variant parameters cannot override reserved artifact/fixture variables")
                    variables.update(parameters)
                    cwd = expand(job.get("cwd", "{sample}"), variables)
                    sample_env = env | {key: expand(value, variables) for key, value in job.get("env", {}).items()}
                    try:
                        start = time.perf_counter()
                        for path, expected in job.get("fixture_checks", {}).items():
                            if census(path) != expected:
                                raise ValueError(f"external fixture fingerprint mismatch: {path}")
                        if prepared:
                            sample["reset"] = reset(prepared, directory / "state")
                        for number, setup in enumerate(job.get("setup", [])):
                            require_success(command(argv_expand(setup, variables), cwd, sample_env,
                                                    directory / f"setup-{number}", job.get("timeout", 180)))
                        sample["setup_seconds_excluded"] = time.perf_counter() - start
                        argv = argv_expand(job["command"], variables)
                        sample["argv"] = argv
                        if job["timing"] == "hyperfine":
                            raw = directory / "hyperfine.json"
                            # ONE run, ONE command per invocation: orchestration controls chronology.
                            hf = [hyperfine, "-N", "--runs", "1", "--warmup", "0", "--style", "basic",
                                  "--export-json", str(raw), "--output", str(directory / "native-output")]
                            accepted = job.get("accepted_exit_codes", [0])
                            if any(code != 0 for code in accepted):
                                hf.append("--ignore-failure=" + ",".join(str(x) for x in accepted if x != 0))
                            record = command([*hf, "--", shlex.join(argv)], cwd, sample_env,
                                             directory / "measure", job.get("timeout", 180))
                            require_success(record)
                            exported = load(raw)["results"][0]
                            if len(exported["times"]) != 1 or exported["exit_codes"][0] not in accepted:
                                raise ValueError("unexpected hyperfine observation count or command failure")
                            sample["metrics"] = {"whole-process": exported["times"][0]}
                        else:
                            record = command(argv, cwd, sample_env, directory / "measure", job.get("timeout", 180))
                            if record["returncode"] not in job.get("accepted_exit_codes", [0]):
                                require_success(record)
                            sample["metrics"], sample["raw_csv_rows"] = csv_metrics(Path(record["stdout"]).read_text(), job["csv"])
                        start = time.perf_counter()
                        for number, check in enumerate(job.get("verify", [])):
                            require_success(command(argv_expand(check, variables), cwd, sample_env,
                                                    directory / f"verify-{number}", job.get("timeout", 180)))
                        if job.get("read_only"):
                            for path, expected in job.get("fixture_checks", {}).items():
                                if census(path) != expected:
                                    raise ValueError(f"read-only fixture changed during sample: {path}")
                        sample["verification_seconds_excluded"] = time.perf_counter() - start
                        sample["verification"] = "external checks passed" if job.get("verify") else "runner exit only; no external parity assertion"
                        sample["status"] = "passed"
                    except Exception as error:
                        sample.update(status="failed", error=f"{type(error).__name__}: {error}", traceback=traceback.format_exc())
                    finally:
                        sample["load_after"] = host_load()
                        write(directory / "sample.json", sample)
                        checkpoint()
                    # A failed attempt invalidates the campaign, but remaining scheduled observations survive.
        result["status"] = "failed" if any(row["status"] != "passed" for row in result["samples"]) else "passed"
    except BaseException as error:
        result.update(status="failed", error=f"{type(error).__name__}: {error}", traceback=traceback.format_exc())
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
    finally:
        checkpoint()
        write(output / "summary.json", summarize(result))
    return output / "result.json"
