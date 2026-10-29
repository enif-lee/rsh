#!/bin/bash
# Onboarding over ssh against a fresh host (Dockerfile.sshd):
#   1. no rsh there: the first connect installs this build and starts it
#   2. a second connect needs no ssh
#   3. the host runs another build: it is brought back to this client's
#      build and its server upgraded in place, keeping a session
#   4. an incompatible old rsh holds the port: it is replaced
#   5. the server is down: the connect starts it again
#   6. the host runs another build and no binary of this build exists for
#      its platform: the connect warns and goes ahead without ssh
#   7. picking a session in `rsh ls` brings the host to this build as well
# Step 1 also has a login rc file that prints a banner.
# Needs docker, a free 127.0.0.1:2201/tcp and :2222/udp, and the Linux
# binaries from scripts/dist.sh. Usage: cargo build --release && tests/container/onboard.sh
set -u
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
R=$ROOT/target/release/rsh
WORK=$(mktemp -d)
trap 'docker rm -f rsh-onboard >/dev/null 2>&1; rm -rf "$WORK"' EXIT
FAILED=0
check() {
    if [[ "$2" == *"$3"* ]]; then echo "  ok   $1"; else echo "  FAIL $1: got '$2', want '$3'"; FAILED=1; fi
}

ssh-keygen -q -t ed25519 -N '' -f "$WORK/key"
docker build -q -f "$ROOT/tests/container/Dockerfile.sshd" --build-arg KEY="$(cat "$WORK/key.pub")" -t rsh-sshd "$ROOT" >/dev/null
docker rm -f rsh-onboard >/dev/null 2>&1
docker run -d --name rsh-onboard -p 127.0.0.1:2201:22 -p 127.0.0.1:2222:2222/udp rsh-sshd >/dev/null
sleep 1
SSH="ssh -p 2201 -i $WORK/key -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"
remote() { $SSH dev@127.0.0.1 "$@"; }
connect() { RSH_AUTH="$WORK/auth" "$R" --ssh "$SSH" "$@" dev@127.0.0.1 2>"$WORK/err"; }
BUILD="build $("$R" --version | awk '{print $4}') protocol"
platform=$(remote 'uname -m' | tail -1)
case $platform in aarch64|arm64) arch=aarch64 docker_platform=linux/arm64 ;; *) arch=x86_64 docker_platform=linux/amd64 ;; esac

echo "== 1. fresh host"
remote 'sed -i "1i echo Welcome to the box" ~/.bashrc'
out=$(echo 'export K=kept; echo first-$((1+1))' | connect --name keep)
check "command ran" "$out" "first-2"
check "installed message" "$(cat "$WORK/err")" "installing with rsh"
check "remote version" "$(remote '~/.rsh/bin/rsh --version')" "$BUILD"
check "rsh is a command in a login shell" "$(remote 'bash -lc "rsh --version"' | tail -1)" "$BUILD"
check "no PATH note" "$(grep -c 'not on PATH' "$WORK/err")" "0"

echo "== 2. second connect"
out=$(echo 'echo again-$K' | RSH_AUTH="$WORK/auth" "$R" --ssh false --name keep dev@127.0.0.1 2>"$WORK/err")
check "no ssh needed" "$out$(cat "$WORK/err")" "again-kept"

echo "== 3. another build on the host"
docker run --rm --platform "$docker_platform" -v "$ROOT":/src:ro -v "rsh-cargo-$arch":/usr/local/cargo/registry \
    -v rsh-target-other:/target -v "$WORK":/out -w /src -e CARGO_TARGET_DIR=/target -e RSH_BUILD_SALT=other \
    rust:1-alpine sh -c "apk add --no-cache musl-dev >/dev/null && cargo build --release --locked 2>/dev/null && cp /target/release/rsh /out/other"
remote 'cat > ~/.rsh/bin/rsh.new && chmod 755 ~/.rsh/bin/rsh.new && mv -f ~/.rsh/bin/rsh.new ~/.rsh/bin/rsh && ~/.rsh/bin/rsh serve upgrade' < "$WORK/other" >/dev/null
check "host now runs another build" "$(remote '~/.rsh/bin/rsh --version')" "protocol"
out=$(echo 'echo kept-$K' | connect --name keep)
check "brought back to this build" "$(remote '~/.rsh/bin/rsh --version')" "$BUILD"
check "server upgraded in place" "$(cat "$WORK/err")" "sessions kept"
check "session survived" "$out" "kept-kept"

echo "== 4. incompatible old rsh holds the port"
cat > "$WORK/fake" <<'EOF'
#!/bin/bash
case "$1" in
  --version) echo "rsh 0.0.9" ;;
  serve) exec -a "$0 serve --bind 0.0.0.0:2222" socat -u UDP-RECV:2222 OPEN:/dev/null ;;
esac
EOF
remote 'kill $(cut -d" " -f1 ~/.rsh/serve-2222.pid); sleep 0.5; cat > ~/.rsh/bin/rsh.new && chmod 755 ~/.rsh/bin/rsh.new && mv -f ~/.rsh/bin/rsh.new ~/.rsh/bin/rsh; rm -f ~/.rsh/serve-2222.pid' < "$WORK/fake"
remote 'cd ~/.rsh/bin && S=serve && (nohup ./rsh $S >/dev/null 2>&1 &); sleep 0.5'
out=$(echo 'echo fixed-$((40+2))' | connect)
check "replaced and connected" "$out" "fixed-42"
check "forced message" "$(cat "$WORK/err")" "replacing rsh 0.0.9"
check "remote version" "$(remote '~/.rsh/bin/rsh --version')" "$BUILD"

echo "== 5. server down"
remote 'kill $(cut -d" " -f1 ~/.rsh/serve-2222.pid)'
sleep 0.5
out=$(echo 'echo up-again' | connect)
check "started again" "$out" "up-again"
check "reason" "$(cat "$WORK/err")" "does not answer"

echo "== 6. no binary of this build for the host"
remote 'cat > ~/.rsh/bin/rsh.new && chmod 755 ~/.rsh/bin/rsh.new && mv -f ~/.rsh/bin/rsh.new ~/.rsh/bin/rsh && ~/.rsh/bin/rsh serve upgrade' < "$WORK/other" >/dev/null
mkdir -p "$WORK/stale" && cp "$WORK/other" "$WORK/stale/rsh-linux-$arch"
out=$(echo 'echo still-works' | RSH_DIST="$WORK/stale" HOME="$WORK" RSH_AUTH="$WORK/auth" "$R" --ssh false dev@127.0.0.1 2>"$WORK/err")
check "connected without ssh" "$out" "still-works"
check "said why it did not update" "$(cat "$WORK/err")" "no linux-$arch binary of this build"

echo "== 7. attaching from rsh ls updates the host too"
out=$(printf '1\necho via-ls-$((6*7))\n' | RSH_AUTH="$WORK/auth" "$R" --ssh "$SSH" ls dev@127.0.0.1 2>"$WORK/err")
check "attached from the menu" "$out" "via-ls-42"
check "brought to this build" "$(remote '~/.rsh/bin/rsh --version')" "$BUILD"
check "upgraded in place" "$(cat "$WORK/err")" "sessions kept"
exit $FAILED
