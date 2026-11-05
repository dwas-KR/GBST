// SPDX-License-Identifier: GPL-3.0-only
#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod appearance;
mod cache_cleanup;
mod log_cleanup;
use appearance::{AccentChoice, Appearance, ThemeChoice, system_prefers_dark};
use chrono::Local;
use gbst_adb::{AdbUsbState, DirectAdb, GoogleServiceAction};
use gbst_core::apk_catalog::{ApkCatalog, REMOTE_APK_CATALOG_URL};
use gbst_core::downloader::download_apks_for_android;
use gbst_core::language::{
    LanguageOption, completion_popup_text, detect_initial_language, save_language,
    translate_runtime_text,
};
use gbst_core::log_i18n::{log_label, log_label_with_args, translate_log};
use gbst_core::model::{DashboardInfo, DeviceInfo};
use gbst_core::paths;
use gbst_core::plan::build_google_basic_service_plan;
use iced::widget::{
    Space, button, column, container, pick_list, progress_bar, row, scrollable, text,
};
use iced::{Background, Color, Element, Font, Length, Size, Subscription, Task, Theme, window};
use image as image_crate;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const WINDOW_WIDTH: f32 = 820.0;
const WINDOW_HEIGHT: f32 = 720.0;
const APP_DISPLAY_VERSION: &str = env!("CARGO_PKG_VERSION");

const BODY_FONT: u32 = 15;
const LOG_FONT: u32 = 12;
const LPM_FONT_FAMILY: &str = "Malgun Gothic";

const APP_ICON_PNG_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const NAV_HOME_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_home.png");
const NAV_G_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_G.png");
const NAV_TAB_SETTINGS_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_tab_settings.png");
const NAV_SETTINGS_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_settings.png");
const NAV_LOG_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_log.png");
const TABLET_CHECK_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_check.png");
const TABLET_X_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_x.png");
const TABLET_FIX_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_fix.png");
const WARNING_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/waring.png");
const MODEL_LENOVO_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/Lenovo.png");
const MODEL_ETC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/Etc.png");
const MODEL_TB_9707F_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB-9707F.png");
const MODEL_TB320FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB320FC.png");
const MODEL_TB321FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB321FC.png");
const MODEL_TB322FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB322FC.png");
const MODEL_TB323FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB323FC.png");
const MODEL_TB324ZC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB324ZC.png");
const MODEL_TB331FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB331FC.png");
const MODEL_TB335FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB335FC.png");
const MODEL_TB365FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB365FC.png");
const MODEL_TB371FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB371FC.png");
const MODEL_TB375FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB375FC.png");
const MODEL_TB376FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB376FC.png");
const MODEL_TB378FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB378FC.png");
const MODEL_TB520FU_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB520FU.png");
const MODEL_TB522FU_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB522FU.png");
const MODEL_TB710FU_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB710FU.png");

// Match LTBox v3.3.5: use the compact 80px rail to classify width.
// Never subtract the expanding drawer width when choosing the class.
const SIDEBAR_RAIL_WIDTH: f32 = 80.0;
const SIDEBAR_EXPANDED_WIDTH: f32 = 232.0;
const EXPANDED_CONTENT_WIDTH: f32 = 1000.0;

const NAV_BTN_HEIGHT: f32 = 56.0;
const SIDEBAR_ANIM_INTERVAL: Duration = Duration::from_millis(16);
const LIVE_EVENT_DRAIN_INTERVAL: Duration = Duration::from_millis(250);
const DASHBOARD_REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const APK_DOWNLOAD_NOTICE_DISMISS_DELAY: Duration = Duration::from_secs(2);
const SIDEBAR_ANIM_FACTOR: f32 = 0.15;
const SIDEBAR_ANIM_THRESHOLD: f32 = 0.004;

const DEVELOPER_YOUTUBE_URL: &str = "https://www.youtube.com/@dwas_KR?sub_confirmation=1";
const GBST_RELEASES_URL: &str = "https://github.com/dwas-KR/GBST/releases";
const GBST_APK_RELEASE_URL: &str =
    "https://github.com/dwas-KR/GBST-APK/releases/tag/apk-2026.08.29";
const GBST_RELEASES_API_URL: &str =
    "https://api.github.com/repos/dwas-KR/GBST/releases?per_page=20";
const LTBOX_GITHUB_URL: &str = "https://github.com/miner7222/LTBox";
const DONATE_URL: &str = "https://www.youtube.com/@dwas_KR/join";
const FEEDBACK_KOREAN_URL: &str = "https://github.com/dwas-KR/GBST/issues/1";
const FEEDBACK_GLOBAL_URL: &str = "https://github.com/dwas-KR/GBST/issues/3";

fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    {
        let backend_chosen =
            std::env::var_os("WGPU_BACKEND").is_some_and(|value| !value.is_empty());
        if !backend_chosen {
            unsafe {
                std::env::set_var("WGPU_BACKEND", "dx12");
            }
        }
    }

    let _ = paths::ensure_runtime_directories();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("GBST - Google Basic Service Tool")
        .theme(App::theme)
        .default_font(lpm_font())
        .window(window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: true,
            min_size: Some(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            icon: lpm_window_icon(),
            ..window::Settings::default()
        })
        .centered()
        .run()
}

fn gui_disclaimer_file() -> std::path::PathBuf {
    paths::config_root().join("gui_disclaimer.json")
}
fn gui_disclaimer_accepted() -> bool {
    let Ok(text) = fs::read_to_string(gui_disclaimer_file()) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|obj| obj.get("accepted").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}
fn gui_disclaimer_save_accepted() -> std::io::Result<()> {
    let path = gui_disclaimer_file();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?
    }
    fs::write(path, br#"{"accepted":true}"#)
}

fn lpm_window_icon() -> Option<window::Icon> {
    window::icon::from_file_data(APP_ICON_PNG_BYTES, Some(image_crate::ImageFormat::Png)).ok()
}

fn lpm_font() -> Font {
    Font::with_name(LPM_FONT_FAMILY)
}

fn lpm_bold_font() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::with_name(LPM_FONT_FAMILY)
    }
}

fn feedback_url_for_language(language: LanguageOption) -> &'static str {
    if matches!(language, LanguageOption::Korean) {
        FEEDBACK_KOREAN_URL
    } else {
        FEEDBACK_GLOBAL_URL
    }
}

