// SPDX-License-Identifier: GPL-3.0-only
use adb_client::usb::{ADBUSBDevice, find_all_connected_adb_devices};
use adb_client::{ADBDeviceExt, RebootType};
use gbst_core::apk_metadata::{
    apk_version_code_candidates_from_dir, apk_version_code_candidates_from_local_apks,
    read_apk_manifest_info,
};
use gbst_core::error::{GbstError, Result};
use gbst_core::model::{
    AndroidMajor, DashboardInfo, DeviceInfo, FailurePolicy, InstallPlan, LocalApk, PlanStep,
    required_google_packages,
};
use gbst_core::paths;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
#[cfg(windows)]

const ADB_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const ADB_REBOOT_WAIT_TIMEOUT: Duration = Duration::from_secs(120);
const ADB_POLL_INTERVAL: Duration = Duration::from_millis(200);
const ADB_CONNECT_RETRY_ATTEMPTS: usize = 10;
const ADB_CONNECT_RETRY_BACKOFF: Duration = Duration::from_millis(150);
const ADB_TRANSPORT_RETRY_ATTEMPTS: usize = 5;
const ADB_TRANSPORT_RETRY_BACKOFF: Duration = Duration::from_millis(900);
const ADB_PUSH_RETRY_ATTEMPTS: usize = 5;
const ADB_PUSH_RETRY_BACKOFF: Duration = Duration::from_millis(1200);
const ADB_SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
const ADB_SERVER_PROBE_TIMEOUT: Duration = Duration::from_millis(150);
const SPINNER_FRAMES: [&str; 4] = ["│", "╱", "━", "╲"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdbWaitLogMode {
    Detect,
    Reboot,
}

pub const ADB_UNAUTHORIZED_GUIDE: &str = "[ADB] USB 디버깅 인증 필요: 태블릿의 잠금을 해제한 뒤 '이 컴퓨터에서 항상 허용'을 선택하고 '허용'을 누르세요.";

const PREPARATION_LOG: &str =
    "[GBST] PC(노트북)에 파일 및 구성 요소 환경과 구글 서비스 설치, 복구, 업데이트를 준비합니다.";
const DEVICE_CONNECT_PREP_LOG: &str = "[ADB] 기기에 연결할 준비합니다.";
const OTA_DISABLE_LOG: &str = "[ADB] OTA(업데이트) 알림 및 권한을 비활성화 합니다.";
const FINALIZE_LOG: &str = "[GBST] 작업을 마무리 합니다.";
const GOOGLE_RESTORE_DONE_LOG: &str = "[GBST] Google Services 복구 및 업데이트가 완료 되었습니다.";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleServiceAction {
    Normal,
    UpdateRequired,
    RepairRequired,
}

impl GoogleServiceAction {
    pub fn dashboard_status(self) -> &'static str {
        match self {
            Self::Normal => "정상",
            Self::UpdateRequired => "업데이트 필요",
            Self::RepairRequired => "복구 필요",
        }
    }
}

#[derive(Debug, Clone)]
struct GooglePackageState {
    installed_for_user: bool,
    disabled: bool,
    version_code: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdbUsbState {
    NoDevice,
    MultipleDevices,
    Ready,
    Recovery,
    Unauthorized,
    ServerBlocking,
    Error(String),
}

/// The only GBST-owned staging directory on the Android device.
pub const GBST_DEVICE_TEMP_DIR: &str = "/data/local/tmp/dwas_kr/GBST";
const GBST_CLEANUP_MARKER: &str = "__GBST_DEVICE_TEMP_CLEANED__";

pub struct DirectAdb {
    device: Option<ADBUSBDevice>,
    serial: Option<String>,
    connected_once: bool,
    cached_bootmode: Option<&'static str>,
    pinned_serial: Option<String>,
    staged_apks: Vec<String>,
}

impl DirectAdb {
    pub fn new() -> Self {
        Self {
            device: None,
            serial: None,
            connected_once: false,
            cached_bootmode: None,
            pinned_serial: None,
            staged_apks: Vec::new(),
        }
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    pub fn startup_authorization_probe_three_times() {
        prepare_adb_usb_environment();

        for _ in 0..3 {
            if adb_usb_candidate_exists().unwrap_or(false) {
                let mut adb = DirectAdb::new();

                match adb.check_usb_state() {
                    Ok(AdbUsbState::Ready) | Ok(AdbUsbState::Recovery) => break,
                    Ok(AdbUsbState::ServerBlocking) => {
                        adb.reclaim_direct_usb_interface();
                    }
                    Ok(AdbUsbState::NoDevice)
                    | Ok(AdbUsbState::MultipleDevices)
                    | Ok(AdbUsbState::Unauthorized)
                    | Ok(AdbUsbState::Error(_))
                    | Err(_) => {}
                }
            }

            thread::sleep(ADB_POLL_INTERVAL);
        }
    }

    pub fn check_usb_state(&mut self) -> Result<AdbUsbState> {
        let devices = find_all_connected_adb_devices()
            .map_err(|err| GbstError::Adb(format!("ADB USB 기기 검색 실패: {err}")))?;

        if devices.is_empty() {
            self.drop_device();
            return Ok(AdbUsbState::NoDevice);
        }

        if devices.len() > 1 {
            self.drop_device();
            return Ok(AdbUsbState::MultipleDevices);
        }

        match self.connect_device() {
            Ok(_) => {
                let bootmode = self.cached_bootmode.unwrap_or("device");
                if bootmode == "recovery" {
                    Ok(AdbUsbState::Recovery)
                } else {
                    Ok(AdbUsbState::Ready)
                }
            }
            Err(err) => {
                self.drop_device();
                let detail = err.to_string();
                record_adb_diagnostic("USB 상태 조회", &detail);
                let lowered = detail.to_ascii_lowercase();
                if (lowered.contains("busy")
                    || lowered.contains("claim")
                    || lowered.contains("access denied")
                    || lowered.contains("in use"))
                    && adb_server_running()
                {
                    Ok(AdbUsbState::ServerBlocking)
                } else if lowered.contains("unauthorized")
                    || lowered.contains("authentication")
                    || lowered.contains("rsa")
                {
                    Ok(AdbUsbState::Unauthorized)
                } else {
                    Ok(AdbUsbState::Error(format!(
                        "USB 연결 실패: {detail}. 케이블, 디버깅 인증, USB 드라이버 및 다른 ADB 프로그램을 확인하세요."
                    )))
                }
            }
        }
    }

    pub fn wait_ready<F>(&mut self, on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        prepare_adb_usb_environment();
        // Show the authorization prompt only when USB state is Unauthorized.
        self.wait_ready_until(ADB_WAIT_TIMEOUT, on_log)
    }

    pub fn wait_ready_until<F>(&mut self, timeout: Duration, on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        self.wait_ready_until_with_mode(timeout, on_log, AdbWaitLogMode::Detect, None)
    }

    fn wait_ready_until_for_reboot<F>(
        &mut self,
        timeout: Duration,
        previous_boot_id: Option<&str>,
        on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        self.wait_ready_until_with_mode(timeout, on_log, AdbWaitLogMode::Reboot, previous_boot_id)
    }

    fn wait_ready_until_with_mode<F>(
        &mut self,
        timeout: Duration,
        mut on_log: F,
        mode: AdbWaitLogMode,
        previous_boot_id: Option<&str>,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let deadline = Instant::now() + timeout;
        let mut spinner_index = 0usize;
        let mut current_state = "ADB USB 기기 감지 대기 중입니다.".to_string();
        let mut authorization_guide_emitted = false;

        let spinner_key = match mode {
            AdbWaitLogMode::Detect => "adb_detect",
            AdbWaitLogMode::Reboot => "adb_reboot",
        };

        loop {
            let frame = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
            spinner_index = spinner_index.wrapping_add(1);

            match mode {
                AdbWaitLogMode::Detect => {
                    on_log(spinner_log(
                        spinner_key,
                        &format!("[ADB] 기기 감지 중... {frame}"),
                    ));
                }
                AdbWaitLogMode::Reboot => {
                    on_log(spinner_log(
                        spinner_key,
                        &format!("[ADB] 기기 재부팅 중... {frame}"),
                    ));
                }
            }

            if adb_usb_candidate_exists().unwrap_or(false) {
                current_state = match self.check_usb_state() {
                    Ok(AdbUsbState::Ready) | Ok(AdbUsbState::Recovery) => {
                        if mode == AdbWaitLogMode::Reboot {
                            let (current_boot_id, boot_completed) = if previous_boot_id.is_some() {
                                let probe = self
                                    .shell(
                                        "cat /proc/sys/kernel/random/boot_id; getprop sys.boot_completed",
                                    )
                                    .unwrap_or_default();
                                parse_reboot_probe(&probe)
                            } else {
                                (
                                    None,
                                    self.shell("getprop sys.boot_completed").unwrap_or_default(),
                                )
                            };

                            if !reboot_session_is_ready(
                                previous_boot_id,
                                current_boot_id.as_deref(),
                                &boot_completed,
                            ) {
                                self.drop_device();
                                current_state =
                                    "재부팅 후 새 ADB 세션 및 Android 부팅 완료를 기다리는 중입니다."
                                        .to_string();
                                thread::sleep(ADB_POLL_INTERVAL);
                                continue;
                            }
                        }

                        self.connected_once = true;
                        match mode {
                            AdbWaitLogMode::Detect => {
                                on_log(spinner_log(spinner_key, "[ADB] 기기 감지 완료"));
                            }
                            AdbWaitLogMode::Reboot => {
                                on_log(spinner_log(spinner_key, "[ADB] 기기 재부팅 완료"));
                            }
                        }
                        return Ok(());
                    }
                    Ok(AdbUsbState::NoDevice) => "ADB USB 기기 감지 대기 중입니다.".to_string(),
                    Ok(AdbUsbState::MultipleDevices) => {
                        return Err(GbstError::Adb(
                            "ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요."
                                .to_string(),
                        ));
                    }
                    Ok(AdbUsbState::Unauthorized) => {
                        if !authorization_guide_emitted {
                            on_log(ADB_UNAUTHORIZED_GUIDE.to_string());
                            authorization_guide_emitted = true;
                        }
                        "ADB unauthorized 상태입니다.".to_string()
                    }
                    Ok(AdbUsbState::ServerBlocking) => {
                        self.reclaim_direct_usb_interface();
                        "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string()
                    }
                    Ok(AdbUsbState::Error(err)) => err,
                    Err(err) => err.to_string(),
                };
            }

            if Instant::now() >= deadline {
                if current_state.contains("감지 대기") {
                    return Err(GbstError::Adb(
                        "NO_USB_ADB_DEVICE: 태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요."
                            .to_string(),
                    ));
                }

                return Err(GbstError::Adb(format!(
                    "ADB_UNAUTHORIZED_RETRY: ADB 기기 감지 시간 초과. 마지막 상태: {current_state}"
                )));
            }

            thread::sleep(ADB_POLL_INTERVAL);
        }
    }

    pub fn read_device_info(&mut self) -> Result<DeviceInfo> {
        let manufacturer = self
            .shell("getprop ro.product.manufacturer")?
            .trim()
            .to_string();
        let model = self.read_model_identity();
        let android_version = self
            .shell("getprop ro.build.version.release")?
            .trim()
            .to_string();

        if !manufacturer.eq_ignore_ascii_case("Lenovo") {
            return Err(GbstError::Device(format!(
                "Lenovo 기기가 아닙니다. 감지된 제조사: {manufacturer}"
            )));
        }

        let android_major = AndroidMajor::parse(&android_version).ok_or_else(|| {
            GbstError::Device(format!(
                "Android 13~18만 지원합니다. 감지된 버전: {android_version}"
            ))
        })?;

        Ok(DeviceInfo {
            manufacturer,
            model,
            android_version,
            android_major,
        })
    }

    pub fn read_dashboard_info(&mut self) -> Result<DashboardInfo> {
        self.ensure_dashboard_ready()?;

        let mut info = DashboardInfo::unknown();

        let manufacturer_raw = self
            .shell("getprop ro.product.manufacturer")
            .unwrap_or_default()
            .trim()
            .to_string();

        let is_lenovo = manufacturer_raw.eq_ignore_ascii_case("Lenovo");
        info.is_lenovo = is_lenovo;
        info.manufacturer = if manufacturer_raw.is_empty() {
            "알 수 없음".to_string()
        } else if is_lenovo {
            "Lenovo".to_string()
        } else {
            "Lenovo 기기가 아닙니다.".to_string()
        };

        let display_name = self.read_first_non_empty_property(&[
            "ro.product.display",
            "ro.product.vendor.name",
            "ro.product.name",
        ]);
        let model_identity = self.read_model_identity();
        let normalized_display = normalize_model_property(&display_name);
        let normalized_identity = normalize_model_property(&model_identity);

        info.model_name = if !display_name.is_empty()
            && !model_identity.is_empty()
            && !normalized_display.contains(&normalized_identity)
        {
            format!("{display_name} ({model_identity})")
        } else if !display_name.is_empty() {
            display_name
        } else if !model_identity.is_empty() {
            model_identity
        } else {
            "알 수 없음".to_string()
        };

        let android_version = self
            .shell("getprop ro.build.version.release")
            .unwrap_or_default()
            .trim()
            .to_string();
        info.android_major = AndroidMajor::parse(&android_version);
        info.android_version = if android_version.is_empty() {
            "알 수 없음".to_string()
        } else {
            android_version
        };

        let region_raw = self
            .shell("getprop ro.config.zui.region")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        info.rom_type = if manufacturer_raw.is_empty() {
            "알 수 없음".to_string()
        } else if !is_lenovo {
            "Lenovo 기기가 아닙니다.".to_string()
        } else if region_raw == "prc" {
            "PRC(중국 내수롬)".to_string()
        } else if region_raw == "row" {
            "ROW(글로벌롬)".to_string()
        } else if region_raw.is_empty() {
            "알 수 없음".to_string()
        } else {
            "Lenovo 기기가 아닙니다.".to_string()
        };

        info.google_service_status = if info.android_major.is_some() && is_lenovo {
            let android_major = info.android_major.expect("checked above");
            self.assess_google_services_from_apk_dir(android_major)
                .dashboard_status()
                .to_string()
        } else {
            "알 수 없음".to_string()
        };

        Ok(info)
    }

    pub fn shell(&mut self, command: &str) -> Result<String> {
        let output = self.shell_inner(command)?;
        let trimmed = output.trim();
        let package_command = command.starts_with("pm ") || command.starts_with("cmd package ");
        if package_command
            && !command.starts_with("pm install ")
            && (trimmed.starts_with("Failure [")
                || trimmed.starts_with("Error:")
                || trimmed.starts_with("Exception occurred"))
        {
            return Err(GbstError::Adb(format!(
                "Android 패키지 명령 실패: {command}: {trimmed}"
            )));
        }
        Ok(output)
    }

    fn read_first_non_empty_property(&mut self, properties: &[&str]) -> String {
        for property in properties {
            let value = self
                .shell(&format!("getprop {property}"))
                .unwrap_or_default()
                .trim()
                .to_string();
            if !value.is_empty() {
                return value;
            }
        }
        String::new()
    }

    fn read_model_identity(&mut self) -> String {
        let properties = [
            "ro.product.model",
            "ro.product.device",
            "ro.product.product.model",
            "ro.product.product.device",
            "ro.product.vendor.model",
            "ro.product.vendor.device",
        ];
        let mut values = Vec::new();

        for property in properties {
            let value = self
                .shell(&format!("getprop {property}"))
                .unwrap_or_default()
                .trim()
                .to_string();
            if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        }

        if let Some(code) = detect_supported_lenovo_model_code(&values) {
            return code.to_string();
        }

        values.into_iter().next().unwrap_or_default()
    }

    pub fn read_screen_off_timeout(&mut self) -> Result<String> {
        let mut last_error = None;
        for command in [
            "settings get --user 0 system screen_off_timeout",
            "settings get system screen_off_timeout",
        ] {
            match self.shell(command) {
                Ok(output) => {
                    let value = output.trim();
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            GbstError::Adb("기기의 화면 꺼짐 값을 읽지 못했습니다.".to_string())
        }))
    }

    pub fn restore_screen_off_timeout(&mut self, timeout: &str) -> Result<()> {
        let value = timeout.trim();
        if value.is_empty() || value.eq_ignore_ascii_case("null") {
            let _ = self.shell("settings delete --user 0 system screen_off_timeout")?;
            return Ok(());
        }

        if !value.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(GbstError::Adb(format!(
                "screen_off_timeout 복원 값이 올바르지 않습니다: {value}"
            )));
        }

        let _ = self.shell(&format!(
            "settings put --user 0 system screen_off_timeout {value}"
        ))?;
        Ok(())
    }

