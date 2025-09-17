#!/usr/bin/env python3
import os
import re
import json
import sys
import datetime
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DOCS_DIR = REPO_ROOT / "docs" / "cukerust"

# Regexes for parsing step macros and functions
macro_head = re.compile(r"^\s*#\[\s*(given|when|then)\b", re.IGNORECASE)
fn_decl = re.compile(r"^\s*pub\s+async\s+fn\s+([A-Za-z0-9_]+)")
# Generic string literal matcher supporting raw strings r"..", r#".."#, and normal ".."
pat_any_str = r'(r#+?".*?"#+?|r"(?:[^"\\]|\\.)*"|"(?:[^"\\]|\\.)*")'
regex_attr = re.compile(r'regex\s*=\s*' + pat_any_str)
paren_lit = re.compile(r'\(\s*' + pat_any_str + r'\s*\)')

def unquote_pattern(s: str | None) -> str | None:
    if not isinstance(s, str):
        return None
    # raw string with optional hashes r###"..."###
    m = re.match(r'^r(#+)?"([\s\S]*)"\1$', s)
    if m:
        return m.group(2)
    m = re.match(r'^"([\s\S]*)"$', s)
    if m:
        return m.group(1)
    return s

def capture_names(regex_text: str) -> list[str]:
    if not isinstance(regex_text, str):
        return []
    names = re.findall(r"\(\?P<([A-Za-z_][A-Za-z0-9_]*)>", regex_text)
    if names:
        return names
    # Fallback: count unnamed capturing groups, excluding non-capturing (?: ...)
    count = 0
    i = 0
    while i < len(regex_text):
        if regex_text[i] == '(':
            # lookahead for "(?" forms
            if i + 1 < len(regex_text) and regex_text[i+1] == '?':
                # non-capturing or lookaround or named; skip
                i += 1
            else:
                count += 1
        i += 1
    return [str(n) for n in range(1, count + 1)]

def find_steps_roots() -> list[Path]:
    roots: list[Path] = []
    for p in REPO_ROOT.rglob("steps"):
        if p.name != "steps":
            continue
        # Only consider tests/steps directories inside crates
        if p.parent.name == "tests":
            roots.append(p)
    return sorted(roots)


def _parse_mod_modules(mod_file: Path) -> list[str]:
    """Parse `mod.rs`-style files and return declared child module names."""
    try:
        text = mod_file.read_text(encoding="utf-8")
    except Exception:
        return []
    mods: list[str] = []
    mod_decl = re.compile(r"^[\t\s]*(?:pub\s+)?mod\s+([A-Za-z0-9_]+)\s*;", re.M)
    for m in mod_decl.finditer(text):
        mods.append(m.group(1))
    return mods


def _walk_mod_tree(dir_path: Path, collected: set[Path]) -> None:
    """Collect .rs files reachable from a directory with a mod.rs, recursively."""
    mod_rs = dir_path / "mod.rs"
    if not mod_rs.exists():
        return
    collected.add(mod_rs)
    for name in _parse_mod_modules(mod_rs):
        file_rs = dir_path / f"{name}.rs"
        sub_dir = dir_path / name
        if file_rs.exists():
            collected.add(file_rs)
        if (sub_dir / "mod.rs").exists():
            collected.add(sub_dir / "mod.rs")
            _walk_mod_tree(sub_dir, collected)


def collect_step_files() -> list[Path]:
    """Restrict to files actually compiled by following steps/mod.rs graphs."""
    out: set[Path] = set()
    for root in find_steps_roots():
        # Top-level steps/mod.rs is authoritative
        top_mod = root / "mod.rs"
        if top_mod.exists():
            out.add(top_mod)
            # Include any direct child modules declared in steps/mod.rs
            for name in _parse_mod_modules(top_mod):
                file_rs = root / f"{name}.rs"
                sub_dir = root / name
                if file_rs.exists():
                    out.add(file_rs)
                if (sub_dir / "mod.rs").exists():
                    _walk_mod_tree(sub_dir, out)
        else:
            # Fallback: include all .rs under steps/ if no mod.rs (unlikely in this repo)
            for p in root.rglob("*.rs"):
                out.add(p)
    # Filter out any accidental target/.git paths (defense-in-depth)
    out = {p for p in out if not any(seg in {"target", ".git"} for seg in p.parts)}
    return sorted(out)


