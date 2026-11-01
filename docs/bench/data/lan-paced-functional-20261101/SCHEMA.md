# Busy54 addition to the inherited fixed RAM recorder

Eight signed i64 payload columns remain; timestamps use the same process
Instant anchor as row ns, with -1 absent. Moving49 uses bounded131,072 records (80B each,10MiB),
No per-record disk output, policy field, option or dependency. Any drops or
failed clock/PID/single-Sender fence invalidate attribution. These traced
functional observations cannot retroactively identify untraced40/41 causes.

* **36 screen_choice:** gen, mask, screen_at_ns, output_at_ns,
  output_gap_ns, gap_input_ns, raw_due_ns, mid_sequence_since_ns.
  Low8 mask bits are branch:1 repaint false,2 no shadow,3 not all_sent,
  4 no screen_at,11 modeled BDP room blocked,12 TCP/unmodeled BDP room
  blocked,20 known slow,30 no-slow low-queue cadence,40 high-queue halfRTT.
  Branch20 bits8..11 choose1 current output_at>=idle,2 predicted next,
  3 idle,4 no output timestamp; bit12 records modeled branch. Bits16/17/18
  are repaint/shadow/animation. gap_input is branch20's already-computed
  serialization+SLOW_ROOM or link-free gap; branch30 uses sentinel0,
  branch40 stores raw halfRTT **before clamp**. It is not a universal final
  gap. Cadence derives from output_gap clamped8..20ms when animation else20ms.
  Effective next_screen_at is max(raw_due,mid_sequence+20ms) when shadow and
  mid_sequence exist. No extra deadline/rate getter is called. Emission is
  suppressed when neither repaint nor shadow is present; pump/tx flags still
  identify raw mode. None/gates are recorded only when that screen context
  exists. Source-internal caller identity is not added.
* **37 server_timer:** decision_now_ns, raw_min_wait_ns, returned_wait_ns,
  co-binding_min_mask, earliest_nominal_due_ns,0,0,0. Mask1 initial200/10ms,
  2 Sender deadline,4 ACK,8 screen,16 gather,32 closed-child20ms,
  64 forwarding deadline,128 forwarding100ms. Equal saturated minima retain
  all bits, not an invented unique winner. Returned wait is max(raw,1ms).
  Existing candidates are evaluated once in their original order; tracing
  only tracks locals. Past deadlines saturate their wait to0.
* **38 server_poll:** actual_enter_ns, actual_return_ns,
  decision_now_after_state_lock_ns, actual_requested_integer_ms, poll_rc,
  OR_of_network_revents,ready_other_fd_count,total_fd_count.
  Other FDs include forwarding/PTY; pty_read identifies actual PTY work.
  This separates requested delay, poll overshoot and post-poll lock delay.
* **39 screen_refresh:** old_gen,new_gen,passed_decision_now_ns,
  actual_refresh_entry_ns,screen_at_after_ns,update_len,output_end,kind.
  Kind1 accepted delta,2 full repaint/reset. Row ns is after refresh work;
  compare with entry for elapsed work. Passed now may predate PTY work.
* Existing **12 pty_apply** keeps duration/read_len/session_id; old zero
  columns become packed_ground_tail,output_gen,output_at_ns,output_gap_ns,
  animation_frames. packed bit0 is Terminal::at_ground's actual boolean;
  higher bits are retained tail byte count. This deliberately distinguishes
  parser ground from an overflow-cleared tail. No new frame parser/buffer.

Reuse source-ledger prewrite clocks and existing11/12/13/14/15/16,25/26,
34/35 and client arrival/gate/poll/write records. PTY read chunks are not
producer frame boundaries; split headers/partial chunks cannot establish
individual source-frame arrival from these scalar events alone. Preserve
unknown joins instead of inferring exact frame identity or GPU presentation.
The minimal question is which actual gate/deadline/wake/refresh path limits
this functional run, not whether a global0.9 cadence is safe or optimal.


## 40 busy_sample_commit — one new scratch record site

The shared Sender site may execute on either endpoint; both endpoint traces validate it. A record occurs only when a new busy-rate sample has actually been appended, expired/excess samples pruned, and the existing conditional median capacity assignment completed. Payload: gen, span_from_ns, passed_ack_now_ns, span_wire_sum, sample_rate_f64_bits, old_capacity_f64_bits, new_capacity_f64_bits, retained_sample_count. None capacity uses -1; rate f64 bits are signed i64 transport and must be decoded without JS Number precision loss.

The row ns is actual instrumentation time after the busy median phase, before later probe/refloor logic in the same on_ack. New capacity is this phase output, not a claim about final ACK-exit state. Payload from/now is the existing sample interval's passed Instant pair, not row time or kernel arrival. Rate is computed once by the existing wire/elapsed expression and reused for push and trace. No added rate/deadline getter or timer; old capacity is a copied local, not new policy state. Count1..15 follows existing retention; below3 the original capacity remains unchanged, at3+ the existing median is assigned.

An absent event is legitimately unexercised, not a failed experiment or proof of zero busy demand. Parser validates presence when observed, signed clocks/order/minimum30ms span, wire/rate consistency, finite positive f64 values and original count/assignment invariants. Arithmetic float comparison is a schema check, not performance acceptance tolerance. Exact interval overlap with existing36/37/38 wait context does not prove continuous application demand, an empty kernel/path queue, physical spare capacity or GPU presentation. No second ACK site, threshold, history-clear policy, margin change or new knob is added.

Generation binding is endpoint-specific: each server commit must equal the most recent preceding tx_stream generation in this isolated single-Sender trace; future tx_stream records cannot approve an earlier event. A client input Sender retains gen0. Missing prior server stream or stale/unknown gen is rejected when a commit is present. A trace with no commits remains legitimately unexercised.
