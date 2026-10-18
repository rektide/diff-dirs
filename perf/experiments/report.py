"""Descriptive paired summaries, not independent-sample significance claims."""
from collections import defaultdict
import math
from pathlib import Path
import statistics

from .common import load


def spread(values):
    center = statistics.median(values)
    return {"n": len(values), "median": center, "min": min(values), "max": max(values),
            "mad": statistics.median(abs(value - center) for value in values)}


def summarize(result):
    baseline = result["variant_order"][0]
    observations = defaultdict(dict)
    failures = []
    for row in result["samples"]:
        expected = result["registry"][row["variant"]]["fingerprint"]
        if row["variant_fingerprint"] != expected:
            raise ValueError("sample variant fingerprint differs from registered artifact")
        if row["warmup"]:
            continue
        if row["status"] != "passed":
            failures.append(row["sample"])
            continue
        for metric, value in row["metrics"].items():
            observations[(row["job"], row["block"], metric)][row["variant"]] = (row, value)
    paired = defaultdict(list)
    rejected = []
    for (job, block, metric), variants in observations.items():
        if baseline not in variants:
            rejected.append({"job": job, "block": block, "metric": metric, "reason": "missing baseline"})
            continue
        before, old = variants[baseline]
        for name in result["variant_order"][1:]:
            if name not in variants:
                rejected.append({"job": job, "block": block, "metric": metric, "reason": f"missing {name}"})
                continue
            after, new = variants[name]
            for field in ("fixture_fingerprint", "job_fingerprint", "environment_fingerprint"):
                if before[field] != after[field]:
                    raise ValueError(f"cannot pair observations: {field} differs")
            if old <= 0 or not math.isfinite(old) or new <= 0 or not math.isfinite(new):
                raise ValueError("latency observations must be finite and positive")
            paired[(job, metric, name)].append({"block": block, "baseline_seconds": old,
                                               "candidate_seconds": new, "relative_percent": (new / old - 1) * 100,
                                               "baseline_sample": before["sample"], "candidate_sample": after["sample"]})
    groups = []
    for (job, metric, variant), rows in sorted(paired.items()):
        changes = [row["relative_percent"] for row in rows]
        summary = spread(changes)
        # Transparent descriptive heuristic; explicitly NOT a CI/p-value or load correction.
        direction = "mixed/inconclusive" if min(changes) <= 0 <= max(changes) else "consistent observed direction"
        groups.append({"job": job, "metric": metric, "baseline": baseline, "candidate": variant,
                       "paired_blocks": rows, "relative_percent": summary,
                       "baseline_seconds": spread([row["baseline_seconds"] for row in rows]),
                       "candidate_seconds": spread([row["candidate_seconds"] for row in rows]),
                       "assessment": "inconclusive: fewer than six matched blocks" if len(rows) < 6 else direction})
    return {"schema": 1, "status": "failed/inconclusive" if failures or rejected or result["status"] != "passed" else "descriptive",
            "failed_samples": failures, "unpaired": rejected, "groups": groups,
            "limits": "No confidence interval, causal load adjustment, fastest-run selection, or cold-cache claim. Negative change is faster."}


def compare(paths):
    """Audit matching campaigns, never silently pool different variant/fixture identities."""
    results = [load(path) for path in paths]
    keys = ("harness_fingerprint", "manifest_fingerprint", "registry_fingerprint", "environment", "prepared", "job_inputs")
    for result in results[1:]:
        for key in keys:
            # Prepared paths necessarily differ; compare semantic template metadata/fixture identity only.
            def semantic(value):
                return {name: {k: item[k] for k in ("fixture", "scenario", "destinations", "metadata")}
                        for name, item in value.items()}
            left, right = results[0][key], result[key]
            if (semantic(left) if key == "prepared" else left) != (semantic(right) if key == "prepared" else right):
                raise ValueError(f"incompatible campaigns: {key} differs; report each separately")
    return [summarize(result) for result in results]