    pub fn screen_off_timeout_display_value(raw: &str) -> String {
        screen_off_timeout_display_value(raw)
    }

    pub fn read_screen_brightness(&mut self) -> Result<u32> {
        for command in [
            "settings get system screen_brightness",
            "settings get --user 0 system screen_brightness",
        ] {
            if let Ok(output) = self.shell(command) {
                if let Some(value) = parse_screen_brightness_value(&output) {
                    return Ok(value);
                }
            }
        }

        for command in [
            "cmd display get-brightness",
            "settings get --user 0 system screen_brightness_float",
            "settings get system screen_brightness_float",
        ] {
            if let Ok(output) = self.shell(command) {
                if let Some(value) = parse_screen_brightness_float(&output) {
                    return Ok(value);
                }
            }
        }

        Err(GbstError::Adb(
            "기기의 화면 밝기 값을 올바르게 읽지 못했습니다.".to_string(),
        ))
    }

    pub fn restore_screen_brightness(&mut self, brightness: u32) -> Result<()> {
        if brightness > 4095 {
            return Err(GbstError::Adb(format!(
                "screen_brightness 복원 값이 올바르지 않습니다: {brightness}"
            )));
        }

        let _ = self.shell(&format!(
            "settings put system screen_brightness {brightness}"
        ))?;
        Ok(())
    }

    pub fn assess_google_services_from_downloaded_apks(
        &mut self,
        android_major: AndroidMajor,
        apks: &[LocalApk],
    ) -> GoogleServiceAction {
        let local_versions = apk_version_code_candidates_from_local_apks(apks);
        self.assess_google_services_with_versions(android_major, &local_versions)
    }

    pub fn assess_google_services_from_apk_dir(
        &mut self,
        android_major: AndroidMajor,
    ) -> GoogleServiceAction {
        let apk_dir = paths::apk_download_dir(android_major.value());
        let local_versions = apk_version_code_candidates_from_dir(&apk_dir);
        self.assess_google_services_with_versions(android_major, &local_versions)
    }

    fn assess_google_services_with_versions(
        &mut self,
        android_major: AndroidMajor,
        local_versions: &std::collections::BTreeMap<String, Vec<u64>>,
    ) -> GoogleServiceAction {
        let mut update_required = false;

        for &package in required_google_packages(android_major) {
            let state = self.read_google_package_state(package);

            if !state.installed_for_user || state.disabled {
                return GoogleServiceAction::RepairRequired;
            }

            if let (Some(installed), Some(downloaded_candidates)) =
                (state.version_code, local_versions.get(package))
            {
                if !downloaded_candidates.is_empty()
                    && downloaded_candidates
                        .iter()
                        .all(|downloaded| *downloaded > installed)
                {
                    update_required = true;
                }
            }
        }

        if update_required {
            GoogleServiceAction::UpdateRequired
        } else {
            GoogleServiceAction::Normal
        }
    }

    /// Post-install safety net. The existing four-stage Google restore plan is
    /// unchanged: only re-enable a package that is still disabled for user 0
    /// after the last reboot, then verify the actual PackageManager state.
    /// Missing packages are reported, never silently counted as healthy.
    pub fn restore_disabled_google_packages_after_install<F>(
        &mut self,
        android_major: AndroidMajor,
        mut on_log: F,
    ) -> bool
    where
        F: FnMut(String),
    {
        let mut changed = false;
        for &package in required_google_packages(android_major) {
            let mut state = self.read_google_package_state(package);
            if !state.installed_for_user {
                on_log(format!(
                    "[ADB] {package}: 사용자 0에 등록되지 않아 최종 검증을 통과할 수 없습니다."
                ));
                continue;
            }
            let Some(command) = post_install_reenable_command(package, &state) else {
                continue;
            };

            on_log(format!(
                "[ADB] {package}: 사용자 0에서 사용 중지 상태입니다. 활성화를 시도합니다."
            ));
            let mut restored = false;
            for attempt in 1..=2 {
                if attempt > 1 {
                    on_log(format!(
                        "[ADB] {package}: 사용자 0 활성화 재시도 {attempt}/2"
                    ));
                }
                if let Err(err) = self.shell(&command) {
                    record_adb_diagnostic(
                        "Google 사용자 0 활성화 명령 오류",
                        &format!("{package}: {err}"),
                    );
                    on_log(format!(
                        "[ADB] {package}: 사용자 0 활성화 명령 오류. 진단 로그를 확인하세요."
                    ));
                }
                // ADB transport success does not prove a package was enabled.
                thread::sleep(Duration::from_millis(500));
                state = self.read_google_package_state(package);
                if state.installed_for_user && !state.disabled {
                    on_log(format!("[ADB] {package}: 사용자 0 활성화 및 검증 완료"));
                    changed = true;
                    restored = true;
                    break;
                }
            }
            if !restored {
                on_log(format!(
                    "[ADB] {package}: 사용자 0 활성화 후에도 사용 중지 상태입니다."
                ));
            }
        }
        changed
    }

    fn read_google_package_state(&mut self, package: &str) -> GooglePackageState {
        let dumpsys = self
            .shell(&format!("dumpsys package {package}"))
            .unwrap_or_default();

        let package_missing = dumpsys.trim().is_empty()
            || dumpsys.contains("Unable to find package")
            || dumpsys.contains("not found");

        if package_missing {
            return GooglePackageState {
                installed_for_user: false,
                disabled: false,
                version_code: None,
            };
        }

        let user_zero_line = dumpsys
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("User 0:"));

        let installed_for_user = self.package_list_contains(package, false)
            || user_zero_line
                .map(|line| !line.contains("installed=false"))
                .unwrap_or_else(|| self.pm_path_exists(package));

        let disabled_by_user_state = user_zero_line
            .and_then(parse_enabled_state_from_user_line)
            .map(|state| matches!(state, 2 | 3 | 4))
            .unwrap_or(false);

        let disabled = disabled_by_user_state || self.package_list_contains(package, true);
        let version_code = self
            .read_active_package_version_code(package)
            .or_else(|| parse_installed_version_code(&dumpsys));

        GooglePackageState {
            installed_for_user,
            disabled,
            version_code,
        }
    }

    fn read_active_package_version_code(&mut self, package: &str) -> Option<u64> {
        let commands = [
            format!("pm list packages --show-versioncode --user 0 {package}"),
            format!("pm list packages --show-versioncode {package}"),
        ];

        for command in commands {
            let output = self.shell(&command).unwrap_or_default();
            if let Some(version_code) = parse_pm_list_version_code(&output, package) {
                return Some(version_code);
            }
        }

        None
    }

    fn pm_path_exists(&mut self, package: &str) -> bool {
        self.shell(&format!("pm path {package}"))
            .map(|output| {
                output
                    .lines()
                    .any(|line| line.trim().starts_with("package:"))
            })
            .unwrap_or(false)
    }

