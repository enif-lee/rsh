
    fn tcp_probe60_near_fixture(delivered: u64, old_ms: u64, new_ms: u64) -> Sender {
        let now = Instant::now();
        let mut tx = Sender::new(1);
        tx.set_lossless(true);
        tx.floor_rtt = Some(100.0);
        tx.capacity = Some(100_000.0);
        tx.push(&vec![0u8; 2 * CHUNK]);
        assert!(tx.next_chunk(now - Duration::from_millis(old_ms)).is_some());
        assert!(tx.next_chunk(now - Duration::from_millis(new_ms)).is_some());
        let wire: u64 = tx.flight.iter().map(|(_, _, wire, _)| wire).sum();
        tx.delivered_wire = delivered - wire;
        tx.probe_from = Some((now - Duration::from_millis(100), 0));
        assert!(tx.on_ack(1, (2 * CHUNK) as u64, now, false));
        tx
    }

    #[test]
    fn tcp_probe60_near_accepts_cumulative_near_floor_ack() {
        assert_eq!(tcp_probe60_near_fixture(7000, 108, 107).capacity, Some(125_000.0));
    }

    #[test]
    fn tcp_probe60_near_rejects_extra_growth_with_one_later_packet() {
        // Both are below the existing queue threshold, but the first is
        // above floor+its own serialization. The other packet cannot hide it.
        assert_eq!(tcp_probe60_near_fixture(7000, 119, 100).capacity, Some(100_000.0));
    }

    #[test]
    fn tcp_probe60_near_preserves_old_eighty_percent_growth() {
        assert_eq!(tcp_probe60_near_fixture(8100, 119, 100).capacity, Some(125_000.0));
    }
