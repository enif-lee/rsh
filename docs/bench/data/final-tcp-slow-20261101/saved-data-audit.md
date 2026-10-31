독립 저장 자료 감사 PASS: 48 jobs/12 blocks, 실제 본문 7652개 / uniform header 7672개. 소스 19765개/161832868B, pending write 6개, pending body join 0개. 원본 데이터 수정과 새 terminal replay/Docker/실측은 하지 않았다.

양쪽 endpoint 파일 192, live-server 96, strict network 96개를 검증했다. 고정 source/analysis/hash, source nonce/PID/clock bracket, all4 window closure→pidfd/SIGUSR2→원래5초 수집, 실제 본문N/T/scene/source join, raw capture read 구조, FPS/1%·5%low/age/maxgap/per-second/CSV 산술이 일치했다. Pending 반환 시각은 null을 유지했다. Resource는 raw job→summary→CSV 동일성과 단위 검증 범위이며 독립 raw 카운터 재측정이 아니다.

Main-A/B는 최종 CAN+PTY-safe4f98/c25 동일 바이너리이고 candidate는 combined295f/fb04다. 4병렬×2회, CPU30은 단일 admission이며 runtime CPU/외부프로세스 값은 문맥이다. FPS1%/5%low와avg를 주로 비교하고 CPU/RSS는 보조로 둔다. 개별 손해로 자동기각하거나 임의 점수를 만들지 않는다. 데이터 유효성과 최종 승격은 별개다.

각 셀 순서: **1%low / 5%low / avg FPS**. 두 반복과 SSH·동일 mainA/A를 모두 보존했다.

|Case|SSH|main-A|main-B|candidate|
|---|---|---|---|---|
|00-lowbw-compress-i0|6.5 / 7.1 / 23.4|42.3 / 45.6 / 55.9|28.2 / 31.1 / 55.1|40.2 / 46.8 / 60.0|
|01-lowbw-compress-i1|6.9 / 7.2 / 23.5|41.1 / 45.9 / 55.9|29.5 / 41.7 / 57.2|43.2 / 46.9 / 60.0|
|02-lowbw-bandwidth-i0|5.8 / 6.5 / 10.0|16.2 / 18.4 / 21.5|12.0 / 18.7 / 22.6|13.7 / 17.8 / 20.2|
|03-lowbw-bandwidth-i1|5.4 / 6.4 / 9.9|13.6 / 17.8 / 20.7|11.5 / 19.3 / 22.9|14.3 / 18.1 / 21.1|
|04-verylow-compress-i0|2.6 / 5.1 / 6.0|25.2 / 26.4 / 33.6|25.9 / 27.4 / 34.6|40.3 / 46.3 / 60.0|
|05-verylow-compress-i1|1.5 / 5.2 / 6.1|25.9 / 26.8 / 33.9|25.5 / 27.2 / 34.4|40.1 / 45.5 / 60.0|
|06-verylow-bandwidth-i0|1.3 / 1.3 / 2.5|5.3 / 5.5 / 6.0|2.9 / 5.4 / 5.7|5.5 / 5.5 / 5.9|
|07-verylow-bandwidth-i1|1.8 / 1.8 / 2.4|5.4 / 5.5 / 5.9|5.5 / 5.6 / 5.9|5.5 / 5.5 / 6.0|
|08-slow64k-compress-i0|1.1 / 1.1 / 1.4|8.9 / 12.7 / 15.6|11.6 / 12.3 / 16.0|5.9 / 12.3 / 15.4|
|09-slow64k-compress-i1|1.0 / 1.0 / 1.5|11.1 / 12.9 / 16.0|11.6 / 12.9 / 15.9|8.8 / 12.7 / 15.7|
|10-slow64k-bandwidth-i0|null / null / 0.5|1.3 / 1.3 / 1.4|1.3 / 1.3 / 1.4|1.4 / 1.4 / 1.6|
|11-slow64k-bandwidth-i1|null / null / 0.6|1.4 / 1.4 / 1.6|1.3 / 1.3 / 1.4|1.3 / 1.3 / 1.6|

1M·256k dashboard candidate60/60avg 및40.1–43.2의1%low/45.5–46.9의5%low 이득이 크다. 그러나1M random은20.2/21.1avg로 main20.7–22.9와 겹치거나 낮고,64k dashboard1%low5.9/8.8은 main8.9–11.6보다 낮다. candidate1M random p95age117/151ms와 main118–123ms를 함께 보존한다. SSH64k random4/5body는 low 둘 다 null이며0으로 바꾸지 않는다. p95age가 없는11jobs도 null이다.

자료: `final41-actual-audit.json`, `final41-actual-metrics.csv`; 스크립트 `final4041-actual-audit.py`. 실패한44partial32/46partial4 및47connect8,49pre-job alias 실패는 이 passing112jobs와 합치지 않는다.
