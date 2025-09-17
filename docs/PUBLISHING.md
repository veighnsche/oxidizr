# Publishing Guide

This repository ships two CLIs:

- `oxidizr-arch` — Arch Linux CLI. Published to AUR.
- `oxidizr-deb` — Debian/Ubuntu CLI. Published as a `.deb` and optionally to an APT repo (Cloudsmith).

Both are wired to publish on Git tags like `v0.1.0`.

## Versioning and tags

- Keep crate versions in sync with tags. Example: `cargo set-version -p oxidizr-arch 0.1.1 && cargo set-version -p oxidizr-deb 0.1.1`.
- Create a signed tag with `v` prefix and push:

```bash
# bump versions across crates as needed
# tag and push
git tag -s v0.1.1 -m "v0.1.1"
git push origin v0.1.1
```

## AUR (oxidizr-arch)

Workflow: `.github/workflows/release-aur.yml`

- Triggers on tags `v*`.
- Updates `PKGBUILD`'s `pkgver` and `pkgrel`, refreshes checksums with `updpkgsums`, generates `.SRCINFO`, and pushes to the AUR Git repo.
- Uses centralized packaging at `cargo/oxidizr-arch/packaging/aur/PKGBUILD` which builds the `oxidizr-arch` crate via `cargo` from the GitHub tag tarball.

Required repository secrets:

- `AUR_SSH_PRIVATE_KEY`: Private key that matches a public key uploaded to your AUR account (https://aur.archlinux.org/).
- `AUR_COMMIT_EMAIL`: Email to attribute AUR commits (fallbacks to `actions@github.com`).

First-time setup notes:

- The workflow will initialize the AUR repository if cloning fails (new package).
- Ensure the package name `oxidizr-arch` is available on AUR.

Manual verification (optional):

```bash
# From an Arch/Manjaro/EndeavourOS machine
cd cargo/oxidizr-arch/packaging/aur
makepkg --printsrcinfo > .SRCINFO
updpkgsums
namcap PKGBUILD
```

## Debian/Ubuntu (oxidizr-deb)

Workflow: `.github/workflows/release-deb.yml`

- Triggers on tags `v*`.
- Builds `oxidizr-deb` in release mode and packages a `.deb` via `cargo-deb`.
- Uploads the `.deb` to the GitHub Release assets automatically.
- Optionally publishes to Cloudsmith (APT repository hosting) if secrets are configured.

Required repository secrets for Cloudsmith (optional):

- `CLOUDSMITH_API_KEY`: Cloudsmith API key.
- `CLOUDSMITH_OWNER`: Cloudsmith owner/org (e.g., `my-org`).
- `CLOUDSMITH_REPO`: Repository slug (e.g., `oxidizr`).
- `DEBIAN_DISTRIBUTIONS`: Comma-separated list of distributions to publish to, e.g.:
  - `ubuntu/jammy,ubuntu/noble,debian/bookworm`

Local build and test of the package:

```bash
# Build release binary
cargo build -p oxidizr-deb --release --locked

# Build .deb
cargo install cargo-deb
cargo deb -p oxidizr-deb --no-build
ls -lh target/debian/*.deb
```

## Notes on dependencies and workspace

- The workspace uses a `[patch.crates-io]` override at the root `Cargo.toml` so building from a source tarball (GitHub tag) uses the local workspace crates. This is intentional and works in both AUR and cargo-deb builds.
- Publish order for crates.io (if you choose to publish libraries) is documented elsewhere, but is not required for AUR/DEB packaging.

## Releasing checklist

- Update CHANGELOG/Release notes.
- Bump crate versions and commit.
- Tag: `vX.Y.Z` and push.
- Verify GitHub Actions ran:
  - AUR workflow pushed to https://aur.archlinux.org/packages/oxidizr-arch
  - Debian workflow produced `.deb` assets on the GitHub Release and (optionally) pushed to Cloudsmith.
