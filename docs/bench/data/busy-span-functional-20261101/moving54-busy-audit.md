# Moving54: busy sample과 strict screen hold의 직접 교차

정책을 그대로 둔 기능 실험54에서, 실제 busy sample의 분모 구간에 future-screen wait 문맥의 poll이 포함되는 witness를 확인했다. 이것은 controller pacing과 측정의 결합을 검토할 직접 자료이며, **sample이 틀렸거나 폐기해야 한다는 증명은 아니다**. 다음 최소 단계는 예정된55의 한 줄 fresh-wait A/B다. 새 threshold/history-clearing/app-limited 상태를 먼저 추가하지 않는다.

Root의 실제16jobs/fourblocks는 functional_valid=True, performance_eligible=False다. Host context는 모두 accepted였지만 사전 quiet gate가 없는 기능 실험이라는 성격은 바뀌지 않는다. 새 kind40은 실제 busy sample append/prune/conditional median 이후의 gen/from/passed ACK now/wire/sample bits/old capacity/new capacity/retained count를 기록한다. 두 정책은 기존4f98/295f이고 새로운 capacity 정책은 없다. Traced IDs는 d742268da4df/3031e105b1aa다.

24개 CSV/metadata의643,005events는 count/order/drop0에 맞고, 최대 단일CSV49,423events, 최대 dump clock 차이는6167ns다. 서버12개에서2,137개 lifetime sample commit을 읽고 wire/elapsed/sample 비트 및 다음25의 capacity 정수값을 대조했다. 이 감사는 source/capture payload를 재생하거나 terminal oracle/GPU를 새로 검증하지 않는다. 원본 기능 무결성과 source/body join은 frozen 분석 결과를 전제로 하고, 새 독립 계산의 중심은 actual sample/hold 구간이다. Measurement peer가 서버12CSV를 별도로 읽어 연결·분모·union 교차를 재계산해0errors/PASS를 보고했다.

## Strict hold 정의

단일 fresh Sender/PID의 기존36/37/38을 다음과 같이 묶었다.

- 37의 timer mask가 **정확히8**이고 future raw wait가 양수여야 한다. 다른 timer와의 tie는 따로 집계한다.
- 직전36은 repaint/shadow/all-sent guards를 통과한 branch20/30/40이어야 하며, `max(raw_due,mid_sequence+20ms)`가37의 nominal due와 일치해야 한다. 49계열 branch30의f=0은 final gap으로 사용하지 않는다.
- 대응38은 poll return code0, network revents0, other-ready0이며 실제 requested ms가37과 일치해야 한다. 이벤트로 일찍 깬 poll은 이 strict 집합에서 제외한다.
- Hold 구간은 **[actual poll enter,min(actual return,due)]**다. Deadline 뒤 overshoot를 의도적 대기에 더하지 않는다. 같은 gen의 구간을 union한 뒤40의 **[from,passed ACK now]**와 교차시킨다. Mark40 row timestamp를 sample 끝으로 쓰지 않는다.

이는 보수적인 관측 부분집합이다. 교차0은 “controller wait가 없었다”는 뜻이 아니다. Eventful poll/tie/과거 deadline 등 제외 사례도 JSON에 남겼다. 양수 교차도 “물리 링크가 비어 있었다”는 뜻은 아니다. Refresh 간격은39의 실제 entry **d**를 쓰고, 오래된 passed decision clock **c**로 바꾸지 않았다.

## 23–30초에 commit된 표본

다음은 sample의 ACK 끝이23–30초에 속한 경우다. 모든 span이23초 뒤에 시작했다는 뜻은 아니다. Candidate dashboard는 반복0에서28개 모두, 반복1에서27/28개가 온전히 그 구간 안에 있다. 혼합 경계 표본은 지우지 않았다. 하향은40의 exact old/new capacity를 비교한 **busy median 단계의 실제 하향**이다.

| Scene | 반복 | Role | Samples | Median 하향 | Hold 교차 | 하향 중 교차 | 교차 비율 중앙값 |
|---|---:|---|---:|---:|---:|---:|---:|
| dashboard | 0 | main-trace-A | 17 | 10 | 7 | 4 | 0.00% |
| dashboard | 0 | main-trace-B | 12 | 6 | 4 | 3 | 0.00% |
| dashboard | 0 | combined-trace | 28 | 17 | 15 | 7 | 6.51% |
| dashboard | 1 | main-trace-B | 7 | 2 | 1 | 0 | 0.00% |
| dashboard | 1 | combined-trace | 28 | 18 | 14 | 10 | 1.48% |
| dashboard | 1 | main-trace-A | 15 | 8 | 6 | 3 | 0.00% |
| random | 0 | main-trace-A | 61 | 22 | 16 | 7 | 0.00% |
| random | 0 | main-trace-B | 52 | 20 | 8 | 4 | 0.00% |
| random | 0 | combined-trace | 66 | 30 | 18 | 8 | 0.00% |
| random | 1 | main-trace-B | 53 | 18 | 21 | 7 | 0.00% |
| random | 1 | combined-trace | 61 | 31 | 16 | 7 | 0.00% |
| random | 1 | main-trace-A | 61 | 24 | 13 | 5 | 0.00% |

