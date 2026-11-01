60은 PR7 후보48의 full32 source에 known-slow TCP quarter fresh-wait55를 결합하는 미빌드 연구 소스다. Margin module을 먼저 유지하고 quarter module을 뒤에 붙여, 이후 PR8을 PR7 위의 작은 변경으로 검토할 수 있다. 새 state/API/options/dependency는 없다. Production 변경은 검토된 두 정책뿐이며 proto/client/global screen_gap/UDP를 추가로 바꾸지 않는다.

가설은 서로 다른 screen_due_at 분기의 이득이 함께 나타날 수 있다는 것이다. 48은 no-slow low-queue TCP animation의 raw period×0.9(기존8..20ms clamp), 55는 known-slow TCP의 fresh wait을 min(FRESH_WAIT,gap/4)로 제한한다. 한 호출에서는 분기가 배타적이지만 startup/추정 feedback가 후속 분기를 바꿀 수 있으므로 효과의 가산성과 회귀 부재를 주장하지 않는다.

Fixture 충돌 하나를 채택했다. 55의 세 번째 fixture가 slow_rate를 지운 뒤 expected deadline을 output_gap으로 검사하던 지점은 결합 소스에서 margin branch30을 통과한다. 그 expectation 한 개만0.9×gap/clamp로 교정했다. Known-slow/UDP/80ms cap/atomicity 의도와 나머지 assertion은 유지하며 margin3 fixture는 원문 exact다. 55 역변환은 이 assertion을 원문으로 복원하는 절차를 명시해 full32 exact로 확인했다. 48 역변환은 adapted quarter module과 quarter predicate만 제거하면 full32 exact다. 두 정책과 두 module을 제거하면 common295f full32 exact다.

현재 근거의 손해를 보존한다. 동일 quarter predicate의 old19 RTT300 random 반복0은24.5avg/4.0·12.0low 대 main49.7/14.0·45.2였다. 다른 parent/방법이므로 새55 결과로 지우지 않는다. Margin51의1M random low1은8.5/9.7로 A/A보다 낮았고, predeclared53의 반복1 low1도11.4 대13.2–17.5였다. 55의57은1M random에서 두 반복 FPS 세 지표가 A/A보다 높았지만 1M dashboard 반복0 low38.6/45.1과256k dashboard 반복1 low39.7/45.3은 A/A보다 낮았고,64k dashboard 반복0 age95 441ms 대360/367ms 비용이 있었다. 56 LAN low는 SSH보다 낮았다. 결합 효과와 비용은 미측정이다.

FPS1%/5%low와avg를 주로 평가하고 CPU/RSS는 보조로 유지한다. Age/traffic/recovery 비용은 별도로 제시하며 임의 점수나 단일 손해 자동탈락 기준을 만들지 않는다. 승격 전 core/slow/moving 모두4병렬×2평가가 필요하다. 미빌드이므로 actual SHA=null/source_build_verified=False다. Planned candidate target6/library116+2ignored, control quarter1fail2pass 및 existingmargin3pass는 아직 실행하지 않은 예상일 뿐이다.

Negative control은 exact48 production에 동일한 adapted quarter3 fixture만 붙였다. Old55 control과 동일 fixture라고 주장하지 않는다. Source/hash 복사만 했으며 build/test/Docker/capture 실행은0이다. Parent와 기존 실제 원자료는 수정하지 않았다.
