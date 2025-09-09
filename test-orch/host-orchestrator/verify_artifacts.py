#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path

def validate_jsonl(path: Path, component_name: str) -> bool:
    if not path.exists():
        print(f"[verify] ❌ missing {component_name} JSONL at {path}")
        return False
    ok = True
    required = {"ts", "component", "run_id"}
    try:
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            if not required.issubset(set(rec.keys())):
                print(f"[verify] ❌ {component_name} JSONL missing required fields at {path}: {rec}")
                ok = False
                break
    except Exception as e:
        print(f"[verify] ❌ failed to read {component_name} JSONL at {path}: {e}")
        ok = False
    return ok


def verify_container_dir(container_dir: Path) -> bool:
    ok = True
    host_jsonl = container_dir / "host.jsonl"
    runner_jsonl = container_dir / "logs" / "runner.jsonl"
    ok &= validate_jsonl(host_jsonl, "host")
    ok &= validate_jsonl(runner_jsonl, "runner")

    # Product logs: require top-level product stdout/stderr logs under logs/
    logs_dir = container_dir / "logs"
    if not ((logs_dir / "product.stdout.log").exists() and (logs_dir / "product.stderr.log").exists()):
        print(f"[verify] ❌ top-level product stdout/stderr logs missing under {logs_dir}")
        ok = False
    return ok


def pick_latest_run_id(root: Path) -> Path:
    runs = [p for p in root.iterdir() if p.is_dir()]
    if not runs:
        raise RuntimeError(f"no runs found under {root}")
    runs.sort(key=lambda p: p.name)
    return runs[-1]


def main():
    ap = argparse.ArgumentParser(description="Verify logging artifacts against policy")
    ap.add_argument("--root", default=".artifacts", help="Artifacts root directory")
    ap.add_argument("--run-id", default="", help="Optional run ID (defaults to latest)")
    args = ap.parse_args()

    root = Path(args.root)
    if not root.exists():
        print(f"[verify] ❌ artifacts root not found: {root}")
        return 2

    run_dir = root / args.run_id if args.run_id else pick_latest_run_id(root)
    if not run_dir.exists():
        print(f"[verify] ❌ run directory not found: {run_dir}")
        return 2

    ok = True
    for container_dir in sorted(run_dir.iterdir()):
        if not container_dir.is_dir():
            continue
        print(f"[verify] Checking {container_dir}")
        ok &= verify_container_dir(container_dir)

    if ok:
        print("[verify] ✅ artifacts verification passed")
        return 0
    print("[verify] ❌ artifacts verification failed")
    return 1


if __name__ == "__main__":
    sys.exit(main())
