57 slow 저장자료 독립감사 PASS. 48 jobs/12 blocks, actual body 8664/uniform header 8685, source 19769 records/161837997 B. Pending write 8개는 end=null을 유지하며 pending body join 0, zero body 0이다. SSH 64k random 두 반복의 각 5 body frames는 low=NULL이고, age95 누락 12 jobs도 NULL이다. 원자료 수정과 새 terminal/body-oracle replay·Docker·실측은 0이다.

Actual candidate 515d/6d7f와 COMBINED295f/fb04 A/A를 비교했다. Main-A/B는 실제 main4f98이 아니다. Source/analysis/parent manifest, capture read 구조와 관측 N/T/scene, source clock/nonce/PID, all-four closure→pidfd SIGUSR2→원래 5초 수집, complete-frame FPS·low·age·maxgap·second bins와 실제 acceptance-per-rep.csv 산술이 모두 일치한다. Endpoint file 192/live server 96/strict network 96개를 확인했다. 자원은 raw job→summary→CSV 동일성과 단위를 검사했으며 저장되지 않은 RX/CPU counter·RSS sample을 독립 재측정한 것은 아니다.

CPU admission은 시작 1 sample 18%이며 각 phase 첫 qualifying total<30% 조건을 확인했다. Runtime CPU/프로세스 부하는 기록 문맥이다. 4병렬×2평가를 유지하고 FPS 1%/5% low와 avg를 주로 평가한다. 데이터 유효성과 최종 승격은 분리하며 final_selection=None이다.

각 셀은 **1% low / 5% low / avg FPS**다. NULL을 0으로 바꾸지 않았다.

|Case|SSH|combined-A|combined-B|quarter55|
|---|---|---|---|---|
|00-lowbw-compress-i0|6.6 / 7.6 / 23.7|40.0 / 45.7 / 60.0|41.6 / 46.4 / 60.0|38.6 / 45.1 / 60.0|
|01-lowbw-compress-i1|6.6 / 7.1 / 23.5|41.3 / 46.5 / 60.0|40.6 / 46.8 / 60.0|41.5 / 45.7 / 60.0|
|02-lowbw-bandwidth-i0|4.6 / 6.8 / 9.9|13.8 / 18.6 / 23.1|10.8 / 17.9 / 21.4|18.0 / 20.2 / 24.2|
|03-lowbw-bandwidth-i1|5.8 / 6.5 / 9.9|14.2 / 18.3 / 21.6|11.2 / 17.9 / 23.0|17.4 / 18.8 / 23.4|
|04-verylow-compress-i0|2.1 / 4.3 / 6.0|41.8 / 46.5 / 60.0|41.4 / 46.3 / 60.0|41.8 / 47.2 / 60.0|
|05-verylow-compress-i1|2.4 / 4.2 / 5.7|42.2 / 45.9 / 60.0|41.6 / 45.9 / 60.0|39.7 / 45.3 / 60.0|
|06-verylow-bandwidth-i0|1.2 / 1.2 / 2.4|5.3 / 5.5 / 6.0|2.8 / 5.5 / 5.7|5.3 / 5.5 / 5.9|
|07-verylow-bandwidth-i1|1.2 / 1.2 / 2.4|5.4 / 5.5 / 6.0|5.3 / 5.5 / 5.9|5.3 / 5.5 / 6.0|
|08-slow64k-compress-i0|1.1 / 1.1 / 1.4|11.6 / 12.6 / 15.6|9.3 / 12.0 / 15.7|9.5 / 12.1 / 15.5|
|09-slow64k-compress-i1|1.3 / 1.3 / 1.4|5.5 / 8.9 / 15.0|8.7 / 12.8 / 16.2|9.0 / 13.3 / 16.1|
|10-slow64k-bandwidth-i0|NULL / NULL / 0.6|1.4 / 1.4 / 1.6|1.3 / 1.3 / 1.4|1.3 / 1.3 / 1.5|
|11-slow64k-bandwidth-i1|NULL / NULL / 0.6|1.4 / 1.4 / 1.6|0.8 / 0.8 / 1.4|1.4 / 1.4 / 1.5|

1M random 후보는 두 반복 모두 세 FPS 지표가 두 A/A보다 높다. 반복0 18.0/20.2/24.2 대 A 13.8/18.6/23.1 및 B 10.8/17.9/21.4, 반복1 17.4/18.8/23.4 대 A 14.2/18.3/21.6 및 B 11.2/17.9/23.0이다. 후보 age95 160/139 ms는 A 178/133 및 B 120/169 ms와 비교해 혼합되어 있다. Candidate RX는 961/926 decimal kbit/s 대 A 925/858·B 867/930이며 실제 body 처리와 같은 구간의 저장 자원 지표다.

손해도 유지한다. 1M dashboard 반복0 후보 low 38.6/45.1은 A 40.0/45.7·B 41.6/46.4보다 낮다. 256k dashboard 반복1 후보 39.7/45.3은 A 42.2/45.9·B 41.6/45.9보다 낮다. 64k dashboard 반복0 후보 avg 15.5와 age95 441 ms는 A 15.6/360 ms·B 15.7/367 ms보다 불리하다. 64k random 후보 avg 1.5/1.5는 A 1.6/1.6보다 낮고 B 1.4/1.4보다 높다. 모든 반복과 A/A spread를 보존하고 단일 손해로 자동 탈락시키거나 유리한 셀만 선택하지 않는다.

Old19의 동일 quarter predicate는 다른 parent6729의 RTT300 random에서 24.5 avg/4.0·12.0 low 대 main49.7/14.0·45.2라는 손해가 있었다. 이번 source55는 combined295f이므로 이 과거 반례를 지우거나 해결됐다고 단정하지 않는다. Untraced57은 branch 활성화나 정책 인과를 직접 입증하지 않는다. Moving58을 별도로 검증해야 하며 old19/50/51/53 및 다른 방법의 절대값을 pool하지 않는다.

자료: fresh57-actual-audit.py/json, fresh57-actual-metrics.csv. JSON은 raw source/capture hashes, pending/null, partial header-body 상태와 window 양쪽 boundary gap을 보존한다. Age는 완료된 body에 조건부이며 입력/GPU/화면 체류 지연을 뜻하지 않는다. 실행 검증은 후보 113 library+4 CLI+25 session+1 stress=143 pass/2 ignored 및 old-policy control 2 pass/1 expected fail의 별도 root 증거이며 이 감사가 빌드나 기능 테스트를 새로 실행한 것은 아니다.