fn ui_text(language: LanguageOption, content: &str) -> String {
    let translated = translate_runtime_text(language, content);
    if language.is_rtl() && !translated.trim().is_empty() {
        format!("\u{200E}{translated}\u{200E}")
    } else {
        translated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavPage {
    Dashboard,
    Google,
    Log,
    Settings,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashboardModelImage {
    Etc,
    Lenovo,
    Tb9707F,
    Tb320Fc,
    Tb321Fc,
    Tb322Fc,
    Tb323Fc,
    Tb324Zc,
    Tb331Fc,
    Tb335Fc,
    Tb365Fc,
    Tb371Fc,
    Tb375Fc,
    Tb376Fc,
    Tb378Fc,
    Tb520Fu,
    Tb522Fu,
    Tb710Fu,
}

const LPM_NAV_MAIN: &[(NavPage, &'static [u8], &str)] = &[
    (NavPage::Dashboard, NAV_HOME_ICON_BYTES, "대시보드"),
    (NavPage::Google, NAV_G_ICON_BYTES, "Google 서비스"),
];

const LPM_NAV_TOOLS: &[(NavPage, &'static [u8], &str)] = &[
    (NavPage::Log, NAV_LOG_ICON_BYTES, "로그 관리"),
    (NavPage::Settings, NAV_SETTINGS_ICON_BYTES, "설정"),
    (NavPage::About, NAV_TAB_SETTINGS_ICON_BYTES, "정보"),
];

#[derive(Debug, Clone)]
enum Message {
    Noop,
    StartupPrepareApks,
    SelectNav(NavPage),
    GoogleStartConfirmed,
    GoogleStartDeclined,
    SidebarHoverEnter,
    SidebarHoverExit,
    WindowResized(f32, f32),
    SidebarAnimTick,
    DashboardRefreshTick,
    ApkDownloadNoticeTick,
    DashboardAutoLoaded(Result<DashboardInfo, String>),
    DetectDevice,
    StartInstall,
    CancelInstall,
    DrainWorker,
    OpenApkLinks,
    OpenApkFolder,
    CheckUsbStatus,
    UsbStatusChecked(Result<String, String>),
    KillAdbServer,
    AdbServerKillFinished(Result<(), String>),
    OpenDiagnosticsFolder,
    OpenLogFolder,
    RequestCacheCleanup,
    ConfirmCacheCleanup,
    CancelCacheCleanup,
    CacheCleanupFinished(Result<cache_cleanup::CleanupReport, String>),
    RequestLogCleanup,
    ConfirmLogCleanup,
    CancelLogCleanup,
    LogCleanupFinished(Result<log_cleanup::LogCleanupReport, String>),
    OpenGitHubRepository,
    OpenLTBoxGitHub,
    OpenGBSTIssues,
    ShowLicenseDialog,
    CloseLicenseDialog,
    OpenLicense,
    OpenYoutube,
    OpenDonate,
    OpenCompletionDonate,
    DismissCompletionNotice,
    OpenFeedback,
    CheckProgramUpdate,
    ProgramUpdateChecked(Result<ProgramUpdateCheckResult, String>),
    DashboardProgramUpdateChecked(Result<ProgramUpdateCheckResult, String>),
    OpenProgramUpdateRelease,
    DismissProgramUpdateNotice,
    ClearLog,
    ExportLog,
    SettingsLanguageSelected(LanguageOption),
    SettingsThemeSelected(ThemeChoice),
    SettingsAccentSelected(AccentChoice),
    WizardNext,
    WizardBack,
    WizardConfirmed(bool),
    WizardRestart,
    OperationElapsedTick,
    DisclaimerToggled(bool),
    DisclaimerContinue,
    DisclaimerExit,
}

fn google_confirmation_required(disclaimer_open: bool, already_running: bool) -> bool {
    !disclaimer_open && !already_running
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoogleNavigationAction {
    Start,
    Queue,
    Ignore,
}

fn google_navigation_action(
    disclaimer_open: bool,
    install_running: bool,
    other_worker_busy: bool,
) -> GoogleNavigationAction {
    if disclaimer_open || install_running {
        GoogleNavigationAction::Ignore
    } else if other_worker_busy {
        GoogleNavigationAction::Queue
    } else {
        GoogleNavigationAction::Start
    }
}

#[derive(Debug)]
enum WorkerEvent {
    Log(String),
    Progress(ProgressSignal),
    DashboardLoaded(Result<DashboardInfo, String>),
    DeviceProbeFinished,
    StartupPrepared(Result<DashboardInfo, String>),
    ApkDownloadCompleted,
    InstallFinished(Result<DashboardInfo, String>),
    Finished(Result<(), String>),
}

#[derive(Debug)]
enum ProgressSignal {
    Status {
        percent: f32,
        key: String,
    },
    Plan {
        completed: usize,
        total: usize,
        within: u8,
        stage: u8,
    },
    Download {
        percent: u8,
    },
    Stage {
        stage: u8,
        percent: u8,
    },
    Upload {
        package: String,
        percent: u8,
    },
}

fn classify_worker_message(line: String) -> WorkerEvent {
    if let Some((package, percent)) = line
        .strip_prefix("[APK] ")
        .and_then(|text| text.split_once(": 기기 전송 진행률 "))
        && let Some(percent) = percent.strip_suffix('%')
        && let Ok(percent) = percent.parse::<u8>()
        && percent <= 100
    {
        return WorkerEvent::Progress(ProgressSignal::Upload {
            package: package.to_string(),
            percent,
        });
    }
    if let Some(percent) = line
        .strip_prefix("__GBST_DOWNLOAD_PROGRESS__|")
        .and_then(|number| number.parse::<u8>().ok())
        && percent <= 100
    {
        return WorkerEvent::Progress(ProgressSignal::Download { percent });
    }
    if let Some((stage, percent)) = line
        .strip_prefix("__GBST_STAGE_PROGRESS__|")
        .and_then(|rest| rest.split_once('|'))
        && let (Ok(stage), Ok(percent)) = (stage.parse::<u8>(), percent.parse::<u8>())
        && (1..=4).contains(&stage)
        && percent <= 100
    {
        return WorkerEvent::Progress(ProgressSignal::Stage { stage, percent });
    }
    if let Some(rest) = line.strip_prefix("__GBST_PROGRESS__|")
        && let Some((percent, key)) = rest.split_once('|')
        && let Ok(percent) = percent.parse::<f32>()
        && percent.is_finite()
    {
        return WorkerEvent::Progress(ProgressSignal::Status {
            percent,
            key: key.to_string(),
        });
    }
    if let Some(rest) = line.strip_prefix("__GBST_PLAN_PROGRESS__|") {
        let fields: Vec<_> = rest.split('|').collect();
        if fields.len() == 4
            && let (Ok(completed), Ok(total), Ok(within), Ok(stage)) = (
                fields[0].parse::<usize>(),
                fields[1].parse::<usize>(),
                fields[2].parse::<u8>(),
                fields[3].parse::<u8>(),
            )
        {
            return WorkerEvent::Progress(ProgressSignal::Plan {
                completed,
                total,
                within,
                stage,
            });
        }
    }
    WorkerEvent::Log(line)
}

#[derive(Debug, Clone)]
struct LogLine {
    spinner_key: Option<String>,
    message: String,
}

#[derive(Debug, Clone)]
struct ProgramUpdateCheckResult {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    asset_name: Option<String>,
}

struct App {
    active_nav: NavPage,
    sidebar_expanded: bool,
    sidebar_anim: f32,
    sidebar_velocity: f32,
    sidebar_label_alpha: f32,
    window_size: (f32, f32),

    language: LanguageOption,
    appearance: Appearance,
    system_dark: bool,
    wizard_step: u8,
    wizard_confirmed: bool,
    pending_google_start: bool,
    google_start_confirmation_open: bool,
    license_dialog_open: bool,
    cache_cleanup_confirmation_open: bool,
    cache_cleanup_running: bool,
    cache_cleanup_report: Option<cache_cleanup::CleanupReport>,
    log_cleanup_confirmation_open: bool,
    log_cleanup_running: bool,
    log_cleanup_report: Option<log_cleanup::LogCleanupReport>,
    operation_started_at: Option<Instant>,
    operation_finalizing: bool,
    disclaimer_open: bool,
    disclaimer_checked: bool,
    busy: bool,
    dashboard_refreshing: bool,
    adb_server_blocking: bool,
    adb_server_kill_in_flight: bool,
    adb_server_kill_error: Option<String>,
    worker_rx: Option<Receiver<WorkerEvent>>,
    install_cancel: Option<Arc<AtomicBool>>,
    device: Option<DeviceInfo>,
    dashboard_info: DashboardInfo,
    apk_download_modal_message: Option<String>,
    apk_download_modal_completed_at: Option<Instant>,
    apk_download_percent: Option<u8>,
    active_stage_percent: Option<u8>,
    log_lines: Vec<LogLine>,
    log_text_cache: String,
    log_cache_dirty: bool,
    plan_completed: usize,
    plan_total: usize,
    overall_progress: f32,
    progress_visible: bool,
    progress_status_key: String,
    progress_stage: u8,
    transfer_percent: Option<u8>,
    transfer_package: Option<String>,
    usb_authorization_pending: bool,
    program_update_checking: bool,
    dashboard_update_notice: Option<ProgramUpdateCheckResult>,
    completion_notice_visible: bool,

    about_logo_handle: iced::widget::image::Handle,
    nav_home_handle: iced::widget::image::Handle,
    nav_g_handle: iced::widget::image::Handle,
    nav_tab_settings_handle: iced::widget::image::Handle,
    nav_log_handle: iced::widget::image::Handle,
    nav_settings_handle: iced::widget::image::Handle,
    nav_home_dark_handle: iced::widget::image::Handle,
    nav_g_dark_handle: iced::widget::image::Handle,
    nav_tab_settings_dark_handle: iced::widget::image::Handle,
    nav_log_dark_handle: iced::widget::image::Handle,
    nav_settings_dark_handle: iced::widget::image::Handle,
    tablet_check_icon_handle: iced::widget::image::Handle,
    tablet_x_icon_handle: iced::widget::image::Handle,
    tablet_fix_icon_handle: iced::widget::image::Handle,
    warning_icon_handle: iced::widget::image::Handle,
    model_lenovo_handle: iced::widget::image::Handle,
    model_etc_handle: iced::widget::image::Handle,
    model_tb_9707f_handle: iced::widget::image::Handle,
    model_tb320fc_handle: iced::widget::image::Handle,
    model_tb321fc_handle: iced::widget::image::Handle,
    model_tb322fc_handle: iced::widget::image::Handle,
    model_tb323fc_handle: iced::widget::image::Handle,
    model_tb324zc_handle: iced::widget::image::Handle,
    model_tb331fc_handle: iced::widget::image::Handle,
    model_tb335fc_handle: iced::widget::image::Handle,
    model_tb365fc_handle: iced::widget::image::Handle,
    model_tb371fc_handle: iced::widget::image::Handle,
    model_tb375fc_handle: iced::widget::image::Handle,
    model_tb376fc_handle: iced::widget::image::Handle,
    model_tb378fc_handle: iced::widget::image::Handle,
    model_tb520fu_handle: iced::widget::image::Handle,
    model_tb522fu_handle: iced::widget::image::Handle,
    model_tb710fu_handle: iced::widget::image::Handle,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let language = detect_initial_language();
        let mut app = Self {
            active_nav: NavPage::Dashboard,
            sidebar_expanded: false,
            sidebar_anim: 0.0,
            sidebar_velocity: 0.0,
            sidebar_label_alpha: 0.0,
            window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            language,
            appearance: Appearance::load(),
            system_dark: system_prefers_dark(),
            wizard_step: 0,
            wizard_confirmed: false,
            pending_google_start: false,
            google_start_confirmation_open: false,
            license_dialog_open: false,
            cache_cleanup_confirmation_open: false,
            cache_cleanup_running: false,
            cache_cleanup_report: None,
            log_cleanup_confirmation_open: false,
            log_cleanup_running: false,
            log_cleanup_report: None,
            operation_started_at: None,
            operation_finalizing: false,
            disclaimer_open: !gui_disclaimer_accepted(),
            disclaimer_checked: false,
            busy: false,
            dashboard_refreshing: false,
            adb_server_blocking: false,
            adb_server_kill_in_flight: false,
            adb_server_kill_error: None,
            worker_rx: None,
            install_cancel: None,
            device: None,
            dashboard_info: DashboardInfo::unknown(),
            apk_download_modal_message: None,
            apk_download_modal_completed_at: None,
            apk_download_percent: None,
            active_stage_percent: None,
            log_lines: Vec::new(),
            log_text_cache: String::new(),
            log_cache_dirty: false,
            plan_completed: 0,
            plan_total: 0,
            overall_progress: 0.0,
            progress_visible: false,
            progress_status_key: "status.preparing".to_string(),
            progress_stage: 0,
            transfer_percent: None,
            transfer_package: None,
            usb_authorization_pending: false,
            program_update_checking: false,
            dashboard_update_notice: None,
            completion_notice_visible: false,
            about_logo_handle: iced::widget::image::Handle::from_bytes(APP_ICON_PNG_BYTES.to_vec()),
            nav_home_handle: smooth_png_handle(NAV_HOME_ICON_BYTES, 21, 21),
            nav_g_handle: smooth_png_handle(NAV_G_ICON_BYTES, 21, 21),
            nav_tab_settings_handle: smooth_png_handle(NAV_TAB_SETTINGS_ICON_BYTES, 21, 21),
            nav_log_handle: smooth_png_handle(NAV_LOG_ICON_BYTES, 21, 21),
            nav_settings_handle: smooth_png_handle(NAV_SETTINGS_ICON_BYTES, 21, 21),
            nav_home_dark_handle: light_nav_png_handle(NAV_HOME_ICON_BYTES, 21, 21),
            nav_g_dark_handle: light_nav_png_handle(NAV_G_ICON_BYTES, 21, 21),
            nav_tab_settings_dark_handle: light_nav_png_handle(NAV_TAB_SETTINGS_ICON_BYTES, 21, 21),
            nav_log_dark_handle: light_nav_png_handle(NAV_LOG_ICON_BYTES, 21, 21),
            nav_settings_dark_handle: light_nav_png_handle(NAV_SETTINGS_ICON_BYTES, 21, 21),
            tablet_check_icon_handle: smooth_png_handle(TABLET_CHECK_ICON_BYTES, 76, 76),
            tablet_x_icon_handle: smooth_png_handle(TABLET_X_ICON_BYTES, 76, 76),
            tablet_fix_icon_handle: smooth_png_handle(TABLET_FIX_ICON_BYTES, 76, 76),
            warning_icon_handle: smooth_png_handle(WARNING_ICON_BYTES, 76, 76),
            model_lenovo_handle: smooth_png_handle(MODEL_LENOVO_IMAGE_BYTES, 180, 108),
            model_etc_handle: smooth_png_handle(MODEL_ETC_IMAGE_BYTES, 180, 108),
            model_tb_9707f_handle: smooth_png_handle(MODEL_TB_9707F_IMAGE_BYTES, 180, 108),
            model_tb320fc_handle: smooth_png_handle(MODEL_TB320FC_IMAGE_BYTES, 180, 108),
            model_tb321fc_handle: smooth_png_handle(MODEL_TB321FC_IMAGE_BYTES, 180, 108),
            model_tb322fc_handle: smooth_png_handle(MODEL_TB322FC_IMAGE_BYTES, 180, 108),
            model_tb323fc_handle: smooth_png_handle(MODEL_TB323FC_IMAGE_BYTES, 180, 108),
            model_tb324zc_handle: smooth_png_handle(MODEL_TB324ZC_IMAGE_BYTES, 180, 108),
            model_tb331fc_handle: smooth_png_handle(MODEL_TB331FC_IMAGE_BYTES, 180, 108),
            model_tb335fc_handle: smooth_png_handle(MODEL_TB335FC_IMAGE_BYTES, 180, 108),
            model_tb365fc_handle: smooth_png_handle(MODEL_TB365FC_IMAGE_BYTES, 180, 108),
            model_tb371fc_handle: smooth_png_handle(MODEL_TB371FC_IMAGE_BYTES, 180, 108),
            model_tb375fc_handle: smooth_png_handle(MODEL_TB375FC_IMAGE_BYTES, 180, 108),
            model_tb376fc_handle: smooth_png_handle(MODEL_TB376FC_IMAGE_BYTES, 180, 108),
            model_tb378fc_handle: smooth_png_handle(MODEL_TB378FC_IMAGE_BYTES, 180, 108),
            model_tb520fu_handle: smooth_png_handle(MODEL_TB520FU_IMAGE_BYTES, 180, 108),
            model_tb522fu_handle: smooth_png_handle(MODEL_TB522FU_IMAGE_BYTES, 180, 108),
            model_tb710fu_handle: smooth_png_handle(MODEL_TB710FU_IMAGE_BYTES, 180, 108),
        };

        app.push_log(format!(
            "[GBST] GUI 초기화 완료 / 언어 설정: {}",
            app.language.display_name()
        ));

        thread::spawn(|| {
            DirectAdb::startup_authorization_probe_three_times();
        });

        (
            app,
            Task::batch([
                Task::perform(
                    check_program_update_worker(APP_DISPLAY_VERSION.to_string()),
                    Message::DashboardProgramUpdateChecked,
                ),
                Task::perform(async {}, |_| Message::StartupPrepareApks),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Noop => Task::none(),
            Message::DisclaimerToggled(checked) => {
                if self.disclaimer_open {
                    self.disclaimer_checked = checked;
                }
                Task::none()
            }
            Message::DisclaimerContinue => {
                if disclaimer_can_continue(self.disclaimer_open, self.disclaimer_checked) {
                    if let Err(err) = gui_disclaimer_save_accepted() {
                        self.push_log(format!("[GBST] 면책 조항 확인 저장 실패: {err}"));
                    }
                    self.disclaimer_open = false;
                }
                Task::none()
            }
            Message::DisclaimerExit => {
                if self.disclaimer_open {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::StartupPrepareApks => {
                if !self.busy
                    && self.worker_rx.is_none()
                    && !self.cache_cleanup_running
                    && !self.log_cleanup_running
                {
                    self.start_startup_prepare_worker();
                }
                Task::none()
            }
            Message::SelectNav(page) => {
                if page == NavPage::Google {
                    if self.cache_cleanup_running || self.log_cleanup_running {
                        return Task::none();
                    }
                    // A click opens an explicit yes/no confirmation. Reopening
                    // the current in-progress job never starts it again.
                    if self.install_cancel.is_some() {
                        self.active_nav = page;
                        return Task::none();
                    }
                    if google_confirmation_required(
                        self.disclaimer_open,
                        self.install_cancel.is_some(),
                    ) {
                        self.google_start_confirmation_open = true;
                    }
                    return Task::none();
                }
                self.google_start_confirmation_open = false;
                self.active_nav = page;
                self.pending_google_start = false;
                if page == NavPage::Dashboard
                    && !self.busy
                    && !self.dashboard_refreshing
                    && self.worker_rx.is_none()
                    && !self.cache_cleanup_running
                    && !self.log_cleanup_running
                {
                    self.dashboard_refreshing = true;
                    return Task::perform(
                        load_dashboard_info_worker(),
                        Message::DashboardAutoLoaded,
                    );
                }
                Task::none()
            }
            Message::GoogleStartConfirmed => {
                if self.cache_cleanup_running
                    || self.log_cleanup_running
                    || !self.google_start_confirmation_open
                {
                    return Task::none();
                }
                self.google_start_confirmation_open = false;
                self.active_nav = NavPage::Google;
                // Keep existing startup/USB worker serialization unchanged.
                match google_navigation_action(
                    self.disclaimer_open,
                    self.install_cancel.is_some(),
                    self.busy || self.worker_rx.is_some() || self.dashboard_refreshing,
                ) {
                    GoogleNavigationAction::Start => {
                        self.pending_google_start = false;
                        self.start_install_worker();
                    }
                    GoogleNavigationAction::Queue => {
                        self.pending_google_start = true;
                    }
                    GoogleNavigationAction::Ignore => {}
                }
                Task::none()
            }
            Message::GoogleStartDeclined => {
                self.google_start_confirmation_open = false;
                self.pending_google_start = false;
                Task::none()
            }
            Message::SidebarHoverEnter => {
                if self.lt_is_compact() {
                    self.sidebar_expanded = true;
                }
                Task::none()
            }
            Message::SidebarHoverExit => {
                self.sidebar_expanded = false;
                Task::none()
            }
            Message::WindowResized(width, height) => {
                if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
                    let previously_compact = self.lt_is_compact();
                    self.window_size = (width, height);
                    if previously_compact != self.lt_is_compact() {
                        self.sidebar_expanded = false;
                        self.sidebar_anim = 0.0;
                        self.sidebar_velocity = 0.0;
                        self.sidebar_label_alpha = 0.0;
                    }
                }
                Task::none()
            }
            Message::SidebarAnimTick => {
                let target = self.sidebar_anim_target();
                self.sidebar_anim = lpm_approach_anim_value(
                    self.sidebar_anim,
                    target,
                    SIDEBAR_ANIM_FACTOR,
                    SIDEBAR_ANIM_THRESHOLD,
                );
                self.sidebar_velocity = target - self.sidebar_anim;
                self.sidebar_label_alpha = lpm_approach_anim_value(
                    self.sidebar_label_alpha,
                    target,
                    0.22,
                    SIDEBAR_ANIM_THRESHOLD,
                );
                Task::none()
            }
            Message::DashboardRefreshTick => {
                if self.active_nav != NavPage::Dashboard
                    || self.busy
                    || self.dashboard_refreshing
                    || self.worker_rx.is_some()
                    || self.cache_cleanup_running
                    || self.log_cleanup_running
                {
                    return Task::none();
                }

                self.dashboard_refreshing = true;
                Task::perform(load_dashboard_info_worker(), Message::DashboardAutoLoaded)
            }
            Message::ApkDownloadNoticeTick => {
                self.dismiss_completed_apk_download_notice_if_ready();
                Task::none()
            }
            Message::DashboardAutoLoaded(result) => {
                self.dashboard_refreshing = false;
                match result {
                    Ok(info) => {
                        self.adb_server_blocking = false;
                        if self.dashboard_info.model_name != info.model_name {
                            self.wizard_confirmed = false;
                            if !self.busy {
                                self.wizard_step = 0;
                            }
                        }
                        self.dashboard_info = info;
                    }
                    Err(err) => {
                        self.adb_server_blocking = is_adb_server_blocked_error(&err);
                        self.dashboard_info = DashboardInfo::unknown();
                        self.wizard_confirmed = false;
                        if !self.busy {
                            self.wizard_step = 0;
                        }
                    }
                }
                self.try_start_pending_google();
                Task::none()
            }
            Message::DetectDevice => {
                if !self.busy
                    && !self.dashboard_refreshing
                    && self.worker_rx.is_none()
                    && !self.cache_cleanup_running
                    && !self.log_cleanup_running
                {
                    self.start_detect_worker();
                }
                Task::none()
            }
            Message::StartInstall => {
                if !self.busy
                    && !self.dashboard_refreshing
                    && self.worker_rx.is_none()
                    && self.wizard_step == 3
                    && self.wizard_confirmed
                    && self.dashboard_info.is_lenovo
                    && self.dashboard_info.android_major.is_some()
                {
                    self.start_install_worker();
                }
                Task::none()
            }
            Message::WizardNext => {
                if wizard_can_advance(
                    self.wizard_step,
                    self.lt_device_connected() && !self.dashboard_refreshing,
                    self.wizard_confirmed,
                    self.busy,
                ) {
                    self.wizard_step += 1;
                }
                Task::none()
            }
            Message::WizardBack => {
                if !self.busy && self.wizard_step > 0 {
                    self.wizard_step -= 1;
                }
                Task::none()
            }
            Message::WizardConfirmed(checked) => {
                if !self.busy {
                    self.wizard_confirmed = checked;
                }
                Task::none()
            }
            Message::WizardRestart => {
                if !self.busy {
                    self.wizard_step = 0;
                    self.wizard_confirmed = false;
                    self.progress_visible = false;
                }
                Task::none()
            }
            Message::CancelInstall => {
                if let Some(flag) = &self.install_cancel {
                    flag.store(true, Ordering::Relaxed);
                    self.push_log("[GBST] 취소를 요청했습니다. 현재 실행 중인 ADB 명령/재부팅 이후 안전한 단계에서 중지됩니다.");
                }
                Task::none()
            }
            Message::OperationElapsedTick => Task::none(),
            Message::DrainWorker => {
                self.drain_worker_events();
                Task::none()
            }
            Message::OpenApkLinks => {
                if let Err(err) = open::that(GBST_APK_RELEASE_URL) {
                    self.push_log(format!("[APK] GitHub APK 링크 파일 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenApkFolder => {
                let dir = self
                    .dashboard_info
                    .android_major
                    .map(|android| paths::apk_download_dir(android.value()))
                    .unwrap_or_else(paths::apk_cache_dir);
                if let Err(err) = fs::create_dir_all(&dir) {
                    self.push_log(format!("[APK] APK 폴더 생성 실패: {err}"));
                } else if let Err(err) = open::that(&dir) {
                    self.push_log(format!("[APK] APK 폴더 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::KillAdbServer => {
                if !adb_server_kill_allowed(
                    self.adb_server_blocking,
                    self.busy
                        || self.worker_rx.is_some()
                        || self.dashboard_refreshing
                        || self.install_cancel.is_some()
                        || self.cache_cleanup_running
                        || self.log_cleanup_running,
                    self.adb_server_kill_in_flight,
                    self.disclaimer_open,
                ) {
                    return Task::none();
                }
                self.adb_server_kill_in_flight = true;
                self.adb_server_kill_error = None;
                self.push_log("[ADB] 외부 ADB 서버 종료를 요청했습니다.");
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(gbst_adb::kill_adb_server_direct)
                            .await
                            .map_err(|error| format!("백그라운드 작업 실패: {error}"))
                            .and_then(|result| result.map_err(|error| error.to_string()))
                    },
                    Message::AdbServerKillFinished,
                )
            }
            Message::AdbServerKillFinished(result) => {
                self.adb_server_kill_in_flight = false;
                match result {
                    Ok(()) => {
                        self.adb_server_blocking = false;
                        self.adb_server_kill_error = None;
                        self.push_log("[ADB] 외부 ADB 서버 종료 완료. USB 연결을 다시 검사합니다.");
                        if !self.busy && self.worker_rx.is_none() && !self.dashboard_refreshing {
                            self.dashboard_refreshing = true;
                            return Task::perform(
                                load_dashboard_info_worker(),
                                Message::DashboardAutoLoaded,
                            );
                        }
                    }
                    Err(error) => {
                        self.adb_server_kill_error = Some(error.clone());
                        self.push_log(format!("[ADB] 외부 ADB 서버 종료 실패: {error}"));
                    }
                }
                Task::none()
            }
            Message::CheckUsbStatus => {
                if self.busy || self.cache_cleanup_running || self.log_cleanup_running {
                    self.push_log("[USB] Google 작업 중에는 USB 진단을 실행할 수 없습니다.");
                    return Task::none();
                }
                self.push_log("[USB] 연결 상태 및 인증 확인을 시작합니다.");
                Task::perform(
                    async {
                        let mut adb = DirectAdb::new();
                        match adb.check_usb_state() {
                        Ok(AdbUsbState::Ready) => Ok("연결 완료: Android ADB 기기가 인증되었습니다.".to_owned()),
                        Ok(AdbUsbState::Recovery) => Ok("기기가 Recovery ADB 모드로 연결되어 있습니다.".to_owned()),
                        Ok(AdbUsbState::NoDevice) => Ok("ADB 기기를 찾지 못했습니다. 데이터 케이블과 USB 디버깅을 확인하세요.".to_owned()),
                        Ok(AdbUsbState::MultipleDevices) => Ok("ADB 기기가 여러 대 감지되었습니다. 한 대만 연결하세요.".to_owned()),
                        Ok(AdbUsbState::Unauthorized) => Ok("ADB 인증이 필요합니다. 태블릿 화면에서 USB 디버깅 허용을 선택하세요.".to_owned()),
                        Ok(AdbUsbState::ServerBlocking) => Ok("USB ADB 인터페이스가 점유 중입니다. 다른 ADB 프로그램을 닫고 재검사하세요.".to_owned()),
                        Ok(AdbUsbState::Error(err)) => Err(err),
                        Err(err) => Err(err.to_string()),
                    }
                    },
                    Message::UsbStatusChecked,
                )
            }
            Message::UsbStatusChecked(result) => {
                match result {
                    Ok(status) => {
                        self.adb_server_blocking = status.contains("USB ADB 인터페이스가 점유 중");
                        self.push_log(format!("[USB] {status}"));
                    }
                    Err(err) => self.push_log(format!("[USB] 진단 오류: {err}")),
                }
                self.active_nav = NavPage::Log;
                Task::none()
            }
            Message::OpenDiagnosticsFolder => {
                let dir = paths::diagnostic_logs_dir();
                if let Err(err) = fs::create_dir_all(&dir) {
                    self.push_log(format!("[USB] 진단 로그 폴더 생성 실패: {err}"));
                } else if let Err(err) = open::that(&dir) {
                    self.push_log(format!("[USB] 진단 로그 폴더 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenLogFolder => {
                let dir = paths::logs_dir();
                if let Err(err) = fs::create_dir_all(&dir) {
                    self.push_log(format!("[Log] 작업 로그 폴더 생성 실패: {err}"));
                } else if let Err(err) = open::that(&dir) {
                    self.push_log(format!("[Log] 작업 로그 폴더 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::RequestCacheCleanup => {
                if self.can_start_cache_cleanup() {
                    self.cache_cleanup_confirmation_open = true;
                }
                Task::none()
            }
            Message::CancelCacheCleanup => {
                self.cache_cleanup_confirmation_open = false;
                Task::none()
            }
            Message::ConfirmCacheCleanup => {
                if !self.cache_cleanup_confirmation_open || !self.can_start_cache_cleanup() {
                    self.cache_cleanup_confirmation_open = false;
                    return Task::none();
                }
                self.cache_cleanup_confirmation_open = false;
                self.cache_cleanup_running = true;
                self.cache_cleanup_report = None;
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            cache_cleanup::clean_owned_apk_cache(&paths::apk_cache_dir())
                                .map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| error.to_string())
                        .and_then(|result| result)
                    },
                    Message::CacheCleanupFinished,
                )
            }
            Message::CacheCleanupFinished(result) => {
                self.cache_cleanup_running = false;
                match result {
                    Ok(report) => {
                        self.push_log(format!(
                            "[GBST] APK 캐시 정리 완료: {}개 파일",
                            report.files_removed,
                        ));
                        self.cache_cleanup_report = Some(report);
                    }
                    Err(error) => {
                        self.push_log(format!("[GBST] APK 캐시 정리에 실패했습니다: {error}"));
                    }
                }
                Task::none()
            }
            Message::RequestLogCleanup => {
                if self.can_start_log_cleanup() {
                    self.log_cleanup_confirmation_open = true;
                }
                Task::none()
            }
            Message::CancelLogCleanup => {
                self.log_cleanup_confirmation_open = false;
                Task::none()
            }
            Message::ConfirmLogCleanup => {
                if !self.log_cleanup_confirmation_open || !self.can_start_log_cleanup() {
                    self.log_cleanup_confirmation_open = false;
                    return Task::none();
                }
                self.log_cleanup_confirmation_open = false;
                self.log_cleanup_running = true;
                self.log_cleanup_report = None;
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            let mut adb = DirectAdb::new();
                            // Clean connected Lenovo tablet first; preserve PC logs
                            // if ADB cannot verify both device deletions.
                            adb.cleanup_dwas_kr_folders_with_retry(|_| {})
                                .map_err(|error| error.to_string())?;
                            log_cleanup::remove_gbst_logs(
                                &paths::logs_dir(),
                                &paths::diagnostic_logs_dir(),
                            )
                            .map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| error.to_string())
                        .and_then(|result| result)
                    },
                    Message::LogCleanupFinished,
                )
            }
            Message::LogCleanupFinished(result) => {
                self.log_cleanup_running = false;
                match result {
                    Ok(report) => {
                        // Log removal also clears the in-memory activity history.
                        // Otherwise the user could export deleted records again.
                        self.log_lines.clear();
                        self.log_cache_dirty = true;
                        self.push_log(format!("[GBST] PC 로그 및 태블릿 dwas_kr 폴더 정리 완료"));
                        self.log_cleanup_report = Some(report);
                    }
                    Err(error) => {
                        self.push_log(format!("[GBST] 로그 및 기기 폴더 정리 중 오류: {error}"))
                    }
                }
                Task::none()
            }
            Message::OpenGitHubRepository => {
                if let Err(err) = open::that("https://github.com/dwas-KR/GBST") {
                    self.push_log(format!("[GBST] GitHub 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenLTBoxGitHub => {
                if let Err(err) = open::that(LTBOX_GITHUB_URL) {
                    self.push_log(format!("[GBST] LTBox GitHub 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenGBSTIssues => {
                if let Err(err) = open::that("https://github.com/dwas-KR/GBST/issues") {
                    self.push_log(format!("[GBST] GitHub 이슈 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::ShowLicenseDialog => {
                self.license_dialog_open = true;
                Task::none()
            }
            Message::CloseLicenseDialog => {
                self.license_dialog_open = false;
                Task::none()
            }
            Message::OpenLicense => {
                let folder = paths::runtime_root();
                let file = if folder.join("LICENSE").is_file() {
                    folder.join("LICENSE")
                } else if folder
                    .parent()
                    .is_some_and(|parent| parent.join("LICENSE").is_file())
                {
                    folder.parent().unwrap().join("LICENSE")
                } else {
                    folder.join("LICENSE")
                };
                if file.is_file() {
                    if let Err(err) = open::that(&file) {
                        self.push_log(format!("[GBST] 라이선스 문서 열기 실패: {err}"));
                    }
                } else {
                    self.push_log("[GBST] 라이선스 파일을 찾지 못했습니다. GitHub 저장소의 LICENSE를 확인해 주세요.");
                    let _ = open::that("https://github.com/dwas-KR/GBST/blob/main/LICENSE");
                }
                Task::none()
            }
            Message::OpenYoutube => {
                if let Err(err) = open::that(DEVELOPER_YOUTUBE_URL) {
                    self.push_log(format!("[설정] YouTube 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenDonate => {
                if let Err(err) = open::that(DONATE_URL) {
                    self.push_log(format!("[설정] 후원 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::OpenCompletionDonate => {
                self.completion_notice_visible = false;
                if let Err(err) = open::that(DONATE_URL) {
                    self.push_log(format!("[설정] 후원 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::DismissCompletionNotice => {
                self.completion_notice_visible = false;
                Task::none()
            }
            Message::OpenFeedback => {
                let feedback_url = feedback_url_for_language(self.language);
                if let Err(err) = open::that(feedback_url) {
                    self.push_log(format!("[설정] 피드백 링크 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::CheckProgramUpdate => {
                if self.program_update_checking {
                    self.push_log("[Update] 이미 최신 릴리즈 확인이 진행 중입니다.");
                    return Task::none();
                }

                self.program_update_checking = true;
                self.push_log("[Update] 최신 GBST 릴리즈를 확인합니다.");

                Task::perform(
                    check_program_update_worker(APP_DISPLAY_VERSION.to_string()),
                    Message::ProgramUpdateChecked,
                )
            }
            Message::ProgramUpdateChecked(result) => {
                self.program_update_checking = false;

                match result {
                    Ok(info) => {
                        if info.update_available {
                            self.push_log(format!(
                                "[Update] 새 GBST 버전을 찾았습니다: 현재 {} → 최신 {}",
                                info.current_version, info.latest_version
                            ));
                            if let Some(asset_name) = &info.asset_name {
                                self.push_log(format!("[Update] 릴리즈 ZIP 파일: {asset_name}"));
                            }
                            self.push_log("[Update] 대시보드에 업데이트 안내 창을 표시합니다.");
                            self.dashboard_update_notice = Some(info);
                            self.active_nav = NavPage::Dashboard;
                        } else {
                            self.push_log(format!(
                                "[Update] 이미 최신 버전을 사용 중입니다: GBST {}",
                                info.current_version
                            ));
                        }
                    }
                    Err(err) => {
                        self.push_log(format!("[Update] 업데이트 확인 실패: {err}"));
                        self.push_log("[Update] 수동 확인을 위해 GitHub Releases 페이지를 엽니다.");
                        if let Err(open_err) = open::that(GBST_RELEASES_URL) {
                            self.push_log(format!(
                                "[Update] GitHub Releases 페이지 열기 실패: {open_err}"
                            ));
                        }
                    }
                }

                Task::none()
            }
            Message::DashboardProgramUpdateChecked(result) => {
                if let Ok(info) = result {
                    if info.update_available {
                        self.dashboard_update_notice = Some(info);
                        if self.active_nav == NavPage::Dashboard {
                            self.push_log("[Update] 새로운 업데이트 파일을 감지했습니다.");
                        }
                    }
                }
                Task::none()
            }
            Message::OpenProgramUpdateRelease => {
                let release_url = self
                    .dashboard_update_notice
                    .as_ref()
                    .map(|info| info.release_url.clone())
                    .unwrap_or_else(|| GBST_RELEASES_URL.to_string());
                self.dashboard_update_notice = None;
                if let Err(err) = open::that(&release_url) {
                    self.push_log(format!("[Update] GitHub Releases 페이지 열기 실패: {err}"));
                }
                Task::none()
            }
            Message::DismissProgramUpdateNotice => {
                self.dashboard_update_notice = None;
                self.push_log("[Update] 이번 업데이트 안내를 다음에 다시 확인합니다.");
                Task::none()
            }
            Message::ClearLog => {
                self.log_lines.clear();
                self.log_cache_dirty = true;
                Task::none()
            }
            Message::ExportLog => {
                if let Err(err) = self.save_gbst_log_to_file("수동") {
                    self.push_log(format!("[Log] 저장 실패: {err}"));
                }
                Task::none()
            }
            Message::SettingsLanguageSelected(language) => {
                self.language = language;
                self.log_cache_dirty = true;
                if let Err(err) = save_language(language) {
                    self.push_log(format!("[설정] 언어 저장 실패: {err}"));
                }
                Task::none()
            }
            Message::SettingsThemeSelected(theme) => {
                self.appearance.theme = theme;
                if let Err(err) = self.appearance.save() {
                    self.push_log(format!("[설정] 화면 테마 저장 실패: {err}"));
                }
                Task::none()
            }
            Message::SettingsAccentSelected(accent) => {
                self.appearance.accent = accent;
                if let Err(err) = self.appearance.save() {
                    self.push_log(format!("[설정] 강조 색상 저장 실패: {err}"));
                }
                Task::none()
            }
        }
    }

    fn theme(&self) -> Theme {
        self.appearance.as_theme(self.system_dark)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions =
            vec![iced::event::listen_with(
                |event, _status, _window_id| match event {
                    iced::Event::Window(iced::window::Event::Resized(size)) => {
                        Some(Message::WindowResized(size.width, size.height))
                    }
                    _ => None,
                },
            )];

        if self.active_nav == NavPage::Dashboard
            && !self.busy
            && !self.dashboard_refreshing
            && self.worker_rx.is_none()
        {
            subscriptions.push(
                iced::time::every(DASHBOARD_REFRESH_INTERVAL)
                    .map(|_| Message::DashboardRefreshTick),
            );
        }

        if self.worker_rx.is_some() {
            subscriptions
                .push(iced::time::every(LIVE_EVENT_DRAIN_INTERVAL).map(|_| Message::DrainWorker));
        }

        if self.apk_download_modal_message.is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(250))
                    .map(|_| Message::ApkDownloadNoticeTick),
            );
        }

        if self.active_nav == NavPage::Google && self.busy && self.install_cancel.is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_secs(1)).map(|_| Message::OperationElapsedTick),
            );
        }
        if !self.sidebar_anim_settled() {
            subscriptions
                .push(iced::time::every(SIDEBAR_ANIM_INTERVAL).map(|_| Message::SidebarAnimTick));
        }

        Subscription::batch(subscriptions)
    }

    fn view(&self) -> Element<'_, Message> {
        self.lt_view()
    }

    fn dashboard_panel(&self) -> Element<'_, Message> {
        let dashboard = &self.dashboard_info;
        let model_image_handle = self.dashboard_model_image_handle(dashboard);

        let model_preview = container(
            iced::widget::image(model_image_handle)
                .width(Length::Fixed(180.0))
                .height(Length::Fixed(108.0)),
        )
        .width(Length::Fixed(190.0))
        .height(Length::Fixed(142.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

        let info_grid = column![
            dashboard_info_row(
                self.language,
                "모델명",
                compact_text(&dashboard.model_name, 52)
            ),
            dashboard_info_row(
                self.language,
                "Android 버전",
                dashboard.android_version.clone()
            ),
            dashboard_info_row(self.language, "제조사", dashboard.manufacturer.clone()),
            dashboard_info_row(self.language, "롬 유형", dashboard.rom_type.clone()),
            dashboard_info_row(
                self.language,
                "구글 서비스 상태",
                dashboard.google_service_status.clone()
            ),
        ]
        .spacing(7)
        .width(Length::Fill);

        let top_divider = container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(140.0))
            .style(lpm_nav_divider_style);

        let content_row = row![model_preview, top_divider, info_grid]
            .spacing(20)
            .align_y(iced::Alignment::Center);

        let main_card = container(
            column![
                text("Google Basic Service Tool")
                    .size(24)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                content_row,
            ]
            .spacing(18),
        )
        .width(Length::Fill)
        .padding(18)
        .style(lpm_nav_dashboard_inner_style);

        let detect_action = dashboard_action_card(
            self.language,
            "후원하기",
            "개발자에게 큰 힘과 응원이 됩니다.",
            "이동",
            Message::OpenDonate,
            false,
        );

        let install_action = dashboard_action_card(
            self.language,
            "Google 서비스\n설치/복구/업데이트",
            "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.",
            "시작",
            Message::StartInstall,
            self.busy,
        );

        let link_action = dashboard_action_card(
            self.language,
            "개발자 유튜브",
            "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.",
            "이동",
            Message::OpenYoutube,
            false,
        );

        let body = column![
            main_card,
            row![detect_action, install_action, link_action]
                .spacing(12)
                .align_y(iced::Alignment::Start),
        ]
        .spacing(12);

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(18)
            .style(lpm_nav_app_background_style)
            .into()
    }

    fn progress_panel(&self) -> Element<'_, Message> {
        if !self.progress_visible {
            return Space::new().height(Length::Fixed(0.0)).into();
        }

        let phase = if !self.busy {
            String::new()
        } else if self.progress_stage > 0 {
            log_label_with_args(
                self.language,
                "progress.phase",
                &[("stage", &self.progress_stage.to_string())],
            )
        } else {
            log_label(self.language, "status.preparing")
        };
        let status = log_label(self.language, &self.progress_status_key);
        let mut panel = column![
            row![
                text(log_label(self.language, "status.overall"))
                    .size(13)
                    .font(lpm_bold_font()),
                text(format!("{:.1}%", self.overall_progress.clamp(0.0, 100.0)))
                    .size(14)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
            ]
            .align_y(iced::Alignment::Center),
            progress_bar(0.0..=100.0, self.overall_progress.clamp(0.0, 100.0))
                .length(Length::Fill)
                .girth(Length::Fixed(12.0)),
            row![
                text(status).size(12),
                text(phase)
                    .size(12)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
            ],
        ]
        .spacing(8);

        if let Some(percent) = self.transfer_percent {
            let package = self.transfer_package.as_deref().unwrap_or("APK");
            panel = panel
                .push(
                    row![
                        text(log_label_with_args(
                            self.language,
                            "progress.transfer",
                            &[("package", package)],
                        ))
                        .size(11)
                        .width(Length::Fill),
                        text(format!("{percent}%")).size(11),
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .push(
                    progress_bar(0.0..=100.0, f32::from(percent))
                        .length(Length::Fill)
                        .girth(Length::Fixed(7.0)),
                );
        }

        container(panel)
            .width(Length::Fill)
            .padding(12)
            .style(lpm_nav_dashboard_inner_style)
            .into()
    }

    fn log_section(&self) -> Element<'_, Message> {
        let log_text = self.build_log_text();

        let log_panel = container(
            scrollable(
                container(text(log_text).size(LOG_FONT))
                    .width(Length::Fill)
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 20.0,
                        bottom: 12.0,
                        left: 12.0,
                    }),
            )
            .anchor_bottom()
            .anchor_left()
            .auto_scroll(true)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .style(log_container_style);

        let log_buttons = row![
            button(text(ui_text(self.language, "로그 내보내기")).size(BODY_FONT))
                .on_press(Message::ExportLog),
            button(text(ui_text(self.language, "로그 지우기")).size(BODY_FONT))
                .on_press(Message::ClearLog),
        ]
        .spacing(8);
        // The existing cancellation mechanism remains internal; no
        // cancellation button is shown in the GBST execution interface.

        container(
            column![
                text(ui_text(self.language, "작업 로그")).size(18),
                self.progress_panel(),
                log_buttons,
                log_panel,
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(14)
        .style(lpm_nav_panel_style)
        .into()
    }

    fn settings_panel(&self) -> Element<'_, Message> {
        let language_picker = pick_list(
            LanguageOption::ALL.as_slice(),
            Some(self.language),
            Message::SettingsLanguageSelected,
        )
        .text_size(12)
        .style(lpm_nav_language_pick_list_style)
        .menu_style(lpm_nav_language_pick_list_menu_style)
        .width(Length::Fixed(170.0));

        container(
            scrollable(
                column![
                    settings_row(
                        self.language,
                        "언어 변경",
                        "GBST 프로그램 표시 언어를 변경합니다.",
                        language_picker.into(),
                    ),
                    settings_row(
                        self.language,
                        "개발자 유튜브",
                        "개발자 YouTube 채널로 이동합니다.",
                        settings_move_button(self.language, "이동", Message::OpenYoutube),
                    ),
                    settings_row(
                        self.language,
                        "후원하기",
                        "개발자에게 큰 힘과 응원이 됩니다.",
                        settings_move_button(self.language, "이동", Message::OpenDonate),
                    ),
                    settings_row(
                        self.language,
                        "USB/ADB 연결 진단",
                        "현재 USB 기기 연결, 인증, 다른 ADB 프로그램의 점유 상태를 검사합니다.",
                        settings_move_button(self.language, "진단", Message::CheckUsbStatus),
                    ),
                    settings_row(
                        self.language,
                        "ADB 진단 로그",
                        "연결 및 명령 실패 원인을 기록한 폴더를 엽니다.",
                        settings_move_button(
                            self.language,
                            "폴더 열기",
                            Message::OpenDiagnosticsFolder
                        ),
                    ),
                    settings_row(
                        self.language,
                        "프로그램 업데이트",
                        "GBST 최신 릴리즈 버전을 확인합니다.",
                        if self.program_update_checking {
                            settings_move_button(self.language, "확인 중", Message::Noop)
                        } else {
                            settings_move_button(self.language, "확인", Message::CheckProgramUpdate)
                        },
                    ),
                    settings_row(
                        self.language,
                        "피드백",
                        "의견을 주시면 프로그램이 완벽해질 수 있습니다.",
                        settings_move_button(self.language, "이동", Message::OpenFeedback),
                    ),
                ]
                .spacing(16),
            )
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([18.0, 16.0])
        .style(lpm_nav_settings_panel_style)
        .into()
    }

    fn nav_icon_handle(&self, page: NavPage) -> iced::widget::image::Handle {
        match page {
            NavPage::Dashboard => self.nav_home_handle.clone(),
            NavPage::Google => self.nav_g_handle.clone(),
            NavPage::Log => self.nav_log_handle.clone(),
            NavPage::Settings => self.nav_settings_handle.clone(),
            NavPage::About => self.nav_tab_settings_handle.clone(),
        }
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let label_t = ((self.sidebar_anim - 0.4) / 0.5).clamp(0.0, 1.0);
        let label_alpha = ease_out_cubic(label_t);

        let mut nav = column![].spacing(1).padding([16.0, 0.0]);

        nav = nav.push(lpm_nav_section_header(
            self.language,
            "기기 관리",
            label_alpha,
        ));
        for &(page, _icon, label) in LPM_NAV_MAIN {
            nav = nav.push(lpm_nav_button(
                page,
                self.nav_icon_handle(page),
                label,
                self.language,
                self.active_nav == page,
                label_alpha,
            ));
        }

        nav = nav.push(lpm_nav_section_header(
            self.language,
            "프로그램",
            label_alpha,
        ));
        for &(page, _icon, label) in LPM_NAV_TOOLS {
            nav = nav.push(lpm_nav_button(
                page,
                self.nav_icon_handle(page),
                label,
                self.language,
                self.active_nav == page,
                label_alpha,
            ));
        }

        let width =
            SIDEBAR_RAIL_WIDTH + (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_RAIL_WIDTH) * self.sidebar_anim;

        let panel = container(nav)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .style(lpm_nav_menu_panel_style);

        let divider = container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(lpm_nav_divider_style);

        let shell = row![panel, divider].height(Length::Fill);

        iced::widget::mouse_area(shell)
            .on_enter(Message::SidebarHoverEnter)
            .on_exit(Message::SidebarHoverExit)
            .on_press(Message::Noop)
            .interaction(iced::mouse::Interaction::Idle)
            .into()
    }

    fn sidebar_anim_target(&self) -> f32 {
        if self.lt_is_compact() && self.sidebar_expanded {
            1.0
        } else {
            0.0
        }
    }

    fn sidebar_anim_settled(&self) -> bool {
        let target = self.sidebar_anim_target();
        (self.sidebar_anim - target).abs() <= SIDEBAR_ANIM_THRESHOLD
            && (self.sidebar_label_alpha - target).abs() <= SIDEBAR_ANIM_THRESHOLD
    }

    fn start_startup_prepare_worker(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.busy = true;
        self.dashboard_info = DashboardInfo::unknown();
        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;
        self.apk_download_percent = None;

        thread::spawn(move || {
            let result = prepare_startup_apk_download_flow(|line| {
                let download_completed = is_apk_download_done_message(&line);
                // The startup prefetch path must also consume typed download
                // signals; never print internal progress tokens as log text.
                let _ = tx.send(classify_worker_message(line));
                if download_completed {
                    let _ = tx.send(WorkerEvent::ApkDownloadCompleted);
                }
            })
            .map_err(|err| err.to_string());

            let _ = tx.send(WorkerEvent::StartupPrepared(result));
        });
    }

    fn start_detect_worker(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.busy = true;
        self.dashboard_info = DashboardInfo::unknown();
        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;
        self.push_log("[ADB] 기기 감지를 시작합니다.");

        thread::spawn(move || {
            let result = (|| -> Result<DashboardInfo, String> {
                paths::ensure_runtime_directories().map_err(|err| err.to_string())?;
                let mut adb = DirectAdb::new();
                adb.wait_ready(|line| {
                    let _ = tx.send(WorkerEvent::Log(line));
                })
                .map_err(|err| err.to_string())?;
                adb.read_dashboard_info().map_err(|err| err.to_string())
            })();

            let _ = tx.send(WorkerEvent::DashboardLoaded(result));
            let _ = tx.send(WorkerEvent::DeviceProbeFinished);
        });
    }

    fn try_start_pending_google(&mut self) {
        if self.pending_google_start
            && !self.cache_cleanup_running
            && !self.log_cleanup_running
            && self.active_nav == NavPage::Google
            && !self.disclaimer_open
            && !self.busy
            && self.worker_rx.is_none()
        {
            self.pending_google_start = false;
            self.start_install_worker();
        }
    }

    fn can_start_cache_cleanup(&self) -> bool {
        !self.busy
            && !self.dashboard_refreshing
            && self.worker_rx.is_none()
            && self.install_cancel.is_none()
            && !self.pending_google_start
            && !self.cache_cleanup_running
            && !self.log_cleanup_running
            && !self.disclaimer_open
    }

    fn can_start_log_cleanup(&self) -> bool {
        self.can_start_cache_cleanup() && self.lt_device_connected()
    }

    fn start_install_worker(&mut self) {
        if self.cache_cleanup_running || self.log_cleanup_running {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.busy = true;
        // Keep verified device identity visible while the Google worker runs.
        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;
        self.apk_download_percent = None;
        self.completion_notice_visible = false;
        self.active_nav = NavPage::Google;
        self.wizard_step = 3;
        self.operation_started_at = Some(Instant::now());
        self.operation_finalizing = false;
        self.usb_authorization_pending = false;
        self.progress_visible = true;
        self.plan_completed = 0;
        self.plan_total = 0;
        self.overall_progress = 0.0;
        self.progress_stage = 0;
        self.progress_status_key = "status.preparing".to_string();
        self.transfer_percent = None;
        self.transfer_package = None;
        self.apk_download_percent = None;
        self.active_stage_percent = None;
        self.push_log("[GBST] 기기에 Google Service 설치, 복구, 업데이트를 시작합니다.");
        let cancel = Arc::new(AtomicBool::new(false));
        self.install_cancel = Some(cancel.clone());

        thread::spawn(move || {
            let result = run_install_flow(cancel, |line| {
                let _ = tx.send(classify_worker_message(line));
            })
            .map_err(|err| err.to_string());
            let _ = tx.send(WorkerEvent::InstallFinished(result));
        });
    }

    fn drain_worker_events(&mut self) {
        let mut events = Vec::new();
        let mut disconnect = false;

        if let Some(rx) = self.worker_rx.as_ref() {
            loop {
                match rx.try_recv() {
                    Ok(event) => events.push(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnect = true;
                        break;
                    }
                }
            }
        }

        for event in events {
            match event {
                WorkerEvent::Log(line) => self.push_log(line),
                WorkerEvent::Progress(progress) => self.apply_progress_signal(progress),
                WorkerEvent::StartupPrepared(result) => {
                    self.usb_authorization_pending = false;
                    self.busy = false;
                    self.worker_rx = None;
                    self.apk_download_modal_message = None;
                    self.apk_download_modal_completed_at = None;
                    match result {
                        Ok(info) => {
                            self.dashboard_info = info;
                        }
                        Err(err) => {
                            let user_error = clean_user_error(&err);
                            self.dashboard_info = DashboardInfo::unknown();
                            self.push_log(format!(
                                "[GBST] APK 사전 다운로드 준비 실패: {user_error}"
                            ));
                        }
                    }
                }
                WorkerEvent::ApkDownloadCompleted => {
                    self.apk_download_modal_message = None;
                    self.apk_download_modal_completed_at = None;
                }
                WorkerEvent::DeviceProbeFinished => {
                    self.busy = false;
                    self.worker_rx = None;
                }
                WorkerEvent::DashboardLoaded(result) => match result {
                    Ok(info) => {
                        self.adb_server_blocking = false;
                        self.push_log(format!(
                            "[Device] 감지된 기기에 정보, 제조사: {} / 모델명: {} / Android 버전: {}",
                            info.manufacturer,
                            info.model_name,
                            info.android_version
                        ));
                        self.dashboard_info = info;
                    }
                    Err(err) => {
                        self.adb_server_blocking = is_adb_server_blocked_error(&err);
                        self.dashboard_info = DashboardInfo::unknown();
                        self.push_log(format!("[Dashboard] 감지 실패: {err}"));
                    }
                },
                WorkerEvent::InstallFinished(result) => {
                    self.usb_authorization_pending = false;
                    self.busy = false;
                    self.worker_rx = None;
                    self.install_cancel = None;
                    match result {
                        Ok(info) => {
                            self.dashboard_info = info;
                            if self.progress_visible {
                                self.overall_progress = 100.0;
                                self.progress_status_key = "status.finished".to_string();
                                self.transfer_percent = None;
                            }
                            self.push_log("[GBST] 작업이 완료되었습니다.");
                            if let Err(err) = self.save_gbst_log_to_file("완료") {
                                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
                            }
                            self.completion_notice_visible = true;
                        }
                        Err(err) => {
                            if self.progress_visible {
                                self.progress_status_key =
                                    if err.contains("취소") || err.contains("cancel") {
                                        "status.cancelled".to_string()
                                    } else {
                                        "status.failed".to_string()
                                    };
                                self.transfer_percent = None;
                            }
                            let user_error = clean_user_error(&err);
                            self.push_log(format!("[GBST] 작업 실패: {user_error}"));
                            if let Err(save_err) = self.save_gbst_log_to_file("실패") {
                                self.push_log(format!(
                                    "[Log] 작업 로그 자동 저장 실패: {save_err}"
                                ));
                            }
                        }
                    }
                }
                WorkerEvent::Finished(result) => {
                    self.usb_authorization_pending = false;
                    self.busy = false;
                    self.worker_rx = None;
                    match result {
                        Ok(()) => {
                            self.push_log("[GBST] 작업이 완료되었습니다.");
                            if let Err(err) = self.save_gbst_log_to_file("완료") {
                                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
                            }
                        }
                        Err(err) => {
                            let user_error = clean_user_error(&err);
                            self.push_log(format!("[GBST] 작업 실패: {user_error}"));
                            if let Err(save_err) = self.save_gbst_log_to_file("실패") {
                                self.push_log(format!(
                                    "[Log] 작업 로그 자동 저장 실패: {save_err}"
                                ));
                            }
                        }
                    }
                }
            }
        }

        if disconnect && self.busy {
            self.usb_authorization_pending = false;
            self.busy = false;
            self.worker_rx = None;
            self.push_log("[GBST] 작업 스레드 연결이 종료되었습니다.");
            if let Err(err) = self.save_gbst_log_to_file("종료") {
                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
            }
        }
        // Launch a queued Google operation only after old worker-channel
        // disconnect bookkeeping is finished. Otherwise a disconnected
        // startup worker could erroneously tear down the fresh install.
        self.try_start_pending_google();
    }

    fn apply_progress_signal(&mut self, progress: ProgressSignal) {
        match progress {
            ProgressSignal::Status { percent, key } => {
                self.overall_progress = self.overall_progress.max(percent.clamp(0.0, 99.0));
                self.progress_status_key = key;
            }
            ProgressSignal::Plan {
                completed,
                total,
                within,
                stage,
            } => {
                if self.progress_stage != stage {
                    self.active_stage_percent = None;
                }
                self.plan_completed = completed;
                self.plan_total = total;
                self.progress_stage = stage;
                self.overall_progress = self
                    .overall_progress
                    .max(overall_progress_from_plan(completed, total, within));
                self.progress_status_key = "status.install".to_string();
                if within == 0 {
                    self.transfer_percent = None;
                }
            }
            ProgressSignal::Download { percent } => {
                self.apk_download_percent = Some(percent);
                // LTBox-style overall gauge reserves the first 12% for APK
                // preparation; this does not alter any download/install step.
                if self.progress_visible {
                    self.overall_progress = self
                        .overall_progress
                        .max(1.0 + 11.0 * f32::from(percent) / 100.0);
                    self.progress_status_key = "status.preparing".to_string();
                }
            }
            ProgressSignal::Stage { stage, percent } => {
                if self.progress_stage != stage {
                    self.active_stage_percent = None;
                }
                self.progress_stage = stage;
                self.active_stage_percent = Some(percent);
            }
            ProgressSignal::Upload { package, percent } => {
                self.transfer_percent = Some(percent);
                self.transfer_package = Some(package);
                let within = ((u32::from(percent) * 70) / 100) as u8;
                self.overall_progress = self.overall_progress.max(overall_progress_from_plan(
                    self.plan_completed,
                    self.plan_total,
                    within,
                ));
            }
        }
        self.log_cache_dirty = true;
    }

    fn push_log(&mut self, line: impl Into<String>) {
        let raw = line.into();
        // The finalization timeline entry must reflect an actual worker
        // log, not the 92%-reserved work-plan boundary.
        if self.progress_visible && raw.contains("작업을 마무리 합니다.") {
            self.operation_finalizing = true;
        }
        if let Some((package, percent)) = raw
            .strip_prefix("[APK] ")
            .and_then(|line| line.split_once(": 기기 전송 진행률 "))
            && let Some(percent) = percent.strip_suffix('%')
            && let Ok(percent) = percent.parse::<u8>()
        {
            self.transfer_percent = Some(percent.min(100));
            self.transfer_package = Some(package.to_string());
            let within = ((u32::from(percent.min(100)) * 70) / 100) as u8;
            self.overall_progress = self.overall_progress.max(overall_progress_from_plan(
                self.plan_completed,
                self.plan_total,
                within,
            ));
        }
        if raw.contains("설치 성공 및 패키지 등록 확인") {
            self.transfer_percent = None;
            self.transfer_package = None;
        }
        let Some((spinner_key, message)) = normalize_log_line(raw) else {
            return;
        };

        // Permission guidance follows actual USB auth state, rather than
        // interrupting users whose tablets are already authorized.
        if message.starts_with("[ADB] USB 디버깅 인증 필요:") {
            self.usb_authorization_pending = true;
        } else if message.contains("[ADB] 기기 감지 완료")
            || message.contains("[ADB] 기기 재부팅 완료")
        {
            self.usb_authorization_pending = false;
        }

        // Downloading APKs is part of the total operation. Map its existing
        // 0–100% download gauge into the first 12% of the overall track.
        if self.progress_visible
            && message.starts_with("[APK] [")
            && message.contains("Android ")
            && message.contains("Google Service APK 파일")
            && let Some((_, rest)) = message.split_once("] ")
            && let Some((percent, _)) = rest.split_once("% ")
            && let Ok(percent) = percent.parse::<u8>()
        {
            let percent = percent.min(100);
            self.overall_progress = self
                .overall_progress
                .max(1.0 + 11.0 * f32::from(percent) / 100.0);
            self.progress_status_key = "status.preparing".to_string();
        }

        // Keep original source logs. Translate only when rendering or exporting
        // so switching the selected locale updates earlier lines as well.
        let is_apk_download_notice = is_apk_download_notice_message(&message);

        if let Some(key) = spinner_key.as_deref() {
            let mut replaced_spinner_line = false;
            if let Some(last) = self.log_lines.last_mut() {
                if last.spinner_key.as_deref() == Some(key) {
                    last.message = message.clone();
                    self.log_cache_dirty = true;
                    replaced_spinner_line = true;
                }
            }

            if is_apk_download_notice {
                self.update_apk_download_modal_from_log(message.clone());
            }

            if replaced_spinner_line {
                return;
            }
        } else if self
            .log_lines
            .last()
            .is_some_and(|last| last.spinner_key.is_none() && last.message == message)
        {
            if is_apk_download_notice {
                self.update_apk_download_modal_from_log(message.clone());
            }
            return;
        }

        if is_apk_download_notice {
            self.update_apk_download_modal_from_log(message.clone());
        }

        self.log_lines.push(LogLine {
            spinner_key,
            message,
        });
        self.log_cache_dirty = true;
    }

    fn update_apk_download_modal_from_log(&mut self, message: String) {
        self.apk_download_modal_message = Some(message.clone());
        if is_apk_download_done_message(&message) {
            self.apk_download_modal_completed_at = Some(Instant::now());
        } else {
            self.apk_download_modal_completed_at = None;
        }
    }

    fn dismiss_completed_apk_download_notice_if_ready(&mut self) {
        let Some(completed_at) = self.apk_download_modal_completed_at else {
            return;
        };

        if completed_at.elapsed() < APK_DOWNLOAD_NOTICE_DISMISS_DELAY {
            return;
        }

        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;
        self.apk_download_percent = None;
    }

    fn build_log_text(&self) -> String {
        if self.log_lines.is_empty() {
            ui_text(self.language, "로그가 없습니다.")
        } else {
            self.log_lines
                .iter()
                .map(|line| translate_log(self.language, &line.message))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn save_gbst_log_to_file(&mut self, flow_label: &str) -> std::io::Result<PathBuf> {
        let log_dir = paths::logs_dir();
        fs::create_dir_all(&log_dir)?;

        let base_name = format!("GBST_{}", Local::now().format("%Y-%m-%d_%H-%M"));
        let mut log_path = log_dir.join(format!("{base_name}.txt"));
        let mut duplicate_index = 2usize;

        while log_path.exists() {
            log_path = log_dir.join(format!("{base_name}_{duplicate_index}.txt"));
            duplicate_index += 1;
        }

        self.push_log(format!(
            "[Log] GBST {flow_label} 작업 로그를 {}에 저장합니다.",
            log_path.display()
        ));

        fs::write(&log_path, self.build_log_text())?;
        Ok(log_path)
    }

    fn should_show_dashboard_update_notice(&self) -> bool {
        self.active_nav == NavPage::Dashboard
            && self
                .dashboard_update_notice
                .as_ref()
                .map(|info| info.update_available)
                .unwrap_or(false)
    }

    fn usb_authorization_notice_view(&self) -> Element<'_, Message> {
        let content = container(
            column![
                text(log_label(self.language, "adb.authorization_title"))
                    .size(19)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                text(log_label(self.language, "adb.authorization_guide"))
                    .size(13)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::Word),
                text(log_label(self.language, "adb.authorization_wait"))
                    .size(12)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::Word),
            ]
            .spacing(14)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(465.0))
        .padding([22.0, 24.0])
        .style(lpm_nav_dashboard_update_popup_card_style);

        let scrim = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(lpm_nav_dashboard_update_popup_scrim_style);

        let centered = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }

    fn apk_download_notice_view(&self) -> Element<'_, Message> {
        let message = self
            .apk_download_modal_message
            .as_deref()
            .unwrap_or_default();

        let popup_content = container(
            column![
                text(ui_text(self.language, "파일 다운로드"))
                    .size(18)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::None),
                text(ui_text(
                    self.language,
                    "Google Service APK 파일을 준비하고 있습니다."
                ))
                .size(12)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
                container(
                    text(translate_log(self.language, message))
                        .size(12)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .wrapping(iced::widget::text::Wrapping::Word),
                )
                .width(Length::Fill)
                .padding([10.0, 12.0])
                .style(lpm_nav_dashboard_apk_download_card_style),
                row![
                    text(log_label(self.language, "status.preparing"))
                        .size(12)
                        .width(Length::Fill),
                    text(format!("{}%", self.apk_download_percent.unwrap_or(0)))
                        .size(13)
                        .font(lpm_bold_font()),
                ]
                .align_y(iced::Alignment::Center),
                progress_bar(
                    0.0..=100.0,
                    f32::from(self.apk_download_percent.unwrap_or(0)),
                )
                .length(Length::Fill)
                .girth(Length::Fixed(6.0)),
            ]
            .spacing(12)
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(420.0))
        .padding([18.0, 20.0])
        .style(lpm_nav_dashboard_update_popup_card_style);

        let scrim = iced::widget::mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(lpm_nav_dashboard_update_popup_scrim_style),
        )
        .on_press(Message::Noop);

        let centered = container(popup_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }

    fn completion_notice_view(&self) -> Element<'_, Message> {
        let content = completion_popup_text(self.language);

        let popup_body = column![
            text(ui_text(self.language, content.title))
                .size(19)
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            text(ui_text(self.language, content.first_line))
                .size(11.5)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            text(ui_text(self.language, content.second_line))
                .size(11.5)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            text(ui_text(self.language, content.third_line))
                .size(11.5)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            row![
                button(
                    container(text(ui_text(self.language, content.donate_button)).size(11.5))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fixed(180.0))
                .height(Length::Fixed(34.0))
                .padding(0.0)
                .style(lpm_nav_settings_move_button_style)
                .on_press(Message::OpenCompletionDonate),
                button(
                    container(text(ui_text(self.language, content.close_button)).size(11.5))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(34.0))
                .padding(0.0)
                .style(lpm_nav_settings_move_button_style)
                .on_press(Message::DismissCompletionNotice),
            ]
            .spacing(14)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(14)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center);

        let popup_content = container(
            container(popup_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(620.0))
        .height(Length::Fixed(320.0))
        .padding([22.0, 30.0])
        .style(lpm_nav_dashboard_update_popup_card_style);

        let scrim = iced::widget::mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(lpm_nav_dashboard_update_popup_scrim_style),
        )
        .on_press(Message::Noop);

        let centered = container(popup_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }

    fn dashboard_update_notice_view(&self) -> Element<'_, Message> {
        let version_label = self
            .dashboard_update_notice
            .as_ref()
            .map(|info| {
                format!(
                    "현재 {} → 최신 {}",
                    info.current_version, info.latest_version
                )
            })
            .unwrap_or_else(|| "새로운 버전을 감지했습니다.".to_string());

        let popup_content = container(
            column![
                text(ui_text(self.language, "새로운 업데이트가 있습니다."))
                    .size(18)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::None),
                text(ui_text(
                    self.language,
                    "현재 버전의 문제를
해결하고 업그레이드한 파일을
감지했습니다."
                ))
                .size(12)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
                text(version_label)
                    .size(10)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::None),
                row![
                    button(
                        container(text(ui_text(self.language, "파일 업데이트 (권장)")).size(11))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    )
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(28.0))
                    .padding(0.0)
                    .style(lpm_nav_settings_move_button_style)
                    .on_press(Message::OpenProgramUpdateRelease),
                    button(
                        container(text(ui_text(self.language, "다음에 하기")).size(11))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    )
                    .width(Length::Fixed(96.0))
                    .height(Length::Fixed(28.0))
                    .padding(0.0)
                    .style(lpm_nav_settings_move_button_style)
                    .on_press(Message::DismissProgramUpdateNotice),
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(10)
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(260.0))
        .height(Length::Fixed(210.0))
        .padding([18.0, 20.0])
        .style(lpm_nav_dashboard_update_popup_card_style);

        let scrim = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(lpm_nav_dashboard_update_popup_scrim_style);

        let centered = container(popup_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }
}

fn overall_progress_from_plan(completed: usize, total: usize, within: u8) -> f32 {
    if total == 0 {
        return 12.0;
    }
    let fraction =
        (completed.min(total) as f32 + f32::from(within.min(100)) / 100.0) / total as f32;
    (12.0 + 80.0 * fraction).clamp(12.0, 92.0)
}

fn normalize_log_line(raw: String) -> Option<(Option<String>, String)> {
    if raw == "__BLANK__" {
        return Some((None, String::new()));
    }

    let trimmed = raw.trim().to_string();

    if trimmed.is_empty() || is_hidden_log_text(&trimmed) {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("__SPINNER__|") {
        let mut parts = rest.splitn(2, '|');
        let key = parts.next()?.trim();
        let message = parts.next().unwrap_or_default().trim();

        if key.is_empty() || message.is_empty() || is_hidden_log_text(message) {
            return None;
        }

        return Some((Some(key.to_string()), message.to_string()));
    }

    Some((None, trimmed))
}

fn is_hidden_log_text(text: &str) -> bool {
    let trimmed = text.trim();

    if trimmed.starts_with("[GBST] 작업 실패:")
        || trimmed.starts_with("[Error]")
        || trimmed.contains("Failure")
        || trimmed.contains("INSTALL_FAILED_")
        || trimmed.contains("[경고]")
        || trimmed.contains("[WARN]")
        || trimmed.contains("ADB shell 실패")
        || trimmed.contains("USB Error")
        || trimmed.contains("Input/Output Error")
    {
        return false;
    }

    // Never hide failures or warnings: the message explains which step did
    // not work, even when the operation's FailurePolicy is Continue.
    trimmed.is_empty()
        || trimmed == "Success"
        || trimmed.starts_with("Success")
        || trimmed.starts_with("Package ")
        || trimmed.starts_with("Reset all modes for:")
        || trimmed.starts_with("[OK]")
}

const ADB_SERVER_BLOCKING_MARKER: &str = "__GBST_ADB_SERVER_BLOCKING__:";

fn is_adb_server_blocked_error(error: &str) -> bool {
    error.starts_with(ADB_SERVER_BLOCKING_MARKER)
}

fn adb_server_kill_allowed(
    blocked: bool,
    operation_in_progress: bool,
    kill_in_flight: bool,
    disclaimer_open: bool,
) -> bool {
    blocked && !operation_in_progress && !kill_in_flight && !disclaimer_open
}

async fn load_dashboard_info_worker() -> Result<DashboardInfo, String> {
    let mut adb = DirectAdb::new();
    match adb.read_dashboard_info() {
        Ok(info) => Ok(info),
        Err(error) => {
            // Match LTBox's precise BUSY vs UNAUTHORIZED distinction; the
            // warning is displayed only after a direct USB status probe.
            if matches!(adb.check_usb_state(), Ok(AdbUsbState::ServerBlocking)) {
                Err(format!("{ADB_SERVER_BLOCKING_MARKER} {error}"))
            } else {
                Err(error.to_string())
            }
        }
    }
}

fn prepare_startup_apk_download_flow<F>(mut on_log: F) -> Result<DashboardInfo, anyhow::Error>
where
    F: FnMut(String),
{
    paths::ensure_runtime_directories()?;
    remove_downloaded_apk_folders(|line| on_log(line))?;

    let mut adb = DirectAdb::new();
    adb.wait_ready(|line| on_log(line))?;

    let device = match adb.read_device_info() {
        Ok(device) => device,
        Err(err) => {
            let text = err.to_string();
            if text.contains("Lenovo") || text.contains("레노버") {
                on_log(
                    "[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다."
                        .to_string(),
                );
            }
            return Err(err.into());
        }
    };

    on_log(format!(
        "[Device] 감지된 기기에 정보, 제조사: {} / 모델명: {} / Android 버전: {}",
        device.manufacturer, device.model, device.android_version
    ));

    let android_major = device.android_major.value();
    let catalog = ApkCatalog::load_from_github_text(REMOTE_APK_CATALOG_URL, |line| on_log(line))?;
    let entries = catalog.entries_for_android(android_major)?;
    let _ = download_apks_for_android(android_major, &entries, |line| on_log(line))?;

    let info = adb.read_dashboard_info()?;
    Ok(info)
}

fn run_install_flow<F>(
    cancel: Arc<AtomicBool>,
    mut on_log: F,
) -> Result<DashboardInfo, anyhow::Error>
where
    F: FnMut(String),
{
    paths::ensure_runtime_directories()?;
    on_log("__GBST_PROGRESS__|1|status.preparing".to_string());
    let mut adb = DirectAdb::new();

    // Finally-style cleanup is outside the workflow so even early errors,
    // cancellation and failed post-install verification reach the same path.
    let workflow_result = run_install_flow_on_device(&mut adb, cancel.as_ref(), &mut on_log);

    if workflow_result.is_ok() {
        on_log("__GBST_PROGRESS__|98|status.cleanup".to_string());
    } else {
        on_log("__GBST_PROGRESS__|0|status.cleanup".to_string());
    }
    on_log("[ADB] 작업 종료 후 태블릿의 네 GBST 데이터/캐시 폴더 정리...".to_string());
    let cleanup_result = adb.cleanup_dwas_kr_folders_with_retry(|line| on_log(line));

    match (workflow_result, cleanup_result) {
        (Ok(info), Ok(())) => {
            on_log("[ADB] 삭제 완료".to_string());
            on_log("__GBST_PROGRESS__|99|status.cleanup".to_string());
            Ok(info)
        }
        (Err(err), Ok(())) => {
            on_log("[ADB] 작업 중단 후 GBST 임시 파일 정리 완료".to_string());
            Err(err)
        }
        (Ok(_), Err(err)) => Err(anyhow::anyhow!(
            "Google 서비스 작업 후 임시 파일 정리에 실패했습니다. 기기에 남아 있을 수 있습니다: {err}"
        )),
        (Err(workflow_error), Err(cleanup_error)) => Err(anyhow::anyhow!(
            "Google 서비스 작업 실패: {workflow_error}; 기기 임시 파일 정리도 확인되지 않았습니다: {cleanup_error}"
        )),
    }
}

fn run_install_flow_on_device<F>(
    adb: &mut DirectAdb,
    cancel: &AtomicBool,
    on_log: &mut F,
) -> Result<DashboardInfo, anyhow::Error>
where
    F: FnMut(String),
{
    adb.wait_ready(|line| on_log(line))?;

    let device = match adb.read_device_info() {
        Ok(device) => device,
        Err(err) => {
            let text = err.to_string();
            if text.contains("Lenovo") || text.contains("레노버") {
                on_log(
                    "[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다."
                        .to_string(),
                );
            }
            return Err(err.into());
        }
    };

    on_log(format!(
        "[Device] 감지된 기기에 정보, 제조사: {} / 모델명: {} / Android 버전: {}",
        device.manufacturer, device.model, device.android_version
    ));

    if !device.manufacturer.eq_ignore_ascii_case("Lenovo") {
        on_log("[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다.".to_string());
        return Err(anyhow::anyhow!("본 프로그램은 레노버 태블릿만 지원합니다."));
    }

    // Remove only the two explicitly user-authorized dwas_kr directories
    // before staging any APK. This follows the existing Lenovo check above.
    on_log("[ADB] 작업 시작 전 태블릿의 네 GBST 데이터/캐시 폴더를 정리합니다.".to_string());
    adb.cleanup_dwas_kr_folders_with_retry(|line| on_log(line))?;
    on_log("[ADB] 작업 시작 전 네 GBST 데이터/캐시 폴더 정리 및 검증 완료".to_string());

    let original_screen_timeout = match adb.read_screen_off_timeout() {
        Ok(value) => {
            let trimmed = value.trim().to_string();
            on_log(format!(
                "[Device] 사용자가 설정한 화면 꺼짐 값: {}",
                DirectAdb::screen_off_timeout_display_value(&trimmed)
            ));
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(err) => {
            on_log(format!(
                "[Device] 사용자가 설정한 화면 꺼짐 값: 알 수 없음 ({err})"
            ));
            None
        }
    };

    let original_screen_brightness = match adb.read_screen_brightness() {
        Ok(value) => {
            on_log(format!("[Device] 사용자가 설정한 화면 밝기 값: {value}"));
            Some(value)
        }
        Err(err) => {
            on_log(format!(
                "[Device] 사용자가 설정한 화면 밝기 값: 알 수 없음 ({err})"
            ));
            None
        }
    };

    // Only restore temporary UI changes; intentional OTA configuration remains intact.
    let temporary_settings = [("global", "mode_ringer"), ("system", "user_rotation")]
        .into_iter()
        .filter_map(|(scope, key)| {
            let value = adb.shell(&format!("settings get {scope} {key}")).ok()?;
            let value = value.trim().to_string();
            if value == "null" || value.parse::<u8>().ok().is_some_and(|n| n <= 3) {
                Some((scope, key, value))
            } else {
                on_log(format!("[Device] {key} 원래 값 확인 실패: 복원 제외"));
                None
            }
        })
        .collect::<Vec<_>>();

    let operation_result = (|| -> Result<(), anyhow::Error> {
        let android_major = device.android_major.value();

        let catalog =
            ApkCatalog::load_from_github_text(REMOTE_APK_CATALOG_URL, |line| on_log(line))?;
        let entries = catalog.entries_for_android(android_major)?;
        let local_apks = download_apks_for_android(android_major, &entries, |line| on_log(line))?;
        on_log("__GBST_PROGRESS__|12|status.install".to_string());
        if cancel.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!(
                "사용자가 APK 준비 이후 작업을 취소했습니다."
            ));
        }
        match adb.assess_google_services_from_downloaded_apks(device.android_major, &local_apks) {
            GoogleServiceAction::RepairRequired => {
                on_log(
                    "[GBST] 기기에 Google Services가 정상적이지 않으므로 복구를 진행합니다."
                        .to_string(),
                );
            }
            GoogleServiceAction::UpdateRequired => {
                on_log(
                    "[GBST] 기기에 Google Services APK 버전이 낮으므로 업데이트를 진행합니다."
                        .to_string(),
                );
            }
            GoogleServiceAction::Normal => {}
        }

        let plan = build_google_basic_service_plan(device.android_major, local_apks)?;

        adb.execute_plan_interruptible(&plan, cancel, |line| on_log(line))?;
        on_log("__GBST_PROGRESS__|92|status.verifying".to_string());
        Ok(())
    })();

    // Temporary APKs are removed by run_install_flow's finalizer after
    // *all* post-install verification and user-setting restoration.
    if let Some(brightness) = original_screen_brightness {
        on_log(format!(
            "[Device] 사용자가 설정한 화면 밝기 값({brightness})으로 복원합니다."
        ));
        if let Err(err) = adb.restore_screen_brightness(brightness) {
            on_log(format!(
                "    [경고] 화면 밝기 값 복원 실패, 계속 진행: {err}"
            ));
        }
    }

    if let Some(timeout) = original_screen_timeout.as_deref() {
        on_log("[Device] 사용자가 설정한 값으로 복원합니다.".to_string());
        if let Err(err) = adb.restore_screen_off_timeout(timeout) {
            on_log(format!(
                "    [경고] 화면 꺼짐 값 복원 실패, 계속 진행: {err}"
            ));
        }
    }

    for (scope, key, value) in temporary_settings {
        let command = if value == "null" {
            format!("settings delete {scope} {key}")
        } else {
            format!("settings put {scope} {key} {value}")
        };
        if let Err(err) = adb.shell(&command) {
            on_log(format!("[Device] 사용자 설정 {key} 복원 실패: {err}"));
        } else {
            on_log(format!("[Device] 사용자 설정 {key} 복원 완료"));
        }
    }

    operation_result?;

    on_log("[Device] 설치된 Google Services APK 버전과 GBST 정보를 다시 조회합니다.".to_string());

    let mut refreshed_info = None;
    let mut last_refresh_error = None;
    let mut post_install_reenable_checked = false;

    for attempt in 0..3 {
        match adb.read_dashboard_info() {
            Ok(info) => {
                let is_normal = info.google_service_status == "정상";
                // Only after the original plan has finished: recover a user-0
                // package disabled by the device after the last reboot.
                // No APK uninstall/reinstall, plan reorder, or extra reboot.
                if !is_normal
                    && info.google_service_status == "복구 필요"
                    && !post_install_reenable_checked
                    && attempt < 2
                {
                    post_install_reenable_checked = true;
                    adb.restore_disabled_google_packages_after_install(
                        device.android_major,
                        |line| on_log(line),
                    );
                }
                refreshed_info = Some(info);
                if is_normal || attempt == 2 {
                    break;
                }
            }
            Err(err) => {
                last_refresh_error = Some(err);
            }
        }
    }

    let info = refreshed_info.ok_or_else(|| {
        anyhow::anyhow!(
            "복구 완료 후 기기 정보 재조회 실패: {}",
            last_refresh_error
                .map(|err| err.to_string())
                .unwrap_or_else(|| "알 수 없는 오류".to_string())
        )
    })?;

    on_log(format!(
        "[Device] GBST 기기 정보 재조회 완료, Google Services 상태: {}",
        info.google_service_status
    ));
    if info.google_service_status != "정상" {
        return Err(anyhow::anyhow!(
            "Google 서비스 패키지 검사 결과가 정상 상태가 아닙니다: {}. 진단 로그를 확인하세요.",
            info.google_service_status
        ));
    }

    on_log("__GBST_PROGRESS__|97|status.verifying".to_string());
    Ok(info)
}

fn remove_downloaded_apk_folders<F>(mut on_log: F) -> Result<(), anyhow::Error>
where
    F: FnMut(String),
{
    let apk_root = paths::apk_cache_dir();

    on_log("__SPINNER__|apk_cleanup|[APK] 다운로드한 파일 및 폴더 제거... │".to_string());

    fs::create_dir_all(&apk_root)?;
    for android_dir in fs::read_dir(&apk_root)? {
        let path = android_dir?.path();
        if !path.is_dir() {
            if path.is_file() && path.metadata()?.len() == 0 {
                fs::remove_file(path)?;
            }
            continue;
        }

        for entry in fs::read_dir(path)? {
            let file_path = entry?.path();
            if !file_path.is_file() {
                continue;
            }

            let file_name = file_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if file_name.ends_with(".part") || file_path.metadata()?.len() == 0 {
                fs::remove_file(file_path)?;
            }
        }
    }

    on_log("__SPINNER__|apk_cleanup|[APK] 다운로드한 파일 및 폴더 제거 완료".to_string());
    Ok(())
}

fn clean_user_error(error: &str) -> String {
    if error.contains("APK 다운로드 및 검증에 실패했습니다:") {
        return error.to_string();
    }

    if error.contains("다운로드 실패")
        || error.contains("status code 403")
        || error.contains("https://")
        || error.contains("http://")
    {
        return "APK 다운로드에 실패했습니다. GitHub APK 링크 파일 또는 GitHub Release APK 다운로드 링크를 다시 확인해주세요.".to_string();
    }

    if error.contains("Lenovo") || error.contains("레노버") {
        return "본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다.".to_string();
    }

    error.to_string()
}

fn nav_page_title(page: NavPage) -> &'static str {
    match page {
        NavPage::Dashboard => "대시보드",
        NavPage::Google => "Google 작업 시작",
        NavPage::Log => "로그 관리",
        NavPage::Settings => "설정",
        NavPage::About => "정보",
    }
}

fn nav_page_subtitle(page: NavPage) -> &'static str {
    match page {
        NavPage::Dashboard => "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다.",
        NavPage::Google => "Lenovo 태블릿에 Google 기본 서비스를 설치/복구합니다.",
        NavPage::Log => "작업 로그를 확인하고 텍스트 파일로 저장합니다.",
        NavPage::Settings => "GBST 프로그램을 설정합니다.",
        NavPage::About => "GBST 프로젝트와 버전 정보를 확인합니다.",
    }
}

fn lpm_nav_button<'a>(
    page: NavPage,
    icon_handle: iced::widget::image::Handle,
    label: &'static str,
    language: LanguageOption,
    active: bool,
    label_alpha: f32,
) -> Element<'a, Message> {
    let icon_pill: Element<'a, Message> = container(
        iced::widget::image(icon_handle)
            .width(Length::Fixed(21.0))
            .height(Length::Fixed(21.0)),
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(28.0))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme: &Theme| {
        if active {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(224, 228, 255))),
                text_color: Some(Color::from_rgb8(43, 58, 118)),
                border: iced::Border {
                    radius: 14.0.into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            }
        } else {
            container::Style::default()
        }
    })
    .into();

    let mut inner = row![icon_pill].spacing(8).align_y(iced::Alignment::Center);

    if label_alpha > 0.0 {
        let alpha = label_alpha;
        inner = inner.push(
            text(ui_text(language, label))
                .size(lpm_sidebar_label_size())
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .wrapping(iced::widget::text::Wrapping::Word)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(
                        32.0 / 255.0,
                        35.0 / 255.0,
                        47.0 / 255.0,
                        alpha,
                    )),
                }),
        );
    }

    let content = container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center);

    button(content)
        .padding([0.0, 15.0])
        .width(Length::Fill)
        .height(Length::Fixed(NAV_BTN_HEIGHT))
        .on_press(Message::SelectNav(page))
        .style(move |_theme: &Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(Color::from_rgb8(233, 235, 246)))
                } else {
                    None
                },
                text_color: Color::from_rgb8(32, 35, 47),
                border: iced::Border {
                    radius: 18.0.into(),
                    ..iced::Border::default()
                },
                ..iced::widget::button::Style::default()
            }
        })
        .into()
}

fn lpm_nav_section_header<'a>(
    language: LanguageOption,
    label: &'static str,
    label_alpha: f32,
) -> Element<'a, Message> {
    let alpha = if label_alpha > 0.08 { label_alpha } else { 0.0 };

    let header_content: Element<'a, Message> = if alpha > 0.0 {
        text(ui_text(language, label))
            .size(11)
            .font(lpm_bold_font())
            .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH - 30.0))
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .wrapping(iced::widget::text::Wrapping::None)
            .style(move |_theme: &Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(
                    113.0 / 255.0,
                    116.0 / 255.0,
                    130.0 / 255.0,
                    alpha,
                )),
            })
            .into()
    } else {
        Space::new()
            .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH - 30.0))
            .height(Length::Fill)
            .into()
    };

    container(header_content)
        .width(Length::Fill)
        .height(Length::Fixed(31.0))
        .padding([0.0, 0.0])
        .clip(true)
        .style(lpm_nav_section_label_style)
        .into()
}

