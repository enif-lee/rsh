64 사전 고정 WAN random8jobs/2blocks 독립감사 PASS. 실제 body3642/header3646/source3840 records·45166104 B, pending/body join/zero body/누락 low·age95는0이다. Source/capture/clock/PID/pidfd5s/all-four fences 및 실제 fa934/4b52 대295f A/A 신원, 완료 body 통계·CSV 산술이 모두 일치한다. Endpoint32/live server16/strict network16개다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

반복0 실제 순서는 [SSH,A,B,C], 반복1은 [C,B,A,SSH]다. Summary뿐 아니라 raw job/deployment의 slot별 artifact SHA와 variant를 확인했고, 감사 script가 이 정확한 두 순서를 별도 assertion으로 검사했다. C/SSH는0↔3, A/B는1↔2를 교대하여 각 role이 odd/even을 한 번씩 지난다. 물리 core pinning/affinity를 증명하지 않는다.

|Eval|Role(slot)|1%low / 5%low / avg FPS|Age95 ms|RX kbit/s|RSS client/server KiB|CPU client/server %|
|---|---|---|---:|---:|---|---|
|0|ssh(0)|31.3 / 46.5 / 60.0|57|5802|7896/18828|0.21/0.22|
|0|main-A(1)|37.9 / 44.8 / 54.9|71|2133|2436/5524|1.02/4.52|
|0|main-B(2)|42.2 / 45.3 / 54.5|71|2126|2500/5560|1.02/4.43|
|0|candidate(3)|29.8 / 46.8 / 58.1|64|2270|2472/5528|1.1/4.71|
|1|ssh(3)|39.0 / 48.1 / 60.0|58|5802|7700/18840|0.23/0.24|
|1|main-A(2)|41.7 / 46.6 / 55.0|71|2141|2728/5976|1.07/4.58|
|1|main-B(1)|36.4 / 45.3 / 54.4|71|2125|2772/6000|1.07/4.64|
|1|candidate(0)|29.5 / 46.0 / 58.4|67|2280|2756/6008|1.17/4.77|

C의1%low는 odd slot3에서29.8, even slot0에서29.5이며 두 반복 모두 모든 A/A보다 낮다. SSH31.3/39.0보다도 낮다. 따라서61에서 남긴 후보의1%low 비용을 odd parity만으로 설명하거나 면책할 수 없다. A/A는 odd slot1에서37.9→36.4, even slot2에서42.2→41.7로 이 작은 진단에서 위치와의 연관이 보인다. 다만2평가와 다른 role·시점/상호작용만으로 안정적 parity 편향/physical core 원인을 확정하지 않는다.

C avg58.1/58.4는 A/A54.4–55.0보다 높고 age95도64/67ms 대71ms로 낮다. C5%low46.8/46.0은 반복0 양 A/A44.8/45.3보다 높고 반복1 B45.3보다 높지만 A46.6보다 낮다. SSH avg60은 더 높고,5%low46.5/48.1과 C 비교도 혼합이다. RX2270/2280 kbit/s 대 A/A2125–2141 및 CPU/RSS 원값도 함께 남겼다. 자원은 raw→summary→CSV 일치/단위 검토 범위이며 별도 counter/RSS sample 재측정은 아니다.

이 진단은 사전 고정8jobs를 끝낸 것이며 후보를 찾기 위한 winner retry/추가 반복이 아니다. 61 결과와 pool하지 않고 기존61 WAN·RTT300·LAN의 모든 손해를 그대로 남긴다. 기존 compact92 A/A 행은 안정적 parity 편향을 지지하지 않았지만 role과parity가 분리되지 않았다는 설계 교란을 보여줬다. 새 진단이 그 과거 관측을 변경하지 않는다.

실험 baseline은 COMBINED295f이며60 두 정책의 결합 효과다. PR7 위 source layout은 quarter increment versus48 또는 가산성 분리 실험이 아니다. Untraced8jobs는 정책 branch 활성화·인과나 전 영역 성능 승격을 입증하지 않는다. FPS1%/5%low·avg 우선, CPU/RSS 보조, age/traffic 비용 별도/자동 단일지표 veto 없음/final_selection=None을 유지한다. CPU 시작 admission1sample18%와 각 phase 첫 total<30%를 확인했고 runtime 부하는 문맥이다.

근거: combo64-actual-audit.py/json, combo64-actual-metrics.csv,64의 prepared/analysis/source/capture hashes. 모든 부정/양수/null/부분본문과 window boundary/maxgap은 JSON/CSV에 보존했다. Root146pass/2ignored 및 controlquarter2pass1expectedfail+margin3pass는 재사용 artifact의 별도 기능 증거이며 본 감사가 새 tests/build를 한 것은 아니다.
