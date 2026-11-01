72 slow 최종 기준 비교 저장자료 독립감사 PASS.48 jobs/12 blocks, actual body7760/uniform header7776, source19754 records/161756817 B다. Pending write6개 end=null/body join0, zero body0이다. Low 누락4jobs(64k random 반복0 SSH5frames/A10/C10,반복1SSH5)·age95누락11jobs는NULL로 보존했다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

기준 A/A는 **CAN+PTY correctness-fixed4f98/c25(24248+PR4/5)**이며 literal current git main24248이 아니다. 후보65quiet781b/3d4c의 전체 정책 스택 비교이지60fa934 대비 quiet 증가분/additivity 실험이 아니다. 기존 개별 build를 연결했으며 이번 동일시점/컨테이너 matched rebuild가 아니다.

Actual 각 net/scene 반복0 [SSH,A,B,C], 반복1 [C,B,A,SSH]·binary/PID/source/capture hashes·FRAME N/T/scene·prewrite clock/nonce·원래5초 pidfd/all-four fences·완료 body FPS/low/age/gap/second/CSV 산술이 일치했다. Endpoint192/live server96/strict network96개다. CPU admission1sample15%와 각 phase 첫 total<30%를 확인했고 runtimeCPU는 문맥이다. 자원은 raw job→summary→CSV 동일성과 단위/공식 범위이며 원시RX/CPUcounter/RSSsample 독립 재측정은 아니다.

각 셀은 **1%low /5%low /avg FPS**, requested60Hz다.

|Case|SSH|4f98 reference-A|4f98 reference-B|quiet65 stack|
|---|---|---|---|---|
|00-lowbw-compress-i0|5.9 / 7.2 / 23.5|43.3 / 46.4 / 56.0|39.8 / 46.9 / 55.6|42.1 / 46.5 / 60.0|
|01-lowbw-compress-i1|6.5 / 7.0 / 23.5|40.5 / 46.9 / 55.9|43.8 / 46.6 / 55.9|43.4 / 46.7 / 60.0|
|02-lowbw-bandwidth-i0|6.3 / 6.5 / 9.7|17.5 / 19.3 / 22.9|17.7 / 18.6 / 22.4|19.2 / 20.9 / 24.4|
|03-lowbw-bandwidth-i1|5.4 / 6.3 / 9.9|16.4 / 18.3 / 21.2|13.6 / 18.6 / 23.2|14.8 / 19.5 / 24.1|
|04-verylow-compress-i0|2.1 / 5.1 / 6.0|23.8 / 27.0 / 34.0|25.3 / 27.7 / 37.6|43.8 / 47.1 / 60.0|
|05-verylow-compress-i1|2.4 / 4.1 / 6.1|24.9 / 26.8 / 35.0|25.1 / 27.7 / 35.9|38.7 / 44.6 / 60.0|
|06-verylow-bandwidth-i0|1.3 / 1.3 / 2.4|5.4 / 5.6 / 5.9|5.4 / 5.5 / 6.0|5.3 / 5.5 / 5.9|
|07-verylow-bandwidth-i1|1.6 / 1.6 / 2.5|5.4 / 5.5 / 6.0|5.4 / 5.5 / 6.0|5.2 / 5.4 / 5.9|
|08-slow64k-compress-i0|1.3 / 1.3 / 1.5|11.5 / 12.6 / 15.6|11.3 / 12.0 / 15.6|6.9 / 12.6 / 15.6|
|09-slow64k-compress-i1|1.3 / 1.3 / 1.4|11.9 / 12.5 / 16.0|11.3 / 12.8 / 15.7|8.9 / 12.5 / 15.7|
|10-slow64k-bandwidth-i0|NULL / NULL / 0.6|NULL / NULL / 1.2|0.8 / 0.8 / 1.4|NULL / NULL / 1.2|
|11-slow64k-bandwidth-i1|NULL / NULL / 0.6|1.3 / 1.3 / 1.4|1.3 / 1.3 / 1.4|0.8 / 0.8 / 1.5|