    fn package_list_contains(&mut self, package: &str, disabled_only: bool) -> bool {
        let command = if disabled_only {
            format!("pm list packages -d --user 0 {package}")
        } else {
            format!("pm list packages --user 0 {package}")
        };

        let output = self
            .shell(&command)
            .or_else(|_| {
                let fallback = if disabled_only {
                    format!("pm list packages -d {package}")
                } else {
                    format!("pm list packages {package}")
                };
                self.shell(&fallback)
            })
            .unwrap_or_default();

        output
            .lines()
            .any(|line| line.trim() == format!("package:{package}"))
    }

    fn shell_inner(&mut self, command: &str) -> Result<String> {
        let started = Instant::now();
        let mut last_error: Option<String> = None;

        for attempt in 0..ADB_TRANSPORT_RETRY_ATTEMPTS {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();

            let result = match self.connect_device() {
                Ok(device) => device
                    .shell_command(
                        &command,
                        Some(&mut stdout as &mut dyn Write),
                        Some(&mut stderr as &mut dyn Write),
                    )
                    .map_err(|err| err.to_string()),
                Err(err) => Err(err.to_string()),
            };

            match result {
                Ok(_) => {
                    let summary = command
                        .split_whitespace()
                        .take(4)
                        .collect::<Vec<_>>()
                        .join(" ");
                    record_adb_diagnostic(
                        "ADB Shell OK",
                        &format!("{summary} ({}ms)", started.elapsed().as_millis()),
                    );
                    return Ok(String::from_utf8_lossy(&stdout).trim().to_string());
                }
                Err(err) => {
                    self.drop_device();
                    let stderr_text = String::from_utf8_lossy(&stderr).trim().to_string();
                    last_error = Some(if stderr_text.is_empty() {
                        format!("ADB shell 실패 `{command}`: {err}")
                    } else {
                        format!("ADB shell 실패 `{command}`: {err} / stderr={stderr_text}")
                    });

                    if attempt + 1 < ADB_TRANSPORT_RETRY_ATTEMPTS {
                        thread::sleep(ADB_TRANSPORT_RETRY_BACKOFF);
                    }
                }
            }
        }

        let detail = last_error
            .unwrap_or_else(|| format!("ADB shell 실패 `{command}`: 알 수 없는 전송 오류"));
        record_adb_diagnostic("ADB shell 전송", &detail);
        Err(GbstError::Adb(detail))
    }

    pub fn install_apk_with_pm(&mut self, package: &str, apk_path: &Path) -> Result<String> {
        self.install_apk_with_pm_logged(package, apk_path, |_| {})
    }

