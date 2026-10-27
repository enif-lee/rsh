// The path from the server to a client, as the client's arrival reports
// show it (`proto::Feedback`): its bottleneck rate, the queue in front of
// it, the round trip, whether it takes a burst at once, and the share of
// datagrams lost (kept for loss protection to come). Measured from when
// datagrams left the server and arrived at the client (its kernel's
// receive time), so neither end's own work reads as the link's.

use crate::proto::Feedback;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Numbered datagrams remembered for their reports.
const SENT_KEPT: usize = 4096;
/// How long the least one-way delay and round trip are taken over: the
/// path's own delay, with no queue in it, is the least seen lately.
const FLOOR_SPAN: Duration = Duration::from_secs(10);
/// The latest round trips `window_rtt` tells a changed path by.
const LAST_RTTS: usize = 4;
/// Rate samples kept: the rate is their median.
const RATE_KEPT: usize = 24;
/// Rate samples older than this no longer count.
const RATE_SPAN: Duration = Duration::from_secs(10);
/// Datagrams sent back to back (within this, microseconds) leave the
/// bottleneck one after the other when it is slower than the sender: the
/// train's bytes after its first over the spread of their arrivals is the
/// bottleneck's rate. Trains spread less than `SPREAD_US` say nothing (a
/// timer's or the kernel's slack); a link slow enough to matter
/// (`SLOW_RATE`) spreads a full datagram over more than twice that.
const TRAIN_GAP_US: i64 = 1000;
const SPREAD_US: i64 = 2000;
/// Datagrams sent further apart join the train when they arrived clearly
/// further apart than they were sent (by this share and margin,
/// microseconds): the second waited for the bottleneck to finish the first
/// (an ack clock spaces sends out, and a slow link is busy between them).
/// Less is jitter, or a link idle between them.
const SPREAD_SHARE: f64 = 1.2;
const SPREAD_MARGIN_US: f64 = 1000.0;

struct Sent {
    pn: u32,
    at: Instant,
    wire: u32,
    arrived: bool,
    departed: bool,
}

#[derive(Clone, Copy)]
struct Arrival {
    pn: u32,
    at_us: i64,
    sent: Instant,
}

/// Datagrams sent back to back that arrived in order (see `TRAIN_GAP_US`):
/// when the first arrived, and the bytes after it.
#[derive(Clone, Copy)]
struct Train {
    first_us: i64,
    last_us: i64,
    bytes: u64,
}

pub struct PathModel {
    /// What send times are counted from.
    start: Instant,
    next_pn: u32,
    sent: VecDeque<Sent>,
    /// The client clock's last raw reading and its unwrapped value.
    clock: Option<(u32, i64)>,
    last: Option<Arrival>,
    train: Option<Train>,
    /// (when, one-way delay in ms against an arbitrary clock offset),
    /// increasing delays: the front is the least over `FLOOR_SPAN`.
    owd: VecDeque<(Instant, f64)>,
    queue_ms: f64,
    /// (when, round trip less the client's hold, ms), as `owd`.
    rtt: VecDeque<(Instant, f64)>,
    srtt: Option<f64>,
    last_rtts: VecDeque<f64>,
    rates: VecDeque<(Instant, f64)>,
    /// When a train of 600 bytes or more (after its first) last arrived
    /// spread less than `SPREAD_US`: the path took it over `SLOW_RATE`.
    fast_seen: Option<Instant>,
    /// (when, wire bytes) of datagrams sent over the last second.
    sent_log: VecDeque<(Instant, u32)>,
    /// Lost datagrams over all reported or aged out, as a moving share,
    /// and the share of those lost with no queue in front of them.
    loss: f64,
    random_loss: f64,
    /// Bytes the client reported since `delivered_from`, for telling a
    /// link faster than the rate measured, and the rate of the last span.
    delivered: (Instant, u64),
    delivery_rate: f64,
}

