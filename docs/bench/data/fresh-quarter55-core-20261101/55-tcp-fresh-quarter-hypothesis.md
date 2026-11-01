55는 새로운 알고리즘이 아니다. 과거19와 같은 known-slow TCP quarter-gap predicate를 현재combined295f 기반에서 다시 검증하기 위한 소스 초안이다. 실제 성능·개선·회귀 해결·승격은 아직 입증하지 않았다. Source-only이며 build/test/Docker/실측0이다.

현재49b의 후보 dashboard23–30초에서는 branch20의 serialization+5ms work 중앙값22.386/26.228ms, fresh 정렬 추가8–9ms, 실제 entry gap33.663/33.802ms가 관측됐다. Probe 상승 뒤 busy median이 capacity를 다시 낮춘 별도 witness도 있다. TCP의 fresh allowance를 `min(FRESH_WAIT,gap/4)`로 줄이면 일부 next-step 선택 대신 idle deadline을 선택할 수 있다는 조건부 가설이다. 이것이 지속 가능한 물리 throughput을 늘리거나 낮은 sample을 바로잡는다는 뜻은 아니다. 근거:`research_notes/LAN 원거리 low 성능 연구/moving49b-detail-entry-v2.json` 및 독립 peer.

변경은 `server.rs`의 기존 `self.modelled()` 조건에 `|| is_tcp_addr(self.addr)`를 추가한 한 줄이다. Known-slow branch 안에서만 작동한다. Modelled UDP의 기존 quarter cap과 unmodelled UDP의20ms는 그대로다. Global screen gap, unknown-rate branch, TAIL_WAIT, gather/compression, admission/windows/RTO, measured floor gate, server lossless probe0.6/client·UDP0.8, CAN·PTY/account 수정은 변경하지 않았다. Margin48의0.9는 포함하지 않았다. 상태/API/옵션/의존성은 추가하지 않았다.

중요한 반대 근거가 있다. 과거19의 동일 predicate는 RTT300 random 반복0에서 candidate24.5avg/4.0·12.0low를 기록했다. 동시 main-A는49.7/14.0·45.2, main-B는49.4/11.3·44.0이었다. 반복1 candidate50.0/11.1·44.5는 main49.7/9.9·44.3 및48.4/13.0·35.7과 섞였다. 나쁜 반복을 지우거나 두 번째 반복으로 상쇄하지 않는다. 원인은 당시에도 확정되지 않았다. 근거:`19-fresh-quarter-core-screen/core-metrics.csv`, `research_notes/LAN 원거리 low 성능 연구/fresh19-actual-review.md`, `rsh-parity/docs/bench/lowtail-20261031.md`.

19의 parent는 main6729/24248bd이고 probe0.8이었다. 현재 parent295f에는 far TCP transit floor gate, output TCP probe0.6, CAN, private PTY/account lookup이 있다. 19는 schema1의 write 반환 후 ledger 등록, parent의 exec 후 winsize 설정, 연속60초 headroom 방식이었다. 현재 실험 방법은 RAM schema2/pidfd 및 pre-exec geometry/CPU30 admission이다. 이 차이 때문에 절대 수치를 직접 합치지 않지만, 과거 실패를 무효화하는 근거도 아니다. 현재 steady300ms random의 branch30 관측은 known-slow 분기를 직접 행사하지 않을 수 있으나, startup/ACK/압축 state가 바뀌는 효과까지 배제하지 않는다.

앞당긴 diff가 terminal ground 상태에서도 불완전한 program frame을 노출할 수 있다. Body FPS·low·partial exposure가 오히려 나빠지거나 traffic/age/CPU가 늘 수 있다. 현재 busy sample의 continuous-demand 검증 부족은55로 해결되지 않는다. Probe threshold를 더 낮추거나 kernel rate를 대입하지 않는다.

새3개 fixture는 synthetic known-rate TCP에서 next step 거부, 두 UDP 경로 유지,80ms cap 경계 및 unknown-rate/TAIL/unsent/shadow gate를 검사하도록 작성했다. Injected path rate는 deadline 식만 분리하며 실제 estimator 학습을 입증하지 않는다. Current295f에 같은 fixture만 붙인 control은 첫 test 실패/나머지2개 통과를 예상한다. Candidate의3 targeted 및113library+2ignored는 계획일 뿐 실행 결과가 아니다. `55-tcp-fresh-quarter-tests.rs`를 exact suffix로 붙였다.

승격 전에는 core LAN/RTT100/300·slow1M/256k/64k·moving을 각각4병렬×2회로 관측해야 한다. Complete body와header/partial을 구분하고 avg/1%/5%low를 주로, age·회복·traffic·CPU/RSS를 별도 비용으로 기록한다. 특히19의300ms random을 빠뜨리면 안 된다. 일부 지표 손해를 자동탈락시키지는 않지만, 전체 비교 없이55를 채택하지 않는다. 기존 연구·원자료·현재main과margin48 및54 계측 소스는 수정하지 않았다.
