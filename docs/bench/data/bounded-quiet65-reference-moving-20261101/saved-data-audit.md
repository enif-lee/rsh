73 moving 최종 기준 비교 저장자료 독립감사 PASS.16 jobs/4 blocks, actual body10834/uniform header10871, source26366 records/214378810 B다. Pending write1개 end=null/body join0, zero body 및full-window low·age95 누락0이다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

기준 A/A는 **CAN+PTY correctness-fixed4f98/c25(24248+PR4/5)**이며 literal current git main24248이 아니다. 후보65quiet781b/3d4c 전체 정책 스택 비교이지60fa934 대비 quiet 증가분/정책 간 additivity 실험이 아니다. 두 artifact는 기존 개별 build로 이번 동일시점/컨테이너 matched rebuild가 아니다.

Actual 각 scene 반복0 [SSH,A,B,C], 반복1 [C,B,A,SSH]·binary/PID/source/capture hashes·FRAME N/T/scene·prewrite clock/nonce·원래5초 pidfd/all-four fences·완료 body FPS/lows/age/gap/phase/second/fresh-edge와CSV 산술이 일치했다. Endpoint64/live server32/strict network32개다. Pending1개는 actual body join0이므로 pending body수신 증거가 아니다. 자원은 raw job→summary→CSV 동일성과 단위/공식 범위이며 원시RX/CPUcounter/RSSsample 독립 재측정은 아니다.

각 셀은 **1%low /5%low /avg FPS**, requested60Hz다.

|Case|SSH|4f98 reference-A|4f98 reference-B|quiet65 stack|
|---|---|---|---|---|
|00-moving-compress-i0|3.6 / 32.5 / 47.9|2.5 / 5.4 / 15.2|2.3 / 7.4 / 18.5|3.2 / 8.4 / 26.9|
|01-moving-compress-i1|6.3 / 27.0 / 48.2|2.8 / 5.0 / 15.7|2.8 / 5.3 / 18.0|3.4 / 9.8 / 28.2|
|02-moving-bandwidth-i0|3.5 / 16.6 / 16.5|1.6 / 4.8 / 14.2|1.5 / 5.1 / 12.9|2.5 / 6.5 / 21.7|
|03-moving-bandwidth-i1|2.6 / 12.8 / 27.2|1.9 / 6.0 / 15.9|1.4 / 3.7 / 13.9|2.6 / 7.1 / 20.3|

후보는 이번 네 full-window case 모두 양 A/A보다1%low·5%low·avg가 높다. Dashboard26.9/28.2avg 대 A15.2/15.7·B18.5/18.0, random21.7/20.3 대 A14.2/15.9·B12.9/13.9다. 후보 full-window age95도dashboard131/140ms 대 A164/204·B135/164, random208/222ms 대 A354/268·B356/339ms보다 낮다. 이 사실은4f98 기준의 이 cohort 관측이며 다른 moving cohort와 수치를 pool하거나 quiet 단독인과를 입증하지 않는다.

SSH와는 tradeoff가 남는다. Dashboard SSH47.9/48.2avg·5%low32.5/27.0·1%low3.6/6.3는 후보보다 높고 age9512033/11258ms는 훨씬 크다. Random 반복0 후보avg21.7은SSH16.5보다 높지만lows2.5/6.5는SSH3.5/16.6보다 낮다. 반복1 후보2.6/7.1/20.3은SSH2.6/12.8/27.2 대비1%low동률/5%low·avg열세다. 후보 random body651/header653·610/header618도 보존한다.

Final23–30초 후보avg는dashboard42.4/42.7 대 A23.3/23.3·B24.0/21.3, random27.7/24.4 대 A20.0/20.3·B21.3/22.4로 높다. 다만dashboard 반복0 low1 10.9는 A11.1/B11.4보다 낮고 random 반복0 low1 4.9는 A7.6/B9.2보다 낮다. Random 반복0 low5 12.1은 A10.6보다 높고 B14.9보다 낮으며 반복1 low1 8.6은 A동률/B9.7보다 낮고low5 10.0은 A10.3/B12.7보다 낮다. 회복avg만으로 tail비용을 지우지 않는다.

SSH dashboard final23–30초112.3/107.4FPS는 stale catch-up을 포함하고 steadyfresh60을 의미하지 않는다. SSH random 반복0은 이7초 phase body0/low·age95NULL이고 전체30초 창 maxgap13129ms다. 반복1은45.7FPS/age9514175ms다. 다른 cohort의 SSH16.5/27FPS 차이 원인을 추측하지 않는다. Source가 실제23초 이후 만든 첫 dashboardbody edge→arrival delay는 후보134.7/111.6ms 대 A77.6/110.8·B81.4/140.8ms로 혼합이다. Random은후보90.7/104.3ms 대 A85.2/128.7·B98.4/118.2ms로 혼합이다. Source생산위상도 포함하는 지표며 intendedschedule/GPU/화면체류 latency와 다르다.

Native42 common futureT0/fixed30s/all-eight-ready와131points×8ACK×4blocks를 확인했다. Raw ACK에서 모든28공통 outage bounds와524point skews를 재계산해 저장값과 일치했다. WorkerPID/exe/plan/stdout/restoration/sourceT0 coverage를 검증했다. 아래는8방향 common overlap의 ACK 기반 하한–상한(ms)이다.

|Block|max8-worker skew ms|6ms intended|7ms intended|1s intended|
|---|---:|---|---|---|
|00-moving-compress-i0|3.181224|5.791028–5.806028|6.974867–6.999909|999.920917–1000.026001|
|01-moving-compress-i1|1.064630|4.926940–4.936899|6.366864–6.378364|999.884084–999.925375|
|02-moving-bandwidth-i0|1.978553|5.653444–5.664777|6.949749–6.988458|999.929164–999.989456|
|03-moving-bandwidth-i1|1.563134|5.916452–5.930326|6.975249–7.003500|999.183118–999.913955|

6ms common overlap 최저4.926940–4.936899ms,7ms 최저6.366864–6.378364ms다. Coverage valid는 정확한packet 적용시각·프로토콜별 동일packet loss·globalfidelity승인이 아니다. **performance_eligible=False,timing_fidelity_accepted=False,final_selection=None**을 유지하며 consolevalidTrue는 데이터 무결성이다. Moving은descriptivecohort이고 staticeligible/120Hz와 pool하지 않는다.

CPU admission1sample17%와 각 phase 첫 total<30%를 확인했고 runtimeCPU는 문맥이다. FPS1%/5%low·avg 우선/CPU RSS 보조로 모든반복/A/A/age/traffic/partial/boundary 비용을 검토하되 개별손해 자동veto나 숫자가중치를 만들지 않는다. Old19/51/53/57·67–70advanced60비교·71/72costs는 별도보존한다. Untraced 자료로 분기활성화/인과/atomicity/실제20ms service상한을 단정하지 않는다.74별도120Hz 최종기준 비교는 아직 남아 있다.

근거는 mainref73-actual-audit.py/json, mainref73-actual-metrics.csv다. Original hashes·resource scalar·partial/body/header·window boundary/maxgap·actualslot·모든phase/fresh-edge·28bounds를JSON에 남겼다. 기준Linux129pass/2ignored와 후보Linux149pass/2ignored·별도Mac148pass/2ignored는 priorroot source기능 증거며 본감사 새tests/build0이다.
