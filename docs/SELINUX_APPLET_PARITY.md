# SELinux applet parity and policy (Arch + uutils-coreutils)

This note documents why some GNU coreutils applets are intentionally NOT switched to uutils on Arch, the expected CLI behavior, and how we test/support SELinux-related applets.

## Summary

- uutils-coreutils on Arch does not currently ship SELinux-specific applets `chcon` and `runcon`.
- When you run `oxidizr-arch --commit use coreutils`, the CLI links applets to the replacement where possible but will skip applets that the replacement does not provide.
- Skips are logged as JSON events and no dangling symlinks are created. The existing distro-provided applets remain available (since `use` keeps GNU installed).
- `replace coreutils` (purge GNU coreutils) is guarded by coverage preflight and will fail when the replacement does not cover all distro commands (including `chcon`/`runcon`).

## What you will see

During `use coreutils`, the CLI emits JSONL on stderr for each skipped applet, for example:

```
{"event":"use.exec.skip_applet","applet":"chcon","reason":"source_missing","source":"/usr/bin/uu-chcon"}
{"event":"use.exec.skip_applet","applet":"runcon","reason":"source_missing","source":"/usr/bin/uu-runcon"}
```

- "source_missing" means the per-applet uutils binary (`/usr/bin/uu-<applet>`) is not present after installation.
- After apply, GNU `chcon`/`runcon` continue to exist because `use` does not remove GNU packages. Only applets covered by uutils get switched.

`oxidizr-arch status --json` can still report `coreutils: "active"`, because status requires that representative canaries (e.g., `ls`, `cat`, `echo`, `mv`) are switched and point to executable targets, not that every applet is switched.

## Why chcon/runcon are not switched

- The Arch package `uutils-coreutils` does not currently provide `uu-chcon` and `uu-runcon`.
- These applets are tied to SELinux semantics. On systems without SELinux enabled (the default Arch container), replacing them would not be meaningful.
- We choose to skip missing applets rather than link to a non-existent target or attempt to emulate SELinux behavior.

## Policy

- __Use mode (`use coreutils`)__
  - Link all applets that the replacement actually provides.
  - For applets not provided by the replacement (e.g., `chcon`, `runcon`), leave the distro binaries in place. Emit `use.exec.skip_applet` events and proceed.
  - Rationale: Keeps the system functional and avoids dangling symlinks. Users who need SELinux applets continue to use the GNU ones.

- __Replace mode (`replace coreutils`)__
  - Guarded by coverage preflight: the replacement must cover all distro-provided applets for the package on the live root.
  - On Arch, since GNU coreutils provides `chcon` and `runcon` but uutils does not, coverage preflight will fail and `replace coreutils` will be blocked.
  - Rationale: Prevents destructive states on SELinux-enabled workloads by not purging necessary GNU applets.

- __Status semantics__
  - A package is considered "active" when representative applets are symlinked to valid, executable replacement targets.
  - Not all applets must be switched to report active; some may be intentionally left as GNU (e.g., `chcon`, `runcon`).

## Guidance for users and tests

- If your workload depends on `chcon`/`runcon`:
  - Prefer `use coreutils` (keeps GNU installed) rather than `replace coreutils`.
  - Verify availability: `command -v chcon && command -v runcon`.
  - Consider enabling SELinux-specific tests only on SELinux-enabled hosts/containers.

- For test harnesses:
  - Do not attempt to install uutils or mutate `/usr/bin` directly; let the CLI do it.
  - Inspect CLI stderr JSONL for `use.exec.skip_applet` events and assert there are no skips for applets your scenario requires.
  - Accept that `chcon`/`runcon` are skipped on Arch unless uutils adds support.

## Implementation details (citations)

- Per-applet linking (Arch): `cargo/oxidizr-arch/src/commands/use_cmd.rs` resolves `/usr/bin/uu-<applet>` first and falls back to dispatcher only if necessary.
- Skip logging: `use.exec.skip_applet` is emitted when the per-applet source is missing or not executable.
- Coverage preflight: `cargo/oxidizr-cli-core/src/coverage2.rs` (`coverage_preflight`) ensures replacement covers all distro applets before `replace`.
- Arch distro enumeration: `cargo/oxidizr-arch/src/adapters/arch_adapter.rs` uses `pacman -Ql coreutils` to list `/usr/bin/*` applets (includes `chcon`, `runcon`).
- Status check: `cargo/oxidizr-arch/src/commands/status.rs` validates that representative symlink targets exist and are executable.

## Roadmap / nice-to-have

- Add `status --verbose` to enumerate a sample of linked applets and list skipped ones explicitly.
- `doctor --json` could warn on SELinux-enabled systems if `chcon`/`runcon` are not switched to replacements, with guidance to use GNU for those.
- Revisit when upstream uutils adds `chcon`/`runcon`; the CLI will automatically switch them once present.