fn smooth_png_handle(bytes: &'static [u8], width: u32, height: u32) -> iced::widget::image::Handle {
    let Ok(image) = image_crate::load_from_memory(bytes) else {
        return iced::widget::image::Handle::from_bytes(bytes.to_vec());
    };

    let rgba = image.to_rgba8();
    let resized = image_crate::imageops::resize(
        &rgba,
        width,
        height,
        image_crate::imageops::FilterType::Lanczos3,
    );

    iced::widget::image::Handle::from_rgba(width, height, resized.into_raw())
}

/// Recolor the existing 21px navigation PNG while preserving its original
/// alpha and geometry. The light variant is created once in App::new.
fn light_nav_png_handle(
    bytes: &'static [u8],
    width: u32,
    height: u32,
) -> iced::widget::image::Handle {
    let Ok(decoded) = image_crate::load_from_memory(bytes) else {
        return iced::widget::image::Handle::from_bytes(bytes.to_vec());
    };
    let rgba = decoded.to_rgba8();
    let mut resized = image_crate::imageops::resize(
        &rgba,
        width,
        height,
        image_crate::imageops::FilterType::Lanczos3,
    );
    for pixel in resized.pixels_mut() {
        // Preserve transparent background and anti-aliased alpha edges.
        if pixel[3] > 0 {
            pixel[0] = 222;
            pixel[1] = 222;
            pixel[2] = 235;
        }
    }
    iced::widget::image::Handle::from_rgba(width, height, resized.into_raw())
}

