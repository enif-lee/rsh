독립 저장 자료 감사 PASS: 48 jobs/12 blocks, 실제 본문 21249개 / uniform header 21273개. 소스 23040개/192593640B, pending write 0개, pending body join 0개. 원본 데이터 수정과 새 terminal replay/Docker/실측은 하지 않았다.

양쪽 endpoint 파일 192, live-server 96, strict network 96개를 검증했다. 고정 source/analysis/hash, source nonce/PID/clock bracket, all4 window closure→pidfd/SIGUSR2→원래5초 수집, 실제 본문N/T/scene/source join, raw capture read 구조, FPS/1%·5%low/age/maxgap/per-second/CSV 산술이 일치했다. Pending 반환 시각은 null을 유지했다. Resource는 raw job→summary→CSV 동일성과 단위 검증 범위이며 독립 raw 카운터 재측정이 아니다.

Main-A/B는 최종 CAN+PTY-safe4f98/c25 동일 바이너리이고 candidate는 combined295f/fb04다. 4병렬×2회, CPU30은 단일 admission이며 runtime CPU/외부프로세스 값은 문맥이다. FPS1%/5%low와avg를 주로 비교하고 CPU/RSS는 보조로 둔다. 개별 손해로 자동기각하거나 임의 점수를 만들지 않는다. 데이터 유효성과 최종 승격은 별개다.

각 셀 순서: **1%low / 5%low / avg FPS**. 두 반복과 SSH·동일 mainA/A를 모두 보존했다.

|Case|SSH|main-A|main-B|candidate|
|---|---|---|---|---|
|00-lan-compress-i0|44.5 / 49.5 / 60.0|43.2 / 47.0 / 60.0|44.6 / 48.0 / 60.0|44.7 / 47.6 / 60.0|
|01-lan-compress-i1|46.1 / 51.0 / 60.0|43.1 / 47.0 / 60.0|44.1 / 47.1 / 60.0|43.4 / 47.0 / 60.0|
|02-lan-bandwidth-i0|45.0 / 49.7 / 60.0|39.2 / 44.4 / 60.0|40.0 / 45.5 / 60.0|40.7 / 46.5 / 60.0|
|03-lan-bandwidth-i1|43.1 / 48.3 / 60.0|40.5 / 46.0 / 60.0|41.0 / 45.7 / 60.0|40.9 / 46.2 / 60.0|
|04-wan-compress-i0|42.3 / 47.4 / 60.0|43.5 / 46.3 / 56.1|42.0 / 45.8 / 55.7|42.9 / 46.6 / 60.0|
|05-wan-compress-i1|42.4 / 47.8 / 60.0|43.6 / 47.1 / 56.3|43.2 / 45.5 / 56.0|43.0 / 47.1 / 60.0|
|06-wan-bandwidth-i0|31.7 / 46.2 / 60.0|14.1 / 25.9 / 29.2|41.0 / 44.2 / 54.6|34.8 / 45.4 / 54.4|
|07-wan-bandwidth-i1|26.5 / 46.2 / 60.0|40.2 / 45.4 / 54.5|14.2 / 25.4 / 29.2|37.3 / 45.5 / 54.6|
|08-highlat-compress-i0|25.5 / 46.2 / 60.0|25.8 / 27.5 / 40.4|42.2 / 45.2 / 54.7|42.0 / 45.7 / 54.6|
|09-highlat-compress-i1|22.5 / 43.5 / 60.0|24.8 / 28.2 / 40.4|40.2 / 45.7 / 54.7|43.4 / 46.4 / 54.4|
|10-highlat-bandwidth-i0|12.1 / 42.6 / 60.0|11.7 / 45.2 / 50.4|10.7 / 44.6 / 49.9|12.4 / 29.9 / 47.7|
|11-highlat-bandwidth-i1|12.1 / 43.1 / 60.0|9.0 / 42.6 / 48.9|9.7 / 44.7 / 49.4|12.5 / 44.2 / 49.7|

유리/불리한 관측을 함께 유지한다. WAN dashboard candidate60/60avg는 main55.7–56.3보다 높다. 반면 LAN random은 SSH보다 양쪽 low가 낮고,300ms random0회 candidate47.7avg/29.9(5%low)는 SSH60/42.6 및 main49.9–50.4avg/44.6–45.2보다 낮다. WAN random mainA/A는29.2 대54.5–54.6처럼 넓게 갈리므로 한 main을 제외하거나 후행 표본을 지우지 않는다. 원인·분기 활성화는 이 untraced 자료로 입증하지 않는다.

자료: `final40-actual-audit.json`, `final40-actual-metrics.csv`; 스크립트 `final4041-actual-audit.py`. 실패한44partial32/46partial4 및47connect8,49pre-job alias 실패는 이 passing112jobs와 합치지 않는다.
