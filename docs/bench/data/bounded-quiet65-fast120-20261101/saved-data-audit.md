70 requested120Hz guard 저장자료 독립감사 PASS. Actual requested_fps=120·각 job scene_fps=120·source command120을 확인했다. 16 jobs/4 blocks, body14487/uniform header14509, source15360 records/180686720 B다. 각 source960records이며 pending0/zero body0/full-window low·age95 누락0이다. 원자료 수정·새 terminal/body-oracle replay·Docker·실측0이다.

Actual60 fa934/4b52 A/A 대65quiet781b/3d4c의 한 정책 증가분이다. Actual 반복0 [SSH,A,B,C], 반복1 [C,B,A,SSH], artifact/PID/nonce·source/capture hashes·FRAME N/T/scene·prewrite clock·원래5초 pidfd/all-four fences·FPS/lows/age/gap/per-second/CSV 산술이 일치했다. Endpoint64/live server32/strict network32개다. Source raw quota32MiB/4096records는 그대로이며 dropped/collection deadline failure가 없었다. Prospective961records raw bound31852345 B/base64per-row42472356 B는 quota 계획치이며 totalJSON/RAM 상한이 아니다. Actual 최대960records이고 cap·timeout 증가나 실패 제외는 없었다.

각 셀은 **1%low /5%low /avg FPS**, requested target은 **120Hz**다.

|Case|SSH|baseline60-A|baseline60-B|quiet65|
|---|---|---|---|---|
|00-lan-bandwidth-i0|73.2 / 83.0 / 120.0|61.2 / 75.5 / 119.7|63.1 / 75.9 / 119.9|65.7 / 78.8 / 119.5|
|01-lan-bandwidth-i1|79.8 / 85.8 / 120.0|63.5 / 78.5 / 119.9|67.9 / 77.2 / 119.8|67.1 / 75.3 / 119.8|
|02-wan-bandwidth-i0|57.7 / 76.9 / 120.0|44.6 / 70.4 / 103.5|60.5 / 74.2 / 104.7|56.0 / 67.8 / 100.6|
|03-wan-bandwidth-i1|49.9 / 76.8 / 120.0|48.6 / 71.8 / 103.3|46.1 / 68.2 / 101.9|44.9 / 67.4 / 98.0|

LAN 반복0 후보65.7/78.8/119.5는 양 A/A보다 lows가 높지만 avg는 낮다. 반복1 후보67.1/75.3/119.8은 A63.5/78.5/119.9보다1%low만 높고, B67.9/77.2/119.8보다 lows가 낮고 avg동률이다. SSH의 양 low·avg는 두 반복 모두 후보보다 높다. LAN 반복0 후보 body956/header958·maxgap49ms는 A958/958·19ms/B959/959·26ms보다 불리하며, 반복1은959/959·17ms다.

WAN 반복0 후보56.0/67.8/100.6은 A44.6/70.4/103.5 대비1%low만 높고 B60.5/74.2/104.7보다 세 지표가 낮다. 반복1 후보44.9/67.4/98.0은 A48.6/71.8/103.3·B46.1/68.2/101.9보다 세 지표가 낮다. SSH는120/120avg이며 양low도 후보보다 높다. 후보 WAN body805/header805·784/785, age95 65/66ms 대 A64/65·B65/66ms이며 낮은 avg/tails 비용을 그대로 보존한다. 120Hz에서 일관된 tail/FPS 우위나 SSH 동등을 입증하지 못했다. 단일 비용 자동 veto나 새 허용오차를 만들지 않는다.

CPU admission1sample19%와 각 phase 첫 user+system<30%를 확인했고 runtimeCPU는 문맥이다. 자원은 raw job→summary→CSV 동일성과 단위/공식 범위이며 counter/RSS sample 독립 재측정은 아니다. Observations eligible/final_selection=None을 유지한다. 실제120Hz production·원래quota와5초 수집은 이번16jobs에서 유효했지만 모든 workload의 처리 보장이나 quiet raw20ms의 실제 service 상한은 아니다. Untraced 자료로 분기 활성화/인과/atomicity를 단정하지 않는다.

120Hz guard는 별도 prospective cohort이며60Hz66/67/68/69와 pool하지 않는다. Oldquiet18FPS와60Hz RTT300/slow/moving의 모든 비용도 보존한다. FPS1%/5%low·avg 우선/CPU RSS 보조로 검토하며 전체 승격은 아직 없다. 별도 directCAN+PTYmain4f98 기준 비교는 아직 측정하지 않았다.

근거는 guard70-actual-audit.py/json, guard70-actual-metrics.csv다. JSON은 original hashes·partial body/header·window boundary/maxgap·actual balanced slots·requested120 metadata와 resource scalar를 남긴다. Linux149pass/2ignored·Mac148pass/2ignored는 별도 root source 기능 증거이며 이 감사에서 새 build/test를 하지 않았다.
