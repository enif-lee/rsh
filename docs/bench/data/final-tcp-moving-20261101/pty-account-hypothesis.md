# PTY account lookup v2 — source-only

CAN-v1 `pty-name-source`와 combined-v1 `fps-combined-pty-name-source`를 새 이름으로 복제했습니다. 유일한 변경은 user_account 내부 getpwuid를 POSIX getpwuid_r로 바꾼 것입니다. caller-owned passwd/char buffer를 쓰고 sysconf의 양수 제안 크기를 사용하며 미확정·비양수이면16KiB로 시작합니다. ERANGE이면 checked doubling으로 재시도합니다. 조회 오류·null result·reserve 실패·크기 overflow는 기존 ENV 초기값을 유지합니다. 성공 필드는 buffer가 살아 있는 동안 c_string으로 owned String에 복사하고, 이후 shell 비어 있음/경로 확인과 home 빈 값 처리는 그대로입니다. mutex·새API·옵션·의존성·EOF 변경은 없습니다.

계약 근거: [Linux man-pages getpwuid_r](https://man7.org/linux/man-pages/man3/getpwnam.3.html)의 caller buffer/result/ERANGE/sysconf 규칙이며 POSIX.1-2008 API입니다. 16KiB fallback은 해당 문서 예시와 같습니다. 새 플랫폼 동작을 검증했다고 해석하지 않습니다. Linux/macOS 컴파일·실행은 아직 하지 않았습니다.

v1의 Linux ptsname_r와 비Linux 기존 분기는 그대로입니다. 동일한64-spawn fixture와 기존 Bash fixture를 바꾸지 않았고 combined controller 바이트도 그대로입니다. 부모 전체91/32 파일 역변환 exact와 manifest/patch/proof를 보존했습니다. v1 소스·해시·로그는 수정하지 않았습니다.

v1 실제 stress 실패는 status256과 BusyBox `sync: unrecognized option: c`입니다. static passwd 문자열이 잘못 복사되어 shell이 /bin/sync가 됐다면 이 출력과 부합하지만, 선택된 shell 경로와 call/copy overlap을 기록한 것은 아니므로 정확한 경로는 추론입니다. 최초 attach Detached 실패의 원인으로도 확장하지 않습니다. 64 fixture는 그대로이며 v2가 통과할 것이라고 미리 주장하지 않습니다. 빌드·테스트·Docker·실측0회입니다.
