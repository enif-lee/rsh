# Combined FPS 39: actual saved-evidence audit

판정: 저장 증거 감사 PASS, 오류 0건. Data valid/performance eligible이며 최종 선택은 `null`입니다. 단일 비용으로 자동 기각하거나 이 감사를 성능 승격으로 해석하지 않습니다.

48 jobs/12 blocks/4×2. body 7,867, offline uniform-header 7,888, source 19,765, retained payload 161,805,396 B. Pending write 7(종료 시각은 null), pending body join 0, zero-body 0, missing-low jobs 2, missing age-p95 jobs 11입니다. 누락값을 0으로 채우지 않았습니다.

실제 창: 2026-11-01T04:50:43.613920+09:00 → 2026-11-01T04:54:08.264775+09:00.

준비10/하네스33/부모9/분석6 해시, environment sourcehash33, generated scene=frozen transform, 48 raw capture 전체 구조·시간 순서·SHA를 확인했습니다. 모든 source payload의 FRAME n/T/scene·바이트 합·prewrite wall/monotonic bracket·완료/미완료 write-end·actual complete-body join을 재대조했습니다. 연속 raw FRAME 패턴도 해당 source와 연결했습니다(부분 ANSI diff의 모든 header를 raw regex로 복원한 것은 아닙니다). 각 job nonce/PID/starttime/UID/동일job sidecar·실제4closure·pidfd SIGUSR2·원래5초 deadline이 일치합니다. Strict network endpoint 96, before/after endpoint file 192, live-server 96 검사를 통과했습니다.

기준 A/B는 CAN-main `935/c68` 동일 바이너리이며 후보는 결합 `94f/7198`입니다. RAM schema2·child preexec SET/GET assertion은 공통 37 방법입니다. 새로운 ppoll/subms 변경은 없습니다. 원본 입력·metric은 바꾸지 않았습니다.

FPS 순서는 **1% low / 5% low / avg**이고 actual source-body completion으로 독립 재산출했습니다. Low는 인접 gap, max gap은 시작/끝 경계를 포함합니다. 모든 per-second count·age median/p95·CSV 값과 일치합니다.

| Profile / scene / eval | SSH 1/5/avg | CAN A 1/5/avg | CAN B 1/5/avg | Combined 1/5/avg |
|---|---:|---:|---:|---:|
| lowbw / compress / 0 | 6.6/7.2/23.5 | 42.1/45.1/55.5 | 41.2/45.8/55.6 | 42.8/46.8/60.0 |
| lowbw / compress / 1 | 6.3/7.1/23.7 | 40.8/44.8/55.7 | 42.1/45.8/55.9 | 40.7/46.7/60.0 |
| lowbw / bandwidth / 0 | 5.5/6.3/9.9 | 15.8/17.9/20.4 | 9.5/17.9/20.0 | 13.1/17.8/21.2 |
| lowbw / bandwidth / 1 | 5.4/6.8/9.9 | 12.8/19.1/23.6 | 10.0/18.4/22.0 | 12.2/18.6/23.7 |
| verylow / compress / 0 | 1.5/2.3/5.9 | 26.0/27.4/37.1 | 26.7/27.6/40.5 | 41.2/46.2/60.0 |
| verylow / compress / 1 | 4.1/5.0/6.1 | 25.5/27.9/43.0 | 25.9/28.1/42.1 | 42.6/45.4/60.0 |
| verylow / bandwidth / 0 | 1.3/1.3/2.4 | 5.4/5.6/6.0 | 5.3/5.4/6.0 | 5.5/5.5/6.0 |
| verylow / bandwidth / 1 | 1.3/1.3/2.5 | 5.4/5.5/6.0 | 4.3/5.6/5.9 | 5.4/5.5/5.9 |
| slow64k / compress / 0 | 1.3/1.3/1.5 | 11.1/12.6/15.5 | 9.2/12.6/15.5 | 8.6/13.0/16.1 |
| slow64k / compress / 1 | 1.2/1.2/1.5 | 11.0/12.3/15.7 | 11.6/12.8/15.6 | 5.8/12.2/15.5 |
| slow64k / bandwidth / 0 | —/—/0.6 | 1.4/1.4/1.6 | 1.3/1.3/1.5 | 1.3/1.3/1.4 |
| slow64k / bandwidth / 1 | —/—/0.5 | 0.8/0.8/1.4 | 1.4/1.4/1.6 | 1.4/1.4/1.6 |

