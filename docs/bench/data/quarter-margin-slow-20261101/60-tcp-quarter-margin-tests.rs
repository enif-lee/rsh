
#[cfg(test)]
mod tcp_fresh_quarter55_tests {
    use super::*;

    // Synthetic known rate isolates the deadline; no estimator-learning claim.
    fn with_attachment(check: impl FnOnce(&mut Attachment, Instant)) {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("fresh-quarter55-test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("fresh-quarter55-test", Action::Default, (100, 30), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:12345".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        let last = now - Duration::from_secs(2);
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        attach.output.set_path(Some(21_000.0), Some(100.0));
        attach.path = None;
        attach.shadow = Some(Terminal::with_history(100, 30, 0));
        attach.repaint_due = true;
        attach.screen_at = Some(last);
        attach.update_len = 332;
        attach.output_gap = Duration::from_nanos(16_666_666);
        attach.output_at = Some(last + attach.output_gap);
        attach.frames = ANIMATION_FRAMES;
        attach.mid_sequence = None;
        attach.link_free = None;
        check(attach, last);
    }

    #[test]
    fn known_slow_tcp_does_not_wait_past_the_quarter_gap() {
        with_attachment(|attach, last| {
            assert!(is_tcp_addr(attach.addr));
            assert!(!attach.modelled());
            let step = attach.screen_gap();
            let gap = Duration::from_secs_f64((332 + proto::WIRE_OVERHEAD) as f64 / 21_000.0) + SLOW_ROOM;
            assert!(gap > step && step * 2 - gap > gap / 4);
            assert_eq!(attach.screen_due_at(), Some(last + gap));
            assert_eq!(attach.screen_gap(), step, "global cadence unchanged");
        });
    }

    #[test]
    fn both_udp_fresh_wait_paths_keep_their_old_deadlines() {
        with_attachment(|attach, last| {
            let gap = Duration::from_secs_f64((332 + proto::WIRE_OVERHEAD) as f64 / 21_000.0) + SLOW_ROOM;
            attach.addr = "127.0.0.1:12345".parse().unwrap();
            attach.output.set_lossless(false);
            assert!(!attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 2));
            let mut path = PathModel::new(last);
            let arrivals = (0..2).map(|i| (path.number(last, 600, false), 100_000 + i * 10_000)).collect();
            path.feedback(&proto::Feedback { sent_us: 110_000, arrivals }, last + Duration::from_millis(220));
            attach.path = Some(path);
            attach.link_free = Some(last + gap);
            assert!(attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + gap));
        });
    }

    #[test]
    fn eighty_ms_cap_unknown_rate_and_tail_gates_stay_unchanged() {
        with_attachment(|attach, last| {
            attach.output.set_path(Some(50_000.0), Some(100.0));
            attach.update_len = 3_636;
            attach.output_at = Some(last + attach.output_gap * 4);
            let gap = Duration::from_secs_f64((3_636 + proto::WIRE_OVERHEAD) as f64 / 50_000.0) + SLOW_ROOM;
            assert_eq!(gap, Duration::from_millis(80));
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 5));
            attach.update_len = 4_096;
            attach.output_at = Some(last + attach.output_gap * 5);
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 6));
            let began = last + Duration::from_millis(100);
            attach.mid_sequence = Some(began);
            assert_eq!(attach.next_screen_at(), Some(began + TAIL_WAIT));
            attach.mid_sequence = None;
            attach.output = Sender::new(0);
            attach.output.set_lossless(true);
            attach.output_at = Some(last + attach.output_gap);
            assert!(attach.output.slow_rate().is_none());
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap.mul_f64(0.9).clamp(MIN_SCREEN_GAP, SCREEN_GAP)), "unknown-rate TCP follows the margin branch");
            attach.output.push(b"not sent");
            assert_eq!(attach.screen_due_at(), None);
            attach.output = Sender::new(0);
            attach.shadow = None;
            assert_eq!(attach.screen_due_at(), None);
        });
    }
}
