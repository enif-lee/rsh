# Lowbw52 additions to the inherited fixed RAM recorder

Eight signed i64 payload columns remain; timestamps use the same process
Instant anchor as row ns, with -1 absent. Recorder capacity is unchanged from49:131072 records x80 bytes =10MiB.
No per-record disk output, policy field, option or dependency. Any drops or
failed clock/PID/single-Sender fence invalidate attribution. These traced
functional observations cannot retroactively identify untraced50/51 causes.

* **36 screen_choice:** gen, mask, screen_at_ns, output_at_ns,
  output_gap_ns, gap_input_ns, raw_due_ns, mid_sequence_since_ns.
  Low8 mask bits are branch:1 repaint false,2 no shadow,3 not all_sent,
  4 no screen_at,11 modeled BDP room blocked,12 TCP/unmodeled BDP room
  blocked,20 known slow,30 no-slow low-queue cadence,40 high-queue halfRTT.
  Branch20 bits8..11 choose1 current output_at>=idle,2 predicted next,
  3 idle,4 no output timestamp; bit12 records modeled branch. Bits16/17/18
  are repaint/shadow/animation. gap_input is branch20's already-computed
  serialization+SLOW_ROOM or link-free gap; branch30 stores its actual applied8..20ms gap,
  branch40 stores raw halfRTT **before clamp**. It is still not a universal final
  gap: only branch30 is final, branch40 remains pre-clamp. The global cadence helper remains output_gap clamped8..20ms when animating, else20ms. In branch30 only, the margin source uses raw output_gap*0.9 before that clamp for TCP+animation; the baseline uses the unchanged helper.
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
