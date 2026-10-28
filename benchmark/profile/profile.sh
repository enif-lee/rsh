#!/bin/sh
# Where `rsh serve` spends its CPU under one of the lab's scenes.
#
#   [NET='delay 50ms rate 256kbit'] benchmark/profile/profile.sh [scene] [fps] [seconds] [binary]
#
# scene: bandwidth (default) or compress (benchmark/lab/scene.py); fps 60;
# seconds 15. Without a binary it builds this tree for Linux with symbols
# and frame pointers (docker volume rsh-target-prof). Server and client run
# in one container (image rsh-prof: the lab's image with perf) over
# loopback, in a 100x30 terminal as the lab's; after a second of warm-up
# it records the server's CPU (its threads' schedstat, % of one core), how
# often its loop wakes (poll calls a second) and a perf profile of it, and
# prints them. NET, if set, is a netem spec for loopback (both ways share
# it: a delay counts twice). The reports stay in
# benchmark/profile/out/<scene>-<fps>[-net]/.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
SCENE=${1:-bandwidth}
FPS=${2:-60}
SECONDS_=${3:-15}
BIN=${4:-}
NET=${NET:-}
OUT="$HERE/out/$SCENE-$FPS${NET:+-$(echo "$NET" | tr -c 'a-z0-9' '_')}"
mkdir -p "$OUT"
case $(uname -m) in arm64|aarch64) platform=linux/arm64 ;; *) platform=linux/amd64 ;; esac
if [ -z "$BIN" ]; then
    BIN="$HERE/out/rsh-prof"
    docker run --rm --platform "$platform" -v "$ROOT":/src:ro -v rsh-cargo-prof:/usr/local/cargo/registry \
        -v rsh-target-prof:/target -v "$HERE/out":/out -w /src -e CARGO_TARGET_DIR=/target \
        -e CARGO_PROFILE_RELEASE_DEBUG=line-tables-only -e RUSTFLAGS="-C force-frame-pointers=yes" rust:1-alpine \
        sh -c "apk add --no-cache musl-dev >/dev/null && cargo build --release --locked -q && cp /target/release/rsh /out/rsh-prof"
fi
docker image inspect rsh-prof >/dev/null 2>&1 || docker build -q -t rsh-prof "$HERE" >/dev/null
C=rsh-prof-run
docker rm -f $C >/dev/null 2>&1 || true
docker run -d --privileged --name $C --add-host server:127.0.0.1 rsh-prof sleep infinity >/dev/null
trap 'docker rm -f $C >/dev/null 2>&1' EXIT
docker cp "$BIN" $C:/usr/local/bin/rsh
docker cp "$ROOT/benchmark/lab/scene.py" $C:/tmp/scene.py
[ -n "$NET" ] && docker exec $C tc qdisc add dev lo root netem $NET
docker exec $C sh -c "su dev -c 'nohup rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized >/tmp/rsh.log 2>&1 &'; sleep 0.5"
# The client in a pty of the lab's size; the scene runs warm-up + record + 1 s.
docker exec -d $C script -qfc "stty cols 100 rows 30; rsh server -- python3 /tmp/scene.py $SCENE $((SECONDS_ + 2)) $FPS" /tmp/client.log
sleep 1.5
docker exec $C sh -c '
    pid=$(pgrep -x -u dev rsh | head -1)
    ns() { cat /proc/$pid/task/*/schedstat | awk "{s += \$1} END {print s}"; }
    a=$(ns); t=$(date +%s%N)
    perf stat -x, -e syscalls:sys_enter_ppoll -p $pid -o /tmp/stat.txt -- sleep '"$SECONDS_"' 2>/dev/null &
    perf record -q -e cpu-clock -F 1999 --call-graph fp -p $pid -o /tmp/server.data -- sleep '"$SECONDS_"' 2>/dev/null
    b=$(ns); u=$(date +%s%N)
    wait
    echo "server cpu: $(echo "$a $b $t $u" | awk "{printf \"%.2f\", (\$2 - \$1) / (\$4 - \$3) * 100}")% of one core over '"$SECONDS_"' s"
    echo "loop wakes: $(grep ppoll /tmp/stat.txt | cut -d, -f1 | awk "{printf \"%.0f\", \$1 / '"$SECONDS_"'}") a second"
    perf report -i /tmp/server.data --stdio --no-children --sort symbol --percent-limit 0.7 -g none 2>/dev/null | grep -v "^#" | grep -v "^$" > /tmp/self.txt
    perf report -i /tmp/server.data --stdio --children --sort symbol --percent-limit 3 -g none 2>/dev/null | grep -v "^#" | grep -v "^$" > /tmp/children.txt
' | tee "$OUT/cpu.txt"
# Frames that reached the client's terminal (the scene's header rows), a second.
docker exec $C sh -c "sleep 2; grep -ao 'FRAME [0-9]*' /tmp/client.log | sort -u | wc -l" | awk '{printf "client frames: %.1f a second\n", $1 / '"$((SECONDS_ + 2))"'}' | tee -a "$OUT/cpu.txt"
docker cp $C:/tmp/self.txt "$OUT/self.txt"
docker cp $C:/tmp/children.txt "$OUT/children.txt"
docker cp $C:/tmp/server.data "$OUT/server.data" 2>/dev/null || true
echo "self (top):"; head -25 "$OUT/self.txt"
echo "inclusive (top):"; head -40 "$OUT/children.txt"
