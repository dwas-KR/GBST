# GBST v2.0.0 JSON 다국어 개발 가이드

## 현재 지원 범위

- 언어: 한국어(ko), 영어(en), 러시아어(ru), 일본어(ja), 번체 중국어(zh-TW), 베트남어(vi), 그리스어(el), 힌디어(hi), 조지아어(ka), 네덜란드어(nl), 아랍어(ar), 스페인어(es).
- 각 JSON 카탈로그: 343개 키 (2026-11-05 기준).
- 번역 키 검증: python scripts/audit_i18n.py
- Rust 테스트: cargo test --workspace --locked

## 현재 구현 파일

- crates/gbst-core/lang/*.json: 화면과 로그의 번역 문구/플레이스홀더.
- crates/gbst-core/src/log_i18n.rs: 한국어 원문 로그를 의미 기반 키로 매핑하여 선택 언어로 출력.
- crates/gbst-core/src/operation_i18n.rs: 작업 및 상태 문구 번역.
- crates/gbst-core/src/language.rs: 기존 번역 경로 호환, 언어 설정 로드·저장.
- crates/gbst-gui/src/main.rs, ltbox_visual.rs: 언어 변경에 맞춘 UI 재렌더링.
- crates/gbst-core/tests/runtime_log_source_samples.txt: 실행 로그 번역 회귀 테스트 샘플.

이전 설명에 등장했던 crates/gbst-core/examples/locale_log_audit.rs와
generate_log_catalogs.py는 미사용 실험용 파일이므로 최종 프로젝트에서 제거했습니다.
사용하지 않는 예제를 재생성하거나 실행할 필요가 없습니다.

## 번역 추가 방법

1. 신규 UI 문구는 ui.*, 실행 로그는 해당 동작의 의미에 맞는 키를 선택합니다.
2. 12개 JSON에 동일한 키를 추가하고 자연스러운 표현을 입력합니다.
3. {package}, {path}, {error}, {count} 등 템플릿 변수를 12개 언어에서 일치시킵니다.
4. 패키지 ID, APK 경로, SHA-256, 모델명 등 기술적 식별자는 번역하지 않습니다.
5. 신규 한국어 로그는 translate_log()의 키 매핑 또는 동적 매핑에 추가합니다.
6. python scripts/audit_i18n.py, cargo test --workspace --locked를 실행합니다.

## 최근 변경

- 대시보드의 알 수 없음 등 상태값이 선택 언어로 보이도록 개선.
- Google 서비스 단계별 반복 로그는 시작 한 번만 표시, 진행률은 구조화 이벤트로 계속 갱신.
- 태블릿 폴더 4개(/storage/self/primary/dwas_kr, /storage/self/primary/GBST,
  /data/local/tmp/dwas_kr, /data/local/tmp/GBST)의 정리 메시지를 다국어로 반영.
- ADB 서버 점유 경고, 종료 버튼, 종료 중 표시 및 성공/실패 메시지를 다국어로 반영.

## 참고와 라이선스

LTBox v3.3.5의 820×720 논리 픽셀 창 크기와 ADB 서버 충돌 해결 동작을 참고했으나,
GPLv3 코드를 그대로 복사해 GBST에 병합하지 않았습니다. 기존 GBST ADB 클라이언트에
이미 있던 host:kill 기능을 GUI에 연결하고 GBST 고유 구현을 유지합니다.
