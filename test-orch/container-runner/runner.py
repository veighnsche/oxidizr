#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time
import tarfile
from datetime import datetime, timezone
from pathlib import Path

from lib import proc as procmod
from lib import log as logmod
from lib import fs as fsmod
from lib import suites as suitesmod

PROOF_ROOT = Path("/workspace/.proof")
LOGS_DIR = PROOF_ROOT / "logs"
SNAPSHOTS_DIR = PROOF_ROOT / "snapshots"
RESULTS_DIR = PROOF_ROOT / "results"
JSONL_PATH = Path("/var/log/runner.jsonl")
SUMMARY_PATH = PROOF_ROOT / "summary.json"
PROJECT_DIR = Path("/workspace")
TESTS_DIR = PROJECT_DIR / "tests"
TMP_DIR = PROOF_ROOT / "tmp"


def ensure_dirs():
    for d in [PROOF_ROOT, LOGS_DIR, SNAPSHOTS_DIR, RESULTS_DIR, JSONL_PATH.parent, TMP_DIR]:
        d.mkdir(parents=True, exist_ok=True)


def read_os_release():
    data = {}
    try:
        with open("/etc/os-release", "r") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    k, v = line.split("=", 1)
                    data[k] = v.strip().strip('"')
    except Exception:
        pass
    return data


def get_container_id():
    # Try hostname first (Docker sets it to container ID)
    try:
        with open("/etc/hostname", "r") as f:
            hn = f.read().strip()
            if hn:
                return hn
    except Exception:
        pass
    # Fallback to cgroup hash
    try:
        with open("/proc/self/cgroup", "r") as f:
            for line in f:
                for tok in line.strip().split('/'):
                    if len(tok) >= 12 and all(c in "0123456789abcdef" for c in tok.lower()):
                        return tok[:12]
    except Exception:
        pass
    return "unknown"


def stage_preflight(logger: logmod.JSONLLogger) -> None:
    print("[preflight] starting…")
    osr = read_os_release()
    print(f"[preflight] os-release ID={osr.get('ID','?')} VERSION_ID={osr.get('VERSION_ID','?')}")

    # Pacman DB timestamps
    db_dir = Path("/var/lib/pacman/sync")
    if db_dir.is_dir():
        for p in sorted(db_dir.glob("*.db")):
            try:
                ts = datetime.fromtimestamp(p.stat().st_mtime, tz=timezone.utc).isoformat()
                print(f"[preflight] pacman db: {p.name} mtime={ts}")
            except Exception:
                pass

    # Tool versions
    for cmd in (["rustup", "--version"], ["cargo", "--version"], ["pacman", "-V"]):
        res = procmod.run(cmd, timeout=30)
        logger.event(stage="preflight", suite=None, event="cmd_exec", cmd=res.command_str, rc=res.rc,
                     elapsed_ms=res.elapsed_ms, msg=res.stdout.strip()[:200])

    # Verify product can be built (no repairs). Use cargo check as a quick build test.
    res = procmod.run(["cargo", "check"], cwd=str(PROJECT_DIR), timeout=1200)
    logger.event(stage="preflight", suite=None, event="cmd_exec", cmd=res.command_str, rc=res.rc, elapsed_ms=res.elapsed_ms)
    if res.rc != 0:
        print("[preflight] ❌ cargo check failed; aborting")
        sys.exit(res.rc)
    print("[preflight] ✅ cargo check succeeded")


def stage_deps(logger: logmod.JSONLLogger) -> None:
    print("[deps] starting…")
    # Verify-only: ensure required packages are present; do not install or normalize mirrors.
    packages = [
        "base-devel", "sudo", "git", "curl", "rustup", "which", "findutils",
        "python", "python-yaml", "tar", "gzip", "jq"
    ]
    missing = []
    for pkg in packages:
        res = procmod.run(["pacman", "-Qi", pkg], timeout=60)
        logger.event(stage="deps", suite=None, event="verify_pkg", cmd=res.command_str, rc=res.rc, elapsed_ms=res.elapsed_ms, msg=pkg)
        if res.rc != 0:
            missing.append(pkg)
    if missing:
        print("[deps] ❌ missing packages (bake into image): " + ", ".join(missing))
        sys.exit(1)
    print("[deps] ✅ dependencies present")


