#!/bin/bash
# End-to-end check through a real terminal: an outer tmux pane on this
# machine runs `rsh` against the rsh-e2e container (see Dockerfile), and
# keys and mouse bytes are typed into that pane as a terminal would send them.
#
#   docker build -f tests/container/Dockerfile -t rsh-e2e .
#   docker run -d --name rsh-e2e -p 127.0.0.1:47322:2222/udp rsh-e2e
#   cargo build && tests/container/e2e.sh
# RSH_E2E_CONTAINER and RSH_E2E_PORT point it at another container.
set -u
C=${RSH_E2E_CONTAINER:-rsh-e2e}
PORT=${RSH_E2E_PORT:-47322}
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
R=$ROOT/target/debug/rsh
WORK=$(mktemp -d)
trap 'tmux -L rsh-e2e kill-server 2>/dev/null; rm -rf "$WORK"' EXIT
echo "rsh-e2e e2e-token" > "$WORK/auth"
chmod 600 "$WORK/auth"
# Inside full-screen apps there is no line start for `~`, so these sections
# detach with Ctrl-^; the last section checks the default `~` keys.
cat > "$WORK/env.sh" <<ENV
export RSH_AUTH=$WORK/auth RSH_PEER=127.0.0.1:$PORT RSH_ESCAPE_KEY=\$'\\x1e' PS1='outer\$ ' LANG=en_US.UTF-8
ENV
T() { tmux -L rsh-e2e "$@"; }
FAILED=0
# check <what> <actual> <expected substring>
check() {
    if [[ "$2" == *"$3"* ]]; then echo "  ok   $1: $2"; else echo "  FAIL $1: got '$2', want '$3'"; FAILED=1; fi
}
first() { T capture-pane -p -t z | grep -m1 -E "$1"; }
flags() { T display -p -t z "alt=#{alternate_on} mouse_any=#{mouse_any_flag} btn=#{mouse_button_flag} sgr=#{mouse_sgr_flag} keypad=#{keypad_flag}"; }
inner() { docker exec "$C" tmux display -p -t in "$@"; }
# Also drops SGR resets: next to the badge and blank rows, where tmux puts
# them depends on which cells it trims, not on what is shown.
strip_clock() { sed -e 's/[0-9][0-9]:[0-9][0-9] [0-9][0-9]-[A-Za-z]*-[0-9][0-9]//' -e $'s/\x1b\\[0m//g' -e 's/ *$//' "$1"; }
reattach() {
    T capture-pane -p -e -t z > "$WORK/before"
    T send-keys -t z C-^ .; sleep 1
    check "modes off after detach" "$(flags)" "alt=0 mouse_any=0 btn=0 sgr=0 keypad=0"
    T send-keys -t z "clear; $R --name e2e rsh-e2e" Enter; sleep 4
    T capture-pane -p -e -t z > "$WORK/after"
    if diff -q <(strip_clock "$WORK/before") <(strip_clock "$WORK/after") >/dev/null; then
        check "reattach screen" "identical with colors" "identical"
    else
        check "reattach screen" "differs" "identical"; diff "$WORK/before" "$WORK/after" | head -8 | cat -v
    fi
    check "modes after reattach" "$(flags)" "$1"
}

T new-session -d -x 120 -y 36 -s z "sh -i"; sleep 0.3
# Sessions left by an earlier run would bring up the session menu.
(. "$WORK/env.sh"; for name in e2e keys; do "$R" kill rsh-e2e "$name" >/dev/null 2>&1; done)
T send-keys -t z ". $WORK/env.sh; clear; $R --name e2e rsh-e2e" Enter; sleep 2.5

echo "== tmux"
T send-keys -t z 'tmux new -s in \; set -g mouse on \; split-window -h \; send-keys -t 0 "seq 1 300" Enter' Enter; sleep 2
check "inner tmux size" "$(inner '#{window_width}x#{window_height}')" "120x35"
check "tmux mouse modes reach the outer terminal" "$(flags)" "mouse_any=1"
T send-keys -t z -l $'\e[<0;10;5M\e[<0;10;5m'; sleep 0.7
check "click selects left pane" "$(inner '#{pane_index}')" "0"
for _ in 1 2 3 4 5; do T send-keys -t z -l $'\e[<64;10;5M'; done; sleep 0.7
check "wheel scrolls into copy-mode" "$(docker exec "$C" tmux display -p -t in:0.0 'mode=#{pane_mode} scroll=#{scroll_position}')" "mode=copy-mode scroll=20"
T send-keys -t z q; sleep 0.4
W0=$(docker exec "$C" tmux display -p -t in:0.0 '#{pane_width}')
T send-keys -t z -l $'\e[<0;61;10M\e[<32;50;10M\e[<32;45;10M\e[<0;45;10m'; sleep 0.7
check "drag border resizes pane" "$W0->$(docker exec "$C" tmux display -p -t in:0.0 '#{pane_width}')" "60->44"
T send-keys -t z C-b c; sleep 0.4; T send-keys -t z C-b p; sleep 0.4
check "prefix keys" "windows=$(inner '#{session_windows}') active=$(inner '#{window_index}')" "windows=2 active=0"
reattach "alt=1 mouse_any=1 btn=1 sgr=1 keypad=1"
T send-keys -t z C-b d; sleep 0.8

