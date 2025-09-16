# Cross-filesystem (EXDEV)

- Atomic rename across filesystems is not possible for symlinks.
- When policy allows, degrade to unlink+symlink (for links) or safe copy+sync+rename (for files) and mark `degraded=true`.

Policy and facts
- Control via `apply.exdev` (e.g., `DegradedFallback` vs `Fail`).
- Facts include `degraded=true|false` and a `degraded_reason` such as `"exdev_fallback"`.

Operator guidance
- Prefer keeping staging and target on the same filesystem in production to avoid degradation.
- If degradation is acceptable, document its impact and monitor `degraded` telemetry in `apply.result`.

Citations:
- `src/fs/atomic.rs`
- `src/api/apply/handlers.rs`
- `SPEC/SPEC.md`
