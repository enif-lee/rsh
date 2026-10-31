Cadence51 저장자료 독립감사 PASS: 48jobs/12blocks, body8673/header8696, source19775/161882730B. Pending write6개는null이며 실제pending bodyjoin0개다. 원본입력/결과수정0, 신규terminal replay/Docker/실측0이다.

4병렬×2평가의plannedidentity·source/analysis/parentSHA·rawcapture구조와N/T/scene·sourceclock/nonce/PID·all4close→pidfdSIGUSR2→기존5초수집·complete-frame FPS/low/age/maxgap/second/CSV를 대조했다. Endpoint파일192/live-server96/strictnetwork96개가일치했다. CPU30은admission만이며runtimeCPU는문맥이다. Resource는raw job→summary→CSV동일성과단위를검증했으며RX/CPU원본counter와RSSsample을독립재측정한것은아니다.

Main-A/B는동일 COMBINED295f/fb04 reference다(실제main4f98아님). Candidate는cadence48 ea6c/e34a다. Product검증143total=113library+4CLI+25session+1(64spawn),2ignored와oldcontrol2pass/1expectedfail은별도근거다. 50/51/53은기존4f대295f 공개112와합치지않는다. FPS1%/5%low와avg를주로보며CPU/RSS는보조다. 개별회귀를자동탈락시키거나임의가중점수를만들지않고final_selection=None을유지한다.

각셀: **1%low / 5%low / avg FPS**.
|Case|SSH|combined-A|combined-B|cadence0.9|
|---|---|---|---|---|
|00-lowbw-compress-i0|6.5 / 7.2 / 23.7|41.3 / 45.4 / 60.0|40.8 / 45.8 / 60.0|44.1 / 46.8 / 60.0|
|01-lowbw-compress-i1|6.4 / 7.1 / 23.5|41.3 / 46.3 / 60.0|41.2 / 46.6 / 60.0|41.3 / 45.4 / 60.0|
|02-lowbw-bandwidth-i0|5.2 / 6.5 / 10.0|17.5 / 19.0 / 22.7|12.0 / 19.2 / 23.2|8.5 / 18.4 / 22.2|
|03-lowbw-bandwidth-i1|5.1 / 6.3 / 10.0|14.7 / 19.3 / 23.6|13.7 / 19.0 / 23.5|9.7 / 18.7 / 23.2|
|04-verylow-compress-i0|2.6 / 5.1 / 6.0|41.7 / 46.7 / 60.0|42.5 / 47.0 / 60.0|42.0 / 46.9 / 60.0|
|05-verylow-compress-i1|3.0 / 5.2 / 6.1|41.4 / 48.3 / 60.0|41.3 / 45.6 / 60.0|41.6 / 46.8 / 60.0|
|06-verylow-bandwidth-i0|1.5 / 1.5 / 2.4|3.1 / 5.5 / 5.9|3.0 / 5.4 / 5.7|5.5 / 5.5 / 6.0|
|07-verylow-bandwidth-i1|1.5 / 1.5 / 2.5|5.4 / 5.5 / 5.9|5.3 / 5.5 / 5.9|5.4 / 5.5 / 6.0|
|08-slow64k-compress-i0|1.0 / 1.0 / 1.5|9.0 / 11.9 / 15.4|6.3 / 12.5 / 15.4|5.9 / 12.1 / 15.4|
|09-slow64k-compress-i1|1.0 / 1.0 / 1.5|10.3 / 12.4 / 15.6|10.9 / 12.7 / 15.6|8.6 / 12.8 / 16.1|
|10-slow64k-bandwidth-i0|null / null / 0.6|null / null / 1.2|0.8 / 0.8 / 1.5|null / null / 1.2|
|11-slow64k-bandwidth-i1|null / null / 0.6|1.3 / 1.3 / 1.4|null / null / 1.2|0.8 / 0.8 / 1.5|

1M random 후보의1%low8.5/9.7,5%low18.4/18.7,avg22.2/23.2는 combined1%low12.0–17.5,5%low19.0–19.3,avg22.7–23.6보다 낮은 관측이다. Low가 없는5jobs는 frame10이어도 인접 gap9개라면 null이며0으로 바꾸지 않는다. Age95 누락11jobs도 null이다. 최초50 상대 artifact path 준비 실패는 locks/Docker 전0job 장애이며 passing matrix나 zero FPS 사례로 합산하지 않는다.

52의 별도 functional trace는 window branch30을 행사하지 않았다. 따라서51의 tail 차이를 cadence0.9의 직접 작용으로 귀속할 수 없다. 계측 timing, 초기 state, 동일 binary의A/A 편차가 남는다. 이번 untraced 자료로 branch 활성화나 인과, SSH 전면 동등을 주장하지 않는다.

자료:`cadence51-actual-audit.json`, `cadence51-actual-metrics.csv`; 재계산 script:`cadence505153-actual-audit.py`. Null, partial body/header 차이, window 끝 gap, 두 반복과A/A를 보존했다.