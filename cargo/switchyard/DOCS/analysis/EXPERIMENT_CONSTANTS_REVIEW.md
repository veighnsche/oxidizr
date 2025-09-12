# Review: experiments/constants.rs — constants vs policy

Generated: 2025-09-12

Source reviewed:

- `cargo/oxidizr-arch/src/experiments/constants.rs`

Contents found

- `CHECKSUM_BINS: &[&str]`

  ```rust
  pub const CHECKSUM_BINS: &[&str] = &[
      "b2sum",
      "md5sum",
      "sha1sum",
      "sha224sum",
      "sha256sum",
      "sha384sum",
      "sha512sum",
  ];
  ```

**Verified Claims:**
- The `CHECKSUM_BINS` constant is indeed located in `cargo/oxidizr-arch/src/experiments/constants.rs`.
- This constant is used as an allowlist of checksum utilities that should generally be preserved during experiments.
- The list includes all common checksum utilities: b2sum, md5sum, sha1sum, sha224sum, sha256sum, sha384sum, sha512sum.

**Verified Implementation:**
- The constant is correctly defined as a slice of strings containing checksum binary names.
- The purpose aligns with the documented intent to preserve these binaries during swaps.
- The classification as a policy default rather than a true constant is appropriate since different environments may require different checksum utilities.

**Citations:**
- `cargo/oxidizr-arch/src/experiments/constants.rs:L1-L9` - CHECKSUM_BINS definition

Interpretation and classification

- Purpose: This is an allowlist of checksum utilities. In the context of experiments (e.g., coreutils/uutils swaps), these binaries should generally be preserved (not replaced) to avoid breaking package build tools and integrity checks during operations.
- Classification: Policy default, not a "true constant".
  - True constants should be values that do not depend on deployment, distro, or user goals (e.g., schema version strings, stable names of sidecar keys, file suffixes).
  - This allowlist is an operational choice (policy) and may vary by environment, distro, or user preference. It should be configurable.

Recommendations

1) Treat `CHECKSUM_BINS` as a policy default rather than a hard constant

- Keep the list as a default, but allow overriding it via configuration/policy.
- Do not hardwire such lists into the Switchyard library core; keep them in the experiments or policy layer.

2) Expose a policy field for preservation allowlists

- Add a field in policy/config to carry a list of binaries to preserve during swaps, for example:
  - `policy.preserve_bins: Vec<String>` (or a more specific name if scoped to a particular experiment).
- Default value can be the current `CHECKSUM_BINS` sequence.
- Consumers (experiments) can override per plan or per run.

3) Keep experiment-specific defaults in the experiments crate

- Since the source lives in `oxidizr-arch` (experiments), keep the default list there.
- Switchyard should not depend on or embed experiment-specific sets; it should accept policy input from the caller.

4) Align naming with intent

- Optionally rename to make the intent explicit:
  - From: `CHECKSUM_BINS`
  - To: `DEFAULT_PRESERVE_CHECKSUM_BINS`
- This reduces the appearance of a universal constant and communicates configurability.

5) Document the rationale

- In the experiments crate, document why these bins are preserved (build systems and integrity checks rely on stock checksum tools; replacing them during install/build phases can cause breakages).

What should be constants vs policy (general guidance)

- True constants (keep as `const` or `static`):
  - Schema version strings (e.g., `"backup_meta.v1"`).
  - Stable, non-configurable file suffixes (e.g., sidecar `.meta.json`).
  - Fixed keys for structured logs/JSONL fields, unless a versioning scheme is expected.

- Policy defaults / configuration (avoid hard `const`; put in Policy/Config):
  - Allow/deny lists (e.g., preserve/forbid binaries, paths, or extensions).
  - Operational thresholds (timeouts, retry counts, fsync warn thresholds).
  - Environment/distro-specific choices (mount points, helper tool availability, locale requirements).

- Runtime-derived values (never constants):
  - Timestamps, temporary paths, per-run IDs, resolved absolute paths.

Suggested implementation sketch

