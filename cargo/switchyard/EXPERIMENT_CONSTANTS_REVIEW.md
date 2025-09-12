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

Interpretation and classification

- Purpose: This is an allowlist of checksum utilities. In the context of experiments (e.g., coreutils/uutils swaps), these binaries should generally be preserved (not replaced) to avoid breaking package build tools and integrity checks during operations.
- Classification: Policy default, not a “true constant”.
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