fn lpm_sidebar_label_size() -> u32 {
    14
}

impl App {
    fn dashboard_model_image_handle(&self, info: &DashboardInfo) -> iced::widget::image::Handle {
        match dashboard_model_image_from_info(info) {
            DashboardModelImage::Etc => self.model_etc_handle.clone(),
            DashboardModelImage::Lenovo => self.model_lenovo_handle.clone(),
            DashboardModelImage::Tb9707F => self.model_tb_9707f_handle.clone(),
            DashboardModelImage::Tb320Fc => self.model_tb320fc_handle.clone(),
            DashboardModelImage::Tb321Fc => self.model_tb321fc_handle.clone(),
            DashboardModelImage::Tb322Fc => self.model_tb322fc_handle.clone(),
            DashboardModelImage::Tb323Fc => self.model_tb323fc_handle.clone(),
            DashboardModelImage::Tb324Zc => self.model_tb324zc_handle.clone(),
            DashboardModelImage::Tb331Fc => self.model_tb331fc_handle.clone(),
            DashboardModelImage::Tb335Fc => self.model_tb335fc_handle.clone(),
            DashboardModelImage::Tb365Fc => self.model_tb365fc_handle.clone(),
            DashboardModelImage::Tb371Fc => self.model_tb371fc_handle.clone(),
            DashboardModelImage::Tb375Fc => self.model_tb375fc_handle.clone(),
            DashboardModelImage::Tb376Fc => self.model_tb376fc_handle.clone(),
            DashboardModelImage::Tb378Fc => self.model_tb378fc_handle.clone(),
            DashboardModelImage::Tb520Fu => self.model_tb520fu_handle.clone(),
            DashboardModelImage::Tb522Fu => self.model_tb522fu_handle.clone(),
            DashboardModelImage::Tb710Fu => self.model_tb710fu_handle.clone(),
        }
    }
}

