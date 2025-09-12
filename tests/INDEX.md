# E2E Scenarios Index

This index enumerates all end-to-end scenarios under `tests/` with their summaries and supported distros.

- 10-non-root/
  - Summary: (no task.yaml — placeholder)
  - Distros: (n/a)

- 20-enable-default/
  - Summary: Enable default experiment set installs coreutils and sudo-rs
  - Distros: arch, manjaro, cachyos, endeavouros

- 30-disable-default/
  - Summary: Disable default experiment set removes defaults
  - Distros: arch, manjaro, cachyos, endeavouros

- 40-enable-partial/
  - Summary: Enable only the coreutils experiment
  - Distros: arch, manjaro, cachyos, endeavouros

- 50-disable-partial/
  - Summary: Disable only coreutils, leaving sudo-rs enabled
  - Distros: arch, manjaro, cachyos, endeavouros

- 60-disable-in-german/
  - Summary: Disable only coreutils under a German locale, leaving sudo-rs enabled
  - Distros: arch, manjaro, cachyos, endeavouros

- 70-enable-all/
  - Summary: Enable all experiments installs all and wires symlinks/backups
  - Distros: arch, manjaro, cachyos, endeavouros

- 75-flip-checksums/
  - Summary: Flip checksum tools via the dedicated 'checksums' experiment (presence-aware)
  - Distros: arch, manjaro, cachyos, endeavouros

- 76-checksums-only/
  - Summary: Enable only checksums; auto-install provider and flip present checksum applets (presence-aware)
  - Distros: arch, manjaro, cachyos, endeavouros

- 77-coreutils-remove-guard/
  - Summary: Guard coreutils removal when checksums are active; require disabling checksums first
  - Distros: arch, manjaro, cachyos, endeavouros

- 80-disable-all/
  - Summary: Disable all experiments
  - Distros: arch, manjaro, cachyos, endeavouros

- 85-symlink-progress/
  - Summary: Exercise symlink operations to drive host-side progress bar
  - Distros: arch, manjaro, cachyos, endeavouros

- 90-enable-no-compatibility-check/
  - Summary: Enable with --no-compatibility-check should proceed on unsupported distro
  - Distros: arch, manjaro, cachyos, endeavouros

Notes
- See `tests/README.md` for execution instructions and infra requirements.
- Each scenario must be idempotent and include a `restore` section to leave the system clean.
