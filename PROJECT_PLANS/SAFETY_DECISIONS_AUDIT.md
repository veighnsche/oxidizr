# Safety Audit Decisions (Adopted)

These decisions are aligned to `AUDIT_CHECKLIST.md` and grounded in the current codebase.

## 1) xattrs / ACLs / Capabilities — Selective Security Policy

- Minimal guarantee for all targets:
  - Preserve uid/gid/mode and timestamps for regular files (see `src/symlink/ops.rs::replace_file_with_symlink` where permissions are preserved; extend to timestamps).
  - For symlinks, preserve linkness + target only (no fake metadata on links).
- Capabilities:
  - Detect and preserve `security.capability` for managed regular executables.
  - Implementation: add `src/system/security.rs` with `get_capabilities(path) -> Option<String>` and `set_capabilities(path, caps: &str)` using `getcap`/`setcap` or libcap if available. Hook into post-flip validation.
- SELinux/AppArmor labels:
  - Detect if labels are active; if active, attempt relabel on the active profile tree post-flip.
  - Implementation: `labels_active() -> bool` and `restore_labels(profile_root)` (e.g., `restorecon -R`), best-effort with audit logging.
- ACLs:
  - Detect & warn pre/post; provide remediation command in logs.
  - Optional preservation behind `--preserve-acl`.

Acceptance:

- Flip does not drop `security.capability` when present (verified in smoke tests).
- If labels are active, relabel step is attempted and logged.
- Symlink metadata is not synthesized.

## 2) Initramfs / Emergency Environment — Out of Scope + Docs

- Tool does not modify initramfs; flips are userland-only (`/usr/bin` points to profile `.../active/bin/*`).
- Provide operator playbook with:
  - Escape PATH: `export PATH=/usr/lib/oxidizr-arch/profiles/gnu/bin:$PATH`
  - One-step rollback (pointer flip): `oxidizr-arch profile --set gnu` (or `promote` alias if added).
- Optional guidance: how to pin a small rescue set into mkinitcpio/Dracut (docs only; no code in repo).

Acceptance:

- Clear docs state non-impact on initramfs and provide recovery one-liners.

## 3) Binary Signature Verification & SBOM — Op-Scoped Attestations

- Keep pacman repo signature trust; add local attestations:
  - Per-op JSONL bundle and detached Ed25519 signature.
  - Selective hashing of changed/untrusted-provenance applets.
  - Minimal SBOM fragment (SPDX-lite JSON): package name, version, source, applets linked.
- Implementation:
  - Extend `src/logging/audit.rs` with op buffering/finalize, writing `audit-<op_id>.jsonl` and `.sig`.
  - Add CLI `oxidizr-arch audit verify --op <op_id>`.
  - Provenance enrichment via `src/system/worker/packages.rs::{query_file_owner, check_installed, repo_has_package}`.

Acceptance:

- `audit verify` validates signature per op.
- Audit entries record provenance and versions; selective hashes emitted where expected.
- SBOM fragment exists and is referenced in the audit summary.
