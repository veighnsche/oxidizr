# Repository E2E Scenarios (tests/)

This directory contains end-to-end scenarios executed against real or containerized Arch derivatives. Each scenario is self-contained under a numbered folder and provides a `task.yaml` with the steps to run, a short summary, and distro constraints.

How to run
- Use the project’s orchestrator per `test-orch/host-orchestrator/` (see `HOST_ORCH.md`) to execute scenarios inside a clean container.
- Each `task.yaml` defines:
  - `summary`: one-line purpose
  - `distro-check`: supported distros for the scenario
  - `execute`: bash steps to run
  - `restore`: bash steps to clean up after the scenario

Conventions
- Numbered prefixes (10-, 20-, …) define execution order.
- All scenarios must be idempotent and leave the system clean (restore path must be safe to run multiple times).
- Write logs to a temp file when assertions depend on warnings or info lines.

Infra requirements
- A container runtime and base images with required packages.
- Locales: some scenarios (e.g., disable-in-german) require `de_DE.UTF-8` to exist inside the container image.

Adding a new scenario
- Create `NN-name/` with a `task.yaml` containing the four keys described above.
- Keep steps explicit and portable across supported Arch derivatives.

See also
- `tests/INDEX.md` for a catalog of scenarios with summaries and tags.
- `test-orch/` for the host and container orchestration.