1M dashboard avg60/60 및 256k dashboard 60/60은 두 baseline보다 높습니다. 256k dashboard는 두 평가 모두 low1/low5도 개선됩니다. 1M dashboard eval1의 low1은 후보40.7로 A40.8/B42.1보다 낮고 low5/avg는 높으므로 그룹 tradeoff를 유지합니다. 1M random avg21.2/23.7로 개선되나 eval1은 A23.6에 가깝고 low가 혼합입니다. 64k dashboard 후보16.1/15.5, low1 8.6/5.8은 두 baseline보다 낮습니다.

1M random age-p95는 후보141/178ms, A/B132/116 및134/138ms입니다. RX는 후보852/953kbit/s, A/B814/801 및939/886kbit/s로 FPS 이득과 신선도·통신 비용을 함께 보존합니다. 256k dashboard는 후보 age76/76ms, A/B86–89ms로 개선되며 RX229/229가 A/B147–171보다 높습니다.

7개 source pending payload는 보존됐지만 actual complete-body pending join은0입니다. 64k random SSH의 body5/4개는 low1/low5를 계산할 gap 수가 부족하므로 null입니다. age-p95 null11 jobs도 그대로 유지합니다. 본문이0인 job은 없습니다.

후보의 보조 지표(원값):

| Profile / scene / eval | age median/p95 ms | max gap ms | client/server CPU % | client/server RSS KiB | client RX kbit/s |
|---|---:|---:|---:|---:|---:|
| lowbw / compress / 0 | 59/62 | 25 | 0.87/1.14 | 2628/5844 | 227 |
| lowbw / compress / 1 | 59/63 | 26 | 0.89/1.23 | 2628/5804 | 227 |
| lowbw / bandwidth / 0 | 98/141 | 93 | 0.87/2.91 | 2652/6072 | 852 |
| lowbw / bandwidth / 1 | 128/178 | 84 | 0.89/2.92 | 1828/5356 | 953 |
| verylow / compress / 0 | 72/76 | 28 | 1.26/1.79 | 2640/5824 | 229 |
| verylow / compress / 1 | 72/76 | 28 | 1.26/1.86 | 1988/5044 | 229 |
| verylow / bandwidth / 0 | 380/411 | 183 | 0.57/2.37 | 2628/6084 | 256 |
| verylow / bandwidth / 1 | 384/401 | 184 | 0.56/2.11 | 2628/6060 | 256 |
| slow64k / compress / 0 | 184/378 | 182 | 0.59/2.54 | 2320/5560 | 63 |
| slow64k / compress / 1 | 180/352 | 202 | 0.43/1.75 | 2660/6040 | 63 |
| slow64k / bandwidth / 0 | 1045/— | 760 | 0.3/1.65 | 2652/6064 | 64 |
| slow64k / bandwidth / 1 | 1684/— | 720 | 0.2/1.57 | 2624/6108 | 63 |

Startup CPU 20.0%, phase admission [25.0, 26.0, 26.0, 27.0, 26.0, 29.0, 7.0, 12.0, 14.0, 12.0, 19.0, 16.0] 모두 첫 qualifying sample <30%입니다. Runtime 최대 66.0%는 맥락 기록이며 자동 탈락 조건이 아닙니다. 전체 CPU/외부 process 기록과 clock/metadata 검증을 분리했습니다. P-core affinity는 입증하지 않았습니다.

자원 검증 범위는 batch→summary→CSV 일치와 `harness/agent.py:229–263,279–289,397–409` 공식·단위 검토입니다. 별도 원시 RX/CPU counter endpoint·midpoint RSS sample이 없어 독립 counter 재측정은 아닙니다. network는 client 비-loopback RX decimal kbit/s, RSS는 midpoint 이름별 프로세스 합 KiB, CPU는 transport 프로세스/스레드의 한 코어 대비 %이며 Python source/observer CPU는 제외합니다.

한계: 전체48 terminal-emulator 재생·GPU/모든 terminal·moving 회복은 검증하지 않았습니다. Uninstrumented controller이므로 transit/probe60 분기 활성화·인과적 기여·각 단독 이득의 가산성을 주장하지 않습니다. 2회 평가와 큰 A/A 편차를 보존하며 통계적 우열·전체망 최종 승격을 확정하지 않습니다. `run-status.json`의 analysis_pending 문구는 남아 있으나 실제 분석 manifest/결과 및 root terminal64235/83253이 완료 근거입니다.

재현: `PYTHONDONTWRITEBYTECODE=1 python3 "research_notes/LAN 원거리 low 성능 연구/combined3839-actual-audit.py"` → 두 study `valid_saved_audit:true`, `errors:[]`. JSON에 per-job/edge gaps/source·capture SHA와 검증 범위가 있습니다.
