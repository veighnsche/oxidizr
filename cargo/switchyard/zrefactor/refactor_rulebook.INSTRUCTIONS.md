# Refactoring Rulebook — Mark-and-Sweep Conventions (developer workflow)

Goal: Make refactors predictable and easy to sweep. Any file or code slated for removal, move, or replacement must carry a standardized, grep-able marker. You will remove items manually in a final sweep PR.

Principles

- Keep markers textual and consistent across languages.
- Prefer top-of-file markers for whole-file actions.
- For non-commentable formats (JSON), use the Removals Registry.
- Each marker includes a short reason and, where relevant, the destination.

Versioning and breaking changes (pre-1.0)

- We are pre-1.0. Breaking changes are allowed and encouraged when they improve DX, safety, or clarity.
- Always tag breaking refactors explicitly so they are easy to review and document:
  - Add BREAKING to the marker text, for example:
    - Top-of-file: `/// remove this file — BREAKING: superseded by zrefactor/policy_refactor.INSTRUCTIONS.md`
    - Block: `/// BEGIN REMOVE BLOCK — BREAKING: API surface moved` … `/// END REMOVE BLOCK`
    - Registry: `remove: <path> — BREAKING: <reason>`
- Commit message guidance:
  - Use conventional commits and include a breaking indicator, e.g.:
    - `refactor(breaking): move src/api.rs to src/api/mod.rs`
    - Or include a `BREAKING CHANGE:` trailer in the commit body with a short migration snippet.
  - Include a “Migration:” paragraph with concrete steps for consumers.
- Changelog guidance:
  - Under “Unreleased”, add a “Breaking changes” subsection and list each item with a one-line migration hint.

Canonical markers (copy exactly)

- Whole-file removal (top-of-file):
  - Rust/Go/TS/MD: `/// remove this file — <reason or successor path>`
  - Shell/YAML/TOML: `# remove this file — <reason or successor path>`
  - C-like: `// remove this file — <reason or successor path>`
- File move (top-of-file):
  - `/// move this file -> <new/path> — <reason>`
- File replacement (top-of-file):
  - `/// replace this file with <new/path> — <reason>`
- Deprecated shim (top-of-file):
  - `/// deprecated shim — remove in <version>; use <new/path>`
- Block-level removal (inline section):
  - Begin: `/// BEGIN REMOVE BLOCK — <reason>`
  - End:   `/// END REMOVE BLOCK`

Non-commentable formats (JSON, .schema.json)

- Do NOT insert markers that break syntax.
- Instead, record the intent in the Removals Registry:
  - File: `zrefactor/removals_registry.md`
  - Entry format:
    - `remove: <repo-relative-path> — <reason or successor path>`
    - `move: <old-path> -> <new-path> — <reason>`
    - `replace: <old-path> -> <new-path> — <reason>`

Grep quick checks

- List all removals/moves/replacements in tree:

  ```bash
  rg -n "^\s*(///|//|#)\s*(remove this file|move this file|replace this file|deprecated shim|BEGIN REMOVE BLOCK)" -g '!**/target/**' -S
  ```

- Find registry-based removals:

  ```bash
  rg -n "^(remove|move|replace):" cargo/switchyard/zrefactor/removals_registry.md
  ```
- List all BREAKING markers:

  ```bash
  rg -n "\\bBREAKING\\b" cargo/switchyard -S
  ```

Suggested PR cadence

- PR A — Markers only:
  - Add top-of-file and block markers; update `removals_registry.md` for non-commentable files.
  - No behavioral changes. CI should remain green.
- PR B — Implement refactor:
  - Add new code/paths per instructions; keep markers in place.
- PR C — Sweep removals:
  - Manually delete all `remove this file` items; resolve moves/replacements.
  - Remove any remaining `BEGIN/END REMOVE BLOCK` sections.

Acceptance criteria per refactor

- Files slated for deletion have a top-of-file `remove this file` marker (or a registry entry).
- Files slated for move/replacement carry explicit `move this file`/`replace this file` markers.
- Deprecated compatibility surfaces carry `deprecated shim` markers with target version and successor path.
- A final sweep PR includes a commit message: `refactor(sweep): delete files marked with 'remove this file'`.

Examples in this repo (as reference)

- `cargo/switchyard/zrefactor/gating_ownership.md`:

  ```text
  /// remove this file — decision is implemented by `zrefactor/preflight_gating_refactor.INSTRUCTIONS.md` and `zrefactor/policy_refactor.INSTRUCTIONS.md`
  ```

- Narrative docs superseded by INSTRUCTIONS:

  ```text
  /// remove this file — superseded by `zrefactor/policy_refactor.INSTRUCTIONS.md`
  ```

- Backwards-compat doc guidance (inline):

  ```text
  /// remove this re-export: `src/lib.rs` top-level `pub use policy::rescue`
  ```

Optional helper: Removal checklist generator

- One-liner to produce a removal checklist:

  ```bash
  rg -n "^\s*(///|//|#)\s*remove this file" -S | sed 's/:.*//' | sort -u > /tmp/remove_list.txt
  echo "Registry entries:" >> /tmp/remove_list.txt
  rg -n "^remove:" cargo/switchyard/zrefactor/removals_registry.md -S | cut -d':' -f3- >> /tmp/remove_list.txt
  cat /tmp/remove_list.txt
  ```

FAQ

- Why not delete immediately? Keeping markers visible until the sweep PR makes reviews easier and reduces churn while related code is still moving.
- Why `///`? It renders as a doc comment in Rust and is a clear, grep-able convention. For other languages, `//` or `#` variants are accepted; the substring `remove this file` is the stable key.
