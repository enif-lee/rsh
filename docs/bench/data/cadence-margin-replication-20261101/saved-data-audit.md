Cadence53 저장자료 독립감사 PASS: 16jobs/4blocks, body4503/header4511, source6890/56014900B. Pending write0개는null이며 실제pending bodyjoin0개다. 원본입력/결과수정0, 신규terminal replay/Docker/실측0이다.

4병렬×2평가의plannedidentity·source/analysis/parentSHA·rawcapture구조와N/T/scene·sourceclock/nonce/PID·all4close→pidfdSIGUSR2→기존5초수집·complete-frame FPS/low/age/maxgap/second/CSV를 대조했다. Endpoint파일64/live-server32/strictnetwork32개가일치했다. CPU30은admission만이며runtimeCPU는문맥이다. Resource는raw job→summary→CSV동일성과단위를검증했으며RX/CPU원본counter와RSSsample을독립재측정한것은아니다.

Main-A/B는동일 COMBINED295f/fb04 reference다(실제main4f98아님). Candidate는cadence48 ea6c/e34a다. Product검증143total=113library+4CLI+25session+1(64spawn),2ignored와oldcontrol2pass/1expectedfail은별도근거다. 50/51/53은기존4f대295f 공개112와합치지않는다. FPS1%/5%low와avg를주로보며CPU/RSS는보조다. 개별회귀를자동탈락시키거나임의가중점수를만들지않고final_selection=None을유지한다.

각셀: **1%low / 5%low / avg FPS**.
|Case|SSH|combined-A|combined-B|cadence0.9|
|---|---|---|---|---|
|00-lowbw-compress-i0|6.4 / 7.1 / 23.6|43.3 / 46.8 / 60.0|42.5 / 46.0 / 60.0|42.9 / 46.3 / 60.0|
|01-lowbw-compress-i1|6.4 / 7.1 / 23.7|41.5 / 45.4 / 60.0|42.2 / 45.6 / 60.0|44.9 / 47.9 / 60.0|
|02-lowbw-bandwidth-i0|4.6 / 6.4 / 9.9|13.4 / 18.1 / 22.4|13.4 / 18.6 / 22.6|13.3 / 18.0 / 21.5|
|03-lowbw-bandwidth-i1|5.4 / 6.5 / 10.0|13.2 / 18.8 / 22.9|17.5 / 18.6 / 22.6|11.4 / 19.4 / 23.6|

53은51의1M 증상을 다시 확인하도록 사전 선언한 focused16jobs다. 51에 후행 pool하지 않고 별도로 보고한다. Random 후보0회는13.3/18.0/21.5로 combined13.4/18.1–18.6/22.4–22.6보다 조금 낮다. 1회는11.4/19.4/23.6으로 combined1%low13.2–17.5보다 낮지만5%low18.6–18.8과avg22.6–22.9보다 높다. 51의 큰1%low 저하8.5/9.7은 같은 형태로 두 회 모두 반복되지 않았지만 후보를 면책하는 결과도 아니다. Dashboard1회44.9/47.9/60은 combined보다 높다. 모든 원값을 남긴다.

52의 별도 functional trace는 window branch30을 행사하지 않았다. 따라서51의 tail 차이를 cadence0.9의 직접 작용으로 귀속할 수 없다. 계측 timing, 초기 state, 동일 binary의A/A 편차가 남는다. 이번 untraced 자료로 branch 활성화나 인과, SSH 전면 동등을 주장하지 않는다.

자료:`cadence53-actual-audit.json`, `cadence53-actual-metrics.csv`; 재계산 script:`cadence505153-actual-audit.py`. Null, partial body/header 차이, window 끝 gap, 두 반복과A/A를 보존했다.