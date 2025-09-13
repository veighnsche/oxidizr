#!/usr/bin/env python3
"""
Switchyard Spec Traceability & Lint Tool

- Validates that all Gherkin steps match steps-contract.yaml
- Maps scenario tags to requirement IDs in requirements.yaml
- Reports coverage of MUST/MUST_NOT requirements
- Emits a markdown report at ../traceability.md
- Exits non-zero on:
  * unmatched step lines
  * uncovered MUST/MUST_NOT requirements

Usage:
  python3 tools/traceability.py
"""
from __future__ import annotations
import json
import os
import re
import sys
import textwrap
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Set

try:
    import yaml  # PyYAML
except Exception as e:
    print("ERROR: PyYAML is required to run this tool (pip install pyyaml)", file=sys.stderr)
    sys.exit(2)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # SPEC/
FEATURES_DIR = os.path.join(ROOT, 'features')
REQS_PATH = os.path.join(ROOT, 'requirements.yaml')
STEPS_CONTRACT_PATH = os.path.join(FEATURES_DIR, 'steps-contract.yaml')
AUDIT_SCHEMA_PATH = os.path.join(ROOT, 'audit_event.v2.schema.json')
PREFLIGHT_SCHEMA_PATH = os.path.join(ROOT, 'preflight.yaml')
REPORT_PATH = os.path.join(ROOT, 'traceability.md')

@dataclass
class StepPattern:
    kind: str  # Given | When | Then
    pattern: re.Pattern

@dataclass
class Scenario:
    feature: str
    name: str
    tags: Set[str] = field(default_factory=set)
    steps: List[Tuple[str, str]] = field(default_factory=list)  # (kind, text)

@dataclass
class Coverage:
    req_to_scenarios: Dict[str, List[str]] = field(default_factory=dict)

    def add(self, req: str, scenario: str) -> None:
        self.req_to_scenarios.setdefault(req, []).append(scenario)


def load_requirements(path: str) -> Dict[str, dict]:
    with open(path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    reqs = {}
    for entry in data.get('requirements', []):
        reqs[entry['id']] = entry
    return reqs


def load_step_contract(path: str) -> List[StepPattern]:
    with open(path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    patterns = []
    for entry in data.get('steps', []):
        kind = entry['kind']
        patt = re.compile(entry['pattern'])
        patterns.append(StepPattern(kind=kind, pattern=patt))
    return patterns


def parse_features(dir_path: str, step_contract: List[StepPattern]) -> Tuple[List[Scenario], List[str]]:
    errors: List[str] = []
    scenarios: List[Scenario] = []
    step_index: Dict[str, List[re.Pattern]] = {}
    for sp in step_contract:
        step_index.setdefault(sp.kind, []).append(sp.pattern)

    for fname in sorted(os.listdir(dir_path)):
        if not fname.endswith('.feature'):
            continue
        path = os.path.join(dir_path, fname)
        with open(path, 'r', encoding='utf-8') as f:
            lines = [ln.rstrip('\n') for ln in f]

        pending_tags: Set[str] = set()
        current_scenario: Optional[Scenario] = None
        last_kind: Optional[str] = None

        def flush_scenario():
            nonlocal current_scenario
            if current_scenario is not None:
                scenarios.append(current_scenario)
            current_scenario = None

        for ln in lines:
            striped = ln.strip()
            if striped.startswith('@'):
                # tags may be space-separated on one line
                parts = striped.split()
                for p in parts:
                    if p.startswith('@'):
                        pending_tags.add(p[1:])  # drop '@'
                continue
            if striped.lower().startswith('scenario:'):
                name = striped[len('scenario:'):].strip()
                flush_scenario()
                current_scenario = Scenario(feature=fname, name=name, tags=set(pending_tags))
                pending_tags.clear()
                last_kind = None
                continue
            if re.match(r'^(given|when|then|and|but)\b', striped, re.IGNORECASE):
                if current_scenario is None:
                    # step outside scenario
                    continue
                # Determine kind
                kind_word = striped.split()[0].capitalize()
                if kind_word in ('And', 'But'):
                    kind = last_kind or 'Then'
                else:
                    kind = kind_word
                    last_kind = kind
                # Validate against contract
                patterns = step_index.get(kind, [])
                matched = any(patt.match(striped) for patt in patterns)
                if not matched:
                    errors.append(f"{fname}: step not covered by contract: '{striped}' (kind={kind})")
                current_scenario.steps.append((kind, striped))
                continue
        flush_scenario()
    return scenarios, errors


def build_coverage(reqs: Dict[str, dict], scenarios: List[Scenario]) -> Coverage:
    cov = Coverage()
    for sc in scenarios:
        for tag in sc.tags:
            if tag in reqs:
                cov.add(tag, f"{sc.feature} :: {sc.name}")
    return cov


def write_report(reqs: Dict[str, dict], cov: Coverage, scenarios: List[Scenario], errors: List[str]) -> None:
    must_ids = [rid for rid, r in reqs.items() if str(r.get('level', '')).upper() in ('MUST','MUST_NOT')]
    lines = []
    lines.append('# Switchyard Spec Traceability Report')
    lines.append('')
    lines.append('## Summary')
    lines.append(f"Total requirements: {len(reqs)}")
    lines.append(f"MUST/MUST_NOT: {len(must_ids)}")
    covered_must = sum(1 for rid in must_ids if rid in cov.req_to_scenarios)
    lines.append(f"Covered MUST/MUST_NOT: {covered_must}")
    lines.append(f"Uncovered MUST/MUST_NOT: {len(must_ids) - covered_must}")
    lines.append('')

    if errors:
        lines.append('## Lint Errors')
        for e in errors:
            lines.append(f"- {e}")
        lines.append('')

    lines.append('## Uncovered MUST/MUST_NOT Requirements')
    any_uncovered = False
    for rid in sorted(must_ids):
        if rid not in cov.req_to_scenarios:
            any_uncovered = True
            r = reqs[rid]
            lines.append(f"- {rid} — {r.get('title','')} ({r.get('section','')})")
    if not any_uncovered:
        lines.append('- None')
    lines.append('')

    lines.append('## Coverage Matrix (Requirement → Scenarios)')
    for rid in sorted(reqs.keys()):
        scs = cov.req_to_scenarios.get(rid, [])
        title = reqs[rid].get('title','')
        lines.append(f"- {rid} — {title}")
        if scs:
            for sc in scs:
                lines.append(f"  - {sc}")
        else:
            lines.append("  - (no scenarios)")
    lines.append('')

    with open(REPORT_PATH, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines) + '\n')


def main() -> int:
    reqs = load_requirements(REQS_PATH)
    step_contract = load_step_contract(STEPS_CONTRACT_PATH)
    scenarios, lint_errors = parse_features(FEATURES_DIR, step_contract)
    cov = build_coverage(reqs, scenarios)
    write_report(reqs, cov, scenarios, lint_errors)

    # Determine exit status
    must_uncovered = [rid for rid, r in reqs.items() if str(r.get('level','')).upper() in ('MUST','MUST_NOT') and rid not in cov.req_to_scenarios]
    if lint_errors or must_uncovered:
        if must_uncovered:
            print("Uncovered MUST/MUST_NOT requirements:", ', '.join(sorted(must_uncovered)), file=sys.stderr)
        for e in lint_errors:
            print(e, file=sys.stderr)
        print(f"Traceability report written to {os.path.relpath(REPORT_PATH, ROOT)}", file=sys.stderr)
        return 1
    print(f"OK: All MUST/MUST_NOT covered by at least one scenario. Report at {os.path.relpath(REPORT_PATH, ROOT)}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