1M dashboard 후보avg60/60은 A56.0/55.9·B55.6/55.9보다 높다. Lows는 혼합이다: 반복0 후보42.1/46.5는 A43.3/46.4 대비low1낮음/low5높음, B39.8/46.9 대비low1높음/low5낮음이다. 반복1 후보43.4/46.7도 A40.5/46.9 대비low1높음/low5낮음, B43.8/46.6 대비low1낮음/low5높음이다. Age95후보63/63ms는 A74/74·B73/73ms보다 낮다.

1M random 반복0 후보19.2/20.9/24.4는 A17.5/19.3/22.9·B17.7/18.6/22.4보다 세 지표가 높다. 반복1 후보14.8/19.5/24.1은 B13.6/18.6/23.2보다 높고 A16.4/18.3/21.2 대비1%low만 낮다. 후보 age95160/146ms는 A124/124·B120/128ms보다 높다. FPS 이득과18–40ms freshness 비용을 함께 남긴다.

256k dashboard 후보43.8/47.1/60·38.7/44.6/60은 두 A/A의 양반복1%/5%low·avg보다 모두 높고 age9576/76ms는 기준88–89ms보다 낮다. 대신256k random 반복0 후보5.3/5.5/5.9는 A5.4/5.6/5.9 대비lows낮음/avg동률, B5.4/5.5/6 대비low1·avg낮음/low5동률이다. 반복1 후보5.2/5.4/5.9는 A5.4/5.5/6·B5.4/5.5/6보다 세 지표가 낮다. 후보 age95412/420ms도 A404/395·B408/409ms보다 높다.

64k dashboard 반복0 후보6.9/12.6/15.6은1%low가 A11.5/B11.3보다 낮고5%low는A동률/B12.0보다 높으며avg동률이다. 반복1 후보8.9/12.5/15.7은 A11.9/12.5/16 대비low1·avg낮음/low5동률, B11.3/12.8/15.7 대비lows낮음/avg동률이다. Age95후보429/345ms는 반복0 A413/B352보다 높고 반복1 A374/B350보다 낮다. 후보maxgap171/158ms도 A149/129·B130/124ms보다 길다.

Sparse64k random 후보반복0 body10/header12·lowNULL·avg1.2, 반복1 body12/header13·low0.8/0.8·avg1.5다. 반복1avg는 두A/A1.4보다 높지만 lows0.8은 두A/A1.3보다 낮고maxgap1285ms는 A776/B753ms보다 길다. 누락low/age를0으로 만들거나 이점만 고르지 않는다. Source pending6개는 실제 body join0이므로 pending body수신 증거가 아니다.

SSH의 낮은 대역폭 FPS/low는 대체로 후보보다 낮지만 sparseNULL은 비교 불능으로 남긴다. 후보전체SSH동등이나 무회귀를 주장하지 않는다. Observations eligible/final_selection=None이며 데이터 유효성과 최종 승격을 분리한다. FPS1%/5%low·avg 우선/CPU RSS 보조로 모든 반복/A/A/age/traffic/partial/boundary 비용을 검토하되 개별 비용 자동 veto나 숫자가중치를 만들지 않는다. Old19/51/53/57·67–70advanced60비교와120Hzguard 비용은 별도보존하며 pool/승자재시도/원인면책을 하지 않는다. Untraced 자료로 분기활성화/인과/atomicity/실제20ms service상한을 단정하지 않는다.73moving·74별도120Hz 최종비교는 아직 남아 있다.

근거는 mainref72-actual-audit.py/json, mainref72-actual-metrics.csv다. Original hashes·resource scalar·partial/body/header·window boundary/maxgap·actualslot과모든행을JSON에 남겼다. 기준Linux129pass/2ignored와 후보Linux149pass/2ignored·별도Mac148pass/2ignored는 priorrootsource 기능 증거이며 본감사 새tests/build0이다.