- In Switchyard policy (if you want the library to carry the knob):
  - Add `preserve_bins: Vec<String>` with a sensible empty default; let the experiments crate populate this list when constructing the `Policy`.
- In the experiments crate:
  - Keep `DEFAULT_PRESERVE_CHECKSUM_BINS` as a slice or vector.
  - When building the `Policy`, assign `policy.preserve_bins = DEFAULT_PRESERVE_CHECKSUM_BINS.iter().map(|s| s.to_string()).collect();`
  - Allow users to override via CLI/config.

Traceability and next steps

- If desired, add this item to `idiomatic_todo.md` under a new section (Policy surfacing) to track the field introduction and migration of any call sites relying on a hard-coded list.

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:23 CEST)

- **Invariant:** Checksum binary preservation across environments
- **Assumption (from doc):** The `CHECKSUM_BINS` list represents universal utilities that should be preserved in all deployment environments
- **Reality (evidence):** Hardcoded list at `cargo/oxidizr-arch/src/experiments/constants.rs:L1-L9` includes standard Unix utilities (b2sum, md5sum, sha*sum); however, availability varies by distribution (Alpine Linux may lack some, embedded systems may have different toolsets)
- **Gap:** One-size-fits-all preservation list doesn't account for environment-specific toolchain differences; consumers may encounter missing binaries or need different preservation policies
- **Mitigations:** Make preservation list configurable via Policy fields; provide environment detection to adjust default lists; document per-distro considerations
- **Impacted users:** CLI tools targeting multiple distributions and embedded system integrators
- **Follow-ups:** Implement configurable preserve_bins policy field; add distribution-aware defaults

- **Invariant:** Configuration-vs-constant classification consistency
- **Assumption (from doc):** Clear distinction between true constants and policy defaults guides implementers on what should be configurable
- **Reality (evidence):** Document correctly identifies `CHECKSUM_BINS` as policy default rather than true constant; however, no enforcement mechanism prevents hardcoding of operational choices in Switchyard core
- **Gap:** Without architectural boundaries, future constants might inappropriately hardcode environment-specific choices
- **Mitigations:** Establish coding standards that require policy parameters for environment-dependent values; add linting rules to detect hardcoded operational lists
- **Impacted users:** Library maintainers and contributors who need clear guidance on constant vs configuration boundaries
- **Follow-ups:** Add architectural decision record (ADR) for constant vs policy classification; implement lint rules

- **Invariant:** Experiment-specific constants remain in experiments crate
- **Assumption (from doc):** Experiment-specific defaults should not leak into Switchyard core library
- **Reality (evidence):** `CHECKSUM_BINS` correctly lives in `oxidizr-arch` experiments crate at `src/experiments/constants.rs`; Switchyard accepts policy input from caller
- **Gap:** No formal interface contract prevents experiments from bypassing policy system; dependency boundaries could be violated in future
- **Mitigations:** Define clear API contract for experiment-to-library communication; use type system to enforce policy-based configuration
- **Impacted users:** Experiment developers and library integrators maintaining clean separation of concerns
- **Follow-ups:** Document experiment-library interface contract; add integration tests verifying boundary enforcement

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:23 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: Make preserve list a policy knob instead of hard constant
  - Category: Policy/Default Mismatch
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Hardcoded preservation lists reduce portability across distros; exposing a policy knob enables environment-specific behavior with minimal code churn.
  - Evidence: `cargo/oxidizr-arch/src/experiments/constants.rs:L1-L23` defines `CHECKSUM_BINS`; recommendation already suggests `policy.preserve_bins`.
  - Next step: Add `preserve_bins: Vec<String>` to `src/policy/config.rs`; thread into apply-stage where replacements are decided; default from experiments crate.

- Title: Guardrails to prevent policy-internals from hardcoding environment choices in core
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: A documented rule and ADR reduce future drift of env-dependent lists into core.
  - Evidence: This doc’s guidance; no ADR/lints currently enforce separation.
  - Next step: Add ADR on constants vs policy classification; update `CODING_STANDARDS.md` with a checklist; consider a simple lint/CI grep for suspicious hardcoded allowlists.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