impl PathModel {
    pub fn new(now: Instant) -> Self {
        Self {
            start: now,
            next_pn: 0,
            sent: VecDeque::new(),
            clock: None,
            last: None,
            train: None,
            owd: VecDeque::new(),
            queue_ms: 0.0,
            rtt: VecDeque::new(),
            srtt: None,
            last_rtts: VecDeque::new(),
            rates: VecDeque::new(),
            fast_seen: None,
            sent_log: VecDeque::new(),
            loss: 0.0,
            random_loss: 0.0,
            delivered: (now, 0),
            delivery_rate: 0.0,
        }
    }

    /// A number for a datagram of `wire` bytes going out now; a copy of
    /// one sent already does not count in what `room_for_copies` weighs.
    pub fn number(&mut self, now: Instant, wire: usize, copy: bool) -> u32 {
        let pn = self.next_pn;
        self.next_pn = self.next_pn.wrapping_add(1);
        self.sent.push_back(Sent { pn, at: now, wire: wire as u32, arrived: false, departed: false });
        if !copy {
            self.sent_log.push_back((now, wire as u32));
        }
        while self.sent_log.front().is_some_and(|(when, _)| now.duration_since(*when) > Duration::from_secs(1)) {
            self.sent_log.pop_front();
        }
        while self.sent.len() > SENT_KEPT {
            let old = self.sent.pop_front().expect("kept");
            if !old.arrived {
                self.lost();
            }
        }
        pn
    }

    /// The datagrams numbered since the last call went out now: their send
    /// time is when they left, not when they were made (a batch is sealed
    /// whole before it goes).
    pub fn departed(&mut self, now: Instant) {
        for sent in self.sent.iter_mut().rev() {
            if sent.departed {
                break;
            }
            sent.at = now;
            sent.departed = true;
        }
    }

    /// Another path (the client moved): what was learned no longer holds.
    pub fn forget(&mut self, now: Instant) {
        self.last = None;
        self.train = None;
        self.owd.clear();
        self.queue_ms = 0.0;
        self.rtt.clear();
        self.srtt = None;
        self.last_rtts.clear();
        self.rates.clear();
        self.fast_seen = None;
        self.delivered = (now, 0);
    }

    fn unwrap(&mut self, raw: u32) -> i64 {
        let value = match self.clock {
            Some((last_raw, last)) => last + i64::from(raw.wrapping_sub(last_raw) as i32),
            None => i64::from(raw),
        };
        // Only move forward: a late report's earlier time is read against
        // the latest, not made the latest.
        if self.clock.is_none_or(|(_, last)| value > last) {
            self.clock = Some((raw, value));
        }
        value
    }

    fn lost(&mut self) {
        self.loss = self.loss * 0.99 + 0.01;
        let random = self.queue_ms < 5.0;
        self.random_loss = self.random_loss * 0.99 + if random { 0.01 } else { 0.0 };
    }

    fn arrived(&mut self) {
        self.loss *= 0.99;
        self.random_loss *= 0.99;
    }