    fn install_apk_with_pm_logged<F>(
        &mut self,
        package: &str,
        apk_path: &Path,
        mut on_log: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        let file_name = apk_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                GbstError::Adb(format!(
                    "APK 파일 이름을 읽을 수 없습니다: {}",
                    apk_path.display()
                ))
            })?;

        // Stage APKs only in GBST's private, temporary ADB shell working tree.
        // No APKs are placed in the user's Download directory.
        let remote_dir = GBST_DEVICE_TEMP_DIR;
        let remote_path = format!("{remote_dir}/{file_name}");

        // A previous interrupted run may have removed a Google package for user 0.
        // Restore the already known package before attempting a version update.
        if !self.package_list_contains_user_zero(package)
            && self.read_existing_package_version_code(package).is_some()
        {
            on_log(format!(
                "[ADB] {package}: 사용자 0 등록 상태 복구를 먼저 시도합니다."
            ));
            if self.try_install_existing_for_user(package) {
                on_log(format!("[ADB] {package}: 사용자 0 패키지 등록 복원 완료"));
            }
        }

        if self.try_reuse_existing_package_for_user(package, apk_path)? {
            on_log(format!(
                "[APK] {package}: 설치 버전이 같거나 높아 기존 패키지를 재사용합니다."
            ));
            return Ok("Existing package reused for user 0".to_string());
        }

        on_log(format!(
            "[APK] {package}: {remote_path} 경로로 APK 전송을 시작합니다."
        ));
        self.prepare_remote_apk_dir(remote_dir)?;
        if !self.staged_apks.contains(&remote_path) {
            self.staged_apks.push(remote_path.clone());
        }
        self.push_file_with_retry(apk_path, &remote_path, package, &mut on_log)?;

        self.drop_device();
        let started = Instant::now();
        on_log(format!(
            "[APK] {package}: APK 전송 검증 완료, Android 패키지 설치 명령을 실행합니다."
        ));
        let has_timeout = self
            .shell("command -v timeout")
            .is_ok_and(|output| output.lines().any(|line| line.trim().ends_with("/timeout")));
        let install_command = if has_timeout {
            format!("timeout -k 5 90 pm install -r -d -g --user 0 {remote_path}")
        } else {
            format!("pm install -r -d -g --user 0 {remote_path}")
        };
        let install_result = self.shell(&install_command);
        on_log(format!(
            "[APK] {package}: 설치 명령 반환, 경과 {}초",
            started.elapsed().as_secs()
        ));
        let _ = self.shell(&format!("rm -f {remote_path}"));

        let output = install_result?;
        if pm_install_explicitly_failed(&output) {
            if is_signature_update_incompatible(&output)
                && self.try_install_existing_for_user(package)
            {
                return Err(GbstError::Adb(format!(
                    "APK 서명 불일치로 설치에 실패했습니다. 기존 패키지는 복구되었습니다: {package}: {}",
                    compact_pm_output(&output)
                )));
            }

            let detail = format!("APK 설치 실패: {package}: {}", compact_pm_output(&output));
            record_adb_diagnostic("APK 설치", &detail);
            return Err(GbstError::Adb(detail));
        }

        if !pm_install_explicitly_succeeded(&output) {
            return Err(GbstError::Adb(format!(
                "APK 설치 완료(Success) 응답을 받지 못했습니다: {package}: {}",
                compact_pm_output(&output)
            )));
        }

        if !self.wait_for_user_package(package, 6, Duration::from_millis(500)) {
            return Err(GbstError::Adb(format!(
                "APK 설치 성공 응답을 받았지만 사용자 0의 패키지 등록을 확인하지 못했습니다: {package}"
            )));
        }

        on_log(format!("[APK] {package}: 설치 성공 및 패키지 등록 확인"));
        Ok(output)
    }

    fn try_reuse_existing_package_for_user(
        &mut self,
        package: &str,
        apk_path: &Path,
    ) -> Result<bool> {
        let local_version = read_apk_manifest_info(apk_path)
            .ok()
            .filter(|info| info.package.trim() == package)
            .and_then(|info| info.version_code);

        let existing_version = self.read_existing_package_version_code(package);
        if !should_reuse_existing_package(existing_version, local_version) {
            return Ok(false);
        }

        Ok(self.try_install_existing_for_user(package))
    }

    fn read_existing_package_version_code(&mut self, package: &str) -> Option<u64> {
        let dumpsys = self
            .shell(&format!("dumpsys package {package}"))
            .unwrap_or_default();

        if dumpsys.trim().is_empty()
            || dumpsys.contains("Unable to find package")
            || dumpsys.contains("not found")
        {
            return None;
        }

        parse_installed_version_code(&dumpsys)
    }

    fn try_install_existing_for_user(&mut self, package: &str) -> bool {
        if self
            .shell(&format!("cmd package install-existing --user 0 {package}"))
            .is_err()
        {
            return false;
        }

        self.package_list_contains_user_zero(package)
    }

    fn package_list_contains_user_zero(&mut self, package: &str) -> bool {
        self.shell(&format!("pm list packages --user 0 {package}"))
            .map(|output| {
                output
                    .lines()
                    .any(|line| line.trim() == format!("package:{package}"))
            })
            .unwrap_or(false)
    }

    /// Remove only GBST's Android staging directory. The shared dwas_kr
    /// parent is removed only when empty, preserving other dwas_kr data.
    /// A positive shell marker and filesystem check are mandatory because
    /// the direct ADB shell protocol may not expose the command exit code.
    pub fn cleanup_gbst_temp_files(&mut self) -> Result<()> {
        let output = self.shell(gbst_device_cleanup_command())?;
        if !output
            .lines()
            .any(|line| line.trim() == GBST_CLEANUP_MARKER)
        {
            return Err(GbstError::Adb(format!(
                "GBST 임시 폴더 삭제를 확인하지 못했습니다: {GBST_DEVICE_TEMP_DIR}: {output}"
            )));
        }
        self.staged_apks.clear();
        Ok(())
    }

    /// Retry cleanup on a fresh USB connection, including after a cancelled
    /// or failed installation. Avoid claiming a clean device on USB errors.
    pub fn cleanup_gbst_temp_files_with_retry<F>(&mut self, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        let mut last_error = None;
        for attempt in 1..=3 {
            self.drop_device();
            match self.cleanup_gbst_temp_files() {
                Ok(()) => return Ok(()),
                Err(err) => {
                    on_log(format!(
                        "[ADB] 임시 폴더 삭제 재시도 {attempt}/3 실패: {err}"
                    ));
                    last_error = Some(err);
                    if attempt < 3 {
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            GbstError::Adb("기기 임시 폴더 삭제 결과를 확인하지 못했습니다.".to_string())
        }))
    }

    /// Remove the four explicitly named tablet cache/data folders requested by the user.
    /// Verify the Lenovo manufacturer and each deletion result before proceeding.
    /// No user-supplied shell paths or wildcards are accepted.
    pub fn cleanup_dwas_kr_folders(&mut self) -> Result<()> {
        let device = self.read_device_info()?;
        if !device.manufacturer.eq_ignore_ascii_case("Lenovo") {
            return Err(GbstError::Adb(
                "기기 임시 폴더 정리는 Lenovo 기기에만 허용됩니다.".to_string(),
            ));
        }
        let output = self.shell(dwas_kr_full_cleanup_command())?;
        if !output
            .lines()
            .any(|line| line.trim() == DWAS_KR_FULL_CLEANUP_MARKER)
        {
            return Err(GbstError::Adb(format!(
                "네 GBST 데이터/캐시 폴더 정리를 확인하지 못했습니다: {output}"
            )));
        }
        self.staged_apks.clear();
        Ok(())
    }

    pub fn cleanup_dwas_kr_folders_with_retry<F>(&mut self, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        let mut last_error = None;
        for attempt in 1..=3 {
            self.drop_device();
            match self.cleanup_dwas_kr_folders() {
                Ok(()) => return Ok(()),
                Err(error) => {
                    on_log(format!(
                        "[ADB] GBST 데이터/캐시 폴더 정리 {attempt}/3 실패: {error}"
                    ));
                    last_error = Some(error);
                    if attempt < 3 {
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }
        Err(last_error
            .unwrap_or_else(|| GbstError::Adb("GBST 데이터/캐시 폴더 정리 확인 실패".to_string())))
    }

    fn prepare_remote_apk_dir(&mut self, remote_dir: &str) -> Result<()> {
        self.reclaim_direct_usb_interface();
        self.shell(&format!("mkdir -p {remote_dir}"))?;
        let probe = self.shell(&format!("test -d {remote_dir} && echo __GBST_DIR_READY__"))?;
        if !probe
            .lines()
            .any(|line| line.trim() == "__GBST_DIR_READY__")
        {
            return Err(GbstError::Adb(format!(
                "기기 임시 폴더 준비 상태를 확인하지 못했습니다: {remote_dir}"
            )));
        }
        self.drop_device();
        Ok(())
    }

    fn reclaim_direct_usb_interface(&mut self) {
        // Unlike the previous implementation, never kill other ADB owners.
        self.drop_device();
    }

    fn push_file_with_retry<F>(
        &mut self,
        apk_path: &Path,
        remote_path: &str,
        package: &str,
        on_log: &mut F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let local_size = std::fs::metadata(apk_path)?.len();
        let local_sha256 = sha256_of_file(apk_path)?;
        let mut last_error: Option<String> = None;

        for attempt in 0..ADB_PUSH_RETRY_ATTEMPTS {
            self.reclaim_direct_usb_interface();
            let file = File::open(apk_path)?;
            let mut last_reported = 0u8;
            let mut progress_callback = |_transferred: u64, _total: u64, percent: u8| {
                if percent >= last_reported.saturating_add(10) || percent == 100 {
                    last_reported = percent;
                    on_log(format!("[APK] {package}: 기기 전송 진행률 {percent}%"));
                }
            };
            let mut reader = ProgressReader::new(file, local_size, &mut progress_callback);

            let result = match self.connect_device() {
                Ok(device) => device
                    .push(&mut reader as &mut dyn Read, &remote_path)
                    .map_err(|err| err.to_string()),
                Err(err) => Err(err.to_string()),
            };

            // Release the progress callback's mutable borrow before emitting
            // status messages about transfer verification or retries.
            drop(reader);
            drop(progress_callback);

            match result {
                Ok(()) => {
                    self.drop_device();
                    match self.remote_file_size(remote_path) {
                        Ok(remote_size) if remote_size == local_size => {
                            match self.remote_file_sha256(remote_path) {
                                Ok(remote_sha256)
                                    if remote_sha256.eq_ignore_ascii_case(&local_sha256) =>
                                {
                                    on_log(format!(
                                        "[APK] {package}: 기기 전송 파일 크기 및 SHA-256 검증 완료"
                                    ));
                                    return Ok(());
                                }
                                Ok(remote_sha256) => {
                                    last_error = Some(format!(
                                        "APK 전송 SHA-256 불일치: PC={local_sha256}, 기기={remote_sha256}"
                                    ));
                                }
                                Err(err) => last_error = Some(err.to_string()),
                            }
                        }
                        Ok(remote_size) => {
                            last_error = Some(format!(
                                "전송 크기 불일치: PC={local_size} bytes / 기기={remote_size} bytes"
                            ));
                        }
                        Err(err) => last_error = Some(err.to_string()),
                    }
                }
                Err(err) => {
                    self.drop_device();
                    last_error = Some(err);
                }
            }

            let _ = self.shell(&format!("rm -f {remote_path}"));
            self.drop_device();
            if attempt + 1 < ADB_PUSH_RETRY_ATTEMPTS {
                on_log(format!(
                    "    [경고] APK 전송 실패, Direct ADB 연결을 새로 만든 뒤 재시도합니다. ({}/{})",
                    attempt + 1,
                    ADB_PUSH_RETRY_ATTEMPTS
                ));
                thread::sleep(ADB_PUSH_RETRY_BACKOFF);
            }
        }

        Err(GbstError::Adb(format!(
            "APK push 실패: {} -> {remote_path}: {}",
            apk_path.display(),
            last_error.unwrap_or_else(|| "알 수 없는 전송 오류".to_string())
        )))
    }

    fn remote_file_size(&mut self, remote_path: &str) -> Result<u64> {
        let output = self.shell(&format!("wc -c < {remote_path}"))?;
        output
            .split_whitespace()
            .find_map(|token| token.parse::<u64>().ok())
            .ok_or_else(|| {
                GbstError::Adb(format!(
                    "기기에 전송된 APK 파일 크기를 확인하지 못했습니다: {remote_path}"
                ))
            })
    }

    fn remote_file_sha256(&mut self, remote_path: &str) -> Result<String> {
        let output = self.shell(&format!("sha256sum {remote_path}"))?;
        output
            .split_whitespace()
            .next()
            .filter(|digest| {
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| {
                GbstError::Adb(format!(
                    "기기에 전송된 APK SHA-256을 확인하지 못했습니다: {remote_path}"
                ))
            })
    }

    pub fn install_apk_candidates_with_pm(
        &mut self,
        package: &str,
        apk_paths: &[PathBuf],
    ) -> Result<String> {
        self.install_apk_candidates_with_pm_logged(package, apk_paths, |_| {})
    }

    fn install_apk_candidates_with_pm_logged<F>(
        &mut self,
        package: &str,
        apk_paths: &[PathBuf],
        mut on_log: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        if apk_paths.is_empty() {
            return Err(GbstError::Adb(format!(
                "설치할 APK 후보가 없습니다: {package}"
            )));
        }

        let mut attempts = Vec::new();

        for apk_path in apk_paths {
            match self.install_apk_with_pm_logged(package, apk_path, |line| on_log(line)) {
                Ok(output) => return Ok(output),
                Err(err) => attempts.push(format!("{}: {err}", apk_path.display())),
            }
        }

        Err(GbstError::Adb(format!(
            "APK 후보 설치가 모두 실패했습니다: {package}: {}",
            attempts.join(" | ")
        )))
    }

    fn wait_for_user_package(
        &mut self,
        package: &str,
        attempts: usize,
        interval: Duration,
    ) -> bool {
        for attempt in 0..attempts.max(1) {
            if self.package_list_contains(package, false) {
                return true;
            }
            if attempt + 1 < attempts {
                thread::sleep(interval);
            }
        }
        false
    }

    fn requested_permissions(&mut self, package: &str) -> Result<Vec<String>> {
        let output = self.shell(&format!("dumpsys package {package}"))?;
        let mut permissions = Vec::new();
        let mut in_requested = false;

        for raw_line in output.lines() {
            let line = raw_line.trim();

            if !in_requested {
                if line.starts_with("requested permissions:") {
                    in_requested = true;
                }
                continue;
            }

            if line.starts_with("install permissions:")
                || line.starts_with("runtime permissions:")
                || line.starts_with("User ")
            {
                break;
            }

            if line.is_empty() {
                continue;
            }

            let mut permission = line.split_whitespace().next().unwrap_or(line).to_string();
            if let Some((left, _)) = permission.split_once(':') {
                permission = left.to_string();
            }

            if permission.starts_with("android.permission.") && !permissions.contains(&permission) {
                permissions.push(permission);
            }
        }

        Ok(permissions)
    }

    fn grant_requested_permissions<F>(
        &mut self,
        package: &str,
        permissions: &[String],
        mut on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let requested = self.requested_permissions(package).unwrap_or_default();
        let targets: Vec<&String> = if requested.is_empty() {
            permissions.iter().collect()
        } else {
            permissions
                .iter()
                .filter(|permission| requested.iter().any(|value| value == *permission))
                .collect()
        };

        if targets.is_empty() {
            on_log(format!("{package}: 부여할 런타임 권한 없음"));
            return Ok(());
        }

        for permission in targets {
            match self.shell(&format!("pm grant {package} {permission}")) {
                Ok(_) => on_log(format!("[OK] pm grant {permission}")),
                Err(err) => on_log(format!("[WARN] pm grant {permission}: {err}")),
            }
        }

        Ok(())
    }

    fn existing_appops(&mut self, package: &str) -> Vec<String> {
        let mut output = self
            .shell(&format!("appops get --user 0 {package}"))
            .unwrap_or_default();

        if output.trim().is_empty() {
            output = self
                .shell(&format!("appops get {package}"))
                .unwrap_or_default();
        }

        let mut ops: Vec<String> = Vec::new();
        for raw_line in output.lines() {
            let mut line = raw_line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            if let Some(rest) = line.strip_prefix("Uid mode:") {
                line = rest.trim().to_string();
            }

            let Some((op, _)) = line.split_once(':') else {
                continue;
            };

            let op = op.trim();
            if op.is_empty() {
                continue;
            }

            if op.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                && !ops.iter().any(|value| value.as_str() == op)
            {
                ops.push(op.to_string());
            }
        }

        ops
    }

    fn allow_existing_appops<F>(
        &mut self,
        package: &str,
        ops: &[String],
        mut on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let existing = self.existing_appops(package);
        let targets: Vec<&String> = if existing.is_empty() {
            ops.iter().collect()
        } else {
            ops.iter()
                .filter(|op| existing.iter().any(|value| value == *op))
                .collect()
        };

        if targets.is_empty() {
            on_log(format!("{package}: 적용할 AppOps 없음"));
        }

        for op in targets {
            for command in [
                format!("cmd appops set --user 0 {package} {op} allow"),
                format!("cmd appops set --user 0 --uid {package} {op} allow"),
                format!("appops set --user 0 {package} {op} allow"),
                format!("appops set --user 0 --uid {package} {op} allow"),
            ] {
                let _ = self.shell(&command);
            }
            on_log(format!("[OK] AppOps allow {op}"));
        }

        let _ = self.shell("appops write-settings");
        let _ = self.shell("cmd appops write-settings");
        Ok(())
    }

    pub fn reboot_system(&mut self) -> Result<()> {
        let device = self.connect_device()?;
        let result = device
            .reboot(RebootType::System)
            .map_err(|err| GbstError::Adb(format!("ADB reboot 실패: {err}")));
        self.drop_device();

        match result {
            Ok(()) => Ok(()),
            Err(err) => {
                let text = err.to_string().to_ascii_lowercase();
                if is_adbd_dropped_after_reboot(&text) {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }

    pub fn execute_plan<F>(&mut self, plan: &InstallPlan, on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        self.execute_plan_interruptible(plan, &AtomicBool::new(false), on_log)
    }

    /// Cooperative cancellation at safe step boundaries; never interrupts a running
    /// package install, file transfer or reboot command mid-flight.
    pub fn execute_plan_interruptible<F>(
        &mut self,
        plan: &InstallPlan,
        cancelled: &AtomicBool,
        mut on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let stage_totals = google_stage_totals(&plan.steps);
        let mut stage_positions = [0usize; 5];
        let mut google_stage: Option<u8> = None;
        let mut google_stage_guide_emitted = [false; 5];
        let mut preparation_emitted = false;
        let mut device_connect_prep_emitted = false;
        let mut finalize_emitted = false;
        let progress_total = plan.steps.len().max(1);

        for (step_index, step) in plan.steps.iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(GbstError::Adb(
                    "사용자 요청으로 작업을 안전한 단계 경계에서 중단했습니다. 이미 적용된 기기 변경은 자동으로 되돌리지 않습니다.".to_owned(),
                ));
            }
            if let Some(stage) = infer_google_stage(step, google_stage) {
                google_stage = Some(stage);
                let stage_index = stage as usize;
                if stage_index < google_stage_guide_emitted.len()
                    && mark_google_stage_started(stage, &mut google_stage_guide_emitted)
                {
                    emit_google_restore_wait_guide(&mut on_log);
                    // Text is emitted once per phase. GUI progress events keep
                    // updating for every completed step below.
                    on_log(google_stage_progress_log(
                        stage,
                        0,
                        stage_totals[stage_index],
                    ));
                }
            }

            let stage = google_stage.unwrap_or_default();
            on_log(format!(
                "__GBST_PLAN_PROGRESS__|{step_index}|{progress_total}|0|{stage}"
            ));
            let display_label = display_label_for_step(step, google_stage);
            let spinner_key = spinner_key_for_step(step, google_stage);

            // Stage percentage represents completed work, not the step merely started.
            let count_completed_step = google_stage.is_some()
                && display_label
                    .as_deref()
                    .is_some_and(|label| label.starts_with("구글 서비스 작업 중"));
            if google_stage.is_none() {
                if let Some(label) = display_label.as_deref() {
                    if label == PREPARATION_LOG {
                        if !preparation_emitted {
                            preparation_emitted = true;
                            on_log(label.to_string());
                        }
                    } else if label == DEVICE_CONNECT_PREP_LOG {
                        if !device_connect_prep_emitted {
                            device_connect_prep_emitted = true;
                            on_log(label.to_string());
                        }
                    } else if label == FINALIZE_LOG {
                        if !finalize_emitted {
                            finalize_emitted = true;
                            on_log(label.to_string());
                        }
                    } else if let Some(key) = spinner_key.as_deref() {
                        on_log(spinner_log(key, label));
                    } else {
                        on_log(label.to_string());
                    }
                }
            }

            if let PlanStep::RebootAndWait { .. } = step {
                if let Some(stage) = google_stage {
                    on_log(format!("__GBST_STAGE_PROGRESS__|{stage}|100"));
                    on_log(google_stage_complete_log(stage));
                }
            }

            match step {
                PlanStep::Shell {
                    label,
                    command,
                    policy,
                } => {
                    let shell_result = if is_intentional_delay_shell(command) {
                        Ok(String::new())
                    } else {
                        self.shell(command)
                    };
                    match shell_result {
                        Ok(out) => {
                            if should_show_command_output(label, out.trim()) {
                                on_log(format!("    {}", out.trim()));
                            }
                        }
                        Err(err) if *policy == FailurePolicy::Continue => {
                            if should_show_recoverable_error(label, &err.to_string()) {
                                on_log(format!("    [경고] 계속 진행: {err}"));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                PlanStep::InstallApk {
                    package,
                    path,
                    policy,
                } => match self.install_apk_with_pm_logged(package, path, |line| on_log(line)) {
                    Ok(out) => {
                        if should_show_command_output("APK 설치", out.trim()) {
                            on_log(format!("    {}", out.trim()));
                        }
                    }
                    Err(err) if *policy == FailurePolicy::Continue => {
                        if should_show_recoverable_error("APK 설치", &err.to_string()) {
                            on_log(format!("    [경고] APK 설치 실패, 계속 진행: {err}"));
                        }
                    }
                    Err(err) => return Err(err),
                },
                PlanStep::InstallApkCandidates {
                    package,
                    paths,
                    policy,
                } => {
                    match self
                        .install_apk_candidates_with_pm_logged(package, paths, |line| on_log(line))
                    {
                        Ok(out) => {
                            if should_show_command_output("APK 설치", out.trim()) {
                                on_log(format!("    {}", out.trim()));
                            }
                        }
                        Err(err) if *policy == FailurePolicy::Continue => {
                            if should_show_recoverable_error("APK 설치", &err.to_string()) {
                                on_log(format!("    [경고] APK 후보 설치 실패, 계속 진행: {err}"));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                PlanStep::Delay {
                    seconds, policy, ..
                } => match self.delay_seconds(*seconds, |line| on_log(line)) {
                    Ok(()) => {}
                    Err(err) if *policy == FailurePolicy::Continue => {
                        if should_show_recoverable_error("대기", &err.to_string()) {
                            on_log(format!("    [경고] 대기 실패, 계속 진행: {err}"));
                        }
                    }
                    Err(err) => return Err(err),
                },
                PlanStep::RebootAndWait { label, policy } => {
                    let reboot_count = reboot_count_for_step(plan.android_major, label);
                    let mut reboot_error = None;

                    for reboot_index in 0..reboot_count {
                        if reboot_count > 1 {
                            on_log(format!(
                                "[ADB] Android 13 단계 재부팅 {}/{}을 진행합니다.",
                                reboot_index + 1,
                                reboot_count
                            ));
                        }

                        if let Err(err) = self.reboot_and_wait_once(|line| on_log(line)) {
                            reboot_error = Some(err);
                            break;
                        }
                    }

                    if let Some(err) = reboot_error {
                        if *policy == FailurePolicy::Continue {
                            if should_show_recoverable_error("재부팅", &err.to_string()) {
                                on_log(format!("    [경고] 재부팅 실패, 계속 진행: {err}"));
                            }
                        } else {
                            return Err(err);
                        }
                    }
                }
                PlanStep::GrantRequestedPermissions {
                    package,
                    permissions,
                    policy,
                    ..
                } => match self.grant_requested_permissions(package, permissions, |_| {}) {
                    Ok(()) => {}
                    Err(err) if *policy == FailurePolicy::Continue => {
                        if should_show_recoverable_error(package, &err.to_string()) {
                            on_log(format!("    [경고] 권한 적용 실패, 계속 진행: {err}"));
                        }
                    }
                    Err(err) => return Err(err),
                },
                PlanStep::AppOpsAllowExisting {
                    package,
                    ops,
                    policy,
                    ..
                } => match self.allow_existing_appops(package, ops, |_| {}) {
                    Ok(()) => {}
                    Err(err) if *policy == FailurePolicy::Continue => {
                        if should_show_recoverable_error(package, &err.to_string()) {
                            on_log(format!("    [경고] AppOps 적용 실패, 계속 진행: {err}"));
                        }
                    }
                    Err(err) => return Err(err),
                },
                PlanStep::HealthCheck {
                    packages, policy, ..
                } => {
                    for package in packages {
                        match self.shell(&format!("pm path {package}")) {
                            Ok(output)
                                if output
                                    .lines()
                                    .any(|line| line.trim_start().starts_with("package:")) => {}
                            Ok(output) => {
                                let err = GbstError::Adb(format!(
                                    "필수 패키지 경로 확인 실패: {package}: {output}"
                                ));
                                if *policy == FailurePolicy::Stop {
                                    return Err(err);
                                }
                                on_log(format!("    [WARN] {err}"));
                            }
                            Err(err) if *policy == FailurePolicy::Continue => {
                                if should_show_recoverable_error(package, &err.to_string()) {
                                    on_log(format!("    [WARN] {package}: {err}"));
                                }
                            }
                            Err(err) => return Err(err),
                        }
                    }
                }
            }

            if count_completed_step {
                if let Some(stage) = google_stage {
                    stage_positions[stage as usize] =
                        stage_positions[stage as usize].saturating_add(1);
                    on_log(format!(
                        "__GBST_STAGE_PROGRESS__|{stage}|{}",
                        google_stage_progress_percent(
                            stage_positions[stage as usize],
                            stage_totals[stage as usize],
                        ),
                    ));
                }
            }
            on_log(format!(
                "__GBST_PLAN_PROGRESS__|{}|{progress_total}|0|{stage}",
                step_index + 1
            ));
            if should_clear_google_stage(step) {
                google_stage = None;
            }
        }
        Ok(())
    }

    fn delay_seconds<F>(&mut self, seconds: u64, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        on_log(format!("[ADB] 작업 안정화 대기: {seconds}초"));
        std::thread::sleep(Duration::from_secs(seconds.min(180)));
        Ok(())
    }

    fn reboot_and_wait_once<F>(&mut self, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        let previous_boot_id = self
            .shell("cat /proc/sys/kernel/random/boot_id")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        on_log(spinner_log("adb_reboot", "[ADB] 기기 재부팅 중... │"));
        self.reboot_system()?;
        self.wait_ready_until_for_reboot(
            ADB_REBOOT_WAIT_TIMEOUT,
            previous_boot_id.as_deref(),
            |line| on_log(line),
        )?;
        Ok(())
    }

    fn ensure_dashboard_ready(&mut self) -> Result<()> {
        match self.check_usb_state()? {
            AdbUsbState::Ready | AdbUsbState::Recovery => Ok(()),
            AdbUsbState::ServerBlocking => {
                self.reclaim_direct_usb_interface();

                match self.check_usb_state()? {
                    AdbUsbState::Ready | AdbUsbState::Recovery => Ok(()),
                    AdbUsbState::Unauthorized => {
                        Err(GbstError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string()))
                    }
                    AdbUsbState::ServerBlocking => Err(GbstError::Adb(
                        "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string(),
                    )),
                    AdbUsbState::NoDevice => Err(GbstError::Adb(
                        "USB ADB 기기를 찾지 못했습니다.".to_string(),
                    )),
                    AdbUsbState::MultipleDevices => Err(GbstError::Adb(
                        "ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요.".to_string(),
                    )),
                    AdbUsbState::Error(err) => Err(GbstError::Adb(err)),
                }
            }
            AdbUsbState::Unauthorized => Err(GbstError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),
            AdbUsbState::NoDevice => Err(GbstError::Adb(
                "USB ADB 기기를 찾지 못했습니다.".to_string(),
            )),
            AdbUsbState::MultipleDevices => Err(GbstError::Adb(
                "ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요.".to_string(),
            )),
            AdbUsbState::Error(err) => Err(GbstError::Adb(err)),
        }
    }

    fn connect_device(&mut self) -> Result<&mut ADBUSBDevice> {
        if self.device.is_none() {
            let key_path = ensure_adb_key()?;
            let mut last_error: Option<String> = None;

            for attempt in 0..ADB_CONNECT_RETRY_ATTEMPTS {
                match ADBUSBDevice::autodetect_with_custom_private_key(key_path.clone()) {
                    Ok(mut device) => {
                        let mut stdout = Vec::new();
                        if let Err(err) = device.shell_command(
                            &"getprop ro.serialno",
                            Some(&mut stdout as &mut dyn Write),
                            None,
                        ) {
                            last_error = Some(err.to_string());
                            if attempt + 1 < ADB_CONNECT_RETRY_ATTEMPTS {
                                thread::sleep(ADB_CONNECT_RETRY_BACKOFF);
                            }
                            continue;
                        }
                        let serial = String::from_utf8_lossy(&stdout).trim().to_string();
                        if let Some(pinned) = self.pinned_serial.as_deref() {
                            if serial.is_empty() || serial != pinned {
                                return Err(GbstError::Adb(format!(
                                    "ADB 기기 변경 감지: 최초={pinned}, 현재={serial}"
                                )));
                            }
                        } else if !serial.is_empty() {
                            self.pinned_serial = Some(serial.clone());
                        }
                        if !serial.is_empty() {
                            self.serial = Some(serial);
                        }

                        let mut bootmode_stdout = Vec::new();
                        let _ = device.shell_command(
                            &"getprop ro.bootmode",
                            Some(&mut bootmode_stdout as &mut dyn Write),
                            None,
                        );
                        let bootmode = String::from_utf8_lossy(&bootmode_stdout).trim().to_string();
                        self.cached_bootmode = Some(if bootmode == "recovery" {
                            "recovery"
                        } else {
                            "device"
                        });

                        self.device = Some(device);
                        return Ok(self.device.as_mut().expect("ADB device just stored"));
                    }
                    Err(err) => {
                        last_error = Some(err.to_string());
                        if attempt + 1 < ADB_CONNECT_RETRY_ATTEMPTS {
                            thread::sleep(ADB_CONNECT_RETRY_BACKOFF);
                        }
                    }
                }
            }

            let detail = last_error.unwrap_or_else(|| "ADB USB 직접 연결 실패".to_string());
            record_adb_diagnostic("ADB USB 직접 연결", &detail);
            return Err(GbstError::Adb(detail));
        }

        Ok(self.device.as_mut().expect("ADB device cached"))
    }

    fn drop_device(&mut self) {
        self.device = None;
        self.serial = None;
        self.cached_bootmode = None;
    }
}

/// Failure journal; intentionally does not store successful shell results or APK content.
fn record_adb_diagnostic(context: &str, detail: &str) {
    let dir = paths::diagnostic_logs_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("adb-diagnostics.log");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|delta| delta.as_secs())
        .unwrap_or_default();
    // Keep the diagnostic journal bounded to 5 MiB to avoid indefinite growth.
    if path
        .metadata()
        .is_ok_and(|metadata| metadata.len() > 5 * 1024 * 1024)
    {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        // Errors may contain private device serials and paths: do not log shell output.
        let safe = detail.replace('\r', " ").replace('\n', " ");
        let _ = writeln!(file, "[{now}] {context}: {safe}");
    }
}

fn normalize_model_property(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

fn detect_supported_lenovo_model_code(values: &[String]) -> Option<&'static str> {
    const MODEL_ALIASES: &[(&str, &str)] = &[
        ("TB9707F", "TB-9707F"),
        ("TB710FU", "TB710FU"),
        ("TB710FC", "TB710FC"),
        ("TB711FU", "TB711FU"),
        ("TB711FC", "TB711FC"),
        ("TB522FU", "TB522FU"),
        ("TB522FC", "TB522FC"),
        ("TB520FU", "TB520FU"),
        ("TB520FC", "TB520FC"),
        ("TB378FC", "TB378FC"),
        ("TB378FU", "TB378FU"),
        ("TB390FU", "TB390FU"),
        ("TB390FC", "TB390FC"),
        ("TB376FC", "TB376FC"),
        ("TB376FU", "TB376FU"),
        ("TB375FC", "TB375FC"),
        ("TB375FU", "TB375FU"),
        ("TB373FU", "TB373FU"),
        ("TB373FC", "TB373FC"),
        ("TB371FC", "TB371FC"),
        ("TB371FU", "TB371FU"),
        ("TB365FC", "TB365FC"),
        ("TB365FU", "TB365FU"),
        ("TB361FU", "TB361FU"),
        ("TB361FC", "TB361FC"),
        ("TB335FC", "TB335FC"),
        ("TB335FU", "TB335FU"),
        ("TB336FU", "TB336FU"),
        ("TB336FC", "TB336FC"),
        ("TB331FC", "TB331FC"),
        ("TB331FU", "TB331FU"),
        ("TB324ZC", "TB324ZC"),
        ("TB323FC", "TB323FC"),
        ("TB323FU", "TB323FU"),
        ("TB322FC", "TB322FC"),
        ("TB322FU", "TB322FU"),
        ("TB321FC", "TB321FC"),
        ("TB321FU", "TB321FU"),
        ("TB320FC", "TB320FC"),
        ("TB320FU", "TB320FU"),
    ];

    for value in values {
        let normalized = normalize_model_property(value);
        for &(alias, canonical) in MODEL_ALIASES {
            if normalized.contains(alias) {
                return Some(canonical);
            }
        }
    }

    None
}

struct ProgressReader<'a, R: Read> {
    inner: R,
    total: u64,
    transferred: u64,
    last_percent: Option<u8>,
    on_progress: &'a mut dyn FnMut(u64, u64, u8),
}

impl<'a, R: Read> ProgressReader<'a, R> {
    fn new(inner: R, total: u64, on_progress: &'a mut dyn FnMut(u64, u64, u8)) -> Self {
        Self {
            inner,
            total,
            transferred: 0,
            last_percent: None,
            on_progress,
        }
    }
}

impl<R: Read> Read for ProgressReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.transferred = self.transferred.saturating_add(read as u64);
        let percent = if self.total == 0 {
            100
        } else {
            ((self.transferred.saturating_mul(100) / self.total).min(100)) as u8
        };

        if self.last_percent != Some(percent) {
            self.last_percent = Some(percent);
            (self.on_progress)(self.transferred, self.total, percent);
        }
        Ok(read)
    }
}

impl Default for DirectAdb {
    fn default() -> Self {
        Self::new()
    }
}

/// Explicit command assembled from fixed, application-owned paths only.
const DWAS_KR_FULL_CLEANUP_MARKER: &str = "__DWAS_KR_FULL_CLEANUP_OK__";

/// Shell command uses only the four literal developer-authorized paths.
/// Symlink leaf guard prevents following an attacker-controlled redirected root.
fn dwas_kr_full_cleanup_command() -> &'static str {
    // Four explicit paths requested by the developer; no globbing, no parent
    // directory deletion. Refuse a symlink at any of the four leaf paths.
    "if [ -L /storage/self/primary/dwas_kr ] || [ -L /storage/self/primary/GBST ] || [ -L /data/local/tmp/dwas_kr ] || [ -L /data/local/tmp/GBST ]; then echo __DWAS_KR_FULL_CLEANUP_BLOCKED__; else rm -rf /storage/self/primary/dwas_kr /storage/self/primary/GBST /data/local/tmp/dwas_kr /data/local/tmp/GBST 2>/dev/null; if [ -e /storage/self/primary/dwas_kr ] || [ -L /storage/self/primary/dwas_kr ] || [ -e /storage/self/primary/GBST ] || [ -L /storage/self/primary/GBST ] || [ -e /data/local/tmp/dwas_kr ] || [ -L /data/local/tmp/dwas_kr ] || [ -e /data/local/tmp/GBST ] || [ -L /data/local/tmp/GBST ]; then echo __DWAS_KR_FULL_CLEANUP_FAILED__; else echo __DWAS_KR_FULL_CLEANUP_OK__; fi; fi"
}

fn gbst_device_cleanup_command() -> &'static str {
    // A symlink in the parent must never redirect rm into another directory.
    "if [ -L /data/local/tmp/dwas_kr ]; then echo __GBST_CLEANUP_FAILED__; else if [ -L /data/local/tmp/dwas_kr/GBST ]; then rm -f /data/local/tmp/dwas_kr/GBST; fi; if [ -e /data/local/tmp/dwas_kr/GBST ]; then rm -rf /data/local/tmp/dwas_kr/GBST; fi; if [ -e /data/local/tmp/dwas_kr/GBST ]; then echo __GBST_CLEANUP_FAILED__; else rmdir /data/local/tmp/dwas_kr 2>/dev/null || true; echo __GBST_DEVICE_TEMP_CLEANED__; fi; fi"
}

fn sha256_of_file(path: &Path) -> Result<String> {
    let mut reader = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let len = reader.read(&mut buffer)?;
        if len == 0 {
            break;
        }
        digest.update(&buffer[..len]);
    }
    Ok(hex::encode(digest.finalize()))
}

pub fn ensure_adb_key() -> Result<PathBuf> {
    let _ = paths::ensure_runtime_directories();

    let stable_path = paths::stable_adb_key_path();
    let lpmbox_path = paths::lpmbox_adb_key_path();

    if stable_path.is_file() {
        if is_valid_adb_private_key(&stable_path) {
            return Ok(stable_path);
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or_default();
        let backup = stable_path.with_extension(format!("invalid-{timestamp}.bak"));
        std::fs::rename(&stable_path, &backup)?;
        record_adb_diagnostic(
            "ADB RSA key",
            "유효하지 않은 기존 키를 별도 백업 후 재생성합니다.",
        );
    }

    if lpmbox_path.is_file() && is_valid_adb_private_key(&lpmbox_path) {
        copy_key_pair(&lpmbox_path, &stable_path)?;
        return Ok(stable_path);
    }

    generate_key_pair(&stable_path)?;
    Ok(stable_path)
}

fn is_valid_adb_private_key(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    rsa::RsaPrivateKey::from_pkcs8_pem(&content).is_ok()
        || rsa::RsaPrivateKey::from_pkcs1_pem(&content).is_ok()
}

fn generate_key_pair(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        GbstError::Adb(format!(
            "ADB key parent 경로가 없습니다: {}",
            path.display()
        ))
    })?;
    std::fs::create_dir_all(parent)?;

    let private_key = rsa::RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048)
        .map_err(|err| GbstError::Adb(format!("ADB RSA key 생성 실패: {err}")))?;

    let pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|err| GbstError::Adb(format!("ADB RSA key PEM 변환 실패: {err}")))?;

    std::fs::write(path, pem.as_bytes())?;
    Ok(())
}

fn copy_key_pair(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(from, to)?;

    let from_pub = from.with_extension("pub");
    let to_pub = to.with_extension("pub");
    if from_pub.is_file() {
        let _ = std::fs::copy(from_pub, to_pub);
    }

    Ok(())
}

fn prepare_adb_usb_environment() {
    let _ = paths::ensure_runtime_directories();
    // ADBUSBDevice gets the key path directly; do not change global process
    // environment or terminate other programs using the ADB server.
    let _ = ensure_adb_key();
}

fn adb_usb_candidate_exists() -> Result<bool> {
    let devices = find_all_connected_adb_devices()
        .map_err(|err| GbstError::Adb(format!("ADB USB 기기 검색 실패: {err}")))?;

    Ok(!devices.is_empty())
}

pub fn adb_server_running() -> bool {
    TcpStream::connect_timeout(&ADB_SERVER_ADDR.into(), ADB_SERVER_PROBE_TIMEOUT).is_ok()
}

pub fn kill_adb_server_direct() -> Result<()> {
    let mut stream = TcpStream::connect_timeout(&ADB_SERVER_ADDR.into(), Duration::from_secs(2))
        .map_err(|err| GbstError::Adb(format!("adb server 연결 실패: {err}")))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| GbstError::Adb(format!("adb server write timeout 설정 실패: {err}")))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| GbstError::Adb(format!("adb server read timeout 설정 실패: {err}")))?;

    let payload = b"host:kill";
    let header = format!("{:04x}", payload.len());

    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(payload))
        .map_err(|err| GbstError::Adb(format!("adb server kill 요청 실패: {err}")))?;

    let mut reply = [0u8; 4];

    match stream.read_exact(&mut reply) {
        Ok(()) => match &reply {
            b"OKAY" => Ok(()),
            b"FAIL" => {
                let mut len_buf = [0u8; 4];
                let _ = stream.read_exact(&mut len_buf);
                let size = std::str::from_utf8(&len_buf)
                    .ok()
                    .and_then(|value| usize::from_str_radix(value, 16).ok())
                    .unwrap_or(0);
                let mut msg_buf = vec![0u8; size.min(1024)];
                let _ = stream.read_exact(&mut msg_buf);

                Err(GbstError::Adb(format!(
                    "adb server가 host:kill 요청을 거부했습니다: {}",
                    String::from_utf8_lossy(&msg_buf)
                )))
            }
            other => Err(GbstError::Adb(format!(
                "adb server host:kill 응답이 예상과 다릅니다: {other:?}"
            ))),
        },
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset
            ) =>
        {
            Ok(())
        }
        Err(err) => Err(GbstError::Adb(format!(
            "adb server host:kill 응답 읽기 실패: {err}"
        ))),
    }
}