fn dashboard_model_image_from_info(info: &DashboardInfo) -> DashboardModelImage {
    if !info.is_lenovo {
        return DashboardModelImage::Etc;
    }

    let model = normalize_model_identifier(&info.model_name);

    if model.contains("TB9707F") {
        DashboardModelImage::Tb9707F
    } else if contains_any_model_alias(&model, &["TB710FU", "TB710FC"]) {
        DashboardModelImage::Tb710Fu
    } else if contains_any_model_alias(&model, &["TB711FU", "TB711FC", "TB522FU", "TB522FC"]) {
        DashboardModelImage::Tb522Fu
    } else if contains_any_model_alias(&model, &["TB520FU", "TB520FC"]) {
        DashboardModelImage::Tb520Fu
    } else if contains_any_model_alias(&model, &["TB378FC", "TB378FU"]) {
        DashboardModelImage::Tb378Fc
    } else if contains_any_model_alias(&model, &["TB390FU", "TB390FC", "TB376FC", "TB376FU"]) {
        DashboardModelImage::Tb376Fc
    } else if contains_any_model_alias(&model, &["TB375FC", "TB375FU", "TB373FU", "TB373FC"]) {
        DashboardModelImage::Tb375Fc
    } else if contains_any_model_alias(&model, &["TB371FC", "TB371FU"]) {
        DashboardModelImage::Tb371Fc
    } else if contains_any_model_alias(&model, &["TB365FC", "TB365FU", "TB361FU", "TB361FC"]) {
        DashboardModelImage::Tb365Fc
    } else if contains_any_model_alias(&model, &["TB335FC", "TB335FU", "TB336FU", "TB336FC"]) {
        DashboardModelImage::Tb335Fc
    } else if contains_any_model_alias(&model, &["TB331FC", "TB331FU"]) {
        DashboardModelImage::Tb331Fc
    } else if contains_any_model_alias(&model, &["TB324ZC"]) {
        DashboardModelImage::Tb324Zc
    } else if contains_any_model_alias(&model, &["TB323FC", "TB323FU"]) {
        DashboardModelImage::Tb323Fc
    } else if contains_any_model_alias(&model, &["TB322FC", "TB322FU"]) {
        DashboardModelImage::Tb322Fc
    } else if contains_any_model_alias(&model, &["TB321FC", "TB321FU"]) {
        DashboardModelImage::Tb321Fc
    } else if contains_any_model_alias(&model, &["TB320FC", "TB320FU"]) {
        DashboardModelImage::Tb320Fc
    } else {
        // Recognized Lenovo device, but no dedicated model illustration.
        // A generic Lenovo tablet is more useful than a question-mark tile.
        DashboardModelImage::Lenovo
    }
}

