# 59 LAN Paced 관측

16개 기능 세션에서 토큰 부족으로 쓰기를 기다리는 gate7은 관측되지 않았다. 초기 30 ms refill만으로 공식 1% low를 설명하는 가설도 지지되지 않았다. 이 결과는 fresh-clock 변경의 토큰 보충 효과를 우선순위에서 낮추지만, catch-up 종료시각에 대한 fresh-clock 효과까지 배제하지 않는다. 새 제품 변경은 제안하지 않는다.

대상은 54의 기존 계측 binary 두 개(d742/3031, CAN+PTY baseline4f98와 controller295f)이며 새 정책을 넣지 않은 LAN 기능 조사다. functional_valid=True, performance_eligible=False이며 02-lan-bandwidth-i0의 host-context rejection도 보존한다. 실제 16 jobs, 4 blocks, 4 parallel × 2 iterations이고 raw trace 24개에서 추출한 scalar cache를 사용했다. 공식 측정 구간은 전부 8초 그대로다.

|Case|Role|Body|1% low|5% low|avg FPS|
|---|---|---:|---:|---:|---:|
|00-lan-compress-i0|ssh|480|43.4|47.1|60.0|
|00-lan-compress-i0|main-trace-A|480|42.1|46.9|60.0|
|00-lan-compress-i0|main-trace-B|480|41.1|46.2|60.0|
|00-lan-compress-i0|combined-trace|480|42.5|46.7|60.0|
|01-lan-compress-i1|main-trace-B|480|41.5|45.7|60.0|
|01-lan-compress-i1|combined-trace|480|42.0|45.7|60.0|
|01-lan-compress-i1|ssh|480|42.7|47.2|60.0|
|01-lan-compress-i1|main-trace-A|480|43.3|46.9|60.0|
|02-lan-bandwidth-i0|ssh|480|42.2|45.6|60.0|
|02-lan-bandwidth-i0|main-trace-A|480|37.7|43.0|60.0|
|02-lan-bandwidth-i0|main-trace-B|480|35.9|43.0|60.0|
|02-lan-bandwidth-i0|combined-trace|480|34.1|43.8|60.0|
|03-lan-bandwidth-i1|main-trace-B|479|39.0|44.6|59.9|
|03-lan-bandwidth-i1|combined-trace|480|40.2|45.3|60.0|
|03-lan-bandwidth-i1|ssh|480|43.6|49.0|60.0|
|03-lan-bandwidth-i1|main-trace-A|480|41.4|45.1|60.0|

12 RSH 세션의 최악 1% 인접 body gaps(각 5개, 총 60개)를 사전에 정한 0.5초 경계로 나누면 57개는 경계 이후, 1개는 경계 이전, 2개는 경계를 가로지른다. 이는 startup-only 설명에 대한 반례이며 startup 구간을 공식 low에서 제외하지 않았다. 토큰 refill의 구간 내 중앙값은 14.784–14.922 ms다. 구간 시작 이전에 이미 수신한 출력에 의해 적응한 상태일 수 있으므로 이 결과를 프로세스 시작 순간의 30 ms 동작 부정으로 확대하지 않는다.

gate7은 refill 이후, token 소모와 write8 이전의 상태다. pending>0일 때 dashboard의 token/catch 하위 두 bit는 ready=2,877회뿐이고 random은 ready=2,882회·catch-only=1,418회다. token-only와 token+catch는 모두 0회다. write8에 찍힌 token bit는 소모 후 상태이므로 대기 원인으로 계산하지 않았다. 따라서 root가 지정한 pending<64 KiB, token 부족, catch-up 해제, 다음 write 전에 poll에 진입한다는 조건을 만족한 row-time fresh-refill 후보는 robust=0, possible=0이다.

정수 T=floor(tokens×1000), refill 간격 E, Δ=row.ns−logical_now에 대해 기록된 토큰은 [T/1000,(T+1)/1000)다. 선형 refill 모형의 robust 조건은 T×E+1000×Δ≥1000×E, possible 조건은 (T+1)×E+1000×Δ>1000×E로 계산했다. 이는 기록된 구간에 대한 대수이며 모든 Rust f64 경계 동등성의 증명은 아니다. 이번 부정 결과는 antecedent인 token 부족 gate 자체가 0이라 경계 반올림에 의존하지 않는다.

gate row 시간은 실제 flush 호출 이후이므로 fresh Instant 호출의 정확한 시각이 아니다. gate의 logical_now staleness 중앙값은 약 0.0058–0.0158 ms이고 최대 1.893468 ms지만, stale한 gate가 있었다는 사실만으로 토큰 지연이나 body gap의 원인을 입증하지 않는다. random에서 catch-only gate는 실제 존재하므로 catch deadline을 row 시점이 넘기는지에 대한 별도 bounded 검사는 아직 수행하지 않았다. 현재 token-only 음성 결과를 catch-up 전체 경로의 면책으로 읽지 않는다.

random의 실제 client write 지속시간 최대값은 세션별 약 1.15–11.70 ms였다. 이는 관측된 구간이지 해당 write와 특정 완성 body의 소유관계나 GPU 표시시간 증명이 아니다. client1/2는 poll return code와 모든 FD readiness를 저장하지 않아 특정 poll 구간이 오직 timeout 때문이었다거나 minimum 1 ms가 실제 지연 원인이라고 단정하지 않는다. source header의 made 시각도 ms 해상도의 별도 시계 값이므로 정확한 producer-ready/전송 완료 시각으로 바꾸지 않는다.

기존 37 sub-ms ppoll 조사는 혼합된 tail 결과와 분기 활성화 미계측 한계가 있었고, 현재 결과가 이를 성능 개선으로 재해석하지 않는다. min-poll 제거는 만료 deadline에서 busy-spin 반례가 있으므로 제안하지 않는다. fresh-clock 한 줄 변경도 현재 토큰 증거만으로 우선할 근거가 없다. 다음으로 좁힐 수 있는 질문은 기존 scalar cache 안의 catch-only gate에서 since+10/300 ms와 row-time의 교차 여부이며 제품 상태·정책·옵션을 추가하지 않아도 확인 가능하다.

근거: frozen 54 combined-source/src/client.rs:1162(old logical now flush), :1282(30 ms), :1300(refill), :1314(catch-up), :1327(flush), :1362(.9 gap EMA), :1368(write). 원본/추출 해시와 계산은 lan59-extract.py, lan59-extracted.json, lan59-paced-analysis.py/json에 기록했다. 원자료 추출은 한 번 수행했고, 본 문안 작성 및 peer는 저장된 derived scalar만 사용했다. 측정자료와 코드 변경은 없다.

독립 peer: lan59-paced-peer-review.md/json/summary.json. source/schema와 작은 JSON 집계·정수 부등식 검토 PASS, 채택 결함 0. 독립 reviewer는 15.5 MB cache 및 24개 raw CSV를 재파싱하지 않았으므로 raw→cache 수치의 완전 독립 재구성은 검증 범위 밖이다.
