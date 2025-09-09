import os
import shlex
import subprocess
import time
from dataclasses import dataclass
from typing import Dict, List, Optional, Union


@dataclass
class ProcResult:
    command: List[str]
    command_str: str
    rc: int
    stdout: str
    stderr: str
    elapsed_ms: int


def _ensure_list(cmd: Union[str, List[str]]) -> List[str]:
    if isinstance(cmd, list):
        return cmd
    return shlex.split(cmd)


def run(cmd: Union[str, List[str]], *, env: Optional[Dict[str, str]] = None,
        cwd: Optional[str] = None, timeout: Optional[int] = None) -> ProcResult:
    """Run a command capturing stdout/stderr, returning rc and timings.

    - Does not raise on non-zero exit.
    - Does not modify environment unless provided.
    - Uses shell=False for safety when cmd is a list.
    """
    argv = _ensure_list(cmd)
    command_str = " ".join(shlex.quote(c) for c in argv)
    start = time.time()
    try:
        cp = subprocess.run(
            argv,
            cwd=cwd,
            env=env if env is not None else os.environ.copy(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
            text=True,
        )
        rc = cp.returncode
        out = cp.stdout or ""
        err = cp.stderr or ""
    except subprocess.TimeoutExpired as te:
        rc = 124  # conventional timeout code
        out = te.stdout.decode() if isinstance(te.stdout, (bytes, bytearray)) else (te.stdout or "")
        err = te.stderr.decode() if isinstance(te.stderr, (bytes, bytearray)) else (te.stderr or "")
    end = time.time()
    return ProcResult(
        command=argv,
        command_str=command_str,
        rc=rc,
        stdout=out,
        stderr=err,
        elapsed_ms=int((end - start) * 1000),
    )
