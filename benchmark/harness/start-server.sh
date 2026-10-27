#!/bin/sh
# Servers of all four tools, each on its default port.
/usr/sbin/sshd
etserver --daemon >/dev/null 2>&1 || (etserver >/tmp/etserver.log 2>&1 &)
su dev -c 'nohup rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized >/tmp/rsh.log 2>&1 &'
exec sleep infinity
