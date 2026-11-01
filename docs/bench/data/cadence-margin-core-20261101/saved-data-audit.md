Cadence50 저장자료 독립감사 PASS: 48jobs/12blocks, body22164/header22185, source23040/192593640B. Pending write0개는null이며 실제pending bodyjoin0개다. 원본입력/결과수정0, 신규terminal replay/Docker/실측0이다.

4병렬×2평가의plannedidentity·source/analysis/parentSHA·rawcapture구조와N/T/scene·sourceclock/nonce/PID·all4close→pidfdSIGUSR2→기존5초수집·complete-frame FPS/low/age/maxgap/second/CSV를 대조했다. Endpoint파일192/live-server96/strictnetwork96개가일치했다. CPU30은admission만이며runtimeCPU는문맥이다. Resource는raw job→summary→CSV동일성과단위를검증했으며RX/CPU원본counter와RSSsample을독립재측정한것은아니다.

Main-A/B는동일 COMBINED295f/fb04 reference다(실제main4f98아님). Candidate는cadence48 ea6c/e34a다. Product검증143total=113library+4CLI+25session+1(64spawn),2ignored와oldcontrol2pass/1expectedfail은별도근거다. 50/51/53은기존4f대295f 공개112와합치지않는다. FPS1%/5%low와avg를주로보며CPU/RSS는보조다. 개별회귀를자동탈락시키거나임의가중점수를만들지않고final_selection=None을유지한다.

각셀: **1%low / 5%low / avg FPS**.
|Case|SSH|combined-A|combined-B|cadence0.9|
|---|---|---|---|---|
|00-lan-compress-i0|47.2 / 51.1 / 60.0|43.2 / 46.9 / 60.0|42.4 / 47.1 / 60.0|42.5 / 47.4 / 60.0|
|01-lan-compress-i1|44.6 / 48.7 / 60.0|43.3 / 46.9 / 60.0|43.9 / 47.0 / 60.0|42.4 / 45.8 / 60.0|
|02-lan-bandwidth-i0|46.6 / 50.4 / 60.0|42.3 / 46.6 / 60.0|41.5 / 45.7 / 60.0|41.5 / 46.5 / 60.0|
|03-lan-bandwidth-i1|46.4 / 49.7 / 60.0|41.3 / 46.5 / 60.0|43.8 / 47.6 / 60.0|43.0 / 46.2 / 60.0|
|04-wan-compress-i0|42.4 / 46.5 / 60.0|42.4 / 47.6 / 60.0|40.9 / 46.1 / 60.0|42.1 / 45.8 / 60.0|
|05-wan-compress-i1|41.9 / 47.1 / 60.0|41.3 / 45.4 / 60.0|43.2 / 47.1 / 60.0|42.7 / 46.0 / 60.0|
|06-wan-bandwidth-i0|40.7 / 46.5 / 60.0|20.8 / 44.1 / 53.5|29.7 / 45.4 / 54.5|28.2 / 44.1 / 57.4|
|07-wan-bandwidth-i1|31.1 / 46.8 / 60.0|29.7 / 45.0 / 54.4|39.9 / 43.9 / 54.5|28.8 / 44.6 / 57.1|
|08-highlat-compress-i0|24.2 / 46.9 / 60.0|39.4 / 45.5 / 54.6|29.5 / 36.1 / 52.7|29.6 / 41.4 / 56.9|
|09-highlat-compress-i1|23.7 / 45.6 / 60.0|41.3 / 45.4 / 54.7|43.0 / 46.7 / 54.5|45.6 / 49.4 / 59.0|
|10-highlat-bandwidth-i0|11.9 / 44.0 / 60.0|11.4 / 44.6 / 49.7|24.2 / 44.7 / 49.5|23.2 / 46.2 / 53.6|
|11-highlat-bandwidth-i1|12.6 / 44.8 / 60.0|8.4 / 44.5 / 49.9|10.2 / 45.1 / 50.5|16.1 / 46.9 / 53.4|

300ms random 후보53.6/53.4avg는 combined49.5–50.5보다 높지만 SSH60에는 못 미친다. WAN dashboard는 모든transport60avg에서도 candidate5%low45.8/46.0이 SSH46.5/47.1 및 combined45.4–47.6과 섞인다. LAN을 포함한 위 표의 불리한 셀도 전부 남긴다. 51/53과 별도host window이며 다른matrix 저하를 상쇄하려고pool하지 않는다.

52의 별도 functional trace는 window branch30을 행사하지 않았다. 따라서51의 tail 차이를 cadence0.9의 직접 작용으로 귀속할 수 없다. 계측 timing, 초기 state, 동일 binary의A/A 편차가 남는다. 이번 untraced 자료로 branch 활성화나 인과, SSH 전면 동등을 주장하지 않는다.

자료:`cadence50-actual-audit.json`, `cadence50-actual-metrics.csv`; 재계산 script:`cadence505153-actual-audit.py`. Null, partial body/header 차이, window 끝 gap, 두 반복과A/A를 보존했다.