fn normalize_model_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

fn contains_any_model_alias(model: &str, aliases: &[&str]) -> bool {
    aliases.iter().any(|alias| model.contains(*alias))
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lpm_approach_anim_value(current: f32, target: f32, factor: f32, threshold: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= threshold {
        target
    } else {
        current + delta * factor
    }
}

fn dashboard_info_row<'a>(
    language: LanguageOption,
    label: &'a str,
    value: String,
) -> Element<'a, Message> {
    let alert = is_dashboard_alert_text(&value);
    container(
        row![
            text(ui_text(language, label))
                .size(12)
                .width(Length::Fixed(128.0)),
            text(ui_text(language, &value))
                .size(12)
                .width(Length::Fill)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(if alert {
                        Color::from_rgb8(203, 0, 0)
                    } else {
                        Color::from_rgb8(32, 35, 47)
                    }),
                }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([7.0, 10.0])
    .style(lpm_nav_rom_summary_item_style)
    .into()
}

fn is_dashboard_alert_text(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed == "알 수 없음"
        || trimmed == "감지 실패(복구 필요)"
        || trimmed == "복구 필요"
        || trimmed == "업데이트 필요"
        || trimmed == "Lenovo 기기가 아닙니다."
}

fn dashboard_action_card<'a>(
    language: LanguageOption,
    title: &'a str,
    description: &'a str,
    button_label: &'a str,
    message: Message,
    disabled: bool,
) -> Element<'a, Message> {
    const DASHBOARD_ACTION_CARD_HEIGHT: f32 = 234.0;
    const DASHBOARD_ACTION_TITLE_HEIGHT: f32 = 72.0;
    const DASHBOARD_ACTION_DESCRIPTION_HEIGHT: f32 = 72.0;
    const DASHBOARD_ACTION_BUTTON_HEIGHT: f32 = 34.0;

    container(
        column![
            container(
                text(ui_text(language, title))
                    .size(17)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::Word)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fixed(DASHBOARD_ACTION_TITLE_HEIGHT))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
            container(
                text(ui_text(language, description))
                    .size(12)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::Word)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fixed(DASHBOARD_ACTION_DESCRIPTION_HEIGHT))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
            Space::new().width(Length::Fill).height(Length::Fill),
            container(lpm_primary_button(
                ui_text(language, button_label),
                message,
                disabled
            ))
            .width(Length::Fill)
            .height(Length::Fixed(DASHBOARD_ACTION_BUTTON_HEIGHT))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        ]
        .spacing(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(DASHBOARD_ACTION_CARD_HEIGHT))
    .padding([16.0, 14.0])
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(lpm_nav_dashboard_action_style)
    .into()
}

