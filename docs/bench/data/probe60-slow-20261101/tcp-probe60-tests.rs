
    // The delivery interval and ACK are processed by the production Sender.
    fn tcp_probe60_fixture(lossless: bool, delivered: u64, queued: bool, elapsed_ms: u64) -> Sender {
        let now = Instant::now();
        let sent = now - Duration::from_millis(if queued { 300 } else { 100 });
        let mut tx = Sender::new(1);
        tx.set_lossless(lossless);
        tx.floor_rtt = Some(100.0);
        tx.capacity = Some(100_000.0);
        tx.push(&vec![0u8; 1000]);
        assert_eq!(tx.next_chunk(sent).unwrap().1.len(), 1000);
        let wire = tx.flight.front().unwrap().2;
        tx.delivered_wire = delivered - wire;
        tx.probe_from = Some((now - Duration::from_millis(elapsed_ms), 0));
        assert!(tx.on_ack(1, 1000, now, false));
        tx
    }

    #[test]
    fn tcp_probe60_grows_at_seventy_percent_only_on_tcp() {
        assert_eq!(tcp_probe60_fixture(true, 7000, false, 100).capacity, Some(125_000.0));
        assert_eq!(tcp_probe60_fixture(false, 7000, false, 100).capacity, Some(100_000.0));
    }

    #[test]
    fn tcp_probe60_preserves_low_offered_load_and_subthreshold_guards() {
        for delivered in [1600, 5900] {
            for tcp in [false, true] {
                assert_eq!(tcp_probe60_fixture(tcp, delivered, false, 100).capacity, Some(100_000.0));
            }
        }
    }

    #[test]
    fn tcp_probe60_preserves_queue_and_short_interval_guards() {
        for tcp in [false, true] {
            let queued = tcp_probe60_fixture(tcp, 7000, true, 100);
            assert_eq!(queued.capacity, Some(100_000.0));
            assert!(queued.probe_from.is_none());
            assert_eq!(tcp_probe60_fixture(tcp, 7000, false, 90).capacity, Some(100_000.0));
        }
    }

    #[test]
    fn tcp_probe60_preserves_existing_high_utilization_growth() {
        for tcp in [false, true] {
            assert_eq!(tcp_probe60_fixture(tcp, 8100, false, 100).capacity, Some(125_000.0));
        }
    }
