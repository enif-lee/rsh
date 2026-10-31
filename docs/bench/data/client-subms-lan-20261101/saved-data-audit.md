# Sub-ms client wait: 37 actual saved-evidence audit

판정: 저장 증거 감사 PASS, 오류 0건. 성능 관측은 유효하며 최종 선택은 `null`입니다. LAN 두 장면의 집중 비교로 전체 성능 승격을 입증하지 않습니다.

`37-client-subms-lan`의 16 jobs/4 blocks/4×2를 확인했습니다. 총 7,680 complete bodies와 7,680 source records, 64,197,880 payload bytes입니다. 모든 job은 body/header 각 480개·avg 60 FPS입니다. pending write/body join·zero body·누락 low는 모두 0입니다. 실제 창은 2026-11-01T04:27:56.071501+09:00 → 2026-11-01T04:28:54.374781+09:00입니다.

준비10·하네스33·부모8·분석6 파일 해시, 실제 environment의 source hash 33개 및 동결 transform과 생성 scene의 바이트 일치, 16개 전체 raw capture의 레코드/순서/새 SHA, 모든 source payload의 FRAME n/T/scene·prewrite clock bracket·write-end projection·body join을 검사했습니다. capture에 연속된 raw FRAME header도 모두 같은 source n/T/scene에 연결됩니다. source nonce/PID/starttime/UID, 실제 네 창 종료 fence, pidfd SIGUSR2와 원래 5초 수집 deadline, strict LAN/TCP firewall·64 endpoint file/32 live-server 검증이 일치합니다. 원본 입력은 바꾸지 않았습니다.

Baseline main-A/B는 CAN 수정 main `935/c68`의 동일 바이너리이고 후보는 `ca0/91e`입니다. 공통 36 방식의 child SET→GET assert→exec를 사용합니다. native ioctl 결과 sidecar나 이후 geometry 보존을 별도로 증명한 것은 아닙니다.

FPS는 인접 complete-body gap의 1%/5% low와 avg 순서입니다. trailing gap은 low에 포함되지 않으므로 별도로 max gap과 경계도 재산출했습니다.

| Scene / eval | SSH 1/5/avg | CAN main-A 1/5/avg | CAN main-B 1/5/avg | Subms 1/5/avg |
|---|---:|---:|---:|---:|
| compress / 0 | 46.3/50.8/60.0 | 41.9/47.6/60.0 | 42.3/47.7/60.0 | 42.8/47.7/60.0 |
| compress / 1 | 47.7/50.3/60.0 | 42.5/46.5/60.0 | 44.2/46.7/60.0 | 43.5/47.3/60.0 |
| bandwidth / 0 | 44.2/51.3/60.0 | 41.6/47.3/60.0 | 42.8/46.2/60.0 | 42.6/47.1/60.0 |
| bandwidth / 1 | 46.4/49.3/60.0 | 40.0/46.3/60.0 | 40.9/45.1/60.0 | 42.0/44.9/60.0 |

후보는 SSH 대비 4개 평가 모두 1%/5% low가 낮고 avg는 같습니다. 두 동일 baseline 대비는 혼합입니다. compress i0는 소폭 개선/동률, i1는 1%가 A보다 높고 B보다 낮습니다. random i0는 1%가 A/B 사이·5%가 A/B 사이이고, i1는 1%가 양쪽보다 높지만 5%가 양쪽보다 낮습니다. 일관된 SSH-tail 해소나 baseline 개선을 보이지 않습니다. 작은 차이를 통계적 유의성·확정 회귀로 일반화하지 않습니다.

| Scene / eval | Subms age median/p95 ms | max gap ms | client/server CPU % | client/server RSS KiB | RX kbit/s |
|---|---:|---:|---:|---:|---:|
| compress / 0 | 4/7 | 26 | 0.89/1.22 | 2692/5784 | 227 |
| compress / 1 | 4/7 | 25 | 0.79/1.11 | 2692/5824 | 228 |
| bandwidth / 0 | 5/8 | 27 | 1.13/1.52 | 2708/5860 | 2722 |
| bandwidth / 1 | 5/9 | 31 | 1.27/1.67 | 2732/5844 | 2717 |

후보 age는 compress 4/7 ms, random 5/8–9 ms로 baseline과 유사하고 SSH 1–2/3–5 ms보다 높습니다. 자원은 보조 지표로 유지하며 CPU 하나로 자동 기각하지 않습니다.

CPU admission은 startup 19%, phase 17/27/24/19%로 첫 <30% 표본을 적용했습니다. phase runtime max 38/38/35/30%, external max 105.6/78.8/62.1/61.2%는 기록된 맥락이며 탈락 조건이 아닙니다. P-core affinity 증거는 없습니다.

자원 대조는 batch→summary→CSV의 저장값 일치와 agent.py:229–263,279–289,397–409의 단위/공식 검토입니다. 원시 RX/CPU counter endpoint·midpoint RSS sample이 따로 남지 않아 독립 재측정은 아닙니다. network는 client 비-loopback 수신 decimal kbit/s, RSS는 midpoint 이름별 프로세스 합 KiB, CPU는 transport 프로세스/스레드의 한 코어 대비 %입니다. Python source/observer CPU를 포함하지 않습니다.

한계: ppoll 분기 횟수·실제 wait duration trace가 없어 활성화나 tail 원인·해결을 주장하지 않습니다. whole terminal emulator 재실행/GPU/모든 terminal 검증은 하지 않았습니다. 같은 CAN baseline·RAM/geometry 방법 내 집중 비교이고 이전 unfixed main 또는 34와 절대 수치 pooling을 하지 않습니다. `run-status.json`은 analysis_pending 문구가 남아 있지만 분석 manifest/결과와 root terminal9586이 완료 근거입니다.

감사 재현: `PYTHONDONTWRITEBYTECODE=1 python3 "research_notes/LAN 원거리 low 성능 연구/subms37-actual-audit.py"` → `valid_saved_audit: true`, `errors: []`. JSON에 per-job 값·경계 gap·source/capture SHA와 검사 범위를 보존했습니다.