    /// Takes in a report the client sent.
    pub fn feedback(&mut self, report: &Feedback, now: Instant) {
        let Some(front) = self.sent.front().map(|s| s.pn) else {
            return;
        };
        let mut highest = None;
        for (i, &(pn, raw)) in report.arrivals.iter().enumerate() {
            let index = pn.wrapping_sub(front) as usize;
            let Some(sent) = self.sent.get_mut(index).filter(|s| s.pn == pn && !s.arrived) else {
                continue;
            };
            sent.arrived = true;
            let (at, wire) = (sent.at, sent.wire);
            highest = Some(highest.map_or(index, |h: usize| h.max(index)));
            self.arrived();
            let at_us = self.unwrap(raw);
            // One-way delay against an unknown clock offset: only its
            // changes mean anything, the queue being the rise over the least.
            let owd = at_us as f64 / 1000.0 - at.duration_since(self.start).as_secs_f64() * 1000.0;
            while self.owd.back().is_some_and(|(_, d)| *d >= owd) {
                self.owd.pop_back();
            }
            self.owd.push_back((now, owd));
            while self.owd.len() > 1 && self.owd.front().is_some_and(|(when, _)| now.duration_since(*when) > FLOOR_SPAN) {
                self.owd.pop_front();
            }
            self.queue_ms = owd - self.owd.front().map_or(owd, |(_, d)| *d);
            let joins = self.last.is_some_and(|last| {
                let send_gap = at.saturating_duration_since(last.sent).as_micros() as i64;
                let busy = (at_us - last.at_us) as f64 > SPREAD_SHARE * send_gap as f64 + SPREAD_MARGIN_US;
                last.pn.wrapping_add(1) == pn && (send_gap < TRAIN_GAP_US || busy)
            });
            match self.train.as_mut().filter(|_| joins) {
                Some(train) if at_us >= train.last_us => {
                    train.last_us = at_us;
                    train.bytes += u64::from(wire);
                    // A long train gives a sample now and then, not one at
                    // its end only.
                    if at_us - train.first_us >= 4 * SPREAD_US {
                        self.end_train(now);
                        self.train = Some(Train { first_us: at_us, last_us: at_us, bytes: 0 });
                    }
                }
                _ => {
                    self.end_train(now);
                    self.train = Some(Train { first_us: at_us, last_us: at_us, bytes: 0 });
                }
            }
            self.last = Some(Arrival { pn, at_us, sent: at });
            self.delivered.1 += u64::from(wire);
            // The round trip of the newest datagram reported, less the time
            // the client held it before this report left.
            if i + 1 == report.arrivals.len() {
                let held = f64::from(report.sent_us.wrapping_sub(raw) as i32) / 1000.0;
                let rtt = now.duration_since(at).as_secs_f64() * 1000.0 - held.max(0.0);
                if rtt > 0.0 {
                    self.srtt = Some(self.srtt.map_or(rtt, |s| 0.875 * s + 0.125 * rtt));
                    self.last_rtts.push_back(rtt);
                    if self.last_rtts.len() > LAST_RTTS {
                        self.last_rtts.pop_front();
                    }
                    self.rebase_if_changed(now);
                    while self.rtt.back().is_some_and(|(_, r)| *r >= rtt) {
                        self.rtt.pop_back();
                    }
                    self.rtt.push_back((now, rtt));
                    while self.rtt.len() > 1 && self.rtt.front().is_some_and(|(when, _)| now.duration_since(*when) > FLOOR_SPAN) {
                        self.rtt.pop_front();
                    }
                }
            }
        }
        // Datagrams more than a few behind the newest one arrived that never
        // did are lost (reordering by a few is no loss); those that did are
        // done with.
        if let Some(highest) = highest {
            let keep_from = front.wrapping_add(highest.saturating_sub(3) as u32);
            while let Some(old) = self.sent.front() {
                let behind = keep_from.wrapping_sub(old.pn) as i32 > 0;
                if !(old.arrived || behind) {
                    break;
                }
                let old = self.sent.pop_front().expect("front");
                if !old.arrived {
                    self.lost();
                }
            }
        }
        // Delivered well over the measured rate with no queue: the link got
        // faster (or the samples were low); they are learned again.
        let span = now.duration_since(self.delivered.0).as_secs_f64();
        if span >= 0.25 {
            let delivered = self.delivered.1 as f64 / span;
            self.delivery_rate = delivered;
            if self.queue_ms < 5.0 && self.rate(now).is_some_and(|rate| delivered > 1.25 * rate) {
                self.rates.clear();
            }
            self.delivered = (now, 0);
        }
    }

    /// A train that ended: its rate, when it spread clearly.
    fn end_train(&mut self, now: Instant) {
        if let Some(train) = self.train.take() {
            let spread = train.last_us - train.first_us;
            // 600 bytes in under `SPREAD_US`: over `SLOW_RATE`.
            if spread < SPREAD_US && train.bytes >= 600 {
                self.fast_seen = Some(now);
            }
            if spread >= SPREAD_US && train.bytes > 0 {
                self.rates.push_back((now, train.bytes as f64 * 1_000_000.0 / spread as f64));
                while self.rates.len() > RATE_KEPT {
                    self.rates.pop_front();
                }
            }
        }
    }