Candidate dashboard의28/28개 표본 중 median 하향은17/18개, strict 교차가 있는 표본은15/14개, 하향 중 교차는7/10개다. 전체 표본의 교차시간 중앙값은2.246856/0.510846ms, 분모 대비 중앙값은6.51%/1.48%, 최대는58.97%/49.63%다. 이 수치를 “하향 원인의6.51%”처럼 해석하지 않는다. 낮은 중앙값과 교차 없는 하향도 그대로 보존한다. 온전히23초 뒤에 시작한 반복1의27개로 좁혀도 하향17개 중 교차9개가 남는다.

Exact witness 두 개:

| 반복 | Sample 구간 s | Wire B | New sample B/s | Old→new median capacity B/s | Strict 교차 |
|---|---|---:|---:|---:|---:|
|0|28.709215–28.740566|348|11100.144|16570.215→16395.223|16.232022ms /51.775%|
|1|28.624935–28.658351|462|13825.799|17007.870→15974.885|16.584639ms /49.631%|

이 표의 new sample은 새 관측 한 개이고 new capacity는 retained history의 median이다. 둘을 같은 값으로 취급하지 않는다. New capacity는 같은 ACK의 이후 probe/refloor 이전 값이다. 실제 lifetime2,137commit 중28개는 이후 같은 ACK에서 refloor clear가 관측됐다. 이28개는 최종23–30초 밖에 있으며, 그 때문에도 commit-stage capacity를 ACK 전체의 마지막 상태로 부르면 안 된다. Commit된 busy ACK는queued=True이므로 같은 ACK의 probe는 queue branch로 끝나며 이번 자료에도 same-ACK26은0개다. 별도 다음 ACK의 probe 상승은 여전히 가능하다.

Candidate dashboard의 실제 refresh-entry 간격 중앙값은33.630542/33.490708ms다. 이 자료는 untraced45나 다른 기능 실행49b의 절대 FPS와 합치지 않는다. Kind40이 추가된 계측 경로 자체가 새 관측 조건이며, 샘플 교차 사실만으로55의 효과도 예측하지 않는다.

## 반례와 다음 판단

**물리 queue와 app hold는 동시에 존재할 수 있다.** 임의의 반례로64kbit/s bottleneck에 실제446B가 남았다고 가정하면, 그 serialization만55.75ms다. Sender가 이미 모든 바이트를 TCP로 넘겨 `all_sent=True`이고, 새 화면에30ms future deadline을 둬도 이전 바이트가 bottleneck에서 계속 drain될 수 있다. 그 동안 pop된 ACK의 busy span과 controller poll이 겹치는 것은 모순이 아니다. 이 lower sample을 일괄 무시하면 실제8M→64k 저하에 높은 capacity를 유지해 window/BDP/drain discount와 screen 송신을 동시에 과대하게 만들 수 있다. 이는 실제54의 물리 queue 크기를 측정한 예가 아니라 제안된 sample-제외 규칙에 대한 구성 가능한 반례다.

따라서 최소 권고는 **예정된55 한 줄 A/B를 먼저 실행하고**, FPS1%/5%/average와 body age/fresh recovery/partial exposure를 함께 판단하는 것이다. 기존19의 동일 predicate adverse와51의 다른 margin 회귀를 보존한다. 더 높은 offered utilization이나 바뀐 sample이 관측돼도 실제 capacity 여유가 자동으로 증명되는 것은 아니다. 54만으로 threshold를 더 내리거나 lower-history를 지우는 policy/state를 만들 근거는 부족하다.

근거 파일: `moving54-busy-audit.py`, `moving54-busy-audit.json`, 독립 `moving54-busy-peer-check.*`. 원본은 `54-busy-sample-trace`의 summary 및24CSV/metadata다. Per-sample exact tuple, 교차/비율, same-ACK25/26/refloor 구분, excluded poll 문맥과 hash를 JSON에 보존했다. 원본/제품/하네스는 변경하지 않았고 새 build/workload/Docker/replay는 수행하지 않았다.
