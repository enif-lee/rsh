#!/bin/sh
set -eu
apk add --no-cache musl-dev >/dev/null
rm -rf /src
mkdir /src
cp -a /inputs/65-tcp-margin-quiet-control-source/. /src/
cd /src
cargo clean --locked --profile dev -p rsh
cargo test --locked --lib --no-run --message-format=json > /out/cadence65-control-tests-build.jsonl
control_lib=$(sed -n '/"kind":\["lib"\]/{/"name":"rsh"/{/"executable":"/s/.*"executable":"\([^\"]*\)".*/\1/p;};}' /out/cadence65-control-tests-build.jsonl)
test -x "$control_lib"
cp "$control_lib" /out/cadence65-control-library-tests
set +e
timeout -s KILL 120 "$control_lib" tcp_margin_quiet65_tests:: --test-threads=1 > /out/cadence65-control-library-tests.log 2>&1
control_status=$?
set -e
printf 'control test exit: %s\n' "$control_status"
test "$control_status" -eq 101

timeout -s KILL 120 "$control_lib" tcp_cadence_margin_tests:: --test-threads=1 > /out/cadence65-control-margin-tests.log 2>&1

timeout -s KILL 120 "$control_lib" tcp_fresh_quarter55_tests:: --test-threads=1 > /out/cadence65-control-quarter-tests.log 2>&1
