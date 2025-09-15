# Plan: src/ Organization for oxidizr-deb

## 1) Goals

- Keep concerns modular and testable.
- Make Debian/Ubuntu UX features optional behind flags where appropriate.
- Preserve a stable CLI surface with clear commands.

## 2) Proposed Layout

```
src/
  main.rs                      # parse args, init logging in future (if added), dispatch
  cli/
    mod.rs
    args.rs                    # clap structs: global flags, subcommands (use/restore/status)
    handler.rs                 # top-level dispatch
    completions.rs             # generate shell completions (bash/zsh/fish)
  commands/
    use.rs                     # package-level 'use' entrypoint
    restore.rs                 # restore package (or all) from backups
    status.rs                  # report current rustified state
  packages/
    coreutils.rs               # internal mapping/policy for coreutils (no applet exposure)
    findutils.rs               # internal mapping/policy for findutils (no applet exposure)
    sudo.rs                    # sudo-specific policy/hardening hooks
  fetch/
    mod.rs
    resolver.rs                # select artifact by arch/distro/channel
    verifier.rs                # SHA-256 (and signature) verification
    sources.rs                 # upstream/distro sources
  adapters/
    debian.rs                  # apt/dpkg lock detection helpers
    alternatives.rs            # optional: update-alternatives integration (package-level)
    divert.rs                  # optional: dpkg-divert integration (package-level)
    preflight.rs               # extra CLI-layer preflight checks (e.g., sudo mode/owner)
  util/
    prompts.rs                 # interactive confirm; --assume-yes
    paths.rs                   # SafePath helpers; merged-/usr detection (internal)
    diagnostics.rs             # user-facing error helpers & tips
  errors.rs                    # CLI error type mapping (v0: 0/1 exits)
```

## 3) Cargo Features (optional)

- `debian-alternatives` → enables `adapters::alternatives` and related flags.
- `debian-divert` → enables `adapters::divert` and related flags.
- `file-logging` → enables JSONL file sinks in the engine.
- `fetch-channels` → enable `--channel stable|latest`.

## 4) Public CLI Surface

- Global flags: `--root`, `--commit`, `--assume-yes`.
- Commands: `rustify <package>`, `restore <package|all>`, `status`, `completions`.

## 5) Testing Strategy

- Unit-tests per module (paths, prompts, apt lock parsing, fetch verification).
- Integration tests for commands against a temp `--root`.
- BDD tests for end-to-end behaviors.

## 6) Acceptance

- Commands compile into small units with clear ownership.
- Debian-specific helpers isolated behind `adapters/debian.rs` and optional features.
- No raw mutating `PathBuf` crosses module boundaries without SafePath construction.
- Applet selection is not exposed in CLI; coreutils/findutils mapping is internal.
- Fetch-and-verify runs before any mutation is planned.
