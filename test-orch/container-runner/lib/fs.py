import json
import os
from pathlib import Path
from typing import Dict, Iterable, List, Set, Tuple

# Align with product-side PRESERVE_BINS used during linking
PRESERVE_BINS: Tuple[str, ...] = (
    "b2sum",
    "md5sum",
    "sha1sum",
    "sha224sum",
    "sha256sum",
    "sha384sum",
    "sha512sum",
)

COREUTILS_CANDIDATE_DIRS = [
    Path("/usr/lib/uutils/coreutils"),
    Path("/usr/lib/cargo/bin/coreutils"),
    Path("/usr/lib/cargo/bin"),
]


def load_coreutils_applets(project_dir: Path) -> List[str]:
    # tests/lib/rust-coreutils-bins.txt
    p = project_dir / "tests" / "lib" / "rust-coreutils-bins.txt"
    out: List[str] = []
    try:
        for line in p.read_text().splitlines():
            line = line.strip()
            if not line:
                continue
            out.append(line)
    except Exception:
        pass
    return out


def build_selected_paths(coreutils_applets: Iterable[str]) -> List[Path]:
    paths: Set[Path] = set()
    for name in coreutils_applets:
        paths.add(Path("/usr/bin") / name)
    # Add common ones from other experiments for context
    for extra in ["sudo", "find", "xargs"]:
        paths.add(Path("/usr/bin") / extra)
    return sorted(paths)


def _stat_entry(p: Path) -> Dict:
    info = {
        "path": str(p),
        "exists": p.exists() or p.is_symlink(),
        "is_symlink": p.is_symlink(),
        "link_target": None,
        "mode": None,
        "uid": None,
        "gid": None,
    }
    try:
        if info["is_symlink"]:
            info["link_target"] = os.readlink(p)
        st = p.lstat() if info["is_symlink"] else p.stat()
        info["mode"] = oct(st.st_mode & 0o777)
        info["uid"] = st.st_uid
        info["gid"] = st.st_gid
    except Exception:
        pass
    return info


def snapshot(paths: Iterable[Path], out_path: Path) -> None:
    data = [_stat_entry(Path(p)) for p in paths]
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(data, indent=2))


def _uu_present(name: str) -> bool:
    candidates = [
        Path(f"/usr/bin/uu-{name}"),
    ] + [d / name for d in COREUTILS_CANDIDATE_DIRS]
    for c in candidates:
        try:
            if c.exists():
                return True
        except Exception:
            continue
    return False


def assert_presence(coreutils_applets: Iterable[str], preserve: Set[str], logger,
                    *, suite_name: str, exec_output: str, expect_symlink: bool = True) -> Tuple[Dict, bool]:
    """Presence-aware assertions:
    - For each applet that appears present (uu-<name> or candidate dir file exists) and not preserved,
      assert that /usr/bin/<name> is a symlink.
    - For missing applets, expect product to have emitted a WARN mentioning the applet name.
    Returns (report_json, ok).
    """
    report = {
        "applets": [],
        "failures": [],
    }
    ok = True

    for name in sorted(set(coreutils_applets)):
        entry = {"name": name}
        present = _uu_present(name)
        entry["uu_present"] = present
        target = Path("/usr/bin") / name
        entry["target"] = str(target)
        entry["is_symlink"] = target.is_symlink()
        report["applets"].append(entry)

        if name in preserve:
            # No requirement imposed on preserved bins
            continue

        if present:
            if expect_symlink:
                if not target.is_symlink():
                    msg = f"expected symlink for {name} at {target}, but not a symlink"
                    report["failures"].append(msg)
                    ok = False
                    logger.event(stage="run_suites", suite=suite_name, level="error", event="assert_fail", msg=msg)
            else:
                if target.is_symlink():
                    msg = f"expected no symlink for {name} at {target} after disable, but found symlink"
                    report["failures"].append(msg)
                    ok = False
                    logger.event(stage="run_suites", suite=suite_name, level="error", event="assert_fail", msg=msg)
        else:
            if expect_symlink:
                # Look for WARN in exec output mentioning this applet
                low = exec_output.lower()
                if ("warn" not in low) or (name.lower() not in low):
                    msg = f"missing applet {name} without corresponding product WARN in logs"
                    report["failures"].append(msg)
                    ok = False
                    logger.event(stage="run_suites", suite=suite_name, level="error", event="assert_fail", msg=msg)

    if ok:
        logger.event(stage="run_suites", suite=suite_name, level="info", event="assert_pass", msg="presence assertions passed")
    return report, ok
