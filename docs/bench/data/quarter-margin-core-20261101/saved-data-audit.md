61 core 저장자료 독립감사 PASS. 48 jobs/12 blocks, actual body22195/uniform header22215, source23040 records/192593640 B. Pending write/body join/zero body/누락 low 및 age95는 모두0이다. 원자료 수정과 새 terminal/body-oracle replay·Docker·실측은0이다.

COMBINED295f/fb04 A/A와 quarter+margin60 후보fa934/4b52를 비교했다. Main-A/B는 actualmain4f98이 아니다. Source/analysis/parent manifests, raw capture 구조·FRAME N/T/scene, source clock/nonce/PID, all-four closure→pidfd SIGUSR2→원래5초 수집, 완료 body FPS·low·age·gap·second bins와 acceptance-per-rep.csv 산술이 일치한다. Endpoint file192/live server96/strict network96개를 확인했다. 자원 검증은 raw job→summary→CSV 동일성과 단위/공식 범위이며 저장되지 않은 counter/RSS sample을 새로 측정한 것은 아니다.

CPU 시작 admission은1 sample16%이며 각 phase 첫 total<30%를 확인했다. Runtime CPU/프로세스 값은 문맥이고 affinity 증거가 아니다. 데이터 유효성과 최종 승격은 별개다. Performance observations eligible이며 final_selection=None을 유지한다.

각 셀은 **1%low / 5%low / avg FPS**다.

|Case|SSH|combined-A|combined-B|quarter+margin60|
|---|---|---|---|---|
|00-lan-compress-i0|45.9 / 49.2 / 60.0|42.8 / 46.9 / 60.0|41.2 / 46.6 / 60.0|44.1 / 47.0 / 60.0|
|01-lan-compress-i1|45.9 / 49.7 / 60.0|42.6 / 47.7 / 60.0|43.3 / 47.5 / 60.0|41.7 / 46.5 / 60.0|
|02-lan-bandwidth-i0|45.0 / 48.2 / 60.0|41.5 / 46.3 / 60.0|41.7 / 46.6 / 60.0|43.0 / 47.6 / 60.0|
|03-lan-bandwidth-i1|46.0 / 48.5 / 60.0|42.5 / 46.4 / 60.0|42.4 / 45.7 / 60.0|41.1 / 45.6 / 60.0|
|04-wan-compress-i0|40.9 / 47.5 / 60.0|42.2 / 47.0 / 60.0|42.0 / 46.7 / 60.0|43.2 / 47.2 / 60.0|
|05-wan-compress-i1|41.0 / 48.1 / 60.0|40.0 / 46.4 / 60.0|41.7 / 47.1 / 60.0|41.8 / 46.2 / 60.0|
|06-wan-bandwidth-i0|36.3 / 46.6 / 60.0|28.9 / 45.0 / 54.1|40.2 / 45.6 / 54.7|29.7 / 47.3 / 58.5|
|07-wan-bandwidth-i1|42.5 / 47.6 / 60.0|33.2 / 45.3 / 55.0|41.6 / 45.8 / 54.7|31.6 / 47.9 / 58.3|
|08-highlat-compress-i0|22.8 / 45.2 / 60.0|43.0 / 46.2 / 54.5|41.7 / 45.8 / 54.6|44.0 / 48.7 / 58.9|
|09-highlat-compress-i1|24.2 / 44.4 / 60.0|41.0 / 45.1 / 54.5|41.0 / 46.0 / 54.6|42.5 / 47.7 / 59.0|
|10-highlat-bandwidth-i0|12.2 / 44.5 / 60.0|8.8 / 44.8 / 49.6|14.9 / 45.4 / 50.0|11.8 / 43.5 / 53.0|
|11-highlat-bandwidth-i1|12.4 / 44.7 / 60.0|14.0 / 44.6 / 49.9|15.0 / 28.8 / 46.9|11.5 / 47.5 / 53.4|

RTT300 dashboard 후보44.0/48.7/58.9 및42.5/47.7/59.0은 두 A/A의 세 FPS 지표보다 높다. SSH avg60에는 못 미치지만 low는 SSH보다 높다. WAN random 후보avg58.5/58.3 및5%low47.3/47.9는 A/A보다 높으나1%low29.7/31.6은 각 B40.2/41.6보다 낮다. RTT300 random avg53.0/53.4는 A/A49.6–50.0/46.9–49.9보다 높다.

손해를 남긴다. RTT300 random 반복0의5%low43.5는 A44.8/B45.4/SSH44.5보다 낮고 반복1의1%low11.5는 A14.0/B15.0/SSH12.4보다 낮다. LAN 반복1은 dashboard41.7/46.5 및 random41.1/45.6으로 두 A/A보다 낮다. WAN dashboard 반복1의5%low46.2도 A46.4/B47.1/SSH48.1보다 낮다. RTT300 random 후보 avg는 여전히 SSH60보다 낮다. 모든 반복/A/A spread를 보존하며 단일 열세 자동탈락이나 유리한 셀 선택을 하지 않는다.

완료 body와 online/header는 다르다. RTT300 random 후보 반복0 body424/header429(53.0 vs uniform53.6), 반복1 body427/header430을 각각 보존한다. Partial body/unknown state와 window boundary gap을 별도로 남겼다. Age는 완료 body에 조건부이며 입력/GPU/모든 터미널 presentation latency를 뜻하지 않는다.

이 비교는295f 대비 두 정책의 결합 효과이며 quarter increment versus48이나 가산성을 분리하지 않는다. PR8 source가 PR7 위의 작은 diff라는 사실은 실험 baseline을 바꾸지 않는다. Untraced61은 branch 활성화와 인과를 입증하지 않는다. Old19의 동일 quarter predicate RTT300 random24.5avg/4.0·12.0low, margin51/53의 낮은1%low,55/57의 dashboard low/age 비용을 유지한다. 다른 cohort 절대값을 pool하지 않는다. FPS1%/5%low·avg를 주로 보고 CPU/RSS는 보조로 평가하며 slow62/moving63 전에는 최종 승격을 주장하지 않는다.

근거는 combo61-actual-audit.py/json 및 combo61-actual-metrics.csv다. JSON에 source/capture hashes, partial body, 모든 자원 scalar와 boundary/maxgap을 보존했다. 별도 root 기능 증거는116library+4CLI+25session+1stress=146pass/2ignored, exact48 control의 adaptedquarter2pass/1expectedfail 및 기존margin3pass다. 본 감사가 새 build/test를 실행한 것은 아니다.
