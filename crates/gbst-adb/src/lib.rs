use adb_client::usb::{find_all_connected_adb_devices, ADBUSBDevice};
use adb_client::{ADBDeviceExt, RebootType};
use gbst_core::apk_metadata::{
    apk_version_code_candidates_from_dir, apk_version_code_candidates_from_local_apks,
};
use gbst_core::error::{GbstError, Result};
use gbst_core::model::{
    AndroidMajor, DashboardInfo, DeviceInfo, FailurePolicy, InstallPlan, LocalApk, PlanStep,
    GOOGLE_REQUIRED_PACKAGES,
};
use gbst_core::paths;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const ADB_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const ADB_POLL_INTERVAL: Duration = Duration::from_secs(1);
const ADB_CONNECT_RETRY_ATTEMPTS: usize = 5;
const ADB_CONNECT_RETRY_BACKOFF: Duration = Duration::from_millis(400);
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

pub const ADB_UNAUTHORIZED_GUIDE: &str =
    "[안내] PC(노트북)와 연결한 태블릿에 잠금 해제 → 메세지 창 왼쪽 중간 체크 박스 체크 → 오른쪽 하단 Allow(허용)를 터치해주세요.";

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

pub struct DirectAdb {
    device: Option<ADBUSBDevice>,
    serial: Option<String>,
    connected_once: bool,
    cached_bootmode: Option<&'static str>,
}

impl DirectAdb {
    pub fn new() -> Self {
        Self {
            device: None,
            serial: None,
            connected_once: false,
            cached_bootmode: None,
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
                        let _ = kill_adb_server_direct();
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
            Err(_) => {
                self.drop_device();

                if adb_server_running() {
                    Ok(AdbUsbState::ServerBlocking)
                } else {
                    Ok(AdbUsbState::Unauthorized)
                }
            }
        }
    }

    pub fn wait_ready<F>(&mut self, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        prepare_adb_usb_environment();

        on_log(ADB_UNAUTHORIZED_GUIDE.to_string());
        self.wait_ready_until(ADB_WAIT_TIMEOUT, on_log)
    }

    pub fn wait_ready_until<F>(&mut self, timeout: Duration, on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        self.wait_ready_until_with_mode(timeout, on_log, AdbWaitLogMode::Detect)
    }

    fn wait_ready_until_for_reboot<F>(&mut self, timeout: Duration, on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        self.wait_ready_until_with_mode(timeout, on_log, AdbWaitLogMode::Reboot)
    }