echo "== claude"
T send-keys -t z 'clear; claude' Enter; sleep 7
check "claude menu drawn" "$(first '❯')" "Dark mode"
T send-keys -t z Down Down; sleep 0.8; check "claude Down x2" "$(first '❯')" "Dark mode (colorblind-friendly)"
T send-keys -t z Up; sleep 0.8; check "claude Up" "$(first '❯')" "Light mode"
reattach "alt=0"
T send-keys -t z Down; sleep 0.8; check "claude Down after reattach" "$(first '❯')" "Dark mode (colorblind-friendly)"
docker exec "$C" pkill -x claude; sleep 1

echo "== codex"
T send-keys -t z 'clear; codex' Enter; sleep 7
check "codex menu drawn" "$(first '^> ')" "1. Sign in with ChatGPT"
T send-keys -t z Down Down; sleep 0.8; check "codex Down x2" "$(first '^> ')" "3. Use an OpenAI API key"
reattach "alt=1"
T send-keys -t z Up; sleep 0.8; check "codex Up after reattach" "$(first '^> ')" "2. Sign in with Device Code"
docker exec "$C" pkill -f codex; sleep 1

echo "== resize"
T resize-window -t z -x 100 -y 30; sleep 1
T send-keys -t z 'clear; stty size' Enter; sleep 1
# The answer is on the top row, beside the badge.
check "remote stty size follows the window" "$(first '^[0-9]+ [0-9]+( |$)')" "30 100"
T send-keys -t z C-^ .; sleep 1
check "modes off after final detach" "$(flags)" "alt=0 mouse_any=0"

echo "== ~ keys and badge"
T send-keys -t z "clear; unset RSH_ESCAPE_KEY; $R --name keys rsh-e2e" Enter; sleep 2.5
T send-keys -t z 'echo a~b' Enter; sleep 1
check "badge" "$(T capture-pane -p -e -t z | head -1)" "keys:rsh-e2e"
check "~ inside a line is text" "$(first '^a~b$')" "a~b"
T send-keys -t z '~h'; sleep 1
check "~h window" "$(first 'this help')" "~h  this help"
T send-keys -t z x; sleep 1
check "window closed" "$(first 'this help' || echo closed)" "closed"
docker pause "$C" >/dev/null; sleep 5
check "disconnected notice" "$(first 'Session disconnected')" "[ Session disconnected ]"
dimmed=$(T capture-pane -p -e -t z | grep -c $'\x1b\\[2m')
check "screen dimmed" "$([ "$dimmed" -gt 0 ] && echo dimmed || echo plain)" "dimmed"
T send-keys -t z 'echo back-$((40+2))' Enter
docker unpause "$C" >/dev/null; sleep 2
check "notice gone" "$(first 'Session disconnected' || echo gone)" "gone"
check "keys typed while down ran" "$(first '^back-42')" "back-42"
T send-keys -t z '~.'; sleep 1
check "~. detaches" "$(first 'detached rsh-e2e:keys')" "detached rsh-e2e:keys"
"$R" kill rsh-e2e keys >/dev/null 2>&1

echo "== --tmux"
tmux_state() { docker exec "$C" tmux list-sessions -F '#{session_name}:#{session_attached}' 2>/dev/null | grep '^box:'; }
docker exec "$C" tmux kill-session -t box >/dev/null 2>&1
T send-keys -t z "clear; $R rsh-e2e box --tmux" Enter; sleep 3
check "--tmux joins the tmux session" "$(tmux_state)" "box:1"
check "badge in tmux" "$(T capture-pane -p -t z | head -1)" "box:rsh-e2e"
T send-keys -t z Enter; sleep 0.3; T send-keys -t z '~l'; sleep 3
check "~l marks the tmux row" "$(first 'tmux +box')" "* tmux"
T send-keys -t z q; sleep 0.5
T send-keys -t z Enter; sleep 0.3; T send-keys -t z '~.'; sleep 2
check "~. leaves tmux" "$(first 'left tmux session')" "[rsh: left tmux session box on rsh-e2e]"
check "tmux session kept without a client" "$(tmux_state)" "box:0"
docker exec "$C" tmux kill-session -t box >/dev/null 2>&1
exit $FAILED