    /// The bottleneck's rate, bytes a second (wire), when datagrams lately
    /// waited on it; None when nothing did (a link faster than the output).
    pub fn rate(&self, now: Instant) -> Option<f64> {
        let mut recent: Vec<f64> =
            self.rates.iter().filter(|(when, _)| now.duration_since(*when) <= RATE_SPAN).map(|(_, r)| *r).collect();
        // One sample is enough: samples come only from trains the
        // bottleneck was busy with, so a slow link shows from the first
        // burst (the first frames of an animation would queue otherwise).
        if recent.is_empty() {
            return None;
        }
        recent.sort_by(f64::total_cmp);
        let median = recent[recent.len() / 2];
        // Never below what the path delivered lately while a queue stood in
        // front of it (it was busy all along): a few low samples must not
        // pace output under what the link carries.
        let busy = if self.queue_ms >= 5.0 { self.delivery_rate } else { 0.0 };
        Some(median.max(busy))
    }

    /// Whether the path has room for copies of what goes now: twice the
    /// rate of originals sent over the last second fits in its rate with a
    /// margin (or no rate shows: nothing waited on it). Copies themselves
    /// do not count, or turning them on would turn them off.
    pub fn room_for_copies(&self, now: Instant) -> bool {
        let Some(rate) = self.rate(now) else {
            return true;
        };
        let sent: u64 = self.sent_log.iter().filter(|(when, _)| now.duration_since(*when) <= Duration::from_secs(1)).map(|(_, w)| u64::from(*w)).sum();
        2.0 * sent as f64 <= 0.8 * rate
    }

    /// How long datagrams wait in front of the bottleneck now, ms.
    pub fn queue_ms(&self) -> f64 {
        self.queue_ms
    }

    /// The least round trip lately, ms, the client's hold taken out.
    pub fn min_rtt(&self) -> Option<f64> {
        self.rtt.front().map(|(_, r)| *r)
    }

    /// The round trip a window is sized by.
    pub fn window_rtt(&self, _now: Instant) -> Option<f64> {
        // The least alone: a recent least takes in a standing queue, and a
        // window sized by it keeps the queue standing (LAN to 512 kbit/s:
        // 224 ms grew to 605 with half a second of queue). A near path that
        // turned far is rebased (`rebase_if_changed`).
        self.min_rtt()
    }

    /// A near path (under 20 ms, a LAN) whose last few round trips are far
    /// over its least turned far: the least round trip and one-way delay
    /// start again from them, so neither the window nor the queue reads the
    /// old path for seconds. Not from a far least: there a queue of this
    /// sender's own (a slow link's at the start) looks the same, and the
    /// recent least of `window_rtt` follows a longer path.
    fn rebase_if_changed(&mut self, now: Instant) {
        let (Some(least), Some(last)) = (self.min_rtt(), self.last_rtts.iter().copied().reduce(f64::min)) else {
            return;
        };
        if self.last_rtts.len() < LAST_RTTS || least >= 20.0 || last <= 3.0 * least + 20.0 {
            return;
        }
        self.rtt.clear();
        self.rtt.push_back((now, last));
        self.fast_seen = None;
        // The newest delay stands for the new path's least until a lower
        // one comes.
        if let Some(&(_, newest)) = self.owd.back() {
            self.owd.clear();
            self.owd.push_back((now, newest));
            self.queue_ms = 0.0;
        }
    }

    pub fn srtt(&self) -> Option<f64> {
        self.srtt
    }

    /// Whether the path lately took a train at well over `SLOW_RATE` and no
    /// bottleneck shows since: what goes may go at the program's pace.
    pub fn fast(&self, now: Instant) -> bool {
        // A queue standing says otherwise, whatever was seen before (a LAN
        // that turned slow, before the new rate shows).
        self.queue_ms < 20.0 && self.rate(now).is_none() && self.fast_seen.is_some_and(|at| now.duration_since(at) <= RATE_SPAN)
    }

    /// What the client took lately, wire bytes a second.
    pub fn delivery_rate(&self) -> f64 {
        self.delivery_rate
    }

    /// Whether reports have come: the model speaks for the path.
    pub fn reported(&self) -> bool {
        self.srtt.is_some()
    }

