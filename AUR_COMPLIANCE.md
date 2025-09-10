# 📝 AUR Compliance & Security Audit Checklist

## 1. **PKGBUILD Basics**

* [ ] **Unique name**: Not duplicating an official package. Use `conflicts=()` and `provides=()` if relevant.
* [ ] **Arch set correctly**: `arch=('x86_64')` or `('any')`.
* [ ] **No binaries**: All sources fetched from verifiable upstream (`source=()` with checksums).
* [ ] **License**: `license=('MIT')`, `('Apache')`, etc., matching upstream.
* [ ] **Maintainer field**: `# Maintainer: Name <email>` present and accurate.
* [ ] **Versioning**: Uses proper `pkgver` and `pkgrel` format.
* [ ] **Build system**: Calls proper tools (`cargo`, `make`, etc.), no custom hacks unless necessary.

---

## 2. **Source Integrity**

* [ ] **Verified source**: Uses tagged GitHub release, git tag, or signed tarball.
* [ ] **Checksums**: `sha256sums=()` or `b2sums=()` provided, no `SKIP` unless unavoidable.
* [ ] **No curl-to-bash**: Never pipes remote scripts into shell.
* [ ] **No unpinned refs**: Avoid fetching from `main`/`master` without fixed commit/tag.

---

## 3. **Install Behavior**

* [ ] **No invasive post\_install actions**: Don’t alter `/usr/bin/*` or overwrite pacman-owned files automatically.
* [ ] **No replaces=()**: Use `conflicts=()` instead.
* [ ] **.install scriptlets minimal**: Only echo messages or update caches (e.g., `gtk-update-icon-cache`), nothing invasive.
* [ ] **Respect filesystem hierarchy**: Binaries in `/usr/bin`, libs in `/usr/lib`, docs in `/usr/share/doc/pkgname/`.

---

## 4. **Security & Trust**

* [ ] **No embedded secrets/keys**: No hardcoded credentials, tokens, or GPG keys.
* [ ] **No network activity during build**: All sources must be in `source=()` or vendored.
* [ ] **No hidden behavior**: PKGBUILD must be transparent; no obfuscation or unnecessary complexity.
* [ ] **Reproducible build**: Two users building the package should get identical binaries.

---

## 5. **Runtime Safety (Special for Your Project)**

* [ ] **Swaps only on user command**: Tool does not automatically mutate system files at install.
* [ ] **Clear CLI separation**: `oxidizr` binary performs flips, PKGBUILD only ships binary/docs.
* [ ] **Backups & restores**: Any mutation of `/usr/bin` must be reversible and logged.
* [ ] **Dry-run support**: Users can see what changes would occur before execution.
* [ ] **Structured logs**: Mutations leave audit records.

---

## 6. **Community Expectations**

* [ ] **PKGBUILD is short & readable**: Easy for reviewers to understand.
* [ ] **Namcap passes**: Run `namcap PKGBUILD` and `namcap pkgname.pkg.tar.zst`.
* [ ] **Docs included**: At least README or man page installed.
* [ ] **Transparency**: Any risky behavior (like binary swapping) explained in README.
* [ ] **Opt-in design**: Users consciously run the tool; no surprises.

---

## 7. **Pre-Submission Final Pass**

* [ ] **Run `makepkg -sric`** in a clean chroot (e.g., with `devtools`) and confirm no missing deps.
* [ ] **No leftover debug or test code** in final install.
* [ ] **Upload only PKGBUILD and support files** (no built artifacts).
* [ ] **Double-check AUR submission page**: name, description, URL, license, dependencies all accurate.

---

✅ **If all boxes are ticked, your package is highly likely to pass community audit and remain trusted in AUR.**
