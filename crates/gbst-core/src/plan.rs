use crate::downloader::group_by_package;
use crate::error::{GbstError, Result};
use crate::model::{AndroidMajor, FailurePolicy, InstallPlan, LocalApk, PlanStep};
use std::collections::BTreeMap;
use std::path::PathBuf;

const PKG_VENDING: &str = "com.android.vending";
const PKG_GSF: &str = "com.google.android.gsf";
const PKG_GMS: &str = "com.google.android.gms";
const PKG_EXT_SHARED: &str = "com.google.android.ext.shared";
const PKG_PARTNER_SETUP: &str = "com.google.android.partnersetup";
const PKG_CONFIG_UPDATER: &str = "com.google.android.configupdater";
const PKG_ONE_TIME_INITIALIZER: &str = "com.google.android.onetimeinitializer";
const PKG_PRINT_RECOMMENDATION: &str = "com.google.android.printservice.recommendation";

const GOOGLE_PACKAGES: &[&str] = &[
    PKG_VENDING,
    PKG_GSF,
    PKG_GMS,
    PKG_EXT_SHARED,
    PKG_PARTNER_SETUP,
    PKG_CONFIG_UPDATER,
    PKG_ONE_TIME_INITIALIZER,
    PKG_PRINT_RECOMMENDATION,
];

const RUNTIME_PERMISSIONS: &[&str] = &[
    "android.permission.CAMERA",
    "android.permission.RECORD_AUDIO",
    "android.permission.ACCESS_COARSE_LOCATION",
    "android.permission.ACCESS_FINE_LOCATION",
    "android.permission.NEARBY_WIFI_DEVICES",
    "android.permission.READ_MEDIA_AUDIO",
    "android.permission.READ_MEDIA_IMAGES",
    "android.permission.READ_MEDIA_VIDEO",
    "android.permission.POST_NOTIFICATIONS",
    "android.permission.BLUETOOTH_CONNECT",
    "android.permission.BLUETOOTH_SCAN",
    "android.permission.BLUETOOTH_ADVERTISE",
    "android.permission.ACTIVITY_RECOGNITION",
    "android.permission.BODY_SENSORS",
    "android.permission.ACCESS_MEDIA_LOCATION",
    "android.permission.GET_ACCOUNTS",
];

const APPOPS: &[&str] = &[
    "CAMERA",
    "RECORD_AUDIO",
    "COARSE_LOCATION",
    "FINE_LOCATION",
    "NEARBY_WIFI_DEVICES",
    "READ_MEDIA_AUDIO",
    "READ_MEDIA_IMAGES",
    "READ_MEDIA_VIDEO",
    "READ_MEDIA_VISUAL_USER_SELECTED",
    "ACCESS_MEDIA_LOCATION",
    "POST_NOTIFICATION",
    "ACTIVITY_RECOGNITION",
    "BODY_SENSORS",
    "BLUETOOTH_CONNECT",
    "BLUETOOTH_SCAN",
    "BLUETOOTH_ADVERTISE",
];

const GLOBAL_PIPE_KEYS: &[&str] = &[
    "camera",
    "location",
    "read_media_aural",
    "read_media_images",
    "read_media_video",
    "record_audio",
    "wifi",
];

pub fn build_google_basic_service_plan(
    android_major: AndroidMajor,
    local_apks: Vec<LocalApk>,
) -> Result<InstallPlan> {
    let android_value = android_major.value();
    let by_package = group_by_package(&local_apks);
    validate_required_packages(android_value, &by_package)?;

    let mut steps = Vec::new();

    push_preparation_stage_1(&mut steps);
    push_preparation_stage_2(&mut steps);
    push_google_restore_stage_1(&mut steps, &by_package)?;
    push_google_restore_stage_2(&mut steps, &by_package)?;
    push_google_restore_stage_3(&mut steps, &by_package)?;
    push_google_restore_stage_4(&mut steps);
    push_final_google_services_followup_steps(&mut steps);

    steps.push(PlanStep::HealthCheck {
        label: "Google Services 복구 상태 확인".to_string(),
        packages: GOOGLE_PACKAGES.iter().map(|pkg| (*pkg).to_string()).collect(),
        policy: FailurePolicy::Continue,
    });

    Ok(InstallPlan {
        android_major,
        steps,
    })
}

