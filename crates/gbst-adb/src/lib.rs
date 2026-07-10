use adb_client::usb::{find_all_connected_adb_devices, ADBUSBDevice};
use adb_client::{ADBDeviceExt, RebootType};
use gbst_core::error::{GbstError, Result};
use gbst_core::model::{
    AndroidMajor, DashboardInfo, DeviceInfo, FailurePolicy, InstallPlan, LocalApk, PlanStep,
};
use gbst_core::paths;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
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
const ADB_CONNECT_RETRY_ATTEMPTS: usize = 3;
const ADB_CONNECT_RETRY_BACKOFF: Duration = Duration::from_millis(150);
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
const CORE_REDACTED_NOTICE: &str = "This code is part of the program's core implementation and has been commented out.";
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
        let model = self
            .shell("getprop ro.product.vendor.model")
            .or_else(|_| self.shell("getprop ro.product.model"))?
            .trim()
            .to_string();
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

        let display_name = self
            .shell("getprop ro.product.display")
            .unwrap_or_default()
            .trim()
            .to_string();
        let device_code = self
            .shell("getprop ro.product.device")
            .unwrap_or_default()
            .trim()
            .to_string();
        let fallback_model = self
            .shell("getprop ro.product.vendor.model")
            .or_else(|_| self.shell("getprop ro.product.model"))
            .unwrap_or_default()
            .trim()
            .to_string();

        info.model_name = if !display_name.is_empty() && !device_code.is_empty() {
            format!("{display_name} ({device_code})")
        } else if !display_name.is_empty() {
            display_name
        } else if !fallback_model.is_empty() {
            fallback_model
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

    pub fn assess_google_services_from_downloaded_apks(&mut self, _apks: &[LocalApk]) -> GoogleServiceAction {
        GoogleServiceAction::Normal
    }

    pub fn assess_google_services_from_apk_dir(&mut self, _android_major: AndroidMajor) -> GoogleServiceAction {
        GoogleServiceAction::Normal
    }

    fn read_google_package_state(&mut self, _package: &str) -> GooglePackageState {
        GooglePackageState {
            installed_for_user: true,
            disabled: false,
            version_code: None,
        }
    }

    fn is_package_path_available(&mut self, _package: &str) -> bool {
        false
    }

    fn package_list_contains(&mut self, _package: &str, _disabled_only: bool) -> bool {
        false
    }

    pub fn install_apk_with_pm(&mut self, _apk_path: &std::path::Path) -> Result<String> {
        Ok(CORE_REDACTED_NOTICE.to_string())
    }

    fn grant_requested_permissions<F>(
        &mut self,
        _package: &str,
        _permissions: &[String],
        mut on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        on_log(CORE_REDACTED_NOTICE.to_string());
        Ok(())
    }

    fn allow_existing_appops<F>(
        &mut self,
        _package: &str,
        _ops: &[String],
        mut on_log: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        on_log(CORE_REDACTED_NOTICE.to_string());
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
        on_log(CORE_REDACTED_NOTICE.to_string());

        for step in &plan.steps {
            match step {
                PlanStep::Delay { seconds, policy, .. } => {
                    match self.delay_seconds(*seconds, |line| on_log(line)) {
                        Ok(()) => {}
                        Err(err) if *policy == FailurePolicy::Continue => {
                            on_log(format!("    [Warning] {err}"));
                        }
                        Err(err) => return Err(err),
                    }
                }
                _ => on_log(CORE_REDACTED_NOTICE.to_string()),
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
                        let _ = device.shell_command(
                            &"getprop ro.serialno",
                            Some(&mut stdout as &mut dyn Write),
                            None,
                        );
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
        self.cached_bootmode = None;
    }
}

impl Default for DirectAdb {
    fn default() -> Self {
        Self::new()
    }
}


pub fn ensure_adb_key() -> Result<PathBuf> {
    let _ = paths::ensure_runtime_directories();

    let stable_path = paths::stable_adb_key_path();
    let runtime_path = paths::runtime_adb_key_path();
    let lpmbox_path = paths::lpmbox_adb_key_path();

    if stable_path.is_file() {
        mirror_key_pair(&stable_path, &runtime_path);
        return Ok(stable_path);
    }

    if runtime_path.is_file() {
        copy_key_pair(&runtime_path, &stable_path)?;
        return Ok(stable_path);
    }

    if lpmbox_path.is_file() {
        copy_key_pair(&lpmbox_path, &stable_path)?;
        mirror_key_pair(&stable_path, &runtime_path);
        return Ok(stable_path);
    }

    generate_key_pair(&stable_path)?;
    mirror_key_pair(&stable_path, &runtime_path);
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

fn mirror_key_pair(from: &Path, to: &Path) {
    if to.is_file() {
        return;
    }

    let _ = copy_key_pair(from, to);
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
    on_log(CORE_REDACTED_NOTICE.to_string());
}

fn display_label_for_step(step: &PlanStep, _stage: Option<u8>) -> Option<String> {
    match step {
        PlanStep::Shell { label, .. }
        | PlanStep::Delay { label, .. }
        | PlanStep::RebootAndWait { label, .. }
        | PlanStep::GrantRequestedPermissions { label, .. }
        | PlanStep::AppOpsAllowExisting { label, .. }
        | PlanStep::HealthCheck { label, .. } => Some(label.clone()),
        PlanStep::InstallApk { .. } => Some(CORE_REDACTED_NOTICE.to_string()),
    }
}

fn spinner_key_for_step(_step: &PlanStep, _stage: Option<u8>) -> Option<String> {
    Some("redacted_workflow".to_string())
}

fn infer_google_stage(_step: &PlanStep, current: Option<u8>) -> Option<u8> {
    current
}

fn should_clear_google_stage(_step: &PlanStep) -> bool {
    false
}

fn should_show_command_output(_label: &str, _output: &str) -> bool {
    false
}

fn should_show_recoverable_error(_label: &str, error: &str) -> bool {
    !error.trim().is_empty()
}

fn google_stage_tupdatels(_steps: &[PlanStep]) -> [usize; 5] {
    [1, 1, 1, 1, 1]
}

fn google_stage_progress_log(stage: u8, _current: usize, _tupdatel: usize) -> String {
    spinner_log(
        &format!("redacted_workflow_{stage}"),
        CORE_REDACTED_NOTICE,
    )
}

fn google_stage_complete_log(stage: u8) -> String {
    spinner_log(
        &format!("redacted_workflow_{stage}"),
        CORE_REDACTED_NOTICE,
    )
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
