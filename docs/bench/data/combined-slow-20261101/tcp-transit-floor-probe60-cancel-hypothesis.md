# Transit floor + broad probe60 + CAN: source-only hypothesis

현재 단계는 source-only 준비입니다.

이미 검토된 `tcp-transit-floor-probe60-source`를 복제했습니다. production 변경은 `Terminal::diff_from`의 SYNC_BEGIN 앞에 CAN 1 byte를 내보내는 것뿐입니다. `diff-cancel-tests.rs`의 검토된 client fixture 3개를 원문 그대로 추가했습니다. 기존 far 7개와 broad probe60 4개 fixture, floor>=20ms gate·lossless0.6/else0.8·server/UDP 경로는 그대로입니다. sub-ms wait·near 조건·새 상태/API/옵션/의존성은 없습니다.

FPS 가설은 far의 WAN/256k dashboard 이득과 broad0.6의 회복/1M random 이득을 한 후보에서 검증하는 것입니다. 결합 효과는 미측정이며 더해진다고 가정하지 않습니다. 1%/5% low와 avg FPS를 우선하고 age/RX 비용은 별도 tradeoff로 보존합니다. CPU/RSS는 보조 지표이며 단일 비용으로 자동 기각하지 않습니다.

CAN은 이전 nested-CSI/SGR 경계에서 확인된 frozen pyte와 Rust/tmux 구현 차이를 완화하는 별도 호환 변경입니다. 새 CAN이 CPU/FPS 이득의 원인이라고 가정하지 않습니다. 원래34 raw/metric은 불변입니다.

역변환은 정확한 fixture suffix 제거와 CAN prefix 제거만으로 부모 25 canonical/31 full 파일 전체 바이트와 일치합니다. manifest/patch/proof를 같이 보존했습니다. target14/full110+2ignored는 계획 숫자이며 실행 결과가 아닙니다. 이번 준비에서 테스트·빌드·Docker·실측은 0회입니다. 후속 빌드/실측은 root가 조율합니다.
