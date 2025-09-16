# Glossary

- Action — A single atomic operation in a Plan (e.g., replace a symlink).
- Apply — Execute a Plan (DryRun or Commit) with audit facts emission.
- Degraded (EXDEV) — Cross-filesystem fallback behavior; fact flag and reason recorded.
- Facts — Structured JSON events (schema v2) emitted for every stage.
- Plan — Deterministic set of Actions derived from PlanInput; carries plan_id.
- Preflight — Policy and environment gating prior to apply.
- Rescue Profile — Fallback toolset (BusyBox or GNU subset) verified by preflight.
- SafePath — Path type that stays within allowed roots; required for mutations.
- Sidecar — Metadata file accompanying a backup payload with integrity hints.
- Smoke Tests — Minimal health verification suite post-apply.
