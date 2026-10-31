독립 저장 자료 감사 PASS: 16 jobs/4 blocks, 실제 본문 11239개 / uniform header 11282개. 소스 26696개/218268391B, pending write 0개, pending body join 0개. 원본 데이터 수정과 새 terminal replay/Docker/실측은 하지 않았다.

양쪽 endpoint 파일 64, live-server 32, strict network 32개를 검증했다. 고정 source/analysis/hash, source nonce/PID/clock bracket, all4 window closure→pidfd/SIGUSR2→원래5초 수집, 실제 본문N/T/scene/source join, raw capture read 구조, FPS/1%·5%low/age/maxgap/per-second/CSV 산술이 일치했다. Pending 반환 시각은 null을 유지했다. Resource는 raw job→summary→CSV 동일성과 단위 검증 범위이며 독립 raw 카운터 재측정이 아니다.

Main-A/B는 최종 CAN+PTY-safe4f98/c25 동일 바이너리이고 candidate는 combined295f/fb04다. 4병렬×2회, CPU30은 단일 admission이며 runtime CPU/외부프로세스 값은 문맥이다. FPS1%/5%low와avg를 주로 비교하고 CPU/RSS는 보조로 둔다. 개별 손해로 자동기각하거나 임의 점수를 만들지 않는다. 데이터 유효성과 최종 승격은 별개다.

각 셀 순서: **1%low / 5%low / avg FPS**. 두 반복과 SSH·동일 mainA/A를 모두 보존했다.

|Case|SSH|main-A|main-B|candidate|
|---|---|---|---|---|
|00-moving-compress-i0|6.8 / 26.3 / 48.2|2.4 / 6.4 / 17.8|2.5 / 6.5 / 17.9|3.0 / 7.6 / 23.8|
|01-moving-compress-i1|7.3 / 24.5 / 47.9|2.2 / 7.0 / 20.7|2.4 / 6.0 / 18.3|2.6 / 8.3 / 21.9|
|02-moving-bandwidth-i0|2.6 / 13.6 / 27.3|1.5 / 6.5 / 14.3|2.4 / 7.1 / 20.2|2.3 / 6.0 / 20.4|
|03-moving-bandwidth-i1|3.2 / 12.9 / 27.2|1.9 / 4.9 / 14.1|1.8 / 5.9 / 15.3|2.1 / 6.7 / 19.3|

Moving은 static과 절대값을 합치지 않는다. Candidate dashboard23.8/21.9avg·1%low3.0/2.6·5%low7.6/8.3은 main보다 대체로 높지만 SSH48.2/47.9·6.8/7.3·26.3/24.5보다 낮다. 반면 source-made 기반 p95age는 candidate146/239ms 대 SSH11274/11430ms다. Random도 SSH가 avg와low에서 앞서지만 candidate p95age291/260ms 대 SSH13779/13729ms라는 비용/이득을 함께 기록한다. Age는 완료된 본문 조건부 값이며 input/GPU latency가 아니다.

실제 native8worker×131point/restore 자료와 rawstdout/hash/PID/exe/plan·commonT0/sourcearm을 frozen validator로 확인하고,131skew 및7outage ACK bounds×4blocks를 별도 산술로 다시 계산했다. 다음은 공통6ms/7ms outage의 ACK bracket 범위다. ACK 시각은 정확한 packet impairment 시각이 아니다.

|Block|max8worker skew ms|6ms common lower–upper ms|7ms common lower–upper ms|
|---|---|---|---|
|00-moving-compress-i0|0.931922|5.970697–6.001030|6.975494–6.993202|
|01-moving-compress-i1|11.585183|5.901321–5.913279|6.943701–6.951576|
|02-moving-bandwidth-i0|2.147178|5.429194–5.438777|6.974910–7.000035|
|03-moving-bandwidth-i1|3.009556|2.981765–2.994056|6.560158–6.569491|

28개 공통 outage lower는 모두 양수지만 bandwidth1회의6ms 공통 구간은2.981765–2.994056ms로 짧다. 따라서 exact realized profile 일치나 전역 timing fidelity를 승인하지 않는다. 다른 block의11.585183ms 최대 skew는6ms edge 자체의 값이 아니므로 그 값만으로 공통 outage가 없다고 추론하지 않는다. performance_eligible=False와 manual fidelity 미승인을 유지한다.

7개 phase와30개 second별 본문 통계,18/23초 edge 뒤 첫 source-made 본문을 저장 complete_frames에서 재검산했다. 마지막23–30초 dashboard candidate27.1/28.1FPS 대 SSH107.6/107.4FPS다. 그러나 source-made≥23초 첫 본문 지연은 candidate143.346/33.834ms 대 SSH2289.194/2297.999ms다. SSH catch-up과 실제 source draw/의도된60Hz schedule의 밀림을 구분해야 하며,60Hz 초과를 곧바로 신선한 정상속도 회복으로 해석하지 않는다. source-made 기준은 frame intended_epoch 기준의 schedule 신선도와 다르다.49 trace가 아직 없어 pacing/probe/reset/queue 원인은 모른다.

자료: `moving45-actual-audit.json`, `moving45-actual-metrics.csv`; 스크립트 `moving45-actual-audit.py`. 실패한44partial32/46partial4 및47connect8,49pre-job alias 실패는 이 passing112jobs와 합치지 않는다.