fn validate_required_packages(android: u32, by_package: &BTreeMap<String, Vec<PathBuf>>) -> Result<()> {
    for package in [
        PKG_PRINT_RECOMMENDATION,
        PKG_ONE_TIME_INITIALIZER,
        PKG_CONFIG_UPDATER,
        PKG_PARTNER_SETUP,
        PKG_GMS,
        PKG_GSF,
        PKG_VENDING,
    ] {
        if !by_package.contains_key(package) {
            return Err(GbstError::MissingPackage {
                android,
                package: package.to_string(),
            });
        }
    }
    Ok(())
}

fn push_shell(steps: &mut Vec<PlanStep>, label: impl Into<String>, command: impl Into<String>) {
    steps.push(PlanStep::Shell {
        label: label.into(),
        command: command.into(),
        policy: FailurePolicy::Continue,
    });
}

fn push_shell_stop(steps: &mut Vec<PlanStep>, label: impl Into<String>, command: impl Into<String>) {
    steps.push(PlanStep::Shell {
        label: label.into(),
        command: command.into(),
        policy: FailurePolicy::Stop,
    });
}

fn push_delay(steps: &mut Vec<PlanStep>, label: impl Into<String>, seconds: u64) {
    steps.push(PlanStep::Delay {
        label: label.into(),
        seconds,
        policy: FailurePolicy::Continue,
    });
}

fn push_reboot_and_wait(steps: &mut Vec<PlanStep>, label: impl Into<String>) {
    steps.push(PlanStep::RebootAndWait {
        label: label.into(),
        policy: FailurePolicy::Stop,
    });
}

fn push_install_all(
    steps: &mut Vec<PlanStep>,
    by_package: &BTreeMap<String, Vec<PathBuf>>,
    package: &str,
    policy: FailurePolicy,
) -> Result<()> {
    let paths = by_package
        .get(package)
        .ok_or_else(|| GbstError::MissingPackage {
            android: 0,
            package: package.to_string(),
        })?;

    if paths.len() == 1 {
        steps.push(PlanStep::InstallApk {
            package: package.to_string(),
            path: paths[0].clone(),
            policy,
        });
    } else {
        steps.push(PlanStep::InstallApkCandidates {
            package: package.to_string(),
            paths: paths.to_vec(),
            policy,
        });
    }

    Ok(())
}

fn push_wakeup_key_sequence(steps: &mut Vec<PlanStep>) {
    push_shell(steps, "화면 깨우기", "input keyevent KEYCODE_WAKEUP");
    for idx in 1..=3 {
        push_shell(steps, format!("잠금 해제 보조 키 입력 {idx}"), "input keyevent 82");
    }
    push_shell(steps, "키 이벤트 65", "input keyevent 65");
    push_shell(steps, "키 이벤트 5", "input keyevent 5");
    push_shell(steps, "키 이벤트 110", "input keyevent 110");
    push_delay(steps, "기기 깨우기 후 2초 대기", 2);
}

fn push_ota_disable_steps(steps: &mut Vec<PlanStep>) {
    push_shell(steps, "OTA 네트워크 권한 비활성화", "settings put system ota_network_permission 0");
    push_shell(steps, "Lenovo OTA 새 버전 알림 초기화", "settings put secure lenovo_ota_new_version_found 0");
    push_shell(steps, "Setup Wizard OTA 자동 업데이트 비활성화", "settings put secure setup_wizard_privacy_auto_update 0");
    push_shell(steps, "Lenovo OTA 프로세스 비활성화", "settings put secure lenovo_ota_process 0");
    push_shell(steps, "Setup Wizard OTA key 비활성화", "settings put global setup_wizard_privacy_ota_key 0");
    push_shell(steps, "Setup Wizard OTA 자동 업데이트 global 비활성화", "settings put global setup_wizard_privacy_auto_update 0");
    push_shell(steps, "OTA 자동 업데이트 비활성화", "settings put global ota_disable_automatic_update 1");
    push_delay(steps, "OTA 알림 비활성화 후 2초 대기", 2);
}

