#!/usr/bin/env python3
"""Full-screen animation at a fixed frame rate: row 0 carries the frame
number and the time it was written, the other rows change every frame."""
import os, sys, time
seconds, fps = float(sys.argv[1]), float(sys.argv[2])
cols, rows = os.get_terminal_size()
out = sys.stdout.buffer
out.write(b"\x1b[?25l\x1b[2J")
start = time.time()
n = 0
while True:
    now = time.time()
    if now - start >= seconds:
        break
    head = "FRAME %06d T=%d" % (n, int(now * 1000))
    body = []
    for r in range(1, rows - 1):
        ch = chr(ord("a") + (n + r) % 26)
        body.append(("%s%s" % (ch, chr(ord("A") + (n * 7 + r) % 26))) * (cols // 2))
    frame = "\x1b[H" + head.ljust(cols)[:cols] + "".join("\r\n" + line[:cols] for line in body)
    out.write(frame.encode())
    out.flush()
    n += 1
    next_at = start + n / fps
    time.sleep(max(0.0, next_at - time.time()))
out.write(b"\x1b[?25h\x1b[2J\x1b[H")
out.flush()
print("frames %d" % n)
