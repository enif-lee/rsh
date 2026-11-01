# 59 catch deadline 추가 관측

기존 scalar cache만 읽어 catch-only gate 1,418개를 확인했다. 모두 random 세션의 10 ms budget이며, clear/update의 300 ms catch-only 관측은 0개다. 기존 logical-now에서는 아직 catching_up이고 이후 gate row 시각에서 since+budget이 만료되는 사례는 0개였다. 다음 poll로 이어지는 후보도 0개다.

이는 이번 12개 RSH LAN 세션의 관측 범위에서 fresh-clock 변경에 직접 활성 근거를 추가하지 않는다. row 시각은 실제 flush 호출 이후의 상한 시각이므로 실제 호출 Instant나 특정 body의 인과를 뜻하지 않는다. 관측되지 않은 300 ms 경로나 다른 네트워크 상황으로 결과를 일반화하지 않는다. 기존 token-negative, startup-only 반례 및 공식 전체 8초 low는 그대로다.

계산은 frozen combined-source/src/client.rs:1307의 catching_up 및 :1314의 catch_up 조건을 그대로 썼다. mask bit 3/4가 있으면 300 ms, 아니면 10 ms이며, mask 하위 두 bit가 2인 pending gate만 선택했다. 캐시 SHA는 d18fee88feeec44bea1fde53739451c46c13d177e29d3ab8d1df1f04671a5dfa다. 원시 CSV·capture를 다시 파싱하거나 새 측정·테스트·Docker·빌드를 하지 않았다.

근거: lan59-catch-edge-analysis.py/json. 독립 reviewer가 작은 script/JSON 범위를 검토해 PASS와 채택 결함 0건을 전달했다. 캐시 또는 원시 CSV의 독립 재파싱은 이 후속 peer 범위 밖이다.