fn pm_install_explicitly_succeeded(output: &str) -> bool {
    output
        .lines()
        .map(str::trim)
        .any(|line| line.eq_ignore_ascii_case("Success") || line.starts_with("Success"))
}

fn pm_install_explicitly_failed(output: &str) -> bool {
    let lowered = output.to_ascii_lowercase();
    lowered.contains("failure")
        || lowered.contains("install_failed_")
        || lowered.contains("error:")
        || lowered.contains("exception")
}

fn compact_pm_output(output: &str) -> String {
    let compact = output.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "빈 응답".to_string()
    } else {
        compact
    }
}

/// Create a user-scoped activation command only when actually required.
fn post_install_reenable_command(package: &str, state: &GooglePackageState) -> Option<String> {
    (state.installed_for_user && state.disabled).then(|| format!("pm enable --user 0 {package}"))
}

fn parse_enabled_state_from_user_line(line: &str) -> Option<u32> {
    line.split_whitespace()
        .find_map(|part| part.strip_prefix("enabled="))
        .and_then(|value| {
            let digits = value
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>();
            digits.parse::<u32>().ok()
        })
}

fn parse_pm_list_version_code(output: &str, package: &str) -> Option<u64> {
    let expected_package = format!("package:{package}");

    output.lines().find_map(|raw_line| {
        let line = raw_line.trim();
        if !line.starts_with(&expected_package) {
            return None;
        }

        line.split_whitespace().find_map(|token| {
            token
                .strip_prefix("versionCode:")
                .or_else(|| token.strip_prefix("versionCode="))
                .and_then(parse_leading_u64)
        })
    })
}