fn push_preparation_stage_1(steps: &mut Vec<PlanStep>) {
    push_shell_stop(steps, "사전 준비: ADB 연결 확인", "getprop ro.build.version.release");
    push_wakeup_key_sequence(steps);
    push_shell(steps, "무음 모드 설정", "settings put global mode_ringer 0");
    push_shell(steps, "화면 꺼짐 시간 10분 설정", "settings put system screen_off_timeout 600000");
    push_shell(steps, "가로 화면 고정", "settings put system user_rotation 1");
    push_shell(steps, "작업 중 화면 밝기 낮춤", "settings put system screen_brightness 5");
    push_delay(steps, "사전 준비 후 2초 대기", 2);
    push_reboot_and_wait(steps, "사전 준비 재부팅");
    push_delay(steps, "재부팅 감지 성공 후 5초 대기", 5);
    push_wakeup_key_sequence(steps);
}

fn push_preparation_stage_2(steps: &mut Vec<PlanStep>) {
    push_shell(steps, "중국 입력기 제거: iFlytek", "pm uninstall --user 0 com.iflytek.inputmethod.custom");
    push_shell(steps, "중국 입력기 제거: Sogou", "pm uninstall --user 0 com.sohu.inputmethod.sogou.oem");
    push_shell(steps, "Lenovo OTA 앱 제거", "pm uninstall --user 0 com.lenovo.ota");

    push_shell(steps, "Lenovo tbengine 복구", "cmd package install-existing --user 0 com.lenovo.tbengine");
    push_shell(steps, "ZUI homesettings 복구", "cmd package install-existing --user 0 com.zui.homesettings");
    push_shell(steps, "Lenovo ue.device 복구", "cmd package install-existing --user 0 com.lenovo.ue.device");

    push_ota_disable_steps(steps);
}

fn push_google_restore_stage_1(
    steps: &mut Vec<PlanStep>,
    by_package: &BTreeMap<String, Vec<PathBuf>>,
) -> Result<()> {
    push_shell(steps, "PartnerSetup 제거", format!("pm uninstall --user 0 {PKG_PARTNER_SETUP}"));
    push_shell(steps, "Google ext.shared 제거", format!("pm uninstall --user 0 {PKG_EXT_SHARED}"));
    push_shell(steps, "ConfigUpdater 제거", format!("pm uninstall --user 0 {PKG_CONFIG_UPDATER}"));
    push_shell(steps, "OneTimeInitializer 제거", format!("pm uninstall --user 0 {PKG_ONE_TIME_INITIALIZER}"));
    push_shell(steps, "PrintService Recommendation 제거", format!("pm uninstall --user 0 {PKG_PRINT_RECOMMENDATION}"));
    push_delay(steps, "Google 서비스 복구 루틴 1 삭제 후 2초 대기", 2);

    push_install_all(steps, by_package, PKG_PARTNER_SETUP, FailurePolicy::Stop)?;
    push_install_all(steps, by_package, PKG_CONFIG_UPDATER, FailurePolicy::Stop)?;
    push_install_all(steps, by_package, PKG_ONE_TIME_INITIALIZER, FailurePolicy::Stop)?;
    push_install_all(steps, by_package, PKG_PRINT_RECOMMENDATION, FailurePolicy::Stop)?;
    push_delay(steps, "Google 서비스 복구 루틴 1 설치 후 2초 대기", 2);

    for package in [
        PKG_PARTNER_SETUP,
        PKG_VENDING,
        PKG_GSF,
        PKG_GMS,
        PKG_EXT_SHARED,
        PKG_CONFIG_UPDATER,
        PKG_ONE_TIME_INITIALIZER,
        PKG_PRINT_RECOMMENDATION,
    ] {
        push_shell(steps, format!("패키지 복구: {package}"), format!("cmd package install-existing --user 0 {package}"));
    }
    push_delay(steps, "Google 서비스 복구 루틴 1 복원 후 2초 대기", 2);

    for package in [
        PKG_PARTNER_SETUP,
        PKG_VENDING,
        PKG_GSF,
        PKG_GMS,
        PKG_EXT_SHARED,
        PKG_CONFIG_UPDATER,
        PKG_ONE_TIME_INITIALIZER,
        PKG_PRINT_RECOMMENDATION,
    ] {
        push_shell(steps, format!("패키지 활성화: {package}"), format!("pm enable {package}"));
    }
    push_delay(steps, "Google 서비스 복구 루틴 1 활성화 후 2초 대기", 2);

    for package in [
        PKG_PARTNER_SETUP,
        PKG_VENDING,
        PKG_GSF,
        PKG_GMS,
        PKG_EXT_SHARED,
        PKG_CONFIG_UPDATER,
        PKG_ONE_TIME_INITIALIZER,
        PKG_PRINT_RECOMMENDATION,
    ] {
        push_shell(steps, format!("패키지 데이터 초기화: {package}"), format!("pm clear {package}"));
    }
    push_ota_disable_steps(steps);
    push_reboot_and_wait(steps, "Google 서비스 복구 루틴 1 완료 후 재부팅");
    push_delay(steps, "재부팅 감지 성공 후 5초 대기", 5);
    push_wakeup_key_sequence(steps);

    Ok(())
}