def parse_steps(step_files: list[Path]) -> list[dict]:
    entries: list[dict] = []
    for f in step_files:
        try:
            src = f.read_text(encoding="utf-8").splitlines()
        except Exception as e:
            print(f"WARN: failed to read {f}: {e}", file=sys.stderr)
            continue
        for i, line in enumerate(src):
            m = macro_head.match(line)
            if not m:
                continue
            kind = m.group(1).capitalize()
            # Flatten a small window of following lines to capture multi-line attributes
            window = [l.rstrip() for l in src[i:i + 8]]
            attr_blob = " ".join(window)
            attr_line = window[0].strip()
            pattern_src = None
            # Prefer explicit regex= attribute (search in flattened blob)
            mr = regex_attr.search(attr_blob)
            if mr:
                pattern_src = mr.group(1).strip()
            else:
                # Fallback: literal in parens #[then("...")] in flattened blob
                ml = paren_lit.search(attr_blob)
                if ml:
                    pattern_src = ml.group(1).strip()
            func = None
            for j in range(i + 1, min(i + 12, len(src))):
                mf = fn_decl.match(src[j])
                if mf:
                    func = mf.group(1)
                    break
            regex_norm = unquote_pattern(pattern_src)
            entries.append({
                "kind": kind,
                "file": str(f.relative_to(REPO_ROOT).as_posix()),
                "line": i + 1,
                "function": func,
                "attr": attr_line,
                "pattern_src": pattern_src,
                "regex": regex_norm,
                "tags": [kind],
                "captures": capture_names(regex_norm) if regex_norm else [],
                "notes": f"attribute: {attr_line}",
            })
    return entries


def scan_feature_tags() -> dict:
    # Find all .feature files and count tags (tokens beginning with @)
    tag_counts: dict[str, int] = {}
    paths: list[str] = []
    for p in REPO_ROOT.rglob("*.feature"):
        if any(seg in {"target", ".git"} for seg in p.parts):
            continue
        paths.append(str(p.relative_to(REPO_ROOT).as_posix()))
        try:
            for line in p.read_text(encoding="utf-8").splitlines():
                s = line.strip()
                if not s.startswith("@"):
                    continue
                for tok in s.split():
                    if tok.startswith("@") and len(tok) > 1:
                        tag_counts[tok] = tag_counts.get(tok, 0) + 1
        except Exception as e:
            print(f"WARN: failed reading {p}: {e}", file=sys.stderr)
    return {
        "feature_files": paths,
        "unique_tags": sorted(tag_counts.keys()),
        "counts": dict(sorted(tag_counts.items(), key=lambda kv: (-kv[1], kv[0]))),
    }


def main() -> int:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)

    step_files = collect_step_files()
    steps = parse_steps(step_files)

    # Compute stats
    by_kind: dict[str, int] = {"Given": 0, "When": 0, "Then": 0}
    for s in steps:
        k = s.get("kind")
        if k:
            by_kind[k] = by_kind.get(k, 0) + 1
    # detect ambiguous: same kind + regex pair appears >1
    seen: dict[tuple[str, str], int] = {}
    for s in steps:
        k = s.get("kind")
        r = s.get("regex")
        if not (k and r):
            continue
        key = (k, r)
        seen[key] = seen.get(key, 0) + 1
    ambiguous = sum(1 for v in seen.values() if v > 1)

    payload = {
        "generated_at": datetime.datetime.utcnow().isoformat() + "Z",
        "repo_root": REPO_ROOT.name,
        "step_files_scanned": [str(p.relative_to(REPO_ROOT).as_posix()) for p in step_files],
        "steps": steps,
        "stats": {
            "total": len(steps),
            "by_kind": by_kind,
            "ambiguous": ambiguous,
        },
    }
    (DOCS_DIR / "step_index.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    tags = scan_feature_tags()
    (DOCS_DIR / "tags.json").write_text(
        json.dumps({"generated_at": datetime.datetime.utcnow().isoformat() + "Z", **tags}, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    print(f"Wrote {len(steps)} steps from {len(step_files)} files to {DOCS_DIR/ 'step_index.json'}", file=sys.stderr)
    print(f"Wrote tag index with {len(tags['unique_tags'])} unique tags to {DOCS_DIR / 'tags.json'}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