    /// Share of datagrams lost lately, and of those with no queue in front.
    pub fn loss(&self) -> (f64, f64) {
        (self.loss, self.random_loss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Datagrams of `size` sent `send_gap_us` apart through a link of `rate`
    /// bytes a second and `delay_ms`, reported in groups of four.
    fn run(model: &mut PathModel, start: Instant, count: u32, size: usize, send_gap_us: u64, rate: f64, delay_ms: u64) -> Instant {
        let mut link_free = 0f64;
        let mut pending = Vec::new();
        let mut now = start;
        for i in 0..count {
            now = start + Duration::from_micros(u64::from(i) * send_gap_us);
            let pn = model.number(now, size, false);
            let sent_us = now.duration_since(start).as_micros() as f64;
            let out = sent_us.max(link_free) + size as f64 / rate * 1e6;
            link_free = out;
            let arrive = out + delay_ms as f64 * 1000.0;
            pending.push((pn, arrive as u32));
            if pending.len() == 4 {
                let last = pending.last().expect("four").1;
                let report = Feedback { sent_us: last + 500, arrivals: std::mem::take(&mut pending) };
                let at = start + Duration::from_micros(u64::from(last) + 500 + delay_ms * 1000);
                model.feedback(&report, at.max(now));
            }
        }
        now
    }

    #[test]
    fn a_slow_link_shows_its_rate_and_a_fast_one_none() {
        let start = Instant::now();
        // 32 KB/s, datagrams of 1 KB sent back to back.
        let mut slow = PathModel::new(start);
        let now = run(&mut slow, start, 64, 1000, 10, 32_000.0, 50);
        let rate = slow.rate(now).expect("a rate");
        assert!((rate - 32_000.0).abs() < 1_000.0, "{rate}");
        // A link far faster than what is sent: nothing waits on it.
        let mut fast = PathModel::new(start);
        let now = run(&mut fast, start, 64, 1000, 20_000, 10e6, 150);
        assert_eq!(fast.rate(now), None);
        let rtt = fast.min_rtt().expect("a round trip");
        assert!((rtt - 300.0).abs() < 5.0, "{rtt}");
    }

    #[test]
    fn sends_spaced_out_still_show_a_busy_link_and_not_an_idle_one() {
        let start = Instant::now();
        // An ack clock sends 500 bytes every 12 ms into 32 KB/s (15.6 ms
        // each): the link is busy all along.
        let mut busy = PathModel::new(start);
        let now = run(&mut busy, start, 64, 500, 12_000, 32_000.0, 50);
        let rate = busy.rate(now).expect("a rate");
        assert!((rate - 32_000.0).abs() < 2_000.0, "{rate}");
        // Every 20 ms into the same link: it idles between them.
        let mut idle = PathModel::new(start);
        let now = run(&mut idle, start, 64, 500, 20_000, 32_000.0, 50);
        assert_eq!(idle.rate(now), None);
    }

    #[test]
    fn a_lan_that_turned_far_is_learned_at_once() {
        let start = Instant::now();
        let mut model = PathModel::new(start);
        run(&mut model, start, 40, 300, 30_000, 10e6, 0);
        assert!(model.min_rtt().is_some_and(|rtt| rtt < 2.0));
        // The same session over RTT 100 ms: within a few round trips the
        // least round trip is the new path's, not the LAN's for seconds.
        let later = start + Duration::from_secs(2);
        let now = run(&mut model, later, 20, 300, 30_000, 10e6, 50);
        let rtt = model.window_rtt(now).expect("a round trip");
        assert!((rtt - 100.0).abs() < 5.0, "{rtt}");
        assert!(model.queue_ms() < 5.0, "the new path's delay is no queue: {}", model.queue_ms());
    }

    #[test]
    fn a_queue_shows_as_the_rise_over_the_least_delay() {
        let start = Instant::now();
        let mut model = PathModel::new(start);
        // Sent twice as fast as the link takes: a queue grows.
        run(&mut model, start, 40, 1000, 15_000, 32_000.0, 50);
        assert!(model.queue_ms() > 100.0, "{}", model.queue_ms());
    }
}
