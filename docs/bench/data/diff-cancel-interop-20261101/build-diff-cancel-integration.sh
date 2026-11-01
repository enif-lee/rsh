#!/bin/sh
set -eu
apk add --no-cache musl-dev openssh-client >/dev/null
apk info -vv > /out/diff-cancel-integration-apk-packages.txt
mkdir /src
cp -a /inputs/diff-cancel-integration-source/. /src/
cd /src
cargo clean --locked --profile dev -p rsh
cargo test --locked --tests --no-run --message-format=json > /out/diff-cancel-integration-build.jsonl
cargo test --locked --test binary_cli -- --test-threads=4
cargo test --locked --test session -- --test-threads=4
