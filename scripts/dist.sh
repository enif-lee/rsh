#!/bin/sh
# Builds rsh for every platform onboarding may meet and puts the binaries
# where the client looks for them (default ~/.rsh/dist, or the first
# argument, e.g. dist/ for a release), with their SHA256SUMS. Linux builds are static (musl) and made in docker, so they run
# on any distribution; all builds come from this tree and share its build id.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
# Not group-writable whatever the umask: the client does not trust what
# such a directory holds (remote::trusted).
umask 022
OUT=${1:-$HOME/.rsh/dist}
mkdir -p "$OUT"
# What this run builds: only these go in SHA256SUMS (not a binary left in
# $OUT by an earlier build).
built=""
for arch in x86_64 aarch64; do
    case $arch in
        x86_64) platform=linux/amd64 ;;
        aarch64) platform=linux/arm64 ;;
    esac
    echo "building rsh-linux-$arch"
    docker run --rm --platform "$platform" \
        -v "$ROOT":/src:ro \
        -v "rsh-cargo-$arch":/usr/local/cargo/registry \
        -v "rsh-target-$arch":/target \
        -v "$OUT":/out \
        -w /src -e CARGO_TARGET_DIR=/target \
        rust:1-alpine \
        sh -c "apk add --no-cache musl-dev >/dev/null && cargo build --release --locked && cp /target/release/rsh /out/rsh-linux-$arch.new && mv -f /out/rsh-linux-$arch.new /out/rsh-linux-$arch"
    built="$built rsh-linux-$arch"
done
echo "building the local binary"
(cd "$ROOT" && cargo build --release --locked)
local_platform=$("$ROOT/target/release/rsh" --version >/dev/null && uname -sm | awk '{ os = tolower($1); arch = $2; if (arch == "arm64") arch = "aarch64"; if (arch == "amd64") arch = "x86_64"; print os "-" arch }')
cp "$ROOT/target/release/rsh" "$OUT/rsh-$local_platform.new" && mv -f "$OUT/rsh-$local_platform.new" "$OUT/rsh-$local_platform"
built="$built rsh-$local_platform"
# On macOS the other architecture too, when rustup has its target.
if [ "$(uname -s)" = Darwin ]; then
    for arch in aarch64 x86_64; do
        target=$arch-apple-darwin
        [ "darwin-$arch" = "$local_platform" ] && continue
        if rustup target list --installed 2>/dev/null | grep -qx "$target"; then
            echo "building rsh-darwin-$arch"
            (cd "$ROOT" && cargo build --release --locked --target "$target")
            cp "$ROOT/target/$target/release/rsh" "$OUT/rsh-darwin-$arch.new" && mv -f "$OUT/rsh-darwin-$arch.new" "$OUT/rsh-darwin-$arch"
            built="$built rsh-darwin-$arch"
        else
            echo "skipping rsh-darwin-$arch: rustup target add $target"
        fi
    done
fi
# The sums a release carries (install.sh and rsh check downloads with them).
(cd "$OUT" && for f in $built; do shasum -a 256 "$f" 2>/dev/null || sha256sum "$f"; done) > "$OUT/SHA256SUMS"
"$ROOT/target/release/rsh" --version
ls -l "$OUT"