fn push_google_restore_stage_2(
    steps: &mut Vec<PlanStep>,
    by_package: &BTreeMap<String, Vec<PathBuf>>,
) -> Result<()> {
    push_shell(steps, "Play Store 제거", format!("pm uninstall --user 0 {PKG_VENDING}"));
    push_shell(steps, "Google Play Services 제거", format!("pm uninstall --user 0 {PKG_GMS}"));
    push_delay(steps, "Google 서비스 복구 루틴 2 삭제 후 2초 대기", 2);

    push_install_all(steps, by_package, PKG_GMS, FailurePolicy::Stop)?;
    push_install_all(steps, by_package, PKG_VENDING, FailurePolicy::Stop)?;
    push_delay(steps, "Google 서비스 복구 루틴 2 설치 후 2초 대기", 2);

    for package in [PKG_GMS, PKG_GSF, PKG_VENDING] {
        push_shell(steps, format!("패키지 복구: {package}"), format!("cmd package install-existing --user 0 {package}"));
    }
    push_delay(steps, "Google 서비스 복구 루틴 2 복원 후 2초 대기", 2);

    for package in [PKG_GMS, PKG_GSF, PKG_VENDING] {
        push_shell(steps, format!("패키지 활성화: {package}"), format!("pm enable {package}"));
    }
    push_shell_stop(steps, "Play Store 사용 가능 설정", "settings put global phone_play_store_availability 1");
    push_delay(steps, "Google 서비스 복구 루틴 2 활성화 후 2초 대기", 2);

    push_shell(steps, "Google Play Services 데이터 초기화", format!("pm clear {PKG_GMS}"));
    push_shell(steps, "Play Store 데이터 초기화", format!("pm clear {PKG_VENDING}"));
    push_delay(steps, "Google 서비스 복구 루틴 2 데이터 초기화 후 2초 대기", 2);

    push_ota_disable_steps(steps);
    push_reboot_and_wait(steps, "Google 서비스 복구 루틴 2 완료 후 재부팅");
    push_delay(steps, "재부팅 감지 성공 후 5초 대기", 5);
    push_wakeup_key_sequence(steps);

    Ok(())
}

