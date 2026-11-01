69 moving 저장자료 독립감사 PASS. 16 jobs/4 blocks, actual body12980/uniform header12999, source26052 records/210664124 B다. Pending write2개 end=null/body join0, zero body 및 full-window low·age95 누락0이다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

Actual60 fa934/4b52 A/A 대65quiet781b/3d4c의 한 정책 증가분이다. 각 scene 실제 반복0 [SSH,A,B,C], 반복1 [C,B,A,SSH]를 확인했다. Source/capture hashes·FRAME N/T/scene·prewrite clock/nonce/PID·pidfd원래5초/all-four fences, body FPS/low/age/gap/phase/second/fresh-edge와 CSV 산술이 일치했다. Endpoint64/live server32/strict network32개다. 두 pending payload의 actual body join은 없어 pending body 수신 증거로 쓰지 않는다. 자원은 raw job→summary→CSV 동일성과 단위/공식 범위이며 counter/RSS sample 재측정은 아니다.

각 셀은 **1% low /5% low /avg FPS**다.

|Case|SSH|baseline60-A|baseline60-B|quiet65|
|---|---|---|---|---|
|00-moving-compress-i0|3.8 / 29.9 / 47.9|2.6 / 12.4 / 28.5|3.7 / 7.4 / 26.7|3.2 / 7.0 / 26.4|
|01-moving-compress-i1|6.0 / 27.2 / 48.1|3.0 / 12.7 / 29.8|4.3 / 11.6 / 30.7|3.5 / 13.8 / 31.2|
|02-moving-bandwidth-i0|3.6 / 7.3 / 16.5|2.9 / 7.6 / 22.6|2.5 / 8.0 / 21.4|2.7 / 7.5 / 21.9|
|03-moving-bandwidth-i1|3.3 / 12.4 / 16.5|2.4 / 6.6 / 21.1|2.9 / 6.8 / 21.8|2.3 / 7.5 / 21.7|

Dashboard 반복0 후보3.2/7.0/26.4는 A2.6/12.4/28.5 대비1%low만 높고, B3.7/7.4/26.7보다 세 지표가 낮다. 반복1 후보3.5/13.8/31.2는 A3.0/12.7/29.8보다 세 지표가 높지만 B4.3/11.6/30.7보다1%low는 낮다. Random 반복0 후보2.7/7.5/21.9는 A2.9/7.6/22.6보다 세 지표가 낮고 B2.5/8.0/21.4 대비5%low만 낮다. 반복1 후보2.3/7.5/21.7는 A2.4/6.6/21.1 대비1%low만 낮으며 B2.9/6.8/21.8 대비5%low만 높다. 개선과 손해를 A/A 및 반복별로 보존한다.

SSH dashboard avg47.9/48.1·5%low29.9/27.2는 후보보다 높다. 반면 이번 random SSH는 avg16.5/16.5로 모든 RSH보다 낮으며 final23–30초 body0/0 및 low·age95=NULL이며, 전체30초 창의 maxgap은13115/13203ms다. 이전 moving SSH 약27FPS cohort와 pool하거나 원인을 추측하지 않는다. Full-window SSH body494/495 대 uniform header495/496도 그대로 보존한다. Dashboard SSH final23–30초107/107FPS는 stale catch-up을 포함하고 steady fresh60을 의미하지 않는다.

후보 full-window age95는 dashboard129/151ms 대 A139/164·B134/210ms, random214/203ms 대 A237/244·B225/290ms로 낮다. Final23–30초 dashboard 후보11.9/24.0/42.1은 A22.3/29.1/44.4·B15.2/28.0/43.4보다 세 지표가 낮다. 반복1 후보16.5/26.7/46.6은 A10.4/24.7/42.4보다 높고 B18.7/26.5/43.4 대비1%low만 낮다. Random final23–30초 후보7.2/11.1/26.6은 A5.5/9.8/29.1 대비avg만 낮고 B7.5/11.2/26.1 대비lows가 낮다. 반복1 후보5.5/10.1/25.4는 A5.7/11.6/25.4 대비lows가 낮고 avg동률이며 B5.5/10.3/26.4 대비1%low동률/나머지 낮다.

Source가 실제23초 이후 만든 첫 dashboard body의 edge→arrival delay는 후보76.7/54.8ms 대 A84.5/68.8·B94.5/68.9ms다. Random은 후보144.5/95.2ms 대 A123.8/66.7·B95.3/125.7ms로 혼합이다. 이 지표는 source 생산 위상도 포함하며 intended schedule 지연·GPU·화면체류 지연과 다르다. 30초 전체와 회복 phase, source-made freshness를 구분한다.

Native42 common futureT0/fixed30s/all-eight-ready와131points×8ACK×4blocks를 확인했다. Raw ACK에서 모든28공통 outage bounds와524point skews를 재계산해 저장값과 일치했다. Worker PID/exe/plan/stdout/restoration/sourceT0 coverage를 검증했다. 아래는8방향 common overlap의 ACK 기반 하한–상한(ms)이다.

|Block|max8-worker skew ms|6ms intended|7ms intended|1s intended|
|---|---:|---|---|---|
|00-moving-compress-i0|2.815142|5.970036–6.001162|6.977834–7.000834|999.938798–999.981173|
|01-moving-compress-i1|1.436759|5.965911–6.001245|6.229829–6.235704|999.944465–1000.035216|
|02-moving-bandwidth-i0|2.418556|5.437074–5.444324|6.587123–6.596290|998.420540–1000.044467|
|03-moving-bandwidth-i1|4.024066|5.976953–6.001036|6.975250–6.999626|999.955174–1000.032008|

6ms common overlap 최저는5.437074–5.444324ms이고,7ms 최저는6.229829–6.235704ms다. Max skew만으로 특정 outage의 공통 적용 여부를 판단하지 않는다. Coverage valid는 정확한 packet 적용시각·프로토콜별 동일 packet loss·global fidelity 승인이 아니다. **performance_eligible=False, timing_fidelity_accepted=False, final_selection=None**을 유지한다. Console validTrue는 데이터 무결성이다.

CPU admission은1sample9%와 각 phase 첫 total<30%를 확인했고 runtimeCPU는 문맥이다. Source65 Linux149pass/2ignored·control새quiet2pass1expectedfail+retainedmargin3/quarter3pass와 별도 Mac148pass/2ignored는 root source 기능 증거이며 본감사는 새 tests/build0이다. FPS1%/5%low·avg 우선/CPU RSS 보조로 비용을 검토하되 개별 손해 자동탈락이나 임의 점수를 만들지 않는다. Untraced 자료로 quiet 활성화/인과/atomicity를 단정하지 않는다. Oldquiet18FPS 및66/67/68의 비용과 별도120Hz70을 보존하고 cohort를 pool하지 않는다.

근거는 moving69-actual-audit.py/json, moving69-actual-metrics.csv다. 원본 hash·resource scalar·partial body/header·window boundary/maxgap·모든 phase/fresh-edge·28bounds를 JSON에 남겼다.
