# Maintainer: Ismael Gutiérrez González <frodo_gv@hotmail.com>
pkgname=('rpfm-git')
pkgver=5.0.1.4525.g42a0ced7
pkgrel=1
pkgdesc="A modding tool for modern (since Empire) Total War games. Development version."
arch=('x86_64')
url="https://github.com/Frodo45127/rpfm.git"
license=('MIT')
depends=('libgit2' 'xz' 'p7zip' 'qt6-base' 'qt6-imageformats' 'kcompletion' 'kiconthemes'  'ktexteditor' 'kxmlgui' 'kwidgetsaddons' 'breeze-icons')
makedepends=('git' 'rust' 'cmake')
provides=('rpfm')
conflicts=('rpfm-bin')
source=("git+https://github.com/Frodo45127/rpfm.git")
sha256sums=('SKIP')
options=('!lto')
_programname=('rpfm')

pkgver() {
    cd $_programname
    echo "$(grep '^version =' $srcdir/$_programname/Cargo.toml|cut -d\" -f2).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)"
}

prepare() {
    cd $_programname
    git -c protocol.file.allow=always submodule update --init --recursive 3rdparty/src/ritual
}

build() {
    cd $_programname
    env CARGO_INCREMENTAL=0 cargo build --release
}

package() {

    # The executables.
    install -D -m755 "$srcdir/$_programname/target/release/rpfm_server" "$pkgdir/usr/bin/rpfm_server"
    install -D -m755 "$srcdir/$_programname/target/release/rpfm_ui" "$pkgdir/usr/bin/rpfm_ui"

    # The icons.
    cd "$srcdir/$_programname/icons/"
    for icon in *; do
        install -D -m644 $icon "$pkgdir/usr/share/$_programname/icons/$icon"
    done

    # The language files.
    cd "$srcdir/$_programname/locale/"
    for locale in *; do
        install -D -m644 $locale "$pkgdir/usr/share/$_programname/locale/$locale"
    done

    # The UI files.
    cd "$srcdir/$_programname/rpfm_ui/ui_templates/"
    for ui_template in *; do
        install -D -m644 $ui_template "$pkgdir/usr/share/$_programname/ui/$ui_template"
    done

    # Shortcut.
    install -D -m644 "$srcdir/$_programname/install/linux/arch/rpfm.desktop" "$pkgdir/usr/share/applications/rpfm.desktop"

    # License.
    install -D -m644 "$srcdir/$_programname/LICENSE" "$pkgdir/usr/share/licenses/$_programname/LICENSE"
}