def stage_build(logger: logmod.JSONLLogger) -> None:
    print("[build] starting…")
    profile = os.environ.get("CARGO_PROFILE", "release")
    toolchain = os.environ.get("RUSTUP_TOOLCHAIN", "stable")

    # Select default toolchain (toolchain should be pre-installed in image)
    res = procmod.run(["rustup", "default", toolchain], timeout=120)
    logger.event(stage="build", suite=None, event="cmd_exec", cmd=res.command_str, rc=res.rc, elapsed_ms=res.elapsed_ms)
    if res.rc != 0:
        print("[build] ❌ rustup default failed")
        sys.exit(res.rc)

    # Log tool versions for provenance
    for cmd in (["rustup", "show"], ["cargo", "--version"], ["rustc", "--version"]):
        info = procmod.run(cmd, timeout=60)
        logger.event(stage="build", suite=None, event="cmd_exec", cmd=info.command_str, rc=info.rc, elapsed_ms=info.elapsed_ms)

    # Build the product
    env = os.environ.copy()
    env["RUSTFLAGS"] = env.get("RUSTFLAGS", "")
    build_cmd = ["cargo", "build", "--profile", profile]
    t0 = time.time()
    res = procmod.run(build_cmd, cwd=str(PROJECT_DIR), env=env, timeout=7200)
    logger.event(stage="build", suite=None, event="cmd_exec", cmd=res.command_str, rc=res.rc, elapsed_ms=res.elapsed_ms)
    if res.rc != 0:
        print("[build] ❌ cargo build failed")
        sys.exit(res.rc)

    # Record build metadata
    meta = {
        "profile": profile,
        "toolchain": toolchain,
        "duration_ms": int((time.time() - t0) * 1000),
    }
    (RESULTS_DIR / "build_meta.json").write_text(json.dumps(meta, indent=2))
    print("[build] ✅ cargo build succeeded")


def stage_run_suites(logger: logmod.JSONLLogger) -> list:
    print("[run_suites] starting…")
    test_filter = os.environ.get("TEST_FILTER", "")
    suites = suitesmod.discover_suites(str(TESTS_DIR), test_filter=test_filter)
    if not suites:
        print("[run_suites] No suites discovered; nothing to run")
        return []

    results = []
    coreutils_applets = fsmod.load_coreutils_applets(PROJECT_DIR)
    preserve = set(fsmod.PRESERVE_BINS)

    for idx, suite in enumerate(suites, 1):
        name = suite.name
        print(f"[run_suites] {idx}/{len(suites)} Running suite: {name}")
        suite_root = SNAPSHOTS_DIR / name
        suite_root.mkdir(parents=True, exist_ok=True)
        logs_root = LOGS_DIR / name
        logs_root.mkdir(parents=True, exist_ok=True)
        res_root = RESULTS_DIR / name
        res_root.mkdir(parents=True, exist_ok=True)

        selected_paths = fsmod.build_selected_paths(coreutils_applets)
        before_path = suite_root / "before.json"
        after_path = suite_root / "after.json"

        fsmod.snapshot(selected_paths, before_path)

        started = time.time()
        # Enforce fail-on-skip: incompatible suite is treated as failure
        if not suite.compatible:
            status = "fail"
            exec_res = suitesmod.ExecResult(rc=125, stdout="", stderr=f"suite {name} incompatible with this distro")
            duration_ms = 0
        else:
            exec_res = suitesmod.run_execute_block(suite, PROJECT_DIR, logger,
                                               stdout_path=logs_root / "execute.stdout.log",
                                               stderr_path=logs_root / "execute.stderr.log")
            duration_ms = int((time.time() - started) * 1000)
            status = "pass"

        # Evaluate expectation
        expect = (suite.expect or "pass").lower()
        if suite.compatible:
            if expect in ("fail", "xfail"):
                if exec_res.rc != 0:
                    status = "pass"  # expected failure occurred
                else:
                    status = "fail"
            else:
                status = "pass" if exec_res.rc == 0 else "fail"

        # Always snapshot after execute
        fsmod.snapshot(selected_paths, after_path)

        # Presence-aware assertions only if suite appears to enable coreutils
        acts_on_coreutils = suite.compatible and suitesmod.suite_touches_coreutils(suite)
        presence = None
        presence_ok = True
        if acts_on_coreutils:
            expect_symlink = suitesmod.suite_is_enable(suite) and not suitesmod.suite_is_disable(suite)
            presence, presence_ok = fsmod.assert_presence(coreutils_applets, preserve, logger,
                                                          suite_name=name, exec_output=(exec_res.stdout + "\n" + exec_res.stderr),
                                                          expect_symlink=expect_symlink)
            (res_root / "presence.json").write_text(json.dumps(presence, indent=2))
            if not presence_ok:
                status = "fail"

        # Always attempt restore; any failure makes suite FAIL
        restore_rc = 0
        if suite.restore:
            rest_res = suitesmod.run_restore_block(suite, PROJECT_DIR, logger,
                                                   stdout_path=logs_root / "restore.stdout.log",
                                                   stderr_path=logs_root / "restore.stderr.log")
            restore_rc = rest_res.rc
            if restore_rc != 0:
                status = "fail"

        result = {
            "name": name,
            "status": status,
            "duration_ms": duration_ms,
            "artifacts": [
                str(before_path), str(after_path), str(logs_root / "execute.stdout.log"), str(logs_root / "execute.stderr.log"),
                str(logs_root / "restore.stdout.log" if suite.restore else logs_root),
            ],
            "expect": expect,
            "rc": exec_res.rc,
            "restore_rc": restore_rc,
            "presence_ok": presence_ok,
        }
        results.append(result)

        print(f"[run_suites] {'✅ PASS' if status=='pass' else '❌ FAIL'} suite: {name}")

    # Persist results for later collect stage (if run separately)
    try:
        (TMP_DIR / "suites_results.json").write_text(json.dumps(results, indent=2))
    except Exception:
        pass
    return results


