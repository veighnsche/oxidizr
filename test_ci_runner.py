#!/usr/bin/env python3
"""
Simple CI runner for Switchyard.
- Runs cargo tests with optional args
- Streams output
- Exits non-zero on failure

Future: add golden fixtures and schema validation steps here.
"""
import argparse
import os
import subprocess
import sys
import json
import tempfile
import shutil
import difflib
from pathlib import Path


def run(cmd: list[str]) -> int:
    print("+", " ".join(cmd), flush=True)
    proc = subprocess.Popen(cmd, stdout=sys.stdout, stderr=sys.stderr)
    return proc.wait()


def main() -> int:
    parser = argparse.ArgumentParser()
    # Backward-compatible cargo test options
    parser.add_argument("--package", "-p", default=None, help="Cargo package")
    parser.add_argument("--features", default=None, help="Cargo features")
    parser.add_argument("--test", default=None, help="Specific test to run")
    parser.add_argument("--nocapture", action="store_true", help="Pass -- --nocapture")

    # Golden mode
    parser.add_argument("--golden", default=None, help="Run golden diff for a scenario (e.g., minimal-plan)")
    parser.add_argument("--update", action="store_true", help="Update committed golden files when differences are found")
    parser.add_argument("--diffdir", default="golden-diff", help="Directory to write diff files in golden mode")
    args = parser.parse_args()

    env = os.environ.copy()
    env.setdefault("CARGO_TERM_COLOR", "always")

    # Golden scenarios mapping
    scenarios = {
        "minimal-plan": {
            "test_name": "golden_minimal_plan_preflight_apply",
            "committed_dir": Path("cargo/switchyard/tests/golden/minimal-plan"),
            "canon_files": [
                "canon_plan.json",
                "canon_preflight.json",
                "canon_apply_attempt.json",
                "canon_apply_result.json",
            ],
        },
        "two-action-plan": {
            "test_name": "golden_two_action_plan_preflight_apply",
            "committed_dir": Path("cargo/switchyard/tests/golden/two-action-plan"),
            "canon_files": [
                "canon_plan.json",
                "canon_preflight.json",
                "canon_apply_attempt.json",
                "canon_apply_result.json",
            ],
        },
    }

    def load_json(path: Path):
        with path.open("r", encoding="utf-8") as f:
            return json.load(f)

    def write_json(path: Path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.write("\n")

    def diff_json(a, b) -> str:
        a_s = json.dumps(a, indent=2, ensure_ascii=False) + "\n"
        b_s = json.dumps(b, indent=2, ensure_ascii=False) + "\n"
        return "\n".join(
            difflib.unified_diff(
                a_s.splitlines(), b_s.splitlines(), fromfile="expected", tofile="actual", lineterm=""
            )
        )

    def run_one_scenario(sc_name: str) -> int:
        sc = scenarios[sc_name]
        committed = sc["committed_dir"].resolve()
        test_name = sc["test_name"]

        # Prepare output directories
        diffdir = Path(args.diffdir)
        diffdir.mkdir(parents=True, exist_ok=True)

        with tempfile.TemporaryDirectory() as tmpd:
            outdir = Path(tmpd)
            env["GOLDEN_OUT_DIR"] = str(outdir)

            cargo_cmd = ["cargo", "test", "-p", "switchyard", test_name]
            if args.nocapture:
                cargo_cmd += ["--", "--nocapture"]

            rc = subprocess.call(cargo_cmd, env=env)
            if rc != 0:
                return rc

            # Compare each canon file
            any_diff = False
            for name in sc["canon_files"]:
                actual_path = outdir / name
                expected_path = committed / name
                if not actual_path.exists():
                    print(f"[{sc_name}] Missing actual canon output: {actual_path}", file=sys.stderr)
                    any_diff = True
                    continue
                actual = load_json(actual_path)

                if args.update:
                    # Update committed golden
                    write_json(expected_path, actual)
                    print(f"[{sc_name}] Updated {expected_path}")
                else:
                    if not expected_path.exists():
                        print(f"[{sc_name}] Missing committed golden: {expected_path}", file=sys.stderr)
                        any_diff = True
                        continue
                    expected = load_json(expected_path)
                    if expected != actual:
                        any_diff = True
                        d = diff_json(expected, actual)
                        dpath = diffdir / f"{sc_name}-{name}.diff"
                        with dpath.open("w", encoding="utf-8") as f:
                            f.write(d + "\n")
                        print(f"[{sc_name}] Golden mismatch for {name}. Diff written to {dpath}")

            if any_diff and not args.update:
                return 3
            return 0

    # Golden mode execution
    if args.golden:
        if args.golden == "all":
            rc = 0
            for sc_name in scenarios.keys():
                r = run_one_scenario(sc_name)
                if r != 0:
                    rc = r
            if rc != 0:
                print("Golden diffs detected. See uploaded artifacts for details.", file=sys.stderr)
            return rc
        else:
            if args.golden not in scenarios:
                print(f"Unknown scenario: {args.golden}", file=sys.stderr)
                return 2
            r = run_one_scenario(args.golden)
            if r != 0 and not args.update:
                print("Golden diffs detected. See diffs above.", file=sys.stderr)
            return r

    # Default mode: just run cargo test with provided args
    cargo_cmd = ["cargo", "test"]
    if args.package:
        cargo_cmd += ["-p", args.package]
    if args.features:
        cargo_cmd += ["--features", args.features]
    if args.test:
        cargo_cmd += [args.test]
    if args.nocapture:
        cargo_cmd += ["--", "--nocapture"]

    return run(cargo_cmd)


if __name__ == "__main__":
    sys.exit(main())
