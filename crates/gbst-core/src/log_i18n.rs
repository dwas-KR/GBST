// SPDX-License-Identifier: GPL-3.0-only
//! Locale JSON based runtime log renderer. Source logs remain stable Korean IDs
//! for debugging, while GUI renders them in the selected language on demand.
use crate::language::{LanguageOption, translate_runtime_text};
use std::collections::HashMap;
use std::sync::LazyLock;

const CATALOGS: [(&str, &str); 12] = [
    ("ko", include_str!("../lang/ko.json")),
    ("en", include_str!("../lang/en.json")),
    ("ru", include_str!("../lang/ru.json")),
    ("ja", include_str!("../lang/ja.json")),
    ("zh-TW", include_str!("../lang/zh-TW.json")),
    ("vi", include_str!("../lang/vi.json")),
    ("el", include_str!("../lang/el.json")),
    ("hi", include_str!("../lang/hi.json")),
    ("ka", include_str!("../lang/ka.json")),
    ("nl", include_str!("../lang/nl.json")),
    ("ar", include_str!("../lang/ar.json")),
    ("es", include_str!("../lang/es.json")),
];

static TABLES: LazyLock<HashMap<&'static str, HashMap<String, String>>> = LazyLock::new(|| {
    CATALOGS
        .iter()
        .map(|(lang, json)| {
            let catalog: HashMap<String, String> = serde_json::from_str(json)
                .unwrap_or_else(|e| panic!("{lang} log JSON is invalid: {e}"));
            (*lang, catalog)
        })
        .collect()
});

fn format_key(language: LanguageOption, key: &str, args: &[(&str, &str)]) -> Option<String> {
    let template = TABLES
        .get(language.code())
        .and_then(|values| values.get(key))
        .or_else(|| TABLES.get("en").and_then(|values| values.get(key)))?;
    let mut output = template.clone();
    for (arg, value) in args {
        output = output.replace(&format!("{{{arg}}}"), value);
    }
    Some(output)
}

/// Returns legacy translation phrases from the JSON catalogs.
/// The old Rust translation table stays as fallback during migration.
pub fn legacy_phrase(language: LanguageOption, key: &str) -> Option<&'static str> {
    let id = format!("legacy.{key}");
    TABLES.get(language.code())?.get(&id).map(String::as_str)
}

pub fn log_label(language: LanguageOption, key: &str) -> String {
    format_key(language, key, &[]).unwrap_or_else(|| key.to_string())
}

pub fn log_label_with_args(language: LanguageOption, key: &str, args: &[(&str, &str)]) -> String {
    format_key(language, key, args).unwrap_or_else(|| key.to_string())
}

