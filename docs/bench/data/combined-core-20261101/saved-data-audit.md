# Combined FPS 38: actual saved-evidence audit

판정: 저장 증거 감사 PASS, 오류 0건. Data valid/performance eligible이며 최종 선택은 `null`입니다. 단일 비용으로 자동 기각하거나 이 감사를 성능 승격으로 해석하지 않습니다.

48 jobs/12 blocks/4×2. body 21,530, offline uniform-header 21,550, source 23,040, retained payload 192,593,640 B. Pending write 0, pending body join 0, zero-body 0, missing-low jobs 0, missing age-p95 jobs 0입니다. 누락값을 0으로 채우지 않았습니다.

실제 창: 2026-11-01T04:43:59.898849+09:00 → 2026-11-01T04:47:06.640670+09:00.

준비10/하네스33/부모9/분석6 해시, environment sourcehash33, generated scene=frozen transform, 48 raw capture 전체 구조·시간 순서·SHA를 확인했습니다. 모든 source payload의 FRAME n/T/scene·바이트 합·prewrite wall/monotonic bracket·완료/미완료 write-end·actual complete-body join을 재대조했습니다. 연속 raw FRAME 패턴도 해당 source와 연결했습니다(부분 ANSI diff의 모든 header를 raw regex로 복원한 것은 아닙니다). 각 job nonce/PID/starttime/UID/동일job sidecar·실제4closure·pidfd SIGUSR2·원래5초 deadline이 일치합니다. Strict network endpoint 96, before/after endpoint file 192, live-server 96 검사를 통과했습니다.

기준 A/B는 CAN-main `935/c68` 동일 바이너리이며 후보는 결합 `94f/7198`입니다. RAM schema2·child preexec SET/GET assertion은 공통 37 방법입니다. 새로운 ppoll/subms 변경은 없습니다. 원본 입력·metric은 바꾸지 않았습니다.

FPS 순서는 **1% low / 5% low / avg**이고 actual source-body completion으로 독립 재산출했습니다. Low는 인접 gap, max gap은 시작/끝 경계를 포함합니다. 모든 per-second count·age median/p95·CSV 값과 일치합니다.

| Profile / scene / eval | SSH 1/5/avg | CAN A 1/5/avg | CAN B 1/5/avg | Combined 1/5/avg |
|---|---:|---:|---:|---:|
| lan / compress / 0 | 48.3/51.9/60.0 | 48.1/50.2/60.0 | 43.9/48.1/60.0 | 43.9/49.3/60.0 |
| lan / compress / 1 | 46.6/50.8/60.0 | 45.2/49.4/60.0 | 43.7/49.0/60.0 | 45.7/49.1/60.0 |
| lan / bandwidth / 0 | 48.4/51.3/60.0 | 44.0/46.4/60.0 | 45.1/47.7/60.0 | 44.4/48.7/60.0 |
| lan / bandwidth / 1 | 48.4/50.6/60.0 | 43.9/48.2/59.9 | 44.1/47.0/60.0 | 41.6/48.4/60.0 |
| wan / compress / 0 | 40.6/46.8/60.0 | 42.0/46.2/56.2 | 42.4/45.6/55.8 | 43.0/47.0/60.0 |
| wan / compress / 1 | 45.3/48.3/60.0 | 42.2/45.3/55.9 | 42.6/45.8/55.7 | 43.3/46.8/60.0 |
| wan / bandwidth / 0 | 32.8/46.9/60.0 | 28.6/45.9/54.5 | 40.7/46.9/54.7 | 30.5/45.7/54.5 |
| wan / bandwidth / 1 | 37.0/46.3/60.0 | 39.7/45.2/54.5 | 39.5/44.5/54.7 | 21.4/34.2/52.1 |
| highlat / compress / 0 | 24.7/46.7/60.0 | 27.5/29.1/44.7 | 40.5/46.0/54.6 | 40.7/46.5/54.6 |
| highlat / compress / 1 | 23.1/46.0/60.0 | 28.9/40.0/53.0 | 40.9/45.4/55.0 | 40.9/45.1/54.6 |
| highlat / bandwidth / 0 | 13.0/42.8/60.0 | 13.8/44.3/50.0 | 8.4/11.6/17.0 | 12.7/44.2/50.1 |
| highlat / bandwidth / 1 | 14.5/45.6/60.0 | 15.9/43.8/49.1 | 28.0/45.7/50.4 | 16.8/43.6/49.2 |