fn parse_installed_version_code(dumpsys: &str) -> Option<u64> {
    let mut fallback = None;

    for token in dumpsys.split_whitespace() {
        if let Some(value) = token.strip_prefix("longVersionCode=") {
            if let Some(parsed) = parse_leading_u64(value) {
                return Some(parsed);
            }
        }

        if fallback.is_none() {
            if let Some(value) = token.strip_prefix("versionCode=") {
                fallback = parse_leading_u64(value);
            }
        }
    }

    fallback
}

fn parse_leading_u64(value: &str) -> Option<u64> {
    let digits = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse::<u64>().ok()
}

/// Allow the human-readable phase-start message only once per Google stage.
/// Structured percentage updates are emitted independently on every step.
fn mark_google_stage_started(stage: u8, emitted: &mut [bool; 5]) -> bool {
    let index = usize::from(stage);
    if !(1..=4).contains(&stage) || emitted[index] {
        return false;
    }
    emitted[index] = true;
    true
}

fn emit_google_restore_wait_guide<F>(on_log: &mut F)
where
    F: FnMut(String),
{
    on_log("__BLANK__".to_string());
    on_log("[안내] 동일한 질문을 주시는 분들이 많아 안내 드립니다.".to_string());
    on_log("[안내] 프로그램이 작업 중이므로 작업이 멈추거나, 중단되거나,".to_string());
    on_log("[안내] 연결이 끊긴 것이 아니니 안심하시고 작업이 완료 될 때까지".to_string());
    on_log("[안내] PC(노트북), 케이블, 기기를 가만히 둔 상태로 1분~5분 대기해 주세요.".to_string());
}

