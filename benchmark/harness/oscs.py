#!/usr/bin/env python3
"""Prints one of each OSC the comparison checks; each carries a payload
that appears nowhere else, so the client's output shows which got through."""
import sys, time
E, B = "\x1b]", "\x07"
# (name, body after ESC ], what in the client's output shows it arrived)
SEQS = [
    ("0 제목+아이콘", "0;T0xq", "T0xq"),
    ("1 아이콘", "1;I1xq", "I1xq"),
    ("2 제목", "2;T2xq", "T2xq"),
    ("7 작업 디렉터리", "7;file://h/C7xq", "C7xq"),
    ("8 하이퍼링크", "8;;http://L8xq/", "L8xq"),
    ("9 알림", "9;N9xq", "N9xq"),
    ("9;4 진행률", "9;4;1;57", "]9;4;1;57"),
    ("777 알림", "777;notify;N777xq;b", "N777xq"),
    ("52 클립보드 쓰기", "52;c;QzUyeA==", "QzUyeA=="),
    ("52 클립보드 읽기", "52;c;?", "]52;c;?"),
    ("4 팔레트 설정", "4;1;rgb:12/34/56", "rgb:12/34/56"),
    ("4 팔레트 질의", "4;2;?", "]4;2;?"),
    ("10 전경색 질의", "10;?", "]10;?"),
    ("11 배경색 설정", "11;rgb:01/02/03", "rgb:01/02/03"),
    ("104 팔레트 리셋", "104;3", "]104;3"),
    ("133 셸 통합", "133;A;k=q133", "133;A"),
    ("1337 iTerm2", "1337;SetUserVar=K1337xq=dg==", "K1337xq"),
    ("모르는 OSC", "5555;U5555xq", "U5555xq"),
]
if __name__ == "__main__":
    for _, body, _ in SEQS:
        sys.stdout.write(E + body + B)
        if body.startswith("8;;"):
            sys.stdout.write("link" + E + "8;;" + B)
        sys.stdout.flush()
        time.sleep(0.05)
    sys.stdout.write("\r\nOSC" + "DONE\r\n")
    sys.stdout.flush()
