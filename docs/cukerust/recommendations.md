# VSCode Extension Recommendations (Impact vs Effort)

This repo uses cucumber-rs with multiple step crates and runs via `cargo test` harness=false test binaries. We recommend shipping features in tiers, guided by the static artifacts we generated under `docs/cukerust/`.

Artifacts for the extension to consume:
- `docs/cukerust/step_index.json` (complete step index; includes kind, file, line, function, normalized `regex`, `captures`, `tags`, `notes`, and global `stats`).
- `docs/cukerust/tags.json` (all feature `unique_tags`, tag `counts`, and the list of `feature_files`).
- `docs/cukerust/survey.json` (runners, env overrides, CI jobs, pain points, quick wins).

## Tier 0 (MVP)
- **Gherkin language support**
  - Syntax highlighting for `.feature` files; lightweight tree for `Feature/Scenario/Step/Examples/Tags`.
- **Static diagnostics (undefined/ambiguous)**
  - Use `step_index.json` to verify each step text maps to at least one step.
  - Detect ambiguous matches (same kind + regex matches multiple steps). Surface inline diagnostics.
- **Basic run commands (CodeLens + Commands palette)**
  - Run current feature file by setting the crate-specific env var and invoking `cargo test` (see `run_matrix.md`).
  - Show output terminal; surface failures by piping through `scripts/bdd_filter_results.py` for switchyard.
- **Step hover**
  - On hover over a step, show the matched pattern, captures, and `file:line` of the definition.

## Tier 1 (Navigation & Authoring)
- **Jump-to-definition**
  - From a `.feature` step line to its Rust function using `file` + `line` in `step_index.json`.
- **Completion for steps**
  - Offer completion items from the `regex` patterns. Show the step kind as an icon/label.
  - Insert text should be a readable exemplar (e.g., de-regex with placeholders for captures).
- **Refined hovers**
  - Include `notes` and function signature when available; show module path from `file`.

## Tier 2 (Quality of Life)
- **Coverage (unused steps)**
  - Static: Steps present in `step_index.json` but not referenced by any feature line -> “unused” list.
- **CodeLens**
  - “Run Feature” / “Run Directory” lenses on feature files and folders.
  - “Show Tag Usage” lens that opens a quick pick from `tags.json`.
- **Log integration**
  - Button to open the last run logs; in CI, fetch `target/bdd-lastrun.log` artifact when available.
- **Live decorations (optional, blocked by runner)**
  - If/when NDJSON (Cucumber Messages) is enabled in test entrypoints, stream results to decorate steps in real time.

## Tier 3 (Advanced)
- **Quick-fix: create step skeleton (scaffold)**
  - Offer a dialog that copies a Rust step template to the clipboard (non-destructive suggestion), pre-filled with a regex and function name derived from the step text. (Leave the actual file creation to the user.)
- **Tag-aware hooks UI**
  - Surface tag usage with counts; provide quick filters to run selected tags (once the runner supports it).
- **Workspace multi-root & multi-crate**
  - Support multiple step crates (switchyard, oxidizr-arch, oxidizr-deb, oxidizr-cli-core) in one workspace.

## Repo-specific extras
- **Env-driven feature selection**
  - Use `SWITCHYARD_BDD_FEATURE_PATH`, `OXIDIZR_ARCH_BDD_FEATURE_PATH`, `OXIDIZR_DEB_BDD_FEATURE_PATH` to run subsets by path (see `survey.json` and `run_matrix.md`).
- **SPEC tag browser**
  - Tags include many `@REQ-*` items. Provide a tag view summarizing requirements coverage via `tags.json`.
- **Fail-on-skipped awareness**
  - The runners use `.fail_on_skipped()`. Add a preflight check that highlights undefined steps that would fail.
- **Ambiguity awareness**
  - Use `stats.ambiguous` and per-step pattern matching to surface ambiguity in Problems.

## Implementation notes
- **Index refresh**
  - The extension can watch `docs/cukerust/step_index.json` and `docs/cukerust/tags.json` for changes and refresh providers.
- **Pattern matching**
  - Match step text to `regex` from the index, scoped by step kind (Given/When/Then). Use Rust-style regex semantics.
- **CI awareness**
  - Surface links to relevant CI jobs (`.github/workflows/ci.yml`), especially the `bdd` gate that uploads `bdd-lastrun.log`.

## Known blockers and limitations
- **No NDJSON yet**
  - Test entrypoints do not emit Cucumber Messages NDJSON, so live decorations cannot be implemented without harness changes. See `survey.json` → `blocked_reasons`.
- **Name/Tag/Line filters**
  - Current entrypoints only support selection by path via env vars; running by scenario name/tags/line is not wired yet.
