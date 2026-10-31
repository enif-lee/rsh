
#[cfg(test)]
mod diff_cancel_interop_tests {
    use super::*;

    fn cells(term: &Terminal) -> String {
        (94..97).map(|col| term.cell(24, col).unwrap().ch).collect()
    }

    fn fixture() -> (Vec<u8>, Vec<u8>, Terminal) {
        let base = b"\x1b[?7l\x1b[1;1Hbefore\x1b[25;1Hrow-body\x1b[25;95H";
        let tail = b"\x1b[";
        let rest = b"0m\x1b[1;1Hafter \x1b[25;95H";
        let mut before = Terminal::with_size(100, 30);
        before.feed(base);
        before.feed(tail);
        assert!(!before.at_ground());
        let mut snapshot = before.render(0);
        snapshot.extend_from_slice(tail);
        let mut shadow = Terminal::with_size(100, 30);
        shadow.feed(&snapshot);
        assert!(!shadow.at_ground(), "snapshot includes the unfinished program tail");
        let mut next = Terminal::with_size(100, 30);
        next.feed(base);
        next.feed(tail);
        next.feed(rest);
        assert!(next.at_ground());
        let delta = next.diff_from(&shadow).expect("same geometry/modes can use a screen delta");
        (snapshot, delta, next)
    }

    #[test]
    fn diff_cancel_rust_parser_controls_whole_and_split() {
        let cases: &[(&str, &[u8], &str)] = &[
            ("literal-negative", b"[0m", "[0m"),
            ("ordinary-sgr", b"\x1b[0m", "   "),
            ("nested-escape", b"\x1b[\x1b[0m", "   "),
            ("explicit-can", b"\x1b[\x18\x1b[0m", "   "),
        ];
        for (name, bytes, expected) in cases {
            for split in [false, true] {
                let mut term = Terminal::with_size(100, 30);
                let mut input = b"\x1b[25;95H".to_vec();
                input.extend_from_slice(bytes);
                if split {
                    for byte in input { term.feed(&[byte]); }
                } else {
                    term.feed(&input);
                }
                assert_eq!(cells(&term), *expected, "{name}, split={split}");
                assert!(term.at_ground(), "{name}, split={split}");
                assert_eq!(term.cursor(), (24, if *expected == "[0m" { 97 } else { 94 }));
            }
        }
    }

    #[test]
    fn diff_cancel_prefix_explicitly_cancels_unfinished_sequence() {
        let (_, delta, _) = fixture();
        assert!(delta.starts_with(b"\x18\x1b[?2026h"), "screen delta has an explicit CAN before its first escape");
    }

    #[test]
    fn diff_cancel_snapshot_to_client_delta_emits_interop_bytes() {
        let (snapshot, delta, expected) = fixture();
        let mut client_snapshot = Vec::new();
        let mut client_delta = Vec::new();
        without_sync(&snapshot, &mut client_snapshot);
        without_sync(&delta, &mut client_delta);
        for split in [false, true] {
            let mut tty = Terminal::with_size(100, 30);
            for bytes in [&client_snapshot, &client_delta] {
                if split {
                    for byte in bytes { tty.feed(&[*byte]); }
                } else {
                    tty.feed(bytes);
                }
            }
            assert!(expected.same_screen(&tty), "actual client sync filtering preserves the final screen, split={split}");
            assert!(tty.at_ground());
            assert_eq!(cells(&tty), "   ");
        }
        let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join("");
        eprintln!("DIFF_CANCEL_INTEROP snapshot={}", hex(&snapshot));
        eprintln!("DIFF_CANCEL_INTEROP delta={}", hex(&delta));
        eprintln!("DIFF_CANCEL_INTEROP client_snapshot={}", hex(&client_snapshot));
        eprintln!("DIFF_CANCEL_INTEROP client_delta={}", hex(&client_delta));
        eprintln!("DIFF_CANCEL_INTEROP expected_render={}", hex(&expected.render(0)));
    }
}