fn is_apk_download_notice_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();

    let is_apk = message.contains("[APK]") || normalized.contains("[apk]");
    let is_google_service_apk = message.contains("Google Service APK")
        || normalized.contains("google service apk")
        || message.contains("Google Service APK 檔案")
        || message.contains("Google Service APKファイル");
    let is_download_state = message.contains("다운로드")
        || normalized.contains("download")
        || normalized.contains("descarga")
        || normalized.contains("descargando")
        || normalized.contains("tải")
        || normalized.contains("λήψη")
        || normalized.contains("загруз")
        || message.contains("下載")
        || message.contains("ダウンロード")
        || message.contains("تنزيل")
        || message.contains("ჩამოტვირთ");

    is_apk && is_google_service_apk && is_download_state
}

fn is_apk_download_done_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();

    (message.contains("100%") || message.contains("다운로드 완료"))
        && (message.contains("다운로드 완료")
            || normalized.contains("download complete")
            || normalized.contains("download voltooid")
            || normalized.contains("descarga")
            || message.contains("下載完成")
            || message.contains("ダウンロード完了")
            || message.contains("اكتمل تنزيل")
            || message.contains("λήψη")
            || message.contains("загруз")
            || message.contains("tải xong")
            || message.contains("पूर्ण")
            || message.contains("დასრულდა"))
}

