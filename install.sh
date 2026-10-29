#!/bin/sh
# Installs rsh: the latest release's binary for this machine (Linux or
# macOS, x86_64 or arm64), checked against the release's SHA256SUMS, at
# ~/.rsh/bin/rsh and linked from ~/.local/bin/rsh.
#
#   curl -fsSL https://raw.githubusercontent.com/enif-lee/rsh/main/install.sh | bash
#
# RSH_VERSION=v0.2.0 installs that release instead. Run it again to
# upgrade; rsh also moves itself to a newer release once a day
# (RSH_NO_UPDATE=1 turns that off).
set -eu

REPO=enif-lee/rsh
# ~/.rsh not group-writable whatever the umask (rsh does not trust such
# directories).
umask 022

say() { printf 'rsh install: %s\n' "$*"; }
die() { say "$*" >&2; exit 1; }

case $(uname -s) in
    Linux) os=linux ;;
    Darwin) os=darwin ;;
    *) die "no release for $(uname -s); build from source: https://github.com/$REPO" ;;
esac
case $(uname -m) in
    x86_64 | amd64) arch=x86_64 ;;
    arm64 | aarch64) arch=aarch64 ;;
    *) die "no release for $(uname -m); build from source: https://github.com/$REPO" ;;
esac
name=rsh-$os-$arch

if [ -n "${RSH_VERSION:-}" ]; then
    base=https://github.com/$REPO/releases/download/$RSH_VERSION
else
    base=https://github.com/$REPO/releases/latest/download
fi

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        die "needs curl or wget"
    fi
}

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
trap 'exit 1' INT TERM

say "downloading $name"
fetch "$base/$name" "$tmp/rsh" || die "could not download $base/$name"
fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS" || die "could not download $base/SHA256SUMS"
want=$(awk -v f="$name" '$2 == f { print $1 }' "$tmp/SHA256SUMS")
[ -n "$want" ] || die "$name is not in the release's SHA256SUMS"
[ "$(sha256 "$tmp/rsh")" = "$want" ] || die "$name does not match the release's SHA256SUMS"

bin=$HOME/.rsh/bin
mkdir -p "$bin"
chmod 755 "$tmp/rsh"
mv -f "$tmp/rsh" "$bin/rsh.new"
mv -f "$bin/rsh.new" "$bin/rsh"

link=$HOME/.local/bin/rsh
mkdir -p "$HOME/.local/bin"
if [ -e "$link" ] && [ ! -L "$link" ]; then
    say "$link is another program; left alone (this rsh is $bin/rsh)"
else
    old=$(readlink "$link" 2>/dev/null || true)
    if [ -n "$old" ] && [ "$old" != "$bin/rsh" ]; then
        say "$link pointed at $old; now at $bin/rsh"
    fi
    ln -sf "$bin/rsh" "$link"
fi

say "installed $("$bin/rsh" --version)"
case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) say "add ~/.local/bin to PATH, e.g. echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.profile" ;;
esac