fn display_label_for_step(step: &PlanStep, google_stage: Option<u8>) -> Option<String> {
    match step {
        PlanStep::Shell { label, .. } => display_label_from_text(label, google_stage),
        PlanStep::InstallApk { .. } | PlanStep::InstallApkCandidates { .. } => google_stage
            .map(|stage| format!("구글 서비스 작업 중... ({stage}/4)"))
            .or_else(|| Some("[APK] APK 설치 중".to_string())),
        PlanStep::Delay { label, .. } => {
            if let Some(stage) = google_stage {
                if is_google_stage_delay(label) {
                    return Some(format!("구글 서비스 작업 중... ({stage}/4)"));
                }
            }

            if is_ota_label(label) {
                Some(OTA_DISABLE_LOG.to_string())
            } else {
                None
            }
        }
        PlanStep::RebootAndWait { .. } => Some("[ADB] 기기 재부팅 중...".to_string()),
        PlanStep::GrantRequestedPermissions { .. } | PlanStep::AppOpsAllowExisting { .. } => {
            Some("구글 서비스 작업 중... (4/4)".to_string())
        }
        PlanStep::HealthCheck { .. } => Some(GOOGLE_RESTORE_DONE_LOG.to_string()),
    }
}

fn display_label_from_text(label: &str, google_stage: Option<u8>) -> Option<String> {
    if is_wakeup_detail_label(label) {
        return None;
    }

    if label == "화면 깨우기" {
        return Some(DEVICE_CONNECT_PREP_LOG.to_string());
    }

    if is_ota_label(label) {
        return Some(OTA_DISABLE_LOG.to_string());
    }

    if let Some(stage) = google_stage {
        if is_google_stage_detail_label(label) || is_google_stage_delay(label) {
            return Some(format!("구글 서비스 작업 중... ({stage}/4)"));
        }
    }

    if is_preparation_label(label) {
        return Some(PREPARATION_LOG.to_string());
    }

    if label.contains("Play Store 초기 설정")
        || label.contains("최종 초기화")
        || label.contains("최종 설정")
        || label.contains("작업 후")
        || label.contains("Android 설정 앱")
    {
        return Some(FINALIZE_LOG.to_string());
    }

    Some(label.to_string())
}

fn spinner_key_for_step(step: &PlanStep, google_stage: Option<u8>) -> Option<String> {
    match step {
        PlanStep::Shell { label, .. } => {
            if is_ota_label(label) {
                Some("ota".to_string())
            } else if label == "화면 깨우기" {
                Some("wakeup".to_string())
            } else if is_preparation_label(label) {
                Some("preparation".to_string())
            } else if let Some(stage) = google_stage {
                if is_google_stage_detail_label(label) || is_google_stage_delay(label) {
                    Some(format!("google_restore_{stage}"))
                } else {
                    None
                }
            } else if label.contains("Play Store 초기 설정")
                || label.contains("최종 초기화")
                || label.contains("최종 설정")
                || label.contains("작업 후")
                || label.contains("Android 설정 앱")
            {
                Some("finalize".to_string())
            } else {
                None
            }
        }
        PlanStep::InstallApk { .. } | PlanStep::InstallApkCandidates { .. } => google_stage
            .map(|stage| format!("google_restore_{stage}"))
            .or_else(|| Some("install_apk".to_string())),
        PlanStep::Delay { label, .. } => {
            if is_ota_label(label) {
                Some("ota".to_string())
            } else if let Some(stage) = google_stage {
                if is_google_stage_delay(label) {
                    Some(format!("google_restore_{stage}"))
                } else {
                    None
                }
            } else {
                None
            }
        }
        PlanStep::RebootAndWait { .. } => Some("adb_reboot".to_string()),
        PlanStep::GrantRequestedPermissions { .. } | PlanStep::AppOpsAllowExisting { .. } => {
            Some("google_restore_4".to_string())
        }
        PlanStep::HealthCheck { .. } => Some("health_check".to_string()),
    }
}

fn infer_google_stage(step: &PlanStep, current: Option<u8>) -> Option<u8> {
    let label = match step {
        PlanStep::Shell { label, .. }
        | PlanStep::Delay { label, .. }
        | PlanStep::RebootAndWait { label, .. }
        | PlanStep::GrantRequestedPermissions { label, .. }
        | PlanStep::AppOpsAllowExisting { label, .. }
        | PlanStep::HealthCheck { label, .. } => label.as_str(),
        PlanStep::InstallApk { .. } | PlanStep::InstallApkCandidates { .. } => return current,
    };

    if label.contains("루틴 1")
        || matches!(
            label,
            "PartnerSetup 제거"
                | "Google ext.shared 제거"
                | "ConfigUpdater 제거"
                | "OneTimeInitializer 제거"
                | "PrintService Recommendation 제거"
        )
    {
        return Some(1);
    }

    if label.contains("루틴 2")
        || matches!(
            label,
            "Play Store 제거"
                | "Google Play Services 제거"
                | "Google Play Services 데이터 초기화"
                | "Play Store 데이터 초기화"
                | "Play Store 사용 가능 설정"
        )
    {
        return Some(2);
    }

    if label.contains("루틴 3")
        || label.contains("Google Services Framework")
        || label.starts_with("GSF 패키지")
    {
        return Some(3);
    }

    if label.contains("루틴 4")
        || label.contains("요청 권한")
        || label.contains("AppOps")
        || label.contains("Global pipe key")
        || (current.is_none()
            && (label.starts_with("패키지 복구:") || label.starts_with("패키지 활성화:")))
    {
        return Some(4);
    }

    current
}

fn should_clear_google_stage(step: &PlanStep) -> bool {
    match step {
        PlanStep::RebootAndWait { label, .. } => {
            label.contains("루틴") && label.contains("완료 후 재부팅")
        }
        _ => false,
    }
}

fn is_ota_label(label: &str) -> bool {
    label.contains("OTA 네트워크")
        || label.contains("OTA 새 버전")
        || label.contains("Setup Wizard OTA")
        || label.contains("OTA 프로세스")
        || label.contains("OTA 자동 업데이트")
        || label.contains("OTA 알림 비활성화")
}

fn is_wakeup_detail_label(label: &str) -> bool {
    label.contains("잠금 해제 보조 키 입력")
        || label.starts_with("키 이벤트 ")
        || label == "기기 깨우기 후 2초 대기"
}

fn is_preparation_label(label: &str) -> bool {
    label.contains("중국 입력기")
        || label.contains("Lenovo OTA 앱 제거")
        || label.contains("Lenovo tbengine")
        || label.contains("ZUI homesettings")
        || label.contains("Lenovo ue.device")
        || label.contains("무음 모드")
        || label.contains("화면 꺼짐")
        || label.contains("가로 화면")
        || label.contains("화면 밝기 낮춤")
        || label.contains("사전 준비")
}

fn is_google_stage_detail_label(label: &str) -> bool {
    label.contains("제거")
        || label.contains("APK 설치")
        || label.contains("패키지 복구")
        || label.contains("패키지 활성화")
        || label.contains("데이터 초기화")
        || label.contains("Play Store 사용 가능 설정")
        || label.contains("요청 권한")
        || label.contains("AppOps")
        || label.contains("Global pipe key")
        || label.contains("루틴")
}

fn is_google_stage_delay(label: &str) -> bool {
    label.contains("Google 서비스 복구 루틴")
}

fn should_show_command_output(_label: &str, _output: &str) -> bool {
    false
}

fn should_show_recoverable_error(_label: &str, error: &str) -> bool {
    // Continue-policy commands still preserve actionable failure diagnostics.
    !error.trim().is_empty()
}

fn google_stage_totals(steps: &[PlanStep]) -> [usize; 5] {
    let mut totals = [0usize; 5];
    let mut current_stage: Option<u8> = None;

    for step in steps {
        if let Some(stage) = infer_google_stage(step, current_stage) {
            current_stage = Some(stage);
        }

        if let Some(stage) = current_stage {
            if display_label_for_step(step, current_stage)
                .as_deref()
                .is_some_and(|label| label.starts_with("구글 서비스 작업 중"))
            {
                totals[stage as usize] = totals[stage as usize].saturating_add(1);
            }
        }

        if should_clear_google_stage(step) {
            current_stage = None;
        }
    }

    for total in totals.iter_mut().skip(1) {
        if *total == 0 {
            *total = 1;
        }
    }

    totals
}

/// The original stage totals still determine the phase percentage,
/// but that number is now a structured GUI signal instead of a text bar.
fn google_stage_progress_percent(current: usize, total: usize) -> u8 {
    let total = total.max(1);
    let current = current.min(total);
    ((current as f64 * 100.0 / total as f64).round() as u8).min(100)
}

fn google_stage_progress_log(stage: u8, _current: usize, _total: usize) -> String {
    spinner_log(
        &format!("google_restore_{stage}"),
        &format!("[APK] 구글 서비스 작업 중... ({stage}/4)"),
    )
}

fn google_stage_complete_log(stage: u8) -> String {
    spinner_log(
        &format!("google_restore_{stage}"),
        &format!("[APK] 구글 서비스 작업 {stage}단계 완료"),
    )
}

fn should_reuse_existing_package(
    existing_version: Option<u64>,
    local_version: Option<u64>,
) -> bool {
    matches!((existing_version, local_version), (Some(existing), Some(local)) if existing >= local)
}

fn is_signature_update_incompatible(output: &str) -> bool {
    output
        .to_ascii_uppercase()
        .contains("INSTALL_FAILED_UPDATE_INCOMPATIBLE")
}

fn is_intentional_delay_shell(command: &str) -> bool {
    let command = command.trim();
    command
        .strip_prefix("sleep ")
        .is_some_and(|seconds| seconds.trim().parse::<f64>().is_ok())
}

fn screen_off_timeout_display_value(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        return "알 수 없음".to_string();
    }

    if value.parse::<u64>().is_ok_and(|parsed| parsed >= 600_000) {
        return "화면 꺼짐 비활성화".to_string();
    }

    value.to_string()
}

fn parse_reboot_probe(output: &str) -> (Option<String>, String) {
    let mut lines = output.lines().map(str::trim);
    let boot_id = lines
        .next()
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let boot_completed = lines.next().unwrap_or_default().to_string();
    (boot_id, boot_completed)
}

fn reboot_session_is_ready(
    previous_boot_id: Option<&str>,
    current_boot_id: Option<&str>,
    boot_completed: &str,
) -> bool {
    if boot_completed.trim() != "1" {
        return false;
    }

    match previous_boot_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(previous) => current_boot_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some_and(|current| current != previous),
        None => true,
    }
}

fn parse_screen_brightness_value(output: &str) -> Option<u32> {
    for line in output.lines() {
        let value = line.trim();
        if value.is_empty() || value.eq_ignore_ascii_case("null") {
            continue;
        }
        if !value.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }

        if let Ok(parsed) = value.parse::<u32>() {
            if parsed <= 4095 {
                return Some(parsed);
            }
        }
    }

    None
}

