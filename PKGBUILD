# Maintainer: Flaviu Tamas <tamasflaviu@gmail.com>
pkgname=docker2hosthosts
pkgver=0.1.0
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')

build() {
    return 0
}

package() {
    cargo install --root="$pkgdir" docker2hosthosts
}
