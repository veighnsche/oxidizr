import json
import os
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional

import yaml

from . import proc as procmod


@dataclass
class Suite:
    name: str
    path: Path
    summary: str
    execute: str
    restore: Optional[str]
    distro_check: List[str]
    expect: Optional[str]
    compatible: bool


@dataclass
class ExecResult:
    rc: int
    stdout: str
    stderr: str


def _read_os_release_id() -> str:
    try:
        txt = Path("/etc/os-release").read_text()
        for line in txt.splitlines():
            if line.startswith("ID="):
                return line.split("=", 1)[1].strip().strip('"').lower()
    except Exception:
        pass
    return "unknown"


def _parse_suite(task_path: Path, current_id: str) -> Suite:
    data = yaml.safe_load(task_path.read_text()) or {}
    name = task_path.parent.name
    summary = data.get("summary", "")
    execute = data.get("execute", "")
    restore = data.get("restore")
    distro_check = [str(x).lower() for x in (data.get("distro-check") or [])]
    expect = data.get("expect")
    compatible = (not distro_check) or (current_id in distro_check)
    return Suite(
        name=name,
        path=task_path,
        summary=summary,
        execute=execute,
        restore=restore,
        distro_check=distro_check,
        expect=expect,
        compatible=compatible,
    )


def discover_suites(tests_root: str, *, test_filter: str = "") -> List[Suite]:
    root = Path(tests_root)
    found: List[Path] = []
    for p in sorted(root.rglob("task.yaml")):
        found.append(p)
    if test_filter:
        found = [p for p in found if p.parent.name == test_filter]
        if not found:
            raise RuntimeError(f"test filter '{test_filter}' did not match any discovered suites")

    current_id = _read_os_release_id()
    suites = [_parse_suite(p, current_id) for p in found]
    return suites


def _run_script(script: str, workdir: Path, *, stdout_path: Path, stderr_path: Path,
                product_stdout_path: Path, product_stderr_path: Path) -> ExecResult:
    # Write temp script with minimal prelude, no functions/traps
    with tempfile.NamedTemporaryFile("w", delete=False, prefix="suite-", suffix=".sh") as tf:
        tf.write("#!/usr/bin/env bash\nset -euo pipefail\n\n")
        # Provide a wrapper for oxidizr-arch that tees raw stdout/stderr to top-level product logs
        tf.write(f'PRODUCT_STDOUT="{product_stdout_path}"\n')
        tf.write(f'PRODUCT_STDERR="{product_stderr_path}"\n')
        tf.write('touch "${PRODUCT_STDOUT}" "${PRODUCT_STDERR}"\n')
        tf.write('oxidizr-arch() { command oxidizr-arch "$@" 1>>"${PRODUCT_STDOUT}" 2>>"${PRODUCT_STDERR}"; }\n\n')
        tf.write(script)
        tmp_path = Path(tf.name)
    try:
        os.chmod(tmp_path, 0o700)
    except Exception:
        pass

    res = procmod.run(["bash", str(tmp_path)], cwd=str(workdir), timeout=int(os.environ.get("SUITE_TIMEOUT_SEC", "900")))
    try:
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_text(res.stdout)
        stderr_path.write_text(res.stderr)
    except Exception:
        pass
    try:
        tmp_path.unlink()
    except Exception:
        pass
    return ExecResult(rc=res.rc, stdout=res.stdout, stderr=res.stderr)


def run_execute_block(suite: Suite, project_dir: Path, logger, *, stdout_path: Path, stderr_path: Path,
                      product_stdout_path: Path, product_stderr_path: Path) -> ExecResult:
    logger.event(stage="run_suites", suite=suite.name, event="exec_start", msg=f"suite={suite.name}")
    res = _run_script(suite.execute, project_dir, stdout_path=stdout_path, stderr_path=stderr_path,
                      product_stdout_path=product_stdout_path, product_stderr_path=product_stderr_path)
    logger.event(stage="run_suites", suite=suite.name, event="exec_done", rc=res.rc)
    return res


def run_restore_block(suite: Suite, project_dir: Path, logger, *, stdout_path: Path, stderr_path: Path,
                      product_stdout_path: Path, product_stderr_path: Path) -> ExecResult:
    logger.event(stage="restore", suite=suite.name, event="restore_start")
    res = _run_script(suite.restore or "true", project_dir, stdout_path=stdout_path, stderr_path=stderr_path,
                      product_stdout_path=product_stdout_path, product_stderr_path=product_stderr_path)
    logger.event(stage="restore", suite=suite.name, event="restore_done", rc=res.rc)
    return res


def suite_touches_coreutils(suite: Suite) -> bool:
    exe = (suite.execute or "").lower()
    # Heuristic: if suite enables or disables or mentions coreutils explicitly
    if "coreutils" in exe:
        return True
    if "oxidizr-arch" in exe and ("enable" in exe or "disable" in exe):
        return True
    return False


def suite_is_enable(suite: Suite) -> bool:
    exe = (suite.execute or "").lower()
    return ("oxidizr-arch" in exe) and (" enable" in exe)


def suite_is_disable(suite: Suite) -> bool:
    exe = (suite.execute or "").lower()
    return ("oxidizr-arch" in exe) and (" disable" in exe)