WAN dashboard는 후보 60/60 FPS로 두 baseline 약55.7–56.2보다 높고 low도 개선됩니다. LAN은 avg 60을 유지하지만 SSH보다 lows가 낮고 baseline A/A 사이 혼합입니다. WAN random eval1은 후보 21.4/34.2/52.1로 A/B 39.7/45.2/54.5 및 39.5/44.5/54.7보다 낮습니다. RTT300 random eval0의 동일 baseline A/B가 50.0 대17.0 FPS로 크게 갈리므로 단일 역할 차이를 제품 효과로 일반화하지 않습니다.

WAN random eval1 후보는 **body417 / offline uniform-header421 / online421(52.6 FPS)**이며 body avg는52.1입니다. incomplete-state 체류0.820928%와 body/header 차이4개를 그대로 보존했습니다. 이전 약437이라는 임시 live 수치를 사용하지 않습니다. RTT300 random eval1 후보 body394/header397, incomplete4.175293%도 별도로 남깁니다. 이 차이를 parser 손상·GPU 표시·특정 controller 원인으로 판정하지 않았습니다.

후보의 보조 지표(원값):

| Profile / scene / eval | age median/p95 ms | max gap ms | client/server CPU % | client/server RSS KiB | client RX kbit/s |
|---|---:|---:|---:|---:|---:|
| lan / compress / 0 | 4/5 | 24 | 0.71/1.02 | 1876/4732 | 227 |
| lan / compress / 1 | 4/7 | 24 | 0.93/1.29 | 2624/5792 | 227 |
| lan / bandwidth / 0 | 5/8 | 25 | 1.21/1.55 | 2636/5888 | 2718 |
| lan / bandwidth / 1 | 5/8 | 30 | 1.27/1.61 | 2668/5824 | 2722 |
| wan / compress / 0 | 55/58 | 26 | 0.87/1.22 | 2624/5796 | 228 |
| wan / compress / 1 | 55/58 | 24 | 0.82/1.11 | 2628/5796 | 228 |
| wan / bandwidth / 0 | 63/71 | 87 | 1.12/4.68 | 2028/5348 | 2130 |
| wan / bandwidth / 1 | 63/71 | 114 | 1.12/4.62 | 2688/6100 | 2055 |
| highlat / compress / 0 | 160/169 | 109 | 0.87/3.26 | 2712/6088 | 219 |
| highlat / compress / 1 | 162/170 | 91 | 0.88/3.27 | 2272/5592 | 218 |
| highlat / bandwidth / 0 | 163/171 | 296 | 1.16/4.69 | 2732/6180 | 2000 |
| highlat / bandwidth / 1 | 163/172 | 338 | 1.21/5.08 | 2792/6224 | 1987 |

Startup CPU 11.0%, phase admission [10.0, 13.0, 17.0, 18.0, 16.0, 17.0, 26.0, 15.0, 22.0, 20.0, 29.0, 24.0] 모두 첫 qualifying sample <30%입니다. Runtime 최대 52.0%는 맥락 기록이며 자동 탈락 조건이 아닙니다. 전체 CPU/외부 process 기록과 clock/metadata 검증을 분리했습니다. P-core affinity는 입증하지 않았습니다.

자원 검증 범위는 batch→summary→CSV 일치와 `harness/agent.py:229–263,279–289,397–409` 공식·단위 검토입니다. 별도 원시 RX/CPU counter endpoint·midpoint RSS sample이 없어 독립 counter 재측정은 아닙니다. network는 client 비-loopback RX decimal kbit/s, RSS는 midpoint 이름별 프로세스 합 KiB, CPU는 transport 프로세스/스레드의 한 코어 대비 %이며 Python source/observer CPU는 제외합니다.

한계: 전체48 terminal-emulator 재생·GPU/모든 terminal·moving 회복은 검증하지 않았습니다. Uninstrumented controller이므로 transit/probe60 분기 활성화·인과적 기여·각 단독 이득의 가산성을 주장하지 않습니다. 2회 평가와 큰 A/A 편차를 보존하며 통계적 우열·전체망 최종 승격을 확정하지 않습니다. `run-status.json`의 analysis_pending 문구는 남아 있으나 실제 분석 manifest/결과 및 root terminal64235/83253이 완료 근거입니다.

재현: `PYTHONDONTWRITEBYTECODE=1 python3 "research_notes/LAN 원거리 low 성능 연구/combined3839-actual-audit.py"` → 두 study `valid_saved_audit:true`, `errors:[]`. JSON에 per-job/edge gaps/source·capture SHA와 검증 범위가 있습니다.