def stage_collect(logger: logmod.JSONLLogger, suites_results: list) -> None:
    print("[collect] starting…")
    # Copy audit log and runner JSONL under proofs
    try:
        audit_src = Path("/var/log/oxidizr-arch-audit.log")
        if audit_src.exists():
            (LOGS_DIR / "oxidizr-arch-audit.log").write_bytes(audit_src.read_bytes())
    except Exception:
        pass
    try:
        if JSONL_PATH.exists():
            (LOGS_DIR / "runner.jsonl").write_bytes(JSONL_PATH.read_bytes())
    except Exception:
        pass

    osr = read_os_release()
    started_at = os.environ.get("RUN_STARTED_AT")
    finished_at = datetime.now(timezone.utc).isoformat()

    summary = {
        "distro": osr.get("ID", "unknown"),
        "suites": suites_results,
        "started_at": started_at,
        "finished_at": finished_at,
        "container_id": get_container_id(),
        "harness_policy": "No harness mutation of product-owned artifacts; fail-on-skip enforced",
    }
    SUMMARY_PATH.write_text(json.dumps(summary, indent=2))
    print("[collect] ✅ summary written")

    # Package artifacts into a tar.gz for convenience
    try:
        tar_path = PROOF_ROOT / "proofs.tar.gz"
        with tarfile.open(tar_path, "w:gz") as tf:
            def _add_if_exists(path: Path, arcname: str):
                if path.exists():
                    tf.add(str(path), arcname=arcname)

            _add_if_exists(LOGS_DIR, "logs")
            _add_if_exists(SNAPSHOTS_DIR, "snapshots")
            _add_if_exists(RESULTS_DIR, "results")
            _add_if_exists(SUMMARY_PATH, "summary.json")
        print(f"[collect] 📦 packaged artifacts at {tar_path}")
    except Exception as e:
        print(f"[collect] warning: failed to create tar: {e}")


def main():
    ensure_dirs()
    # Envelope defaults
    osr = read_os_release()
    run_id = os.environ.get("RUN_ID")
    container_id = get_container_id()
    logger = logmod.JSONLLogger(JSONL_PATH, component="runner", run_id=run_id, container_id=container_id, distro=osr.get("ID"))

    parser = argparse.ArgumentParser(description="oxidizr-arch container runner v2")
    parser.add_argument("stage", choices=["all", "preflight", "deps", "build", "run-suites", "collect"], help="Stage to run")
    # Back-compat: host may pass a no-op token 'internal-runner'. Strip any occurrence.
    sys.argv = [arg for arg in sys.argv if arg != "internal-runner"]
    args = parser.parse_args()

    if args.stage == "preflight":
        stage_preflight(logger)
        return
    if args.stage == "deps":
        stage_deps(logger)
        return
    if args.stage == "build":
        stage_build(logger)
        return
    if args.stage == "run-suites":
        res = stage_run_suites(logger)
        # cache in env for collect
        os.environ["SUITES_RESULTS_JSON"] = json.dumps(res)
        # non-zero if any suite failed
        failed = any(r.get("status") == "fail" for r in res)
        sys.exit(1 if failed else 0)
    if args.stage == "collect":
        suites_results = []
        try:
            suites_results = json.loads(os.environ.get("SUITES_RESULTS_JSON", "[]"))
        except Exception:
            pass
        # Fallback to file if env var missing (separate process case)
        if not suites_results:
            try:
                suites_results = json.loads((TMP_DIR / "suites_results.json").read_text())
            except Exception:
                suites_results = []
        stage_collect(logger, suites_results)
        failed = any(r.get("status") == "fail" for r in suites_results)
        return 1 if failed else 0

    # all
    os.environ["RUN_STARTED_AT"] = datetime.now(timezone.utc).isoformat()
    stage_preflight(logger)
    stage_deps(logger)
    stage_build(logger)
    suites_results = stage_run_suites(logger)
    stage_collect(logger, suites_results)


if __name__ == "__main__":
    sys.exit(main() or 0)
