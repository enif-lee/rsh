#!/bin/sh
set -eu
apk add --no-cache musl-dev >/dev/null
rustc -Vv
cargo --version
apk info -v musl musl-dev
printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\nRSH_BUILD_SALT=%s\n' "${RUSTFLAGS:-}" "${CARGO_ENCODED_RUSTFLAGS:-}" "${RSH_BUILD_SALT:-}"
for pair_variant in baseline candidate; do
    cd /
    rm -rf /src
    mkdir /src
    cp -a /inputs/$pair_variant/. /src/
    cd /src
    cargo clean --release --locked -p rsh
    cargo build --release --locked
    cp /target/release/rsh /out/rsh-$pair_variant-linux-aarch64
    /out/rsh-$pair_variant-linux-aarch64 --version
    sha256sum /out/rsh-$pair_variant-linux-aarch64
done