    fn wait_ready_until_with_mode<F>(
        &mut self,
        timeout: Duration,
        mut on_log: F,
        mode: AdbWaitLogMode,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        let deadline = Instant::now() + timeout;
        let mut spinner_index = 0usize;
        let mut current_state = "ADB USB 기기 감지 대기 중입니다.".to_string();
        let mut authorization_guide_emitted = mode == AdbWaitLogMode::Detect;

        let spinner_key = match mode {
            AdbWaitLogMode::Detect => "adb_detect",
            AdbWaitLogMode::Reboot => "adb_reboot",
        };

        loop {
            let frame = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
            spinner_index = spinner_index.wrapping_add(1);

            match mode {
                AdbWaitLogMode::Detect => {
                    on_log(spinner_log(spinner_key, &format!("[ADB] 기기 감지 중... {frame}")));
                }
                AdbWaitLogMode::Reboot => {
                    on_log(spinner_log(spinner_key, &format!("[ADB] 기기 재부팅 중... {frame}")));
                }
            }

            if adb_usb_candidate_exists().unwrap_or(false) {
                current_state = match self.check_usb_state() {
                    Ok(AdbUsbState::Ready) | Ok(AdbUsbState::Recovery) => {
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
                            "ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요.".to_string(),
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
                        let _ = kill_adb_server_direct();
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
        let manufacturer = self.shell("getprop ro.product.manufacturer")?.trim().to_string();
        let model = self.read_model_identity();
        let android_version = self.shell("getprop ro.build.version.release")?.trim().to_string();

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
        self.shell_inner(command)
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
            "ro.product.vendor.model",
            "ro.product.model",
            "ro.product.product.model",
            "ro.product.device",
            "ro.product.vendor.device",
            "ro.product.product.device",
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
        Ok(self
            .shell("settings get system screen_off_timeout")?
            .trim()
            .to_string())
    }

    pub fn restore_screen_off_timeout(&mut self, timeout: &str) -> Result<()> {
        let value = timeout.trim();
        if value.is_empty() || value.eq_ignore_ascii_case("null") {
            let _ = self.shell("settings delete system screen_off_timeout")?;
            return Ok(());
        }

        if !value.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(GbstError::Adb(format!(
                "screen_off_timeout 복원 값이 올바르지 않습니다: {value}"
            )));
        }

        let _ = self.shell(&format!("settings put system screen_off_timeout {value}"))?;
        Ok(())
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

    pub fn assess_google_services_from_downloaded_apks(&mut self, apks: &[LocalApk]) -> GoogleServiceAction {
        let local_versions = apk_version_code_candidates_from_local_apks(apks);
        self.assess_google_services_with_versions(&local_versions)
    }

    pub fn assess_google_services_from_apk_dir(&mut self, android_major: AndroidMajor) -> GoogleServiceAction {
        let apk_dir = paths::apk_download_dir(android_major.value());
        let local_versions = apk_version_code_candidates_from_dir(&apk_dir);
        self.assess_google_services_with_versions(&local_versions)
    }

    fn assess_google_services_with_versions(
        &mut self,
        local_versions: &std::collections::BTreeMap<String, Vec<u64>>,
    ) -> GoogleServiceAction {
        let mut update_required = false;

        for package in GOOGLE_REQUIRED_PACKAGES {
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
            .map(|output| output.lines().any(|line| line.trim().starts_with("package:")))
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
                Ok(_) => return Ok(String::from_utf8_lossy(&stdout).trim().to_string()),
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

        Err(GbstError::Adb(last_error.unwrap_or_else(|| {
            format!("ADB shell 실패 `{command}`: 알 수 없는 전송 오류")
        })))
    }

    pub fn install_apk_with_pm(&mut self, apk_path: &Path) -> Result<String> {
        let file_name = apk_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                GbstError::Adb(format!(
                    "APK 파일 이름을 읽을 수 없습니다: {}",
                    apk_path.display()
                ))
            })?;

        let remote_dir = "/data/local/tmp/gbst";
        let remote_path = format!("{remote_dir}/{file_name}");

        self.shell(&format!("mkdir -p {remote_dir}"))?;
        self.push_file_with_retry(apk_path, &remote_path)?;

        let output = self.shell(&format!("pm install -r -d -g --user 0 {remote_path}"));
        let _ = self.shell(&format!("rm -f {remote_path}"));
        output
    }

    pub fn cleanup_gbst_temp_files(&mut self) -> Result<()> {
        let _ = self.shell("rm -rf /data/local/tmp/gbst")?;
        Ok(())
    }

    fn push_file_with_retry(&mut self, apk_path: &Path, remote_path: &str) -> Result<()> {
        let mut last_error: Option<String> = None;

        for attempt in 0..ADB_PUSH_RETRY_ATTEMPTS {
            let mut file = File::open(apk_path)?;
            let result = match self.connect_device() {
                Ok(device) => device
                    .push(&mut file as &mut dyn Read, &remote_path)
                    .map_err(|err| err.to_string()),
                Err(err) => Err(err.to_string()),
            };

            match result {
                Ok(()) => return Ok(()),
                Err(err) => {
                    self.drop_device();
                    last_error = Some(err);
                    if attempt + 1 < ADB_PUSH_RETRY_ATTEMPTS {
                        thread::sleep(ADB_PUSH_RETRY_BACKOFF);
                    }
                }
            }
        }

        Err(GbstError::Adb(format!(
            "APK push 실패: {} -> {remote_path}: {}",
            apk_path.display(),
            last_error.unwrap_or_else(|| "알 수 없는 전송 오류".to_string())
        )))
    }

    pub fn install_apk_candidates_with_pm(
        &mut self,
        package: &str,
        apk_paths: &[PathBuf],
    ) -> Result<String> {
        if apk_paths.is_empty() {
            return Err(GbstError::Adb(format!(
                "설치할 APK 후보가 없습니다: {package}"
            )));
        }

        let mut attempts = Vec::new();
        let mut ambiguous_result = false;

        for apk_path in apk_paths {
            match self.install_apk_with_pm(apk_path) {
                Ok(output) => {
                    if pm_install_explicitly_failed(&output) {
                        attempts.push(format!(
                            "{}: {}",
                            apk_path.display(),
                            compact_pm_output(&output)
                        ));
                        continue;
                    }

                    if pm_install_explicitly_succeeded(&output)
                        || self.wait_for_user_package(package, 3, Duration::from_millis(500))
                    {
                        return Ok(output);
                    }

                    ambiguous_result = true;
                    attempts.push(format!(
                        "{}: 설치 결과를 확인하지 못했습니다",
                        apk_path.display()
                    ));
                }
                Err(err) => {
                    attempts.push(format!("{}: {err}", apk_path.display()));
                }
            }
        }

        if ambiguous_result
            && self.wait_for_user_package(package, 3, Duration::from_millis(500))
        {
            return Ok(String::new());
        }

        let _ = attempts;
        Ok(String::new())
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
            output = self.shell(&format!("appops get {package}")).unwrap_or_default();
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

    fn allow_existing_appops<F>(&mut self, package: &str, ops: &[String], mut on_log: F) -> Result<()>
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

    pub fn execute_plan<F>(&mut self, plan: &InstallPlan, mut on_log: F) -> Result<()>
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

        for step in &plan.steps {
            if let Some(stage) = infer_google_stage(step, google_stage) {
                google_stage = Some(stage);
                let stage_index = stage as usize;
                if stage_index < google_stage_guide_emitted.len()
                    && !google_stage_guide_emitted[stage_index]
                {
                    emit_google_restore_wait_guide(&mut on_log);
                    google_stage_guide_emitted[stage_index] = true;
                }
            }

            let display_label = display_label_for_step(step, google_stage);
            let spinner_key = spinner_key_for_step(step, google_stage);

            if let Some(stage) = google_stage {
                if display_label
                    .as_deref()
                    .is_some_and(|label| label.starts_with("구글 서비스 작업 중"))
                {
                    stage_positions[stage as usize] = stage_positions[stage as usize].saturating_add(1);
                    on_log(google_stage_progress_log(
                        stage,
                        stage_positions[stage as usize],
                        stage_totals[stage as usize],
                    ));
                }
            } else if let Some(label) = display_label.as_deref() {
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

            if let PlanStep::RebootAndWait { .. } = step {
                if let Some(stage) = google_stage {
                    on_log(google_stage_complete_log(stage));
                }
            }

            match step {
                PlanStep::Shell {
                    label,
                    command,
                    policy,
                } => {
                    match self.shell(command) {
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
                PlanStep::InstallApk { path, policy, .. } => {
                    match self.install_apk_with_pm(path) {
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
                    }
                }
                PlanStep::InstallApkCandidates {
                    package,
                    paths,
                    policy,
                } => {
                    match self.install_apk_candidates_with_pm(package, paths) {
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
                PlanStep::Delay { seconds, policy, .. } => {
                    match self.delay_seconds(*seconds, |line| on_log(line)) {
                        Ok(()) => {}
                        Err(err) if *policy == FailurePolicy::Continue => {
                            if should_show_recoverable_error("대기", &err.to_string()) {
                                on_log(format!("    [경고] 대기 실패, 계속 진행: {err}"));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
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
                } => {
                    match self.grant_requested_permissions(package, permissions, |_| {}) {
                        Ok(()) => {}
                        Err(err) if *policy == FailurePolicy::Continue => {
                            if should_show_recoverable_error(package, &err.to_string()) {
                                on_log(format!("    [경고] 권한 적용 실패, 계속 진행: {err}"));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                PlanStep::AppOpsAllowExisting {
                    package,
                    ops,
                    policy,
                    ..
                } => {
                    match self.allow_existing_appops(package, ops, |_| {}) {
                        Ok(()) => {}
                        Err(err) if *policy == FailurePolicy::Continue => {
                            if should_show_recoverable_error(package, &err.to_string()) {
                                on_log(format!("    [경고] AppOps 적용 실패, 계속 진행: {err}"));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                PlanStep::HealthCheck {
                    packages,
                    policy,
                    ..
                } => {
                    for package in packages {
                        match self.shell(&format!("pm path {package}")) {
                            Ok(_) => {}
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

            if should_clear_google_stage(step) {
                google_stage = None;
            }
        }
        Ok(())
    }

    fn delay_seconds<F>(&mut self, seconds: u64, mut _on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        thread::sleep(Duration::from_secs(seconds));
        Ok(())
    }

    fn reboot_and_wait_once<F>(&mut self, mut on_log: F) -> Result<()>
    where
        F: FnMut(String),
    {
        on_log(spinner_log("adb_reboot", "[ADB] 기기 재부팅 중... │"));
        self.reboot_system()?;
        self.wait_ready_until_for_reboot(ADB_WAIT_TIMEOUT, |line| on_log(line))?;
        thread::sleep(Duration::from_secs(3));
        Ok(())
    }

    fn ensure_dashboard_ready(&mut self) -> Result<()> {
        match self.check_usb_state()? {
            AdbUsbState::Ready | AdbUsbState::Recovery => Ok(()),
            AdbUsbState::ServerBlocking => {
                let _ = kill_adb_server_direct();
                thread::sleep(Duration::from_millis(200));

                match self.check_usb_state()? {
                    AdbUsbState::Ready | AdbUsbState::Recovery => Ok(()),
                    AdbUsbState::Unauthorized => Err(GbstError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),
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
                        if !serial.is_empty() {
                            self.serial = Some(serial);
                        }

                        let mut bootmode_stdout = Vec::new();
                        let _ = device.shell_command(
                            &"getprop ro.bootmode",
                            Some(&mut bootmode_stdout as &mut dyn Write),
                            None,
                        );
                        let bootmode = String::from_utf8_lossy(&bootmode_stdout)
                            .trim()
                            .to_string();
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

            return Err(GbstError::Adb(
                last_error.unwrap_or_else(|| "ADB USB 직접 연결 실패".to_string()),
            ));
        }

        Ok(self.device.as_mut().expect("ADB device cached"))
    }

    fn drop_device(&mut self) {
        self.device = None;
        self.serial = None;
        self.cached_bootmode = None;
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
        ("TB710FC", "TB710FU"),
        ("TB522FU", "TB522FU"),
        ("TB522FC", "TB522FU"),
        ("TB520FU", "TB520FU"),
        ("TB520FC", "TB520FU"),
        ("TB378FC", "TB378FC"),
        ("TB378FU", "TB378FC"),
        ("TB376FC", "TB376FC"),
        ("TB376FU", "TB376FC"),
        ("TB375FC", "TB375FC"),
        ("TB375FU", "TB375FC"),
        ("TB373FU", "TB375FC"),
        ("TB373FC", "TB375FC"),
        ("TB371FC", "TB371FC"),
        ("TB371FU", "TB371FC"),
        ("TB365FC", "TB365FC"),
        ("TB365FU", "TB365FC"),
        ("TB361FU", "TB365FC"),
        ("TB361FC", "TB365FC"),
        ("TB335FC", "TB335FC"),
        ("TB335FU", "TB335FC"),
        ("TB336FU", "TB335FC"),
        ("TB336FC", "TB335FC"),
        ("TB331FC", "TB331FC"),
        ("TB331FU", "TB331FC"),
        ("TB323FC", "TB323FC"),
        ("TB323FU", "TB323FC"),
        ("TB322FC", "TB322FC"),
        ("TB322FU", "TB322FC"),
        ("TB321FC", "TB321FC"),
        ("TB321FU", "TB321FC"),
        ("TB320FC", "TB320FC"),
        ("TB320FU", "TB320FC"),
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

impl Default for DirectAdb {
    fn default() -> Self {
        Self::new()
    }
}


pub fn ensure_adb_key() -> Result<PathBuf> {
    let _ = paths::ensure_runtime_directories();

    let stable_path = paths::stable_adb_key_path();
    let lpmbox_path = paths::lpmbox_adb_key_path();

    if stable_path.is_file() {
        return Ok(stable_path);
    }

    if lpmbox_path.is_file() {
        copy_key_pair(&lpmbox_path, &stable_path)?;
        return Ok(stable_path);
    }

    generate_key_pair(&stable_path)?;
    Ok(stable_path)
}

fn generate_key_pair(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        GbstError::Adb(format!("ADB key parent 경로가 없습니다: {}", path.display()))
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
    configure_stable_adb_vendor_keys();
    terminate_adb_fastboot_processes();

    if adb_server_running() {
        let _ = kill_adb_server_direct();
    }
}

fn configure_stable_adb_vendor_keys() {
    let Ok(key_path) = ensure_adb_key() else {
        return;
    };

    let Some(key_dir) = key_path.parent() else {
        return;
    };

    unsafe {
        std::env::set_var("ADB_VENDOR_KEYS", key_dir);
    }
}

fn adb_usb_candidate_exists() -> Result<bool> {
    let devices = find_all_connected_adb_devices()
        .map_err(|err| GbstError::Adb(format!("ADB USB 기기 검색 실패: {err}")))?;

    Ok(!devices.is_empty())
}

pub fn terminate_adb_fastboot_processes() {
    if cfg!(windows) {
        let _ = hidden_command_output("taskkill", &["/F", "/IM", "adb.exe"]);
        let _ = hidden_command_output("taskkill", &["/F", "/IM", "fastboot.exe"]);
    }
}

fn hidden_command_output(program: &str, args: &[&str]) -> Option<Output> {
    let mut command = Command::new(program);
    command.args(args);

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.output().ok()
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
        || (current.is_none() && (label.starts_with("패키지 복구:") || label.starts_with("패키지 활성화:")))
    {
        return Some(4);
    }

    current
}

fn should_clear_google_stage(step: &PlanStep) -> bool {
    match step {
        PlanStep::RebootAndWait { label, .. } => label.contains("루틴") && label.contains("완료 후 재부팅"),
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
    let text = error.trim();
    !(text.is_empty()
        || text.contains("Failure")
        || text.contains("INSTALL_FAILED_")
        || text.contains("ADB shell 실패")
        || text.contains("USB Error")
        || text.contains("Input/Output Error"))
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

fn compact_progress_bar(current: usize, total: usize) -> String {
    let total = total.max(1);
    let current = current.min(total);
    let percent = if current >= total {
        100
    } else {
        ((current as f32 / total as f32) * 100.0).round() as usize
    };
    let total_blocks = 20usize;
    let filled = ((percent.min(100) * total_blocks) + 50) / 100;
    let empty = total_blocks.saturating_sub(filled);

    format!("[{}{}] {}%", "█".repeat(filled), "·".repeat(empty), percent.min(100))
}

fn google_stage_progress_log(stage: u8, current: usize, total: usize) -> String {
    spinner_log(
        &format!("google_restore_{stage}"),
        &format!(
            "[APK] {} 구글 서비스 작업 중... ({stage}/4)",
            compact_progress_bar(current, total),
        ),
    )
}

fn google_stage_complete_log(stage: u8) -> String {
    spinner_log(
        &format!("google_restore_{stage}"),
        &format!(
            "[APK] {} 구글 서비스 작업 {stage}단계 완료",
            compact_progress_bar(1, 1),
        ),
    )
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
    for token in output.split(|ch: char| {
        !(ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+')
    }) {
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

fn reboot_count_for_step(android_major: AndroidMajor, label: &str) -> usize {
    if android_major != AndroidMajor::Android13 {
        return 1;
    }

    match label {
        "사전 준비 재부팅" => 2,
        "Google 서비스 복구 루틴 1 완료 후 재부팅" => 3,
        "Google 서비스 복구 루틴 2 완료 후 재부팅" => 3,
        "Google 서비스 복구 루틴 3 완료 후 재부팅" => 2,
        _ => 1,
    }
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
    fn android13_reboot_counts_match_each_restore_boundary() {
        assert_eq!(reboot_count_for_step(AndroidMajor::Android13, "사전 준비 재부팅"), 2);
        assert_eq!(reboot_count_for_step(AndroidMajor::Android13, "Google 서비스 복구 루틴 1 완료 후 재부팅"), 3);
        assert_eq!(reboot_count_for_step(AndroidMajor::Android13, "Google 서비스 복구 루틴 2 완료 후 재부팅"), 3);
        assert_eq!(reboot_count_for_step(AndroidMajor::Android13, "Google 서비스 복구 루틴 3 완료 후 재부팅"), 2);
        assert_eq!(reboot_count_for_step(AndroidMajor::Android13, "Google 서비스 복구 루틴 4 완료 후 재부팅"), 1);
        assert_eq!(reboot_count_for_step(AndroidMajor::Android14, "Google 서비스 복구 루틴 1 완료 후 재부팅"), 1);
    }
}
