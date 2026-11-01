#!/bin/sh
set -eu
apk add --no-cache musl-dev openssh-client >/dev/null
apk info -vv > /out/pty-account-pair-apk-packages.txt
rustc -Vv
cargo --version
test -z "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}${RSH_BUILD_SALT:-}"
for pty_variant in pty-name-account fps-combined-pty-account; do
 rm -rf /src
 mkdir /src
 cp -a /inputs/"$pty_variant"-source/. /src/
 cd /src
 cargo clean --locked --profile dev -p rsh
 cargo clean --locked --release -p rsh
 cargo test --locked --tests --no-run --message-format=json > /out/"$pty_variant"-all-tests-build.jsonl
 pty_lib=$(sed -n '/"kind":\["lib"\]/{/"name":"rsh"/{/"executable":"/s/.*"executable":"\([^\"]*\)".*/\1/p;};}' /out/"$pty_variant"-all-tests-build.jsonl)
 test -x "$pty_lib"
 cp "$pty_lib" /out/"$pty_variant"-library-tests
 timeout -s KILL 120 "$pty_lib" --test-threads=4 > /out/"$pty_variant"-library-tests.log 2>&1
 for pty_test in binary_cli session pty_concurrency; do
  pty_exe=$(sed -n '/"name":"'"$pty_test"'"/s/.*"executable":"\([^\"]*\)".*/\1/p' /out/"$pty_variant"-all-tests-build.jsonl)
  test -x "$pty_exe"
  cp "$pty_exe" /out/"$pty_variant"-"$pty_test"-tests
  timeout -s KILL 120 "$pty_exe" --test-threads=4 > /out/"$pty_variant"-"$pty_test"-tests.log 2>&1
 done
 cargo build --locked --release --message-format=json > /out/"$pty_variant"-release-build.jsonl
 cp /target/release/rsh /out/rsh-"$pty_variant"
 /out/rsh-"$pty_variant" --version
 sha256sum /out/rsh-"$pty_variant"
 cd /
done