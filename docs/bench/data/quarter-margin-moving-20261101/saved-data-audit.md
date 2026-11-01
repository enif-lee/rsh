63 moving 저장자료 독립감사 PASS. 16 jobs/4 blocks, actual body12340/uniform header12393, source26684 records/218147700 B. Pending write1개 end=null/body join0, zero body/전체-window low·age95 누락은0이다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

COMBINED295f/fb04 A/A와 quarter+margin60 fa934/4b52를 비교했다. Main-A/B는 actualmain4f98이 아니다. Source/capture hashes·FRAME N/T/scene·prewrite clock/nonce/PID·pidfd원래5초/all-four fences, 완료 body/low/age/gap/phase/second/fresh-edge와 CSV 산술을 확인했다. Endpoint64/live server32/strict network32개다. Source1개의 pending payload는 실제 body join이 없어 pending body수신 증거로 쓰지 않는다. 자원은 raw job→summary→CSV 동일성과 단위/공식 범위이며 counter/RSS sample 재측정은 아니다.

각 셀은 **1%low / 5%low / avg FPS**다.

|Case|SSH|combined-A|combined-B|quarter+margin60|
|---|---|---|---|---|
|00-moving-compress-i0|6.9 / 25.4 / 48.0|2.6 / 10.5 / 20.5|2.9 / 7.6 / 23.0|3.2 / 8.7 / 26.2|
|01-moving-compress-i1|3.6 / 20.2 / 47.9|2.8 / 9.2 / 20.2|2.9 / 6.8 / 21.5|3.6 / 13.4 / 31.1|
|02-moving-bandwidth-i0|3.4 / 13.7 / 27.1|1.8 / 6.3 / 18.4|3.1 / 8.6 / 20.3|2.2 / 6.7 / 20.9|
|03-moving-bandwidth-i1|3.4 / 13.3 / 27.3|2.6 / 7.2 / 20.2|2.3 / 7.1 / 17.7|2.9 / 6.6 / 21.0|

후보 avg는 네 case 모두 두 A/A보다 높다. Dashboard26.2/31.1 대 A20.5/20.2·B23.0/21.5, random20.9/21.0 대 A18.4/20.2·B20.3/17.7이다. Dashboard 반복1 low3.6/13.4는 두 A/A보다 높지만 반복0의5%low8.7은 A10.5보다 낮다. Random 반복0 low1 2.2는 B3.1보다 낮고5%low6.7도 B8.6보다 낮다. Random 반복1 low1 2.9는 두 A/A보다 높지만5%low6.6은 A7.2/B7.1보다 낮다. SSH의 full-window avg와5%low가 네 case 모두 더 높다. 유리한 avg만으로 tail 비용을 지우지 않는다.

후보 full-window dashboard age95 129/183ms 대 A95/131·B129/154ms의 비용이 있다. SSH의11301/11423ms와는 다른 freshness/FPS tradeoff다. Final23–30초 dashboard 후보41.3/45.0FPS 대 A32.0/27.7·B26.7/27.0이며, 후보 low11.5/24.6와21.4/29.6은 두 A/A보다 높다. Random 이 phase의 avg27.0/26.3도 A23.9/25.6·B25.7/24.4보다 높지만 후보1%low3.9/7.1은 두 A/A보다 낮고, 반복1 5%low9.7도 A15.6/B13.5보다 낮다. SSH dashboard106.6/107.4FPS는 stale catch-up을 포함하고 steady fresh60을 뜻하지 않는다. Source가 실제23초 이후 만든 dashboard 첫 body의 후보 수신 delay53.5/83.4ms는 A491.9/135.8·B85.5/66.1ms와 비교해 혼합이다. Source-made freshness와 intended source schedule/GPU/화면체류 지연을 구분한다.

Native42 공통 futureT0/fixed30s/all-eight-ready와131points×8ACK×4blocks를 검사했다. Raw ACK에서 모든28공통 outage bounds와524point skews를 재계산해 저장값과 일치했다. Worker PID/exe/plan/stdout/restoration/sourceT0 coverage를 유지한다. 아래는8방향 common overlap의 ACK 기반 하한–상한(ms)이다.

|Block|max8-worker skew ms|6ms intended|7ms intended|1s intended|
|---|---:|---|---|---|
|00-moving-compress-i0|1.578091|5.121486–5.133653|6.977107–7.000399|999.961923–1000.017340|
|01-moving-compress-i1|1.454022|5.965549–6.002259|6.973815–7.000108|998.625950–999.268751|
|02-moving-bandwidth-i0|3.256976|5.134329–5.144370|6.978163–6.998788|999.924985–999.975194|
|03-moving-bandwidth-i1|2.801598|3.188434–3.196726|6.699787–6.707787|998.941225–998.982559|

Random 반복1의6ms common bound는3.188–3.197ms다. Coverage valid는 packet 적용시각의 정확성·프로토콜별 동일 패킷 손실·global fidelity 승인이 아니다. **performance_eligible=False, timing_fidelity_accepted=False, final_selection=None**을 유지하며 console validTrue는 데이터 무결성이다. 다른 moving/static cohort와 수치를 pool하지 않는다.

CPU admission은1 sample29%와 각 phase 첫 total<30%를 확인했고 runtimeCPU는 문맥이다. 실험은295f 대비 두 정책 결합이며 quarter increment versus48/additivity/인과를 분리하지 않는다. Existing61 role-parity 교란과 old19/51/53/57·현재61/62 비용을 보존한다. Source가 untraced이므로 활성화/원인 단정은 없다. FPS1/5low·avg 우선/CPU RSS 보조로 전체 손익을 검토하며 개별 손해 자동탈락이나 임의 점수를 만들지 않는다.

근거는 moving63-actual-audit.py/json, moving63-actual-metrics.csv다. JSON에 원본 hash, 부분 body/header, window boundary/maxgap, 모든 phase/fresh-edge와28bounds를 남겼다. Source/core 기능146pass/2ignored 및 controlquarter2pass1expectedfail+margin3pass는 별도의 root 증거이며 본 감사는 새 tests/build를 하지 않았다.
