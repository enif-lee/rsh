#!/bin/sh
set -eu
cp /baseline-target/debug/rsh /usr/local/bin/rsh
cp /candidate-rsh /usr/local/bin/rsh-b
chmod +x /usr/local/bin/rsh /usr/local/bin/rsh-b
sha256sum /usr/local/bin/rsh /usr/local/bin/rsh-b
su dev -c 'nohup rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized >/tmp/base.log 2>&1 &'
su dev -c 'nohup rsh-b serve --bind 0.0.0.0:2223 --authorized /home/dev/.rsh/authorized >/tmp/candidate.log 2>&1 &'
iptables -I INPUT -p udp -j DROP
PYTHONPATH=/lab python3 /out/check.py