fn parse_screen_brightness_float(output: &str) -> Option<u32> {
    for token in
        output.split(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+'))
    {
        let token = token.trim();
        if token.is_empty() || !token.contains('.') {
            continue;
        }

        if let Ok(value) = token.parse::<f64>() {
            if (0.0..=1.0).contains(&value) {
                return Some((value * 255.0).round() as u32);
            }
        }
    }

    None
}

fn reboot_count_for_step(_android_major: AndroidMajor, _label: &str) -> usize {
    // One verified reboot at every plan boundary on Android 13-17.
    1
}

fn spinner_log(key: &str, message: &str) -> String {
    format!("__SPINNER__|{key}|{message}")
}

fn is_adbd_dropped_after_reboot(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();

    lower.contains("pipe")
        || lower.contains("broken pipe")
        || lower.contains("no device")
        || lower.contains("device disconnected")
        || lower.contains("unexpected eof")
        || lower.contains("end of file")
        || lower.contains("input/output error")
        || lower.contains("i/o error")
        || lower.contains("closed")
        || lower.contains("reset")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_sane_screen_brightness_values() {
        assert_eq!(parse_screen_brightness_value("80\r\n"), Some(80));
        assert_eq!(parse_screen_brightness_value(" 255 "), Some(255));
        assert_eq!(parse_screen_brightness_value("2568723"), None);
        assert_eq!(parse_screen_brightness_value("null"), None);
    }

    #[test]
    fn converts_normalized_display_brightness_to_android_integer() {
        assert_eq!(parse_screen_brightness_float("0.3137255"), Some(80));
        assert_eq!(parse_screen_brightness_float("Brightness: 1.0"), Some(255));
        assert_eq!(parse_screen_brightness_float("Brightness: 1.5"), None);
    }

    #[test]
    fn preserves_exact_lenovo_sku_instead_of_cross_model_canonicalization() {
        for model in ["TB373FU", "TB361FU", "TB336FU", "TB390FU", "TB711FU"] {
            assert_eq!(
                detect_supported_lenovo_model_code(&[model.to_string()]),
                Some(model)
            );
        }
    }

    #[test]
    fn screen_timeout_max_int_is_presented_as_never_without_changing_raw_value() {
        assert_eq!(
            screen_off_timeout_display_value("2147483647"),
            "화면 꺼짐 비활성화"
        );
        assert_eq!(
            screen_off_timeout_display_value("2147483648"),
            "화면 꺼짐 비활성화"
        );
        assert_eq!(
            screen_off_timeout_display_value("600000"),
            "화면 꺼짐 비활성화"
        );
        assert_eq!(screen_off_timeout_display_value("599999"), "599999");
        assert_eq!(screen_off_timeout_display_value("null"), "알 수 없음");
    }

    #[test]
    fn pm_install_result_is_not_treated_as_success_when_android_reports_failure() {
        assert!(pm_install_explicitly_succeeded("Success\n"));
        assert!(!pm_install_explicitly_failed("Success\n"));
        assert!(pm_install_explicitly_failed(
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE]"
        ));
        assert!(!pm_install_explicitly_succeeded(
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE]"
        ));
    }

    #[test]
    fn transfer_progress_reports_monotonic_percentages() {
        let input = vec![7u8; 1024];
        let mut events = Vec::new();
        {
            let mut callback = |done: u64, total: u64, percent: u8| {
                events.push((done, total, percent));
            };
            let mut reader = ProgressReader::new(std::io::Cursor::new(input), 1024, &mut callback);
            let mut sink = Vec::new();
            std::io::copy(&mut reader, &mut sink).unwrap();
        }
        assert!(events.windows(2).all(|pair| pair[0].2 <= pair[1].2));
        assert_eq!(events.last().map(|event| event.2), Some(100));
    }

    #[test]
    fn reboot_probe_keeps_boot_id_and_completion_from_one_shell_response() {
        let (boot_id, completed) = parse_reboot_probe("new-boot-id\n1\n");
        assert_eq!(boot_id.as_deref(), Some("new-boot-id"));
        assert_eq!(completed, "1");
    }

    #[test]
    fn reboot_readiness_requires_completed_fresh_boot_when_boot_id_is_known() {
        assert!(!reboot_session_is_ready(Some("old"), Some("old"), "1"));
        assert!(!reboot_session_is_ready(Some("old"), Some("new"), "0"));
        assert!(reboot_session_is_ready(Some("old"), Some("new"), "1"));
        assert!(reboot_session_is_ready(None, Some("new"), "1"));
    }

    #[test]
    fn reuses_existing_package_only_when_device_version_is_not_older() {
        assert!(should_reuse_existing_package(
            Some(263232029),
            Some(263232029)
        ));
        assert!(should_reuse_existing_package(
            Some(263232030),
            Some(263232029)
        ));
        assert!(!should_reuse_existing_package(
            Some(263232028),
            Some(263232029)
        ));
        assert!(!should_reuse_existing_package(None, Some(263232029)));
        assert!(!should_reuse_existing_package(Some(263232029), None));
    }

    #[test]
    fn recognizes_signature_mismatch_as_install_existing_fallback_case() {
        assert!(is_signature_update_incompatible(
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE: signatures do not match]"
        ));
        assert!(!is_signature_update_incompatible(
            "Failure [INSTALL_FAILED_VERSION_DOWNGRADE]"
        ));
    }

    #[test]
    fn recognizes_only_explicit_sleep_shell_steps_as_skippable_delays() {
        assert!(is_intentional_delay_shell("sleep 1.2"));
        assert!(is_intentional_delay_shell("  sleep 0.3  "));
        assert!(!is_intentional_delay_shell("echo sleep 1"));
        assert!(!is_intentional_delay_shell("sleep nope"));
    }

    #[test]
    fn full_dwas_kr_cleanup_targets_only_requested_dirs() {
        let command = dwas_kr_full_cleanup_command();
        const CLEANUP_DIRS: [&str; 4] = [
            "/storage/self/primary/dwas_kr",
            "/storage/self/primary/GBST",
            "/data/local/tmp/dwas_kr",
            "/data/local/tmp/GBST",
        ];
        for path in CLEANUP_DIRS {
            assert!(command.contains(path));
            assert!(command.contains(&format!("[ -L {path} ]")));
            assert!(command.contains(&format!("[ -e {path} ]")));
        }
        let deletion = command
            .split_once("else rm -rf ")
            .unwrap()
            .1
            .split_once(" 2>/dev/null")
            .unwrap()
            .0;
        assert_eq!(
            deletion.split_whitespace().collect::<Vec<_>>(),
            CLEANUP_DIRS.to_vec()
        );
        assert!(command.contains(DWAS_KR_FULL_CLEANUP_MARKER));
        assert!(!command.contains("/storage/self/primary/Download"));
        assert!(!command.contains("rm -rf /data/local/tmp;"));
        assert!(!command.contains("rm -rf /storage/self/primary;"));
    }

    #[test]
    fn cleanup_command_is_scoped_to_owned_gbst_directory() {
        let command = gbst_device_cleanup_command();
        assert!(command.contains(GBST_DEVICE_TEMP_DIR));
        assert!(command.contains("rm -rf /data/local/tmp/dwas_kr/GBST"));
        assert!(command.contains("rmdir /data/local/tmp/dwas_kr"));
        assert!(command.contains("__GBST_DEVICE_TEMP_CLEANED__"));
        assert!(!command.contains("rm -rf /data/local/tmp/dwas_kr;"));
        assert!(!command.contains("rm -rf /data/local/tmp/gbst"));
        assert!(!command.contains("/storage/self/primary/Download"));
    }

    #[test]
    fn local_sha256_file_hash_matches_known_digest() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("gbst-adb-sha256-{}-{}", std::process::id(), unique));
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            sha256_of_file(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn cancel_signal_stops_before_the_next_adb_step() {
        let plan = InstallPlan {
            android_major: AndroidMajor::Android16,
            steps: vec![PlanStep::Delay {
                label: "test delay".into(),
                seconds: 1,
                policy: FailurePolicy::Stop,
            }],
        };
        let cancelled = AtomicBool::new(true);
        let mut adb = DirectAdb::new();
        let result = adb.execute_plan_interruptible(&plan, &cancelled, |_| {});
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("중단"));
    }

    #[test]
    fn all_android_versions_reboot_once_at_each_plan_boundary() {
        for label in [
            "사전 준비 재부팅",
            "Google 서비스 복구 루틴 1 완료 후 재부팅",
            "Google 서비스 복구 루틴 2 완료 후 재부팅",
            "Google 서비스 복구 루틴 3 완료 후 재부팅",
            "Google 서비스 복구 루틴 4 완료 후 재부팅",
        ] {
            assert_eq!(reboot_count_for_step(AndroidMajor::Android13, label), 1);
        }
        assert_eq!(
            reboot_count_for_step(
                AndroidMajor::Android14,
                "Google 서비스 복구 루틴 1 완료 후 재부팅"
            ),
            1
        );
    }
}

#[cfg(test)]
mod ltbox_style_progress_tests {
    use super::*;
    #[test]
    fn each_google_stage_emits_one_text_start_while_progress_remains_independent() {
        let mut emitted = [false; 5];
        let mut visible_starts = Vec::new();
        for stage in [1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 4] {
            if mark_google_stage_started(stage, &mut emitted) {
                visible_starts.push(google_stage_progress_log(stage, 0, 10));
            }
        }
        assert_eq!(visible_starts.len(), 4);
        for (index, text) in visible_starts.iter().enumerate() {
            assert!(text.contains(&format!("({}/4)", index + 1)));
        }
        assert!(!mark_google_stage_started(0, &mut emitted));
        assert!(!mark_google_stage_started(4, &mut emitted));
        assert_eq!(google_stage_progress_percent(5, 10), 50);
    }

    #[test]
    fn stage_progress_has_no_textual_unicode_blocks() {
        assert_eq!(google_stage_progress_percent(1, 4), 25);
        assert_eq!(google_stage_progress_percent(4, 4), 100);
        assert_eq!(google_stage_progress_percent(10, 4), 100);
        assert_eq!(google_stage_progress_percent(0, 0), 0);
        let stage = google_stage_progress_log(2, 1, 4);
        let completed = google_stage_complete_log(2);
        assert!(stage.contains("2/4"));
        assert!(completed.contains("2단계 완료"));
        assert!(!stage.contains('█') && !stage.contains('·'));
        assert!(!completed.contains('█') && !completed.contains('·'));
    }
}

#[cfg(test)]
mod gsf_post_install_recovery_tests {
    use super::*;

    fn state(installed_for_user: bool, disabled: bool) -> GooglePackageState {
        GooglePackageState {
            installed_for_user,
            disabled,
            version_code: Some(33),
        }
    }

    #[test]
    fn postflight_only_enables_disabled_packages_for_user_zero() {
        let gsf = "com.google.android.gsf";
        assert_eq!(
            post_install_reenable_command(gsf, &state(true, true)),
            Some("pm enable --user 0 com.google.android.gsf".to_string()),
        );
        assert!(post_install_reenable_command(gsf, &state(true, false)).is_none());
        assert!(post_install_reenable_command(gsf, &state(false, true)).is_none());
    }

    #[test]
    fn reproduces_disabled_user_zero_from_tb128fu_android13() {
        // Captured from the read-only package inspection after the failed run.
        let gsf_line =
            "User 0: installed=true hidden=false suspended=false enabled=2 instant=false";
        let enabled_line =
            "User 0: installed=true hidden=false suspended=false enabled=1 instant=false";
        assert_eq!(parse_enabled_state_from_user_line(gsf_line), Some(2));
        assert_eq!(parse_enabled_state_from_user_line(enabled_line), Some(1));
        assert!(
            post_install_reenable_command(
                "com.google.android.gsf",
                &state(
                    true,
                    parse_enabled_state_from_user_line(gsf_line) == Some(2)
                ),
            )
            .is_some()
        );
    }
}
