# Maintainer: Your Name <you@example.com>

pkgname=coreutils-switch
pkgver=0.1.0
pkgrel=1
epoch=
pkgdesc="oxidizr-arch style coreutils switching tool (dry-run capable)"
arch=('x86_64' 'aarch64')
url="https://github.com/your/repo"
license=('MIT' 'Apache')
depends=('pacman' 'bash')
makedepends=('rust' 'cargo')
provides=('coreutils-switch')
conflicts=()
source=("local://${pkgname}-${pkgver}.tar.gz")
sha256sums=('SKIP')

build() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  cargo build --release --locked
}

package() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  install -Dm755 "target/release/coreutils-switch" "${pkgdir}/usr/bin/coreutils-switch"
  install -Dm644 LICENSE-MIT "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE-MIT"
  install -Dm644 LICENSE-APACHE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE-APACHE"
  install -Dm644 README.md "${pkgdir}/usr/share/doc/${pkgname}/README.md"
}