fn push_google_restore_stage_3(
    steps: &mut Vec<PlanStep>,
    by_package: &BTreeMap<String, Vec<PathBuf>>,
) -> Result<()> {
    push_shell(steps, "Google Services Framework 제거", format!("pm uninstall --user 0 {PKG_GSF}"));
    push_delay(steps, "Google 서비스 복구 루틴 3 삭제 후 2초 대기", 2);

    push_install_all(steps, by_package, PKG_GSF, FailurePolicy::Stop)?;
    push_delay(steps, "Google 서비스 복구 루틴 3 설치 후 2초 대기", 2);

    push_shell(steps, "GSF 패키지 복구", format!("cmd package install-existing --user 0 {PKG_GSF}"));
    push_delay(steps, "Google 서비스 복구 루틴 3 복원 후 2초 대기", 2);

    push_shell(steps, "GSF 패키지 활성화", format!("pm enable {PKG_GSF}"));
    push_ota_disable_steps(steps);
    push_delay(steps, "Google 서비스 복구 루틴 3 활성화 후 2초 대기", 2);
    push_reboot_and_wait(steps, "Google 서비스 복구 루틴 3 완료 후 재부팅");
    push_delay(steps, "재부팅 감지 성공 후 5초 대기", 5);
    push_wakeup_key_sequence(steps);

    Ok(())
}

fn push_google_restore_stage_4(steps: &mut Vec<PlanStep>) {
    push_allow_all_for_google_packages(steps);
    push_ota_disable_steps(steps);
    push_delay(steps, "Google 서비스 복구 루틴 4 권한 부여 후 2초 대기", 2);
    push_reboot_and_wait(steps, "Google 서비스 복구 루틴 4 완료 후 재부팅");
    push_delay(steps, "재부팅 감지 성공 후 5초 대기", 5);
    push_wakeup_key_sequence(steps);
}

fn push_allow_all_for_google_packages(steps: &mut Vec<PlanStep>) {
    for package in GOOGLE_PACKAGES {
        push_shell(steps, format!("패키지 복구: {package}"), format!("cmd package install-existing --user 0 {package}"));
        push_shell(steps, format!("패키지 활성화: {package}"), format!("pm enable {package}"));
        push_shell(steps, format!("AppOps reset(cmd): {package}"), format!("cmd appops reset --user 0 {package}"));
        push_shell(steps, format!("AppOps reset: {package}"), format!("appops reset --user 0 {package}"));

        steps.push(PlanStep::GrantRequestedPermissions {
            label: format!("요청 권한 확인 후 부여: {package}"),
            package: (*package).to_string(),
            permissions: RUNTIME_PERMISSIONS.iter().map(|value| (*value).to_string()).collect(),
            policy: FailurePolicy::Continue,
        });

        steps.push(PlanStep::AppOpsAllowExisting {
            label: format!("기존 AppOps 확인 후 allow 적용: {package}"),
            package: (*package).to_string(),
            ops: APPOPS.iter().map(|value| (*value).to_string()).collect(),
            policy: FailurePolicy::Continue,
        });

        for key in GLOBAL_PIPE_KEYS {
            push_shell(
                steps,
                format!("Global pipe key allow: {package}|{key}"),
                format!("settings put global '{package}|{key}' 1"),
            );
        }
    }
}

fn push_final_google_services_followup_steps(steps: &mut Vec<PlanStep>) {
    push_shell(steps, "Google Play Services 데이터 최종 초기화", format!("pm clear {PKG_GMS}"));
    push_shell(steps, "Play Store 데이터 최종 초기화", format!("pm clear {PKG_VENDING}"));
    push_shell_stop(steps, "Play Store 사용 가능 최종 설정", "settings put global phone_play_store_availability 1");
    push_ota_disable_steps(steps);
    push_shell(steps, "Play Store 초기 설정 키 이벤트 110", "input keyevent 110");
    push_shell(steps, "Play Store 초기 설정 대기 1.2초", "sleep 1.2");
    push_shell(steps, "Play Store 초기 설정 키 이벤트 93", "input keyevent 93");
    push_shell(steps, "Play Store 초기 설정 대기 0.3초", "sleep 0.3");
    push_shell(steps, "Play Store 초기 설정 키 이벤트 117", "input keyevent 117");
    push_shell(steps, "작업 후 화면 밝기 복구", "settings put system screen_brightness 70");
    push_shell(steps, "Android 설정 앱 열기", "am start -n com.android.settings/.Settings");
}
