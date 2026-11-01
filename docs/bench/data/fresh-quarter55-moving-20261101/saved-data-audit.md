58 moving 저장자료 독립감사 PASS. 16 jobs/4 blocks, actual body12331/uniform header12378, source26729 records/218387531 B. Pending write1개 end=null/body join0, zero body와 누락 low/전체-window age95는0이다. 원자료 수정과 새 terminal/body-oracle replay·Docker·실측은0이다.

COMBINED295f/fb04 A/A와 quarter55 후보515d/6d7f를 비교했다. Main-A/B는 actualmain4f98이 아니다. Source/analysis/parent manifests, raw capture 구조·관측 FRAME N/T/scene, source prewrite clock/nonce/PID, all-four closure와 pidfd SIGUSR2·원래5초 수집, 완료 body 통계와 acceptance-per-rep.csv를 검산했다. Endpoint64/live server32/strict network32개를 확인했다. 자원은 raw job→summary→CSV 산술/단위 범위이며 원시 counter/RSS samples를 새로 재측정한 것이 아니다.

각 셀은 **1%low / 5%low / avg FPS**다.

|Case|SSH|combined-A|combined-B|quarter55|
|---|---|---|---|---|
|00-moving-compress-i0|3.7 / 31.5 / 48.0|2.5 / 7.8 / 21.4|2.9 / 9.4 / 23.7|3.6 / 8.6 / 27.9|
|01-moving-compress-i1|6.9 / 32.0 / 49.4|2.8 / 6.9 / 21.2|2.4 / 9.3 / 22.0|3.4 / 7.3 / 25.2|
|02-moving-bandwidth-i0|3.5 / 13.6 / 27.0|2.2 / 6.0 / 19.0|1.9 / 6.3 / 19.8|1.7 / 7.4 / 19.6|
|03-moving-bandwidth-i1|3.5 / 13.4 / 27.6|2.4 / 6.5 / 18.9|2.2 / 7.0 / 19.1|1.9 / 7.1 / 21.3|

Dashboard 후보 avg27.9/25.2와 low1 3.6/3.4는 두 A/A보다 높지만,5%low8.6/7.3은 B9.4/9.3보다 낮다. SSH는 avg48.0/49.4와5%low31.5/32.0으로 여전히 높다. Random 후보 low1 1.7/1.9는 A/A보다 낮고,5%low7.4/7.1은 A/A보다 높다. Random avg19.6/21.3은 반복0 A19.0보다 높고 B19.8보다 낮으며 반복1 양 A/A18.9/19.1보다 높다. FPS 그룹의 손익을 함께 남긴다.

Final23–30초 dashboard 후보 avg43.9/41.9 대 A/A26.1–29.1/27.3–29.4다. 후보5%low26.9/23.2는 A/A18.2–19.1/19.1–19.6보다 높지만, 반복1 low1 10.8은 A11.1/B11.6보다 낮다. 같은 phase SSH112.7/114.1FPS는 밀린 body catch-up이며 steady fresh generation60과 같지 않다. Source가 실제 만든 시각으로23초 이후 새 body를 정한 최초 수신은 후보104.8/102.0ms, A/A101.0–101.5/86.3–132.2ms, SSH2202.9/2114.8ms다. Source-made freshness와 의도된 source schedule/화면 체류 지연을 혼동하지 않는다.

Native42 all-eight workers ready→공통 future T0/fixed30s와131 points×8 ACK records×4 blocks, worker PID/exe/plan/stdout/restoration 및 source T0를 검사했다. 모든28 outage bounds와524 point skews를 원본 ACK에서 재계산해 저장값과 일치했다. 아래는8방향 공통 outage의 ACK 기반 하한–상한(ms)이다. Packet 적용 시각의 정확한 관측이나 프로토콜 간 동일 패킷 손실을 뜻하지 않는다.

|Block|max8-worker skew ms|6ms intended|7ms intended|1s intended|
|---|---:|---|---|---|
|00-moving-compress-i0|1.784010|5.974365–6.003699|6.974246–7.000871|999.910236–1000.025944|
|01-moving-compress-i1|2.633097|5.969532–6.002158|4.361023–4.372940|999.922944–999.968653|
|02-moving-bandwidth-i0|5.970074|5.052652–5.058986|6.973663–7.001246|998.088808–998.131476|
|03-moving-bandwidth-i1|2.449264|5.977657–6.001782|6.802286–6.813037|999.902151–999.955027|

모든 공통 하한은 양수지만 dashboard 반복1의7ms는4.361–4.373ms, random 반복0의6ms는5.053–5.059ms다. Coverage valid와 fidelity 승인을 구분하며 **performance_eligible=False, timing_fidelity_accepted=False, final_selection=None**을 유지한다. Console validTrue는 데이터 무결성이다. 이 자료를 정적 결과나 다른 moving cohort와 pool하거나 exact profile equivalence로 쓰지 않는다.

CPU 시작 admission은1 sample18%이고 각 phase 첫 total<30% sample을 확인했다. Runtime CPU/프로세스 부하는 문맥이며 affinity 증명이 아니다. Source 1개 pending payload는 완료 body join에 없으므로 pending body 수신을 입증하지 않는다. Untraced58은 quarter 분기의 활성화·회복 인과를 직접 보여주지 않는다. Old19 동일 predicate의300ms random24.5avg/4.0·12.0low와56/57 dashboard low·age 비용을 유지하며, FPS1/5low+avg 우선/CPU RSS 보조 정책으로 전체 손익을 평가해야 한다.

근거는 moving58-actual-audit.py/json, moving58-actual-metrics.csv다. JSON에 source/capture hashes, partial body/header, 모든 phase/second/fresh-edge 산술과 bounds를 보존했다. Low는 충분한 인접 gap이 있을 때만 산출하며 window 양쪽 boundary gap은 maxgap에 따로 포함한다. Age는 완료 body에 조건부이고 입력/GPU/모든 터미널 presentation을 뜻하지 않는다.
