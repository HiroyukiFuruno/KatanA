"""Compare two real Windows invocations only when their identities match."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re


def compare(cold, warm, cold_before, cold_after, warm_before, warm_after):
    identities = (cold_before, cold_after, warm_before, warm_after)
    if any(identity != identities[0] for identity in identities):
        raise ValueError("source or execution identity changed")
    identity = identities[0]
    if (identity.get("schema") != 1 or identity.get("cache_eligible") is not True
            or not re.fullmatch(r"windows-target-v1-[a-f0-9]{64}", identity.get("key", ""))):
        raise ValueError("compiled cache identity is unavailable")
    inputs = identity.get("inputs")
    if (identity.get("environment_encoding") != "sha256-per-value"
            or not isinstance(inputs, dict) or not isinstance(inputs.get("source"), dict)
            or not isinstance(inputs.get("environment"), dict)
            or any(value is not None and (not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value))
                   for value in inputs["environment"].values())):
        raise ValueError("redacted execution inputs are unavailable")
    for label, record, hit in (("cold", cold, False), ("warm", warm, True)):
        cache = record.get("compiled_target_cache", {})
        if (record.get("schema") != 1 or record.get("executed_once") is not True
                or record.get("exit_code") != 0 or cache.get("eligible") is not True
                or cache.get("key") != identity["key"] or cache.get("hit") is not hit
                or record.get("reuse_authorized") is not False):
            raise ValueError(f"{label} execution or cache state is invalid")
        elapsed = record.get("elapsed_seconds")
        if type(elapsed) not in (int, float) or not math.isfinite(elapsed) or elapsed <= 0:
            raise ValueError(f"{label} timing is invalid")
        if record.get("lock_observation_phase") != "after_command":
            raise ValueError("lock identity is unavailable")
    for key in ("source", "command_argv", "cargo_lock_sha256"):
        if cold.get(key) != warm.get(key):
            raise ValueError(f"cold/warm {key} differs")
    source = cold.get("source", {})
    if (source.get("RUNNER_OS") != "Windows" or not source.get("RUNNER_ARCH")
            or not re.fullmatch(r"[0-9a-f]{40}", source.get("GITHUB_SHA") or "")
            or not str(source.get("GITHUB_RUN_ID") or "").isdigit()
            or not str(source.get("GITHUB_RUN_ATTEMPT") or "").isdigit()
            or any(not re.fullmatch(r"[0-9a-f]{64}", digest or "")
                   for digest in cold.get("cargo_lock_sha256", {}).values())
            or set(cold.get("cargo_lock_sha256", {})) != {"Cargo.lock", "scripts/screenshot/Cargo.lock"}):
        raise ValueError("run or lock identity is incomplete")
    if inputs["source"].get("head") != source["GITHUB_SHA"]:
        raise ValueError("execution input source differs from workflow source")
    canonical = json.dumps([cold, warm, *identities], sort_keys=True, separators=(",", ":")).encode()
    return {"schema": 1, "diagnostic_only": True, "reuse_authorized": False,
            "full_environment_equivalence_proven": False,
            "inputs_sha256": hashlib.sha256(canonical).hexdigest(), "identity": identity,
            "source": source, "cold_seconds": cold["elapsed_seconds"],
            "warm_seconds": warm["elapsed_seconds"],
            "timing_scope": "command only; excludes runner setup and cache transfer"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cold", type=Path)
    parser.add_argument("warm", type=Path)
    parser.add_argument("cold_before", type=Path)
    parser.add_argument("cold_after", type=Path)
    parser.add_argument("warm_before", type=Path)
    parser.add_argument("warm_after", type=Path)
    args = parser.parse_args()
    values = [json.loads(path.read_text(encoding="utf-8")) for path in vars(args).values()]
    print(json.dumps(compare(*values), sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
