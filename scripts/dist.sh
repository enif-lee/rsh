#!/bin/sh
# Builds rsh for every platform onboarding may meet and puts the binaries
# where the client looks for them (default ~/.rsh/dist, or the first
# argument). Linux builds are static (musl) and made in docker, so they run
# on any distribution; all builds come from this tree and share its build id.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${1:-$HOME/.rsh/dist}
mkdir -p "$OUT"
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
done
echo "building the local binary"
(cd "$ROOT" && cargo build --release --locked)
local_platform=$("$ROOT/target/release/rsh" --version >/dev/null && uname -sm | awk '{ os = tolower($1); arch = $2; if (arch == "arm64") arch = "aarch64"; if (arch == "amd64") arch = "x86_64"; print os "-" arch }')
cp "$ROOT/target/release/rsh" "$OUT/rsh-$local_platform.new" && mv -f "$OUT/rsh-$local_platform.new" "$OUT/rsh-$local_platform"
"$ROOT/target/release/rsh" --version
ls -l "$OUT"