fn lpm_primary_button<'a>(
    label: impl Into<String>,
    message: Message,
    disabled: bool,
) -> Element<'a, Message> {
    let label = label.into();
    let btn = button(
        container(text(label).size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(150.0))
    .height(Length::Fixed(34.0))
    .padding(0.0);

    if disabled {
        btn.style(lpm_nav_disabled_button_style).into()
    } else {
        btn.style(lpm_nav_settings_move_button_style)
            .on_press(message)
            .into()
    }
}

fn lpm_secondary_button<'a>(label: impl Into<String>, message: Message) -> Element<'a, Message> {
    let label = label.into();
    button(
        container(text(label).size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(140.0))
    .height(Length::Fixed(34.0))
    .padding(0.0)
    .style(lpm_nav_additional_option_button_style)
    .on_press(message)
    .into()
}

fn centered_move_button<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    container(
        button(
            container(text(label).size(12))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(38.0))
        .padding(0.0)
        .style(lpm_nav_additional_option_button_style)
        .on_press(message),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

fn settings_move_button<'a>(
    language: LanguageOption,
    label: &'a str,
    message: Message,
) -> Element<'a, Message> {
    button(
        container(text(ui_text(language, label)).size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(100.0))
    .height(Length::Fixed(28.0))
    .padding(0.0)
    .style(lpm_nav_settings_move_button_style)
    .on_press(message)
    .into()
}

fn settings_row<'a>(
    language: LanguageOption,
    title: &'a str,
    desc: &'a str,
    right: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(ui_text(language, title))
                    .size(22)
                    .font(lpm_bold_font()),
                text(ui_text(language, desc)).size(12),
            ]
            .spacing(6)
            .width(Length::Fill),
            right,
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([4.0, 0.0])
    .into()
}

fn compact_text(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut tail: String = text.chars().rev().take(max.saturating_sub(3)).collect();
        tail = tail.chars().rev().collect();
        format!("...{tail}")
    }
}

fn lpm_nav_disabled_button_style(theme: &Theme, _status: button::Status) -> button::Style {
    let c = shell_colors(theme);
    button::Style {
        background: Some(Background::Color(c.row)),
        text_color: c.muted,
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: c.border,
        },
        ..button::Style::default()
    }
}

fn lpm_nav_additional_option_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let c = shell_colors(theme);
    button::Style {
        background: Some(Background::Color(
            if matches!(status, button::Status::Hovered) {
                c.row
            } else {
                c.panel
            },
        )),
        text_color: c.text,
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: c.border,
        },
        ..button::Style::default()
    }
}

fn lpm_nav_settings_move_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let c = shell_colors(theme);
    button::Style {
        background: Some(Background::Color(
            if matches!(status, button::Status::Hovered) {
                c.tint
            } else {
                c.accent
            },
        )),
        text_color: if matches!(status, button::Status::Hovered) {
            c.text
        } else {
            Color::WHITE
        },
        border: iced::Border {
            radius: 10.0.into(),
            ..iced::Border::default()
        },
        ..button::Style::default()
    }
}

fn lpm_nav_language_pick_list_style(
    theme: &Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let c = shell_colors(theme);
    iced::widget::pick_list::Style {
        text_color: c.text,
        placeholder_color: c.muted,
        handle_color: c.text,
        background: Background::Color(
            if matches!(status, iced::widget::pick_list::Status::Active) {
                c.panel
            } else {
                c.row
            },
        ),
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: c.border,
        },
    }
}

fn lpm_nav_language_pick_list_menu_style(theme: &Theme) -> iced::widget::overlay::menu::Style {
    let c = shell_colors(theme);
    iced::widget::overlay::menu::Style {
        background: Background::Color(c.panel),
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: c.border,
        },
        text_color: c.text,
        selected_text_color: c.text,
        selected_background: Background::Color(c.tint),
        shadow: iced::Shadow::default(),
    }
}

fn lpm_nav_dashboard_black_screen_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0, 0, 0))),
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        border: iced::Border {
            radius: 8.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn lpm_nav_app_background_style(theme: &Theme) -> container::Style {
    shell_main_style(theme)
}

fn lpm_nav_menu_panel_style(theme: &Theme) -> container::Style {
    shell_sidebar_style(theme)
}

fn lpm_nav_section_label_style(theme: &Theme) -> container::Style {
    shell_row_style(theme)
}

fn lpm_nav_divider_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.border)),
        ..container::Style::default()
    }
}

fn lpm_nav_footer_style(theme: &Theme) -> container::Style {
    shell_footer_style(theme)
}

fn lpm_nav_header_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_settings_panel_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_panel_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_extra_option_card_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_primary_card_style(theme: &Theme) -> container::Style {
    shell_highlight_style(theme)
}

fn lpm_nav_status_idle_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(222, 244, 231))),
        text_color: Some(Color::from_rgb8(23, 104, 59)),
        ..container::Style::default()
    }
}

fn lpm_nav_status_busy_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 238, 214))),
        text_color: Some(Color::from_rgb8(150, 82, 20)),
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_inner_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_dashboard_action_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_rom_folder_info_card_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_rom_step_panel_style(theme: &Theme) -> container::Style {
    shell_panel_style(theme)
}

fn lpm_nav_rom_summary_item_style(theme: &Theme) -> container::Style {
    shell_row_style(theme)
}

fn lpm_nav_dashboard_apk_download_card_style(theme: &Theme) -> container::Style {
    shell_highlight_style(theme)
}

fn lpm_nav_dashboard_update_popup_scrim_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    let dark = c.page.r < 0.3;
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            0.0,
            0.0,
            0.0,
            if dark { 0.45 } else { 0.20 },
        ))),
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_update_popup_card_style(theme: &Theme) -> container::Style {
    let mut result = shell_panel_style(theme);
    result.border.radius = 17.0.into();
    result.shadow = iced::Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
        offset: iced::Vector::new(0.0, 8.0),
        blur_radius: 18.0,
    };
    result
}

fn log_container_style(theme: &Theme) -> container::Style {
    shell_row_style(theme)
}

async fn check_program_update_worker(
    current_version: String,
) -> Result<ProgramUpdateCheckResult, String> {
    check_program_update_blocking(&current_version)
}

fn check_program_update_blocking(
    current_version: &str,
) -> Result<ProgramUpdateCheckResult, String> {
    let response = ureq::get(GBST_RELEASES_API_URL)
        .header("User-Agent", "GBST")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("GitHub 릴리즈 정보를 가져오지 못했습니다: {err}"))?;

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("GitHub 응답을 읽지 못했습니다: {err}"))?;

    let releases: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("GitHub 응답 JSON 파싱 실패: {err}"))?;

    let releases = releases
        .as_array()
        .ok_or_else(|| "GitHub 릴리즈 응답 형식이 예상과 다릅니다.".to_string())?;

    let mut latest_tag: Option<String> = None;
    let mut latest_url: Option<String> = None;
    let mut latest_asset_name: Option<String> = None;
    let mut latest_version: Option<semver::Version> = None;

    for release in releases {
        if release
            .get("draft")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            continue;
        }
        if release
            .get("prerelease")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            continue;
        }

        let Some(tag) = release.get("tag_name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(parsed) = parse_gbst_version(tag) else {
            continue;
        };

        if latest_version
            .as_ref()
            .map_or(true, |version| parsed > *version)
        {
            latest_version = Some(parsed);
            latest_tag = Some(tag.to_string());
            latest_url = release
                .get("html_url")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned);
            latest_asset_name = release
                .get("assets")
                .and_then(|value| value.as_array())
                .and_then(|assets| find_gbst_release_zip_asset_name(assets.as_slice()));
        }
    }

    let latest_version_text = latest_tag
        .clone()
        .ok_or_else(|| "확인 가능한 GBST 릴리즈 버전을 찾지 못했습니다.".to_string())?;

    let current_parsed = parse_gbst_version(current_version)
        .ok_or_else(|| format!("현재 버전 값을 해석하지 못했습니다: {current_version}"))?;

    let latest_parsed =
        latest_version.ok_or_else(|| "최신 릴리즈 버전을 해석하지 못했습니다.".to_string())?;

    Ok(ProgramUpdateCheckResult {
        current_version: current_version.to_string(),
        latest_version: latest_version_text,
        update_available: latest_parsed > current_parsed,
        release_url: latest_url.unwrap_or_else(|| GBST_RELEASES_URL.to_string()),
        asset_name: latest_asset_name,
    })
}

fn find_gbst_release_zip_asset_name(assets: &[serde_json::Value]) -> Option<String> {
    let mut candidates: Vec<(i32, String)> = Vec::new();

    for asset in assets {
        let Some(name) = asset.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".zip") || lower.contains("source") {
            continue;
        }

        let mut score = 0;
        if lower.contains("gbst") {
            score += 10;
        }
        for token in [
            "win", "windows", "x64", "amd64", "x86", "x86-x64", "x86_x64",
        ] {
            if lower.contains(token) {
                score += 2;
            }
        }
        candidates.push((score, name.to_string()));
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates.into_iter().map(|(_, name)| name).next()
}

fn parse_gbst_version(value: &str) -> Option<semver::Version> {
    let start = value.find(|ch: char| ch.is_ascii_digit())?;
    let mut candidate = String::new();

    for ch in value[start..].chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+') {
            candidate.push(ch);
        } else {
            break;
        }
    }

    if candidate.is_empty() {
        return None;
    }

    let (core, suffix) = split_semver_suffix(&candidate);
    let mut parts = core.split('.').collect::<Vec<_>>();
    while parts.len() < 3 {
        parts.push("0");
    }
    if parts.len() > 3 {
        return None;
    }

    let normalized = format!("{}{}", parts.join("."), suffix);
    semver::Version::parse(&normalized).ok()
}

fn split_semver_suffix(value: &str) -> (&str, &str) {
    let dash = value.find('-');
    let plus = value.find('+');

    match (dash, plus) {
        (Some(left), Some(right)) => {
            let index = left.min(right);
            (&value[..index], &value[index..])
        }
        (Some(index), None) | (None, Some(index)) => (&value[..index], &value[index..]),
        (None, None) => (value, ""),
    }
}

#[cfg(test)]
mod model_image_regression_tests {
    use super::*;

    fn info(model: &str) -> DashboardInfo {
        let mut info = DashboardInfo::unknown();
        info.is_lenovo = true;
        info.model_name = model.to_string();
        info
    }

    #[test]
    fn blocking_adb_kill_is_available_only_when_idle() {
        use super::{adb_server_kill_allowed, is_adb_server_blocked_error};
        assert!(adb_server_kill_allowed(true, false, false, false));
        assert!(!adb_server_kill_allowed(false, false, false, false));
        assert!(!adb_server_kill_allowed(true, true, false, false));
        assert!(!adb_server_kill_allowed(true, false, true, false));
        assert!(!adb_server_kill_allowed(true, false, false, true));
        assert!(is_adb_server_blocked_error(
            "__GBST_ADB_SERVER_BLOCKING__: USB interface busy"
        ));
        assert!(!is_adb_server_blocked_error("unauthorized"));
    }

    #[test]
    fn failure_details_are_not_removed_from_ui_log() {
        use super::is_hidden_log_text;
        for failure in [
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE]",
            "[경고] 계속 진행: ADB shell 실패",
            "[WARN] USB Error while reading device",
            "[ADB] pm install error: Failure",
            "Package error Failure [INSTALL_FAILED_INTERNAL_ERROR]",
        ] {
            assert!(!is_hidden_log_text(failure), "hidden: {failure}");
        }
        assert!(is_hidden_log_text("Success"));
    }

    #[test]
    fn requested_skus_share_only_their_visual_assets() {
        assert_eq!(
            dashboard_model_image_from_info(&info("TB128FU")),
            DashboardModelImage::Lenovo
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB336FU")),
            DashboardModelImage::Tb335Fc
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB361FU")),
            DashboardModelImage::Tb365Fc
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB373FU")),
            DashboardModelImage::Tb375Fc
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB390FU")),
            DashboardModelImage::Tb376Fc
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB711FU")),
            DashboardModelImage::Tb522Fu
        );
        assert_eq!(
            dashboard_model_image_from_info(&info("TB324ZC")),
            DashboardModelImage::Tb324Zc
        );
    }
}

#[cfg(test)]
mod whole_progress_tests {
    use super::{ProgressSignal, WorkerEvent, classify_worker_message, overall_progress_from_plan};
    #[test]
    fn download_and_stage_progress_are_structured_not_text_logs() {
        assert!(matches!(
            classify_worker_message("__GBST_DOWNLOAD_PROGRESS__|75".into()),
            WorkerEvent::Progress(ProgressSignal::Download { percent: 75 })
        ));
        assert!(matches!(
            classify_worker_message("__GBST_STAGE_PROGRESS__|3|40".into()),
            WorkerEvent::Progress(ProgressSignal::Stage {
                stage: 3,
                percent: 40
            })
        ));
        assert!(matches!(
            classify_worker_message("[APK] com.google.android.gms: 기기 전송 진행률 40%".into()),
            WorkerEvent::Progress(ProgressSignal::Upload { package, percent: 40 })
                if package == "com.google.android.gms"
        ));
        assert!(matches!(
            classify_worker_message("__GBST_DOWNLOAD_PROGRESS__|101".into()),
            WorkerEvent::Log(_)
        ));
    }

    #[test]
    fn typed_progress_events_are_not_rendered_as_logs() {
        let progress = classify_worker_message("__GBST_PLAN_PROGRESS__|25|100|70|2".to_string());
        assert!(matches!(
            progress,
            WorkerEvent::Progress(ProgressSignal::Plan {
                completed: 25,
                total: 100,
                within: 70,
                stage: 2
            })
        ));
        let status = classify_worker_message("__GBST_PROGRESS__|98|status.cleanup".to_string());
        assert!(matches!(
            status,
            WorkerEvent::Progress(ProgressSignal::Status { percent: 98.0, .. })
        ));
        let log = classify_worker_message("[APK] APK 설치 중".to_string());
        assert!(matches!(log, WorkerEvent::Log(_)));
    }
    #[test]
    fn whole_progress_is_monotonic_and_reserves_final_verification() {
        assert!((overall_progress_from_plan(0, 100, 0) - 12.0).abs() < 0.01);
        assert!((overall_progress_from_plan(0, 100, 70) - 12.56).abs() < 0.01);
        assert!((overall_progress_from_plan(50, 100, 0) - 52.0).abs() < 0.01);
        assert!((overall_progress_from_plan(100, 100, 0) - 92.0).abs() < 0.01);
        assert!((overall_progress_from_plan(150, 100, 100) - 92.0).abs() < 0.01);
        assert!((overall_progress_from_plan(0, 0, 0) - 12.0).abs() < 0.01);
    }
    #[test]
    fn upload_fraction_cannot_overflow_u8() {
        let fractions = (0u8..=100u8)
            .map(|p| ((u32::from(p) * 70) / 100) as u8)
            .collect::<Vec<_>>();
        assert_eq!(fractions[0], 0);
        assert_eq!(fractions[100], 70);
        assert!(fractions.windows(2).all(|v| v[0] <= v[1]));
    }
}

include!("ltbox_visual.rs");