/// Parse source log messages into semantic keys + named template arguments.
/// Unrecognized older messages fall back to GBST's existing 12-language
/// translator, allowing gradual migration without losing information.
pub fn translate_log(language: LanguageOption, raw: &str) -> String {
    if raw.trim().is_empty() {
        return raw.to_owned();
    }

    let static_key = match raw {
        "[GBST] 기기에 Google Service 설치, 복구, 업데이트를 시작합니다." => {
            Some("gbst.start")
        }
        "[GBST] 기기에 Google Services APK 버전이 낮으므로 업데이트를 진행합니다." => {
            Some("gbst.update")
        }
        "[GBST] Google Services 복구 및 업데이트가 완료 되었습니다." => {
            Some("gbst.services_done")
        }
        "[USB] Google 작업 중에는 USB 진단을 실행할 수 없습니다." => {
            Some("usb.operation_busy")
        }
        "[USB] 연결 상태 및 인증 확인을 시작합니다." => Some("usb.check_start"),
        "[ADB] 외부 ADB 서버 종료를 요청했습니다." => Some("adb.server.kill_start"),
        "[ADB] 외부 ADB 서버 종료 완료. USB 연결을 다시 검사합니다." => {
            Some("adb.server.kill_done")
        }
        "[USB] 연결 완료: Android ADB 기기가 인증되었습니다." => Some("usb.ready"),
        "[USB] 기기가 Recovery ADB 모드로 연결되어 있습니다." => Some("usb.recovery"),
        "[USB] ADB 기기를 찾지 못했습니다. 데이터 케이블과 USB 디버깅을 확인하세요." => {
            Some("usb.no_device")
        }
        "[USB] ADB 기기가 여러 대 감지되었습니다. 한 대만 연결하세요." => {
            Some("usb.multiple")
        }
        "[USB] ADB 인증이 필요합니다. 태블릿 화면에서 USB 디버깅 허용을 선택하세요." => {
            Some("usb.unauthorized")
        }
        "[USB] USB ADB 인터페이스가 점유 중입니다. 다른 ADB 프로그램을 닫고 재검사하세요." => {
            Some("usb.server_blocking")
        }
        "[GBST] 작업이 완료되었습니다." => Some("status.success"),
        "[GBST] PC 로그 및 태블릿 dwas_kr 폴더 정리 완료" => Some("log_cleanup.done"),
        "[ADB] 작업 종료 후 태블릿의 두 dwas_kr 임시 폴더 정리 및 검증 중..." => {
            Some("adb.dwas.cleanup_after")
        }
        "[ADB] /storage/self/primary/dwas_kr 및 /data/local/tmp/dwas_kr 삭제 확인 완료" => {
            Some("adb.dwas.cleanup_done")
        }
        "[ADB] 작업 종료 후 태블릿의 두 dwas_kr 임시 폴더 정리..." => {
            Some("adb.dwas.cleanup_after")
        }
        "[ADB] 작업 종료 후 태블릿의 네 GBST 데이터/캐시 폴더 정리..." => {
            Some("adb.dwas.cleanup_after")
        }
        "[ADB] 삭제 완료" => Some("adb.dwas.cleanup_done"),
        "[ADB] 작업 시작 전 태블릿의 두 dwas_kr 폴더를 정리합니다." => {
            Some("adb.dwas.cleanup_before")
        }
        "[ADB] 작업 시작 전 태블릿의 네 GBST 데이터/캐시 폴더를 정리합니다." => {
            Some("adb.dwas.cleanup_before")
        }
        "[ADB] 작업 시작 전 dwas_kr 폴더 정리 및 검증 완료" => {
            Some("adb.dwas.cleanup_before_done")
        }
        "[ADB] 작업 시작 전 네 GBST 데이터/캐시 폴더 정리 및 검증 완료" => {
            Some("adb.dwas.cleanup_before_done")
        }
        "[GBST] 취소를 요청했습니다. 현재 실행 중인 ADB 명령/재부팅 이후 안전한 단계에서 중지됩니다." => {
            Some("gbst.cancel_requested")
        }
        "[GBST] 라이선스 파일을 찾지 못했습니다. GitHub 저장소의 LICENSE를 확인해 주세요." => {
            Some("gbst.license_missing")
        }
        "[ADB] USB 디버깅 인증 필요:" => Some("adb.authorization_short"),
        "[GBST] 작업 실패:" => Some("gbst.failed_short"),
        "[GBST] 작업을 마무리 합니다." => Some("status.finishing"),
        "[ADB] 이전 작업의 GBST 임시 파일이 남아 있는지 확인합니다." => {
            Some("adb.temp.previous")
        }
        "[ADB] 이전 GBST 임시 파일 정리 확인 완료" => Some("adb.temp.previous_done"),
        "[ADB] GBST 기기 임시 폴더 삭제 및 잔여 파일 검증 중..." => {
            Some("adb.temp.cleaning")
        }
        "[ADB] /data/local/tmp/dwas_kr/GBST 삭제 및 검증 완료" => Some("adb.temp.done"),
        "[ADB] 기기 감지 완료" => Some("adb.detect_done"),
        "[ADB] 기기 재부팅 완료" => Some("adb.reboot_done"),
        "[ADB] 기기 재부팅 중..." => Some("adb.rebooting"),
        "[ADB] 기기에 연결할 준비합니다." => Some("adb.connecting"),
        "[APK] APK 설치 중" => Some("apk.installing"),
        "[APK] GitHub 텍스트를 확인했습니다." => Some("apk.catalog_done"),
        "[APK] 다운로드한 파일 및 폴더 제거 완료" => Some("apk.cache_done"),
        "[안내] PC(노트북)와 연결한 태블릿에 잠금 해제 → 메세지 창 왼쪽 중간 체크 박스 체크 → 오른쪽 하단 Allow(허용)를 터치해주세요." => {
            Some("adb.authorization_guide")
        }
        "[ADB] USB 디버깅 인증 필요: 태블릿의 잠금을 해제한 뒤 '이 컴퓨터에서 항상 허용'을 선택하고 '허용'을 누르세요." => {
            Some("adb.authorization_guide")
        }
        "[안내] 동일한 질문을 주시는 분들이 많아 안내 드립니다." => {
            Some("guide.wait_1")
        }
        "[안내] 프로그램이 작업 중이므로 작업이 멈추거나, 중단되거나," => {
            Some("guide.wait_2")
        }
        "[안내] 연결이 끊긴 것이 아니니 안심하시고 작업이 완료 될 때까지" => {
            Some("guide.wait_3")
        }
        "[안내] PC(노트북), 케이블, 기기를 가만히 둔 상태로 1분~5분 대기해 주세요." => {
            Some("guide.wait_4")
        }
        "[Device] 설치된 Google Services APK 버전과 GBST 정보를 다시 조회합니다." => {
            Some("device.refresh")
        }
        _ => None,
    };
    if let Some(key) = static_key {
        if let Some(value) = format_key(language, key, &[]) {
            return value;
        }
    }

    // Legacy logs can remain in a UI session after an application update.
    // Render the previous byte-count format with the new, concise wording.
    if let Some(value) = raw.strip_prefix("[GBST] APK 캐시 정리 완료: ")
        && let Some((count, _)) = value.split_once("개 파일, ")
    {
        return format_key(language, "apk.cache_removed", &[("count", count)])
            .unwrap_or_else(|| raw.to_string());
    }
    if raw.starts_with("[GBST] PC 로그 및 태블릿 dwas_kr 폴더 정리 완료: ") {
        return format_key(language, "log_cleanup.done", &[]).unwrap_or_else(|| raw.to_string());
    }

    // Every user-facing GBST/ADB cleanup message is a stable semantic JSON
    // key, not a hard-coded Korean phrase displayed in non-Korean locales.
    if let Some(value) = raw
        .strip_prefix("[GBST] APK 캐시 정리 완료: ")
        .and_then(|s| s.strip_suffix("개 파일"))
    {
        return format_key(language, "apk.cache_removed", &[("count", value)])
            .unwrap_or_else(|| raw.to_string());
    }

    // Final verification may discover a user-0 disabled service after the
    // normal four-stage restore. Keep technical package IDs unchanged while
    // translating all recovery and diagnosis messages in the GUI.
    if let Some(rest) = raw.strip_prefix("[ADB] ")
        && let Some((package, action)) = rest.split_once(": ")
        && package.starts_with("com.")
    {
        let key = match action {
            "사용자 0에 등록되지 않아 최종 검증을 통과할 수 없습니다." => {
                Some("postflight.missing")
            }
            "사용자 0에서 사용 중지 상태입니다. 활성화를 시도합니다." => {
                Some("postflight.disabled")
            }
            "사용자 0 활성화 명령 오류. 진단 로그를 확인하세요." => {
                Some("postflight.command_error")
            }
            "사용자 0 활성화 및 검증 완료" => Some("postflight.enabled"),
            "사용자 0 활성화 후에도 사용 중지 상태입니다." => {
                Some("postflight.still_disabled")
            }
            _ => None,
        };
        if let Some(key) = key
            && let Some(text) = format_key(language, key, &[("package", package)])
        {
            return text;
        }
        if let Some(value) = action.strip_prefix("사용자 0 활성화 재시도 ")
            && let Some((attempt, total)) = value.split_once('/')
            && let Some(text) = format_key(
                language,
                "postflight.retry",
                &[("package", package), ("attempt", attempt), ("total", total)],
            )
        {
            return text;
        }
    }

    // Preserve error diagnostics as data, but localize the action and
    // surrounding message. No file paths, package names or USB IDs are
    // modified by this renderer.
    const FAILURE_ACTIONS: &[(&str, &str)] = &[
        ("[USB] 진단 오류: ", "failure.usb_diagnostic"),
        ("[ADB] 외부 ADB 서버 종료 실패: ", "failure.adb.kill"),
        (
            "[USB] 진단 로그 폴더 생성 실패: ",
            "failure.usb_diag_folder_create",
        ),
        (
            "[USB] 진단 로그 폴더 열기 실패: ",
            "failure.usb_diag_folder_open",
        ),
        (
            "[GBST] 면책 조항 확인 저장 실패: ",
            "failure.save_disclaimer",
        ),
        (
            "[APK] GitHub APK 링크 파일 열기 실패: ",
            "failure.open_apk_links",
        ),
        ("[APK] APK 폴더 생성 실패: ", "failure.create_apk_folder"),
        ("[APK] APK 폴더 열기 실패: ", "failure.open_apk_folder"),
        (
            "[Log] 작업 로그 폴더 생성 실패: ",
            "failure.create_log_folder",
        ),
        (
            "[Log] 작업 로그 폴더 열기 실패: ",
            "failure.open_log_folder",
        ),
        (
            "[GBST] APK 캐시 정리에 실패했습니다: ",
            "failure.clean_apk_cache",
        ),
        (
            "[GBST] 로그 및 기기 폴더 정리 중 오류: ",
            "failure.clean_logs_device",
        ),
        ("[GBST] GitHub 링크 열기 실패: ", "failure.open_github"),
        (
            "[GBST] LTBox GitHub 링크 열기 실패: ",
            "failure.open_ltbox_github",
        ),
        (
            "[GBST] GitHub 이슈 링크 열기 실패: ",
            "failure.open_github_issues",
        ),
        ("[GBST] 라이선스 문서 열기 실패: ", "failure.open_license"),
        ("[설정] 화면 테마 저장 실패: ", "failure.save_theme"),
        ("[설정] 강조 색상 저장 실패: ", "failure.save_accent"),
    ];
    for (prefix, action_key) in FAILURE_ACTIONS {
        if let Some(detail) = raw.strip_prefix(prefix) {
            let action = log_label(language, action_key);
            let tag = if prefix.starts_with("[설정]") {
                log_label(language, "log.tag.settings")
            } else {
                prefix
                    .split_once(']')
                    .map(|(tag, _)| &tag[1..])
                    .unwrap_or("GBST")
                    .to_string()
            };
            return format_key(
                language,
                "failure.generic",
                &[("tag", &tag), ("action", &action), ("detail", detail)],
            )
            .unwrap_or_else(|| raw.to_string());
        }
    }

    if let Some(rest) = raw.strip_prefix("[Update] 새 GBST 버전을 찾았습니다: 현재 ")
        && let Some((current, latest)) = rest.split_once(" → 최신 ")
    {
        return format_key(
            language,
            "update.version_found",
            &[("current", current), ("latest", latest)],
        )
        .unwrap_or_else(|| raw.to_string());
    }
    if let Some(error) = raw
        .strip_prefix("[Device] 사용자가 설정한 화면 꺼짐 값: 알 수 없음 (")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return format_key(language, "device.timeout_unknown", &[("error", error)])
            .unwrap_or_else(|| raw.to_string());
    }
    if let Some(error) = raw
        .strip_prefix("[Device] 사용자가 설정한 화면 밝기 값: 알 수 없음 (")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return format_key(language, "device.brightness_unknown", &[("error", error)])
            .unwrap_or_else(|| raw.to_string());
    }
    if let Some(key) = raw
        .strip_prefix("[Device] ")
        .and_then(|rest| rest.strip_suffix(" 원래 값 확인 실패: 복원 제외"))
    {
        return format_key(language, "device.backup_setting_unknown", &[("key", key)])
            .unwrap_or_else(|| raw.to_string());
    }
    if let Some(rest) = raw.strip_prefix("[Device] 사용자 설정 ")
        && let Some((key, error)) = rest.split_once(" 복원 실패: ")
    {
        return format_key(
            language,
            "device.restore_setting_error",
            &[("key", key), ("error", error)],
        )
        .unwrap_or_else(|| raw.to_string());
    }
    if let Some(error) =
        raw.strip_prefix("[APK] 원격 목록을 읽을 수 없어 내장 카탈로그를 사용합니다: ")
    {
        return format_key(language, "apk.catalog_fallback", &[("error", error)])
            .unwrap_or_else(|| raw.to_string());
    }
    if let Some(error) = raw.strip_prefix(
        "[APK] GitHub APK 링크 파일이 평문 카탈로그 형식이 아니어서 프로그램에 포함된 공개 GBST_apk.txt를 사용합니다: "
    ) {
        return format_key(language, "apk.catalog_invalid", &[("error", error)])
            .unwrap_or_else(|| raw.to_string());
    }
    if let Some(rest) = raw.strip_prefix("[ADB] dwas_kr 폴더 정리 ")
        && let Some((attempt, error)) = rest.split_once(" 실패: ")
    {
        return format_key(
            language,
            "adb.dwas.cleanup_retry_failed",
            &[("attempt", attempt), ("error", error)],
        )
        .unwrap_or_else(|| raw.to_string());
    }

    if let Some(lang) = raw.strip_prefix("[GBST] GUI 초기화 완료 / 언어 설정: ") {
        let translated_name = if lang.trim() == "한국어" {
            log_label(language, "language.name.korean")
        } else {
            lang.to_string()
        };
        return format_key(language, "gbst.gui_init", &[("language", &translated_name)])
            .unwrap_or_else(|| raw.into());
    }
    if let Some(rest) = raw.strip_prefix("[ADB] Android ")
        && let Some((android, rest)) = rest.split_once(" 단계 재부팅 ")
        && let Some(numbers) = rest.strip_suffix("을 진행합니다.")
        && let Some((index, total)) = numbers.split_once('/')
    {
        return format_key(
            language,
            "adb.reboot_count",
            &[("android", android), ("index", index), ("total", total)],
        )
        .unwrap_or_else(|| raw.into());
    }
    if let Some(sec) = raw
        .strip_prefix("[ADB] 작업 안정화 대기: ")
        .and_then(|s| s.strip_suffix("초"))
    {
        return format_key(language, "adb.delay", &[("seconds", sec.trim())])
            .unwrap_or_else(|| raw.into());
    }
    if let Some(value) = raw.strip_prefix("[Device] 사용자가 설정한 화면 꺼짐 값: ") {
        let value = value.trim();
        let is_disabled = value == "화면 꺼짐 비활성화"
            || value == "사용 안 함"
            || value.parse::<u64>().is_ok_and(|number| number >= 600_000);
        let key = if is_disabled {
            "device.timeout_disabled"
        } else {
            "device.timeout_value"
        };
        return format_key(language, key, &[("value", value)]).unwrap_or_else(|| raw.into());
    }
    if let Some(value) = raw.strip_prefix("[Device] 사용자가 설정한 화면 밝기 값: ") {
        return format_key(language, "device.brightness", &[("value", value)])
            .unwrap_or_else(|| raw.into());
    }
    if let Some(value) = raw
        .strip_prefix("[Device] 사용자가 설정한 화면 밝기 값(")
        .and_then(|s| s.strip_suffix(")으로 복원합니다."))
    {
        return format_key(language, "device.restore_brightness", &[("value", value)])
            .unwrap_or_else(|| raw.into());
    }
    if raw == "[Device] 사용자가 설정한 값으로 복원합니다." {
        return format_key(language, "device.restore_timeout", &[]).unwrap_or_else(|| raw.into());
    }
    if let Some(rest) = raw
        .strip_prefix("[Device] 사용자 설정 ")
        .and_then(|s| s.strip_suffix(" 복원 완료"))
    {
        return format_key(language, "device.setting_restored", &[("key", rest)])
            .unwrap_or_else(|| raw.into());
    }
    if let Some(rest) =
        raw.strip_prefix("[Device] GBST 기기 정보 재조회 완료, Google Services 상태: ")
    {
        let status = if rest.trim() == "정상" {
            log_label(language, "device.normal")
        } else {
            translate_runtime_text(language, rest.trim())
        };
        return format_key(language, "device.refreshed", &[("status", &status)])
            .unwrap_or_else(|| raw.into());
    }
    if let Some(rest) = raw.strip_prefix("[Device] 감지된 기기에 정보, 제조사: ") {
        if let Some((manufacturer, rest)) = rest.split_once(" / 모델명: ")
            && let Some((model, android)) = rest.split_once(" / Android 버전: ")
        {
            return format_key(
                language,
                "device.detected",
                &[
                    ("manufacturer", manufacturer),
                    ("model", model),
                    ("android", android),
                ],
            )
            .unwrap_or_else(|| raw.into());
        }
    }

    // Persist the raw package identifier and device path verbatim. Translate
    // only human-facing explanatory text; never translate shell arguments.
    for prefix in ["[APK] ", "[ADB] "] {
        if let Some((package, rest)) = raw
            .strip_prefix(prefix)
            .and_then(|line| line.split_once(": "))
        {
            let entries: [(&str, &str); 8] = [
                (
                    "사용자 0 등록 상태 복구를 먼저 시도합니다.",
                    "apk.restore_begin",
                ),
                ("사용자 0 패키지 등록 복원 완료", "apk.restore_done"),
                (
                    "설치 버전이 같거나 높아 기존 패키지를 재사용합니다.",
                    "apk.reused",
                ),
                ("기기 전송 파일 크기 및 SHA-256 검증 완료", "apk.verified"),
                (
                    "APK 전송 검증 완료, Android 패키지 설치 명령을 실행합니다.",
                    "apk.install_begin",
                ),
                ("설치 성공 및 패키지 등록 확인", "apk.installed"),
                ("USB 기기 연결 오류", "adb.usb_error"),
                ("기기 연결을 재시도합니다.", "adb.retry"),
            ];
            if let Some((_, key)) = entries.iter().find(|(original, _)| rest == *original) {
                if let Some(result) = format_key(language, key, &[("package", package)]) {
                    return result;
                }
            }
            if let Some(percent) = rest
                .strip_prefix("기기 전송 진행률 ")
                .and_then(|s| s.strip_suffix('%'))
            {
                if let Some(result) = format_key(
                    language,
                    "apk.transfer",
                    &[("package", package), ("percent", percent)],
                ) {
                    return result;
                }
            }
            if let Some(path) = rest.strip_suffix(" 경로로 APK 전송을 시작합니다.") {
                if let Some(result) = format_key(
                    language,
                    "apk.push_begin",
                    &[("package", package), ("path", path)],
                ) {
                    return result;
                }
            }
            if let Some(seconds) = rest
                .strip_prefix("설치 명령 반환, 경과 ")
                .and_then(|s| s.strip_suffix("초"))
            {
                if let Some(result) = format_key(
                    language,
                    "apk.install_elapsed",
                    &[("package", package), ("seconds", seconds)],
                ) {
                    return result;
                }
            }
        }
    }

    // Progress values now travel as typed GUI signals. Translate the
    // human-readable stage milestones without embedding a text gauge.
    if let Some(rest) = raw.strip_prefix("[APK] Android ")
        && let Some((android, action)) = rest.split_once(" Google Service APK ")
    {
        let key = match action {
            "파일 다운로드 시작" => Some("apk.download_begin"),
            "파일을 다운로드 중..." => Some("apk.downloading"),
            "파일 다운로드 완료" => Some("apk.download_done"),
            _ => None,
        };
        if let Some(key) = key
            && let Some(translated) = format_key(language, key, &[("android", android)])
        {
            return translated;
        }
    }
    if let Some(stage) = raw
        .strip_prefix("[APK] 구글 서비스 작업 중... (")
        .and_then(|rest| rest.strip_suffix(')'))
        && let Some(translated) = format_key(language, "apk.stage", &[("stage", stage)])
    {
        return translated;
    }
    if let Some(stage) = raw
        .strip_prefix("[APK] 구글 서비스 작업 ")
        .and_then(|rest| rest.strip_suffix("단계 완료"))
        && let Some(translated) = format_key(language, "apk.stage_done", &[("stage", stage)])
    {
        return translated;
    }

    // Legacy text-block logs may still exist in a previous session.
    // Use the same bar-free JSON templates when rendering and exporting them.
    if let Some(rest) = raw.strip_prefix("[APK] [")
        && let Some((bar, rest)) = rest.split_once("] ")
        && let Some((percent, rest)) = rest.split_once("% ")
    {
        if let Some(stage) = rest
            .strip_prefix("구글 서비스 작업 중... (")
            .and_then(|s| s.strip_suffix(')'))
        {
            return format_key(
                language,
                "apk.stage",
                &[("bar", bar), ("percent", percent), ("stage", stage)],
            )
            .unwrap_or_else(|| raw.into());
        }
        if let Some(stage) = rest
            .strip_prefix("구글 서비스 작업 ")
            .and_then(|s| s.strip_suffix("단계 완료"))
        {
            return format_key(
                language,
                "apk.stage_done",
                &[("bar", bar), ("percent", percent), ("stage", stage)],
            )
            .unwrap_or_else(|| raw.into());
        }
        // Previous GBST builds also embedded Unicode progress bars into
        // download start and in-progress notices. Translate these legacy
        // lines without carrying any of the old bar glyphs into the new GUI.
        for (suffix, key) in [
            (
                " Google Service APK 파일 다운로드 시작",
                "apk.download_begin",
            ),
            (
                " Google Service APK 파일을 다운로드 중...",
                "apk.downloading",
            ),
        ] {
            if let Some(android) = rest
                .strip_prefix("Android ")
                .and_then(|message| message.strip_suffix(suffix))
                && let Some(translated) = format_key(language, key, &[("android", android)])
            {
                return translated;
            }
        }
        if let Some(android) = rest
            .strip_prefix("Android ")
            .and_then(|s| s.strip_suffix(" Google Service APK 파일 다운로드 완료"))
        {
            return format_key(
                language,
                "apk.download_done",
                &[("bar", bar), ("percent", percent), ("android", android)],
            )
            .unwrap_or_else(|| raw.into());
        }
    }
    if let Some(rest) = raw.strip_prefix("[Log] GBST ")
        && let Some((flow, path)) = rest.split_once(" 작업 로그를 ")
        && let Some(path) = path.strip_suffix("에 저장합니다.")
    {
        let flow_localized = log_label(
            language,
            match flow {
                "완료" => "flow.complete",
                "실패" => "flow.failed",
                "종료" => "flow.closed",
                "수동" => "flow.manual",
                _ => "flow.other",
            },
        );
        return format_key(
            language,
            "log.saved",
            &[("flow", &flow_localized), ("path", path)],
        )
        .unwrap_or_else(|| raw.into());
    }
    translate_runtime_text(language, raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_json_locales_have_the_same_keys_and_template_parameters() {
        use std::collections::BTreeSet;
        let english = TABLES.get("en").unwrap();
        let keys = english.keys().cloned().collect::<BTreeSet<_>>();
        fn placeholders(template: &str) -> BTreeSet<String> {
            let mut fields = BTreeSet::new();
            let mut remaining = template;
            while let Some((_, rest)) = remaining.split_once('{') {
                let Some((name, trailing)) = rest.split_once('}') else {
                    break;
                };
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    fields.insert(name.to_string());
                }
                remaining = trailing;
            }
            fields
        }
        for lang in LanguageOption::ALL {
            let table = TABLES.get(lang.code()).unwrap();
            assert_eq!(
                table.keys().cloned().collect::<BTreeSet<_>>(),
                keys,
                "{} has missing or extra log translation keys",
                lang.code()
            );
            for (key, english_template) in english {
                assert_eq!(
                    placeholders(&table[key]),
                    placeholders(english_template),
                    "{}: template parameter mismatch for {key}",
                    lang.code()
                );
            }
        }
    }

    #[test]
    fn catalogs_parse_and_all_locales_contain_core_keys() {
        let required = [
            "status.overall",
            "status.install",
            "status.finished",
            "apk.transfer",
            "apk.push_begin",
            "apk.verified",
            "apk.installed",
            "adb.delay",
            "device.timeout_disabled",
            "adb.authorization_guide",
            "adb.temp.done",
            "apk.stage",
            "log.saved",
        ];
        for lang in LanguageOption::ALL {
            let entries = TABLES.get(lang.code()).unwrap();
            for key in required {
                assert!(entries.contains_key(key), "{} missing {key}", lang.code());
            }
        }
    }
    #[test]
    fn dynamic_logs_translate_and_preserve_package_and_path() {
        let text = "[APK] com.google.android.gms: 기기 전송 진행률 30%";
        assert_eq!(
            translate_log(LanguageOption::English, text),
            "[APK] com.google.android.gms: Device upload 30%"
        );
        let raw = "[Device] 사용자가 설정한 화면 꺼짐 값: 600000";
        assert_eq!(
            translate_log(LanguageOption::Korean, raw),
            "[Device] 화면 꺼짐 비활성화"
        );
        assert!(!translate_log(LanguageOption::English, raw).contains("600000"));
        let path = "/data/local/tmp/dwas_kr/GBST/com.google.android.gms.apk";
        let txt = format!("[APK] com.google.android.gms: {path} 경로로 APK 전송을 시작합니다.");
        assert!(translate_log(LanguageOption::Japanese, &txt).contains(path));
    }
    /// Audited snippets from GBST GUI, direct USB ADB, APK downloader, and
    /// fallback catalog. Any new Korean user log must receive a JSON mapping.
    #[test]
    fn source_runtime_log_samples_are_localized_in_all_languages() {
        const SAMPLES: &str = include_str!("../tests/runtime_log_source_samples.txt");
        let lines = SAMPLES
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        assert!(
            lines.len() >= 100,
            "source log audit has unexpectedly few samples"
        );
        for language in LanguageOption::ALL {
            if language == LanguageOption::Korean {
                continue;
            }
            for source in &lines {
                let translated = translate_log(language, source);
                assert!(
                    !translated
                        .chars()
                        .any(|ch| ('\u{AC00}'..='\u{D7A3}').contains(&ch)),
                    "{} still contains Korean text: {}\n=> {}",
                    language.code(),
                    source,
                    translated
                );
            }
        }
    }

    #[test]
    fn ltbox_style_download_and_stage_logs_are_bar_free_in_every_locale() {
        let new_lines = [
            "[APK] Android 13 Google Service APK 파일 다운로드 시작",
            "[APK] Android 13 Google Service APK 파일을 다운로드 중...",
            "[APK] Android 13 Google Service APK 파일 다운로드 완료",
            "[APK] 구글 서비스 작업 중... (1/4)",
            "[APK] 구글 서비스 작업 1단계 완료",
        ];
        let previous_lines = [
            "[APK] [██████··············] 31% 구글 서비스 작업 중... (2/4)",
            "[APK] [████████████████████] 100% 구글 서비스 작업 4단계 완료",
            "[APK] [····················] 0% Android 13 Google Service APK 파일 다운로드 시작",
            "[APK] [██████··············] 30% Android 13 Google Service APK 파일을 다운로드 중...",
            "[APK] [████████████████████] 100% Android 13 Google Service APK 파일 다운로드 완료",
        ];
        for language in LanguageOption::ALL {
            for source in new_lines.into_iter().chain(previous_lines) {
                let translated = translate_log(language, source);
                assert!(
                    !translated.contains('█') && !translated.contains('·'),
                    "{} still rendered a text bar: {}",
                    language.code(),
                    translated
                );
                assert!(
                    !translated.contains("{bar}") && !translated.contains("{percent}"),
                    "{} leaked a template variable: {}",
                    language.code(),
                    translated
                );
                if language != LanguageOption::Korean {
                    assert!(
                        !translated
                            .chars()
                            .any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c)),
                        "{} still contains Korean: {}",
                        language.code(),
                        translated
                    );
                }
            }
        }
    }

    #[test]
    fn requested_cleanup_messages_are_translated_and_keep_counts() {
        const RAW: [&str; 4] = [
            "[GBST] PC 로그 및 태블릿 dwas_kr 폴더 정리 완료",
            "[GBST] APK 캐시 정리 완료: 18개 파일",
            "[ADB] 작업 종료 후 태블릿의 두 dwas_kr 임시 폴더 정리...",
            "[ADB] 삭제 완료",
        ];
        for language in LanguageOption::ALL {
            for raw in RAW {
                let result = translate_log(language, raw);
                assert!(!result.is_empty(), "{}: {}", language.code(), raw);
                if language != LanguageOption::Korean {
                    assert!(
                        !result
                            .chars()
                            .any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c)),
                        "{}: {}",
                        language.code(),
                        result
                    );
                }
                if raw.contains("18개 파일") {
                    assert!(result.contains("18"), "{}: {}", language.code(), result);
                }
            }
        }
        assert_eq!(
            translate_log(LanguageOption::Japanese, RAW[3]),
            "[ADB] 削除が完了しました。"
        );
        assert_eq!(
            translate_log(LanguageOption::Japanese, RAW[1]),
            "[GBST] APKキャッシュの削除完了：18件のファイル。"
        );
    }

    #[test]
    fn runtime_locale_switch_retranslates_the_same_raw_line() {
        let raw = "[APK] com.android.vending: 설치 성공 및 패키지 등록 확인";
        let korean = translate_log(LanguageOption::Korean, raw);
        let english = translate_log(LanguageOption::English, raw);
        assert_ne!(korean, english);
        for lang in LanguageOption::ALL {
            let translated = translate_log(lang, raw);
            assert!(translated.contains("com.android.vending"));
            assert!(!translated.is_empty());
        }
    }
}
