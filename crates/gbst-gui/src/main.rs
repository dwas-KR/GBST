#![windows_subsystem = "windows"]
#![allow(dead_code)]

use gbst_adb::{DirectAdb, GoogleServiceAction};
use gbst_core::apk_catalog::{ApkCatalog, REMOTE_APK_CATALOG_URL};
use gbst_core::downloader::download_apks_for_android;
use gbst_core::language::{completion_popup_text, detect_initial_language, save_language, translate_runtime_text, LanguageOption};
use gbst_core::model::{DashboardInfo, DeviceInfo};
use gbst_core::paths;
use gbst_core::plan::build_google_basic_service_plan;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, Space};
use iced::{window, Background, Color, Element, Font, Length, Size, Subscription, Task, Theme};
use image as image_crate;
use chrono::Local;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
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

const SIDEBAR_RAIL_WIDTH: f32 = 64.0;
const SIDEBAR_EXPANDED_WIDTH: f32 = 210.0;
const NAV_BTN_HEIGHT: f32 = 38.0;
const SIDEBAR_ANIM_INTERVAL: Duration = Duration::from_millis(16);
const LIVE_EVENT_DRAIN_INTERVAL: Duration = Duration::from_millis(250);
const DASHBOARD_REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const APK_DOWNLOAD_NOTICE_DISMISS_DELAY: Duration = Duration::from_secs(2);
const SIDEBAR_ANIM_FACTOR: f32 = 0.15;
const SIDEBAR_ANIM_THRESHOLD: f32 = 0.004;

const DEVELOPER_YOUTUBE_URL: &str = "https://www.youtube.com/@dwas_KR?sub_confirmation=1";
const GBST_RELEASES_URL: &str = "https://github.com/dwas-KR/GBST/releases";
const GBST_APK_RELEASE_URL: &str = "https://github.com/dwas-KR/GBST-APK/releases/tag/apk-2026.08.29";
const GBST_RELEASES_API_URL: &str = "https://api.github.com/repos/dwas-KR/GBST/releases?per_page=20";
const DONATE_URL: &str = "https://www.youtube.com/@dwas_KR/join";
const FEEDBACK_KOREAN_URL: &str = "https://github.com/dwas-KR/GBST/issues/1";
const FEEDBACK_GLOBAL_URL: &str = "https://github.com/dwas-KR/GBST/issues/3";

fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    {
        let backend_chosen = std::env::var_os("WGPU_BACKEND").is_some_and(|value| !value.is_empty());
        if !backend_chosen {
            unsafe {
                std::env::set_var("WGPU_BACKEND", "dx12");
            }
        }
    }

    let _ = paths::ensure_runtime_directories();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("Google Basic Service Tool")
        .default_font(lpm_font())
        .window(window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            icon: lpm_window_icon(),
            ..window::Settings::default()
        })
        .centered()
        .run()
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
    (NavPage::Google, NAV_G_ICON_BYTES, "Google 작업 시작"),
];

const LPM_NAV_TOOLS: &[(NavPage, &'static [u8], &str)] = &[
    (NavPage::Log, NAV_LOG_ICON_BYTES, "로그 관리"),
    (NavPage::Settings, NAV_SETTINGS_ICON_BYTES, "설정"),
];

#[derive(Debug, Clone)]
enum Message {
    Noop,
    StartupPrepareApks,
    SelectNav(NavPage),
    SidebarHoverEnter,
    SidebarHoverExit,
    SidebarAnimTick,
    DashboardRefreshTick,
    ApkDownloadNoticeTick,
    DashboardAutoLoaded(Result<DashboardInfo, String>),
    DetectDevice,
    StartInstall,
    DrainWorker,
    OpenApkLinks,
    OpenApkFolder,
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
}

#[derive(Debug)]
enum WorkerEvent {
    Log(String),
    DashboardLoaded(Result<DashboardInfo, String>),
    StartupPrepared(Result<DashboardInfo, String>),
    ApkDownloadCompleted,
    InstallFinished(Result<DashboardInfo, String>),
    Finished(Result<(), String>),
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

    language: LanguageOption,
    busy: bool,
    dashboard_refreshing: bool,
    worker_rx: Option<Receiver<WorkerEvent>>,
    device: Option<DeviceInfo>,
    dashboard_info: DashboardInfo,
    apk_download_modal_message: Option<String>,
    apk_download_modal_completed_at: Option<Instant>,
    log_lines: Vec<LogLine>,
    log_text_cache: String,
    log_cache_dirty: bool,
    program_update_checking: bool,
    dashboard_update_notice: Option<ProgramUpdateCheckResult>,
    completion_notice_visible: bool,

    nav_home_handle: iced::widget::image::Handle,
    nav_g_handle: iced::widget::image::Handle,
    nav_tab_settings_handle: iced::widget::image::Handle,
    nav_log_handle: iced::widget::image::Handle,
    nav_settings_handle: iced::widget::image::Handle,
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
            language,
            busy: false,
            dashboard_refreshing: false,
            worker_rx: None,
            device: None,
            dashboard_info: DashboardInfo::unknown(),
            apk_download_modal_message: None,
            apk_download_modal_completed_at: None,
            log_lines: Vec::new(),
            log_text_cache: String::new(),
            log_cache_dirty: false,
            program_update_checking: false,
            dashboard_update_notice: None,
            completion_notice_visible: false,
            nav_home_handle: smooth_png_handle(NAV_HOME_ICON_BYTES, 21, 21),
            nav_g_handle: smooth_png_handle(NAV_G_ICON_BYTES, 21, 21),
            nav_tab_settings_handle: smooth_png_handle(NAV_TAB_SETTINGS_ICON_BYTES, 21, 21),
            nav_log_handle: smooth_png_handle(NAV_LOG_ICON_BYTES, 21, 21),
            nav_settings_handle: smooth_png_handle(NAV_SETTINGS_ICON_BYTES, 21, 21),
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
            Message::StartupPrepareApks => {
                if !self.busy && self.worker_rx.is_none() {
                    self.start_startup_prepare_worker();
                }
                Task::none()
            }
            Message::SelectNav(page) => {
                if page == NavPage::Google {
                    if !self.busy {
                        self.start_install_worker();
                    } else {
                        self.active_nav = NavPage::Log;
                    }
                    return Task::none();
                }

                self.active_nav = page;

                if page == NavPage::Dashboard
                    && !self.busy
                    && !self.dashboard_refreshing
                    && self.worker_rx.is_none()
                {
                    self.dashboard_info = DashboardInfo::unknown();
                    self.dashboard_refreshing = true;
                    return Task::perform(load_dashboard_info_worker(), Message::DashboardAutoLoaded);
                }

                Task::none()
            }
            Message::SidebarHoverEnter => {
                self.sidebar_expanded = true;
                Task::none()
            }
            Message::SidebarHoverExit => {
                self.sidebar_expanded = false;
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
                Task::none()
            }
            Message::DashboardRefreshTick => {
                if self.active_nav != NavPage::Dashboard
                    || self.busy
                    || self.dashboard_refreshing
                    || self.worker_rx.is_some()
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
                        self.dashboard_info = info;
                    }
                    Err(_) => {
                        self.dashboard_info = DashboardInfo::unknown();
                    }
                }
                Task::none()
            }
            Message::DetectDevice => {
                if !self.busy {
                    self.start_detect_worker();
                }
                Task::none()
            }
            Message::StartInstall => {
                if !self.busy {
                    self.start_install_worker();
                }
                Task::none()
            }
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
                            self.push_log(format!("[Update] GitHub Releases 페이지 열기 실패: {open_err}"));
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
                if let Err(err) = save_language(language) {
                    self.push_log(format!("[설정] 언어 저장 실패: {err}"));
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

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
            subscriptions.push(iced::time::every(LIVE_EVENT_DRAIN_INTERVAL).map(|_| Message::DrainWorker));
        }

        if self.apk_download_modal_message.is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(250)).map(|_| Message::ApkDownloadNoticeTick),
            );
        }

        if !self.sidebar_anim_settled() {
            subscriptions.push(iced::time::every(SIDEBAR_ANIM_INTERVAL).map(|_| Message::SidebarAnimTick));
        }

        Subscription::batch(subscriptions)
    }

    fn view(&self) -> Element<'_, Message> {
        let status = if self.busy { ui_text(self.language, "작업 중") } else { ui_text(self.language, "대기 중") };
        let header_right: Element<'_, Message> = Space::new()
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(1.0))
            .into();

        let header = container(
            row![
                column![
                    text(ui_text(self.language, nav_page_title(self.active_nav))).size(24),
                    text(ui_text(self.language, nav_page_subtitle(self.active_nav))).size(12),
                ]
                .spacing(3)
                .width(Length::Fill),
                header_right,
            ]
            .spacing(12),
        )
        .width(Length::Fill)
        .padding([14.0, 16.0])
        .style(lpm_nav_header_style);

        let main_stack = match self.active_nav {
            NavPage::Dashboard => column![self.dashboard_panel()],
            NavPage::Google => column![self.log_section()],
            NavPage::Log => column![header, self.log_section()],
            NavPage::Settings => column![header, self.settings_panel()],
        }
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill);

        let footer = container(
            row![
                text(format!("● {status}")).size(12).width(Length::Fill),
                text(format!("v{APP_DISPLAY_VERSION}")).size(12).width(Length::Fixed(60.0)).align_x(iced::alignment::Horizontal::Right),
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .padding([6.0, 12.0])
        .style(lpm_nav_footer_style);

        let main_content = container(column![
            container(main_stack)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(12)
                .style(lpm_nav_app_background_style),
            footer,
        ])
        .width(Length::Fill)
        .height(Length::Fill);

        let rail_placeholder = container(Space::new())
            .width(Length::Fixed(SIDEBAR_RAIL_WIDTH))
            .height(Length::Fill);

        let row_base = row![rail_placeholder, main_content].height(Length::Fill);
        let mut layers: Vec<Element<'_, Message>> = vec![row_base.into(), self.sidebar()];
        if self.should_show_dashboard_update_notice() {
            layers.push(self.dashboard_update_notice_view());
        }
        if self.apk_download_modal_message.is_some() {
            layers.push(self.apk_download_notice_view());
        }
        if self.completion_notice_visible {
            layers.push(self.completion_notice_view());
        }

        iced::widget::Stack::with_children(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
            dashboard_info_row(self.language, "모델명", compact_text(&dashboard.model_name, 52)),
            dashboard_info_row(self.language, "Android 버전", dashboard.android_version.clone()),
            dashboard_info_row(self.language, "제조사", dashboard.manufacturer.clone()),
            dashboard_info_row(self.language, "롬 유형", dashboard.rom_type.clone()),
            dashboard_info_row(self.language, "구글 서비스 상태", dashboard.google_service_status.clone()),
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

    fn log_section(&self) -> Element<'_, Message> {
        let log_text = self.build_log_text();

        let log_panel = container(
            scrollable(
                container(text(log_text).size(LOG_FONT))
                    .width(Length::Fill)
                    .padding(iced::Padding { top: 12.0, right: 20.0, bottom: 12.0, left: 12.0 }),
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

        container(
            column![
                row![
                    text(ui_text(self.language, "작업 로그")).size(18),
                    text(ui_text(self.language, "ADB / APK Download / Package Install 진행 상태")).size(12),
                ]
                .spacing(10),
                row![
                    button(text(ui_text(self.language, "로그 내보내기")).size(BODY_FONT)).on_press(Message::ExportLog),
                    button(text(ui_text(self.language, "로그 지우기")).size(BODY_FONT)).on_press(Message::ClearLog),
                ]
                .spacing(8),
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
            .spacing(24),
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
        }
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let label_t = ((self.sidebar_anim - 0.4) / 0.5).clamp(0.0, 1.0);
        let label_alpha = ease_out_cubic(label_t);

        let mut nav = column![].spacing(1).padding([16.0, 0.0]);

        nav = nav.push(lpm_nav_section_header(self.language, "기기 관리", label_alpha));
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

        nav = nav.push(lpm_nav_section_header(self.language, "프로그램", label_alpha));
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

        let width = SIDEBAR_RAIL_WIDTH + (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_RAIL_WIDTH) * self.sidebar_anim;

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
        if self.sidebar_expanded { 1.0 } else { 0.0 }
    }

    fn sidebar_anim_settled(&self) -> bool {
        (self.sidebar_anim - self.sidebar_anim_target()).abs() <= SIDEBAR_ANIM_THRESHOLD
    }

    fn start_startup_prepare_worker(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.busy = true;
        self.dashboard_info = DashboardInfo::unknown();
        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;

        thread::spawn(move || {
            let result = prepare_startup_apk_download_flow(|line| {
                let download_completed = is_apk_download_done_message(&line);
                let _ = tx.send(WorkerEvent::Log(line));
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
        self.active_nav = NavPage::Log;
        self.push_log("[ADB] 기기 감지를 시작합니다.");

        thread::spawn(move || {
            let result = (|| -> Result<DashboardInfo, String> {
                paths::ensure_runtime_directories().map_err(|err| err.to_string())?;
                let mut adb = DirectAdb::new();
                adb.wait_ready(|line| { let _ = tx.send(WorkerEvent::Log(line)); })
                    .map_err(|err| err.to_string())?;
                adb.read_dashboard_info().map_err(|err| err.to_string())
            })();

            let _ = tx.send(WorkerEvent::DashboardLoaded(result));
            let _ = tx.send(WorkerEvent::Finished(Ok(())));
        });
    }

    fn start_install_worker(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.busy = true;
        self.dashboard_info = DashboardInfo::unknown();
        self.apk_download_modal_message = None;
        self.apk_download_modal_completed_at = None;
        self.completion_notice_visible = false;
        self.active_nav = NavPage::Log;
        self.push_log("[GBST] 기기에 Google Service 설치, 복구, 업데이트를 시작합니다.");

        thread::spawn(move || {
            let result = run_install_flow(|line| { let _ = tx.send(WorkerEvent::Log(line)); })
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
                WorkerEvent::StartupPrepared(result) => {
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
                            self.push_log(format!("[GBST] APK 사전 다운로드 준비 실패: {user_error}"));
                        }
                    }
                }
                WorkerEvent::ApkDownloadCompleted => {
                    self.apk_download_modal_message = None;
                    self.apk_download_modal_completed_at = None;
                }
                WorkerEvent::DashboardLoaded(result) => match result {
                    Ok(info) => {
                        self.push_log(format!(
                            "[Device] 감지된 기기에 정보, 제조사: {} / 모델명: {} / Android 버전: {}",
                            info.manufacturer,
                            info.model_name,
                            info.android_version
                        ));
                        self.dashboard_info = info;
                    }
                    Err(err) => {
                        self.dashboard_info = DashboardInfo::unknown();
                        self.push_log(format!("[Dashboard] 감지 실패: {err}"));
                    }
                },
                WorkerEvent::InstallFinished(result) => {
                    self.busy = false;
                    self.worker_rx = None;
                    match result {
                        Ok(info) => {
                            self.dashboard_info = info;
                            self.push_log("[GBST] 작업이 완료되었습니다.");
                            if let Err(err) = self.save_gbst_log_to_file("완료") {
                                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
                            }
                            self.completion_notice_visible = true;
                        }
                        Err(err) => {
                            let user_error = clean_user_error(&err);
                            self.push_log(format!("[GBST] 작업 실패: {user_error}"));
                            if let Err(save_err) = self.save_gbst_log_to_file("실패") {
                                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {save_err}"));
                            }
                        }
                    }
                }
                WorkerEvent::Finished(result) => {
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
                                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {save_err}"));
                            }
                        }
                    }
                }
            }
        }

        if disconnect && self.busy {
            self.busy = false;
            self.worker_rx = None;
            self.push_log("[GBST] 작업 스레드 연결이 종료되었습니다.");
            if let Err(err) = self.save_gbst_log_to_file("종료") {
                self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
            }
        }
    }

    fn push_log(&mut self, line: impl Into<String>) {
        let raw = line.into();
        let Some((spinner_key, message)) = normalize_log_line(raw) else {
            return;
        };

        let message = ui_text(self.language, &message);
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

        self.log_lines.push(LogLine { spinner_key, message });
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
    }

    fn build_log_text(&self) -> String {
        if self.log_lines.is_empty() {
            ui_text(self.language, "로그가 없습니다.")
        } else {
            self.log_lines
                .iter()
                .map(|line| line.message.clone())
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
                text(ui_text(self.language, "Google Service APK 파일을 준비하고 있습니다."))
                    .size(12)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::Word),
                container(
                    text(message)
                        .size(12)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .wrapping(iced::widget::text::Wrapping::Word),
                )
                .width(Length::Fill)
                .padding([10.0, 12.0])
                .style(lpm_nav_dashboard_apk_download_card_style),
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
            .map(|info| format!("현재 {} → 최신 {}", info.current_version, info.latest_version))
            .unwrap_or_else(|| "새로운 버전을 감지했습니다.".to_string());

        let popup_content = container(
            column![
                text(ui_text(self.language, "새로운 업데이트가 있습니다."))
                    .size(18)
                    .font(lpm_bold_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::None),
                text(ui_text(self.language, "현재 버전의 문제를
해결하고 업그레이드한 파일을
감지했습니다."))
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

    if trimmed.starts_with("[GBST] 작업 실패:") || trimmed.starts_with("[Error]") {
        return false;
    }

    trimmed.is_empty()
        || trimmed.contains("Failure")
        || trimmed.contains("[경고]")
        || trimmed.contains("[WARN]")
        || trimmed == "Success"
        || trimmed.starts_with("Success")
        || trimmed.starts_with("Package ")
        || trimmed.starts_with("Reset all modes for:")
        || trimmed.starts_with("[OK]")
        || trimmed.contains("INSTALL_FAILED_")
        || trimmed.contains("ADB shell 실패")
        || trimmed.contains("USB Error")
}

async fn load_dashboard_info_worker() -> Result<DashboardInfo, String> {
    let mut adb = DirectAdb::new();
    adb.read_dashboard_info().map_err(|err| err.to_string())
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
                on_log("[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다.".to_string());
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

fn run_install_flow<F>(mut on_log: F) -> Result<DashboardInfo, anyhow::Error>
where
    F: FnMut(String),
{
    paths::ensure_runtime_directories()?;

    let mut adb = DirectAdb::new();
    adb.wait_ready(|line| on_log(line))?;

    let device = match adb.read_device_info() {
        Ok(device) => device,
        Err(err) => {
            let text = err.to_string();
            if text.contains("Lenovo") || text.contains("레노버") {
                on_log("[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다.".to_string());
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

    let original_screen_timeout = match adb.read_screen_off_timeout() {
        Ok(value) => {
            let trimmed = value.trim().to_string();
            on_log(format!(
                "[Device] 사용자가 설정한 화면 꺼짐 값: {}",
                if trimmed.is_empty() { "알 수 없음" } else { trimmed.as_str() }
            ));
            if trimmed.is_empty() { None } else { Some(trimmed) }
        }
        Err(err) => {
            on_log(format!("[Device] 사용자가 설정한 화면 꺼짐 값: 알 수 없음 ({err})"));
            None
        }
    };

    let original_screen_brightness = match adb.read_screen_brightness() {
        Ok(value) => {
            on_log(format!("[Device] 사용자가 설정한 화면 밝기 값: {value}"));
            Some(value)
        }
        Err(err) => {
            on_log(format!("[Device] 사용자가 설정한 화면 밝기 값: 알 수 없음 ({err})"));
            None
        }
    };

    let operation_result = (|| -> Result<(), anyhow::Error> {
        let android_major = device.android_major.value();

        let catalog = ApkCatalog::load_from_github_text(REMOTE_APK_CATALOG_URL, |line| on_log(line))?;
        let entries = catalog.entries_for_android(android_major)?;
        let local_apks = download_apks_for_android(android_major, &entries, |line| on_log(line))?;
        match adb.assess_google_services_from_downloaded_apks(&local_apks) {
            GoogleServiceAction::RepairRequired => {
                on_log("[GBST] 기기에 Google Services가 정상적이지 않으므로 복구를 진행합니다.".to_string());
            }
            GoogleServiceAction::UpdateRequired => {
                on_log("[GBST] 기기에 Google Services APK 버전이 낮으므로 업데이트를 진행합니다.".to_string());
            }
            GoogleServiceAction::Normal => {}
        }

        let plan = build_google_basic_service_plan(device.android_major, local_apks)?;

        adb.execute_plan(&plan, |line| on_log(line))?;
        Ok(())
    })();

    if let Err(err) = adb.cleanup_gbst_temp_files() {
        on_log(format!("    [경고] 기기 임시 파일 정리 실패, 계속 진행: {err}"));
    } else {
        on_log("[ADB] 기기 임시 폴더 /data/local/tmp/gbst 정리 완료".to_string());
    }

    if let Some(brightness) = original_screen_brightness {
        on_log(format!("[Device] 사용자가 설정한 화면 밝기 값({brightness})으로 복원합니다."));
        if let Err(err) = adb.restore_screen_brightness(brightness) {
            on_log(format!("    [경고] 화면 밝기 값 복원 실패, 계속 진행: {err}"));
        }
    }

    if let Some(timeout) = original_screen_timeout.as_deref() {
        on_log("[Device] 사용자가 설정한 값으로 복원합니다.".to_string());
        if let Err(err) = adb.restore_screen_off_timeout(timeout) {
            on_log(format!("    [경고] 화면 꺼짐 값 복원 실패, 계속 진행: {err}"));
        }
    }

    operation_result?;

    on_log("[Device] 설치된 Google Services APK 버전과 GBST 정보를 다시 조회합니다.".to_string());

    let mut refreshed_info = None;
    let mut last_refresh_error = None;

    for attempt in 0..3 {
        match adb.read_dashboard_info() {
            Ok(info) => {
                let is_normal = info.google_service_status == "정상";
                refreshed_info = Some(info);
                if is_normal || attempt == 2 {
                    break;
                }
            }
            Err(err) => {
                last_refresh_error = Some(err);
            }
        }

        thread::sleep(Duration::from_secs(1));
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
    }
}

fn nav_page_subtitle(page: NavPage) -> &'static str {
    match page {
        NavPage::Dashboard => "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다.",
        NavPage::Google => "Lenovo 태블릿에 Google 기본 서비스를 설치/복구합니다.",
        NavPage::Log => "작업 로그를 확인하고 텍스트 파일로 저장합니다.",
        NavPage::Settings => "GBST 프로그램을 설정합니다.",
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

    let mut inner = row![icon_pill]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if label_alpha > 0.0 {
        let alpha = label_alpha;
        inner = inner.push(
            text(ui_text(language, label))
                .size(lpm_sidebar_label_size())
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .wrapping(iced::widget::text::Wrapping::Word)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(32.0 / 255.0, 35.0 / 255.0, 47.0 / 255.0, alpha)),
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

fn lpm_nav_section_header<'a>(language: LanguageOption, label: &'static str, label_alpha: f32) -> Element<'a, Message> {
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
                color: Some(Color::from_rgba(113.0 / 255.0, 116.0 / 255.0, 130.0 / 255.0, alpha)),
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

fn lpm_sidebar_label_size() -> u32 { 14 }

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
    } else if contains_any_model_alias(&model, &["TB522FU", "TB522FC"]) {
        DashboardModelImage::Tb522Fu
    } else if contains_any_model_alias(&model, &["TB520FU", "TB520FC"]) {
        DashboardModelImage::Tb520Fu
    } else if contains_any_model_alias(&model, &["TB378FC", "TB378FU"]) {
        DashboardModelImage::Tb378Fc
    } else if contains_any_model_alias(&model, &["TB376FC", "TB376FU"]) {
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
    } else if contains_any_model_alias(&model, &["TB323FC", "TB323FU"]) {
        DashboardModelImage::Tb323Fc
    } else if contains_any_model_alias(&model, &["TB322FC", "TB322FU"]) {
        DashboardModelImage::Tb322Fc
    } else if contains_any_model_alias(&model, &["TB321FC", "TB321FU"]) {
        DashboardModelImage::Tb321Fc
    } else if contains_any_model_alias(&model, &["TB320FC", "TB320FU"]) {
        DashboardModelImage::Tb320Fc
    } else {
        DashboardModelImage::Etc
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
    if delta.abs() <= threshold { target } else { current + delta * factor }
}

fn dashboard_info_row<'a>(language: LanguageOption, label: &'a str, value: String) -> Element<'a, Message> {
    let alert = is_dashboard_alert_text(&value);
    container(
        row![
            text(ui_text(language, label)).size(12).width(Length::Fixed(128.0)),
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
            container(lpm_primary_button(ui_text(language, button_label), message, disabled))
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

    message.contains("100%")
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

fn lpm_primary_button<'a>(label: impl Into<String>, message: Message, disabled: bool) -> Element<'a, Message> {
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
        btn.style(lpm_nav_settings_move_button_style).on_press(message).into()
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

fn settings_move_button<'a>(language: LanguageOption, label: &'a str, message: Message) -> Element<'a, Message> {
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

fn settings_row<'a>(language: LanguageOption, title: &'a str, desc: &'a str, right: Element<'a, Message>) -> Element<'a, Message> {
    container(
        row![
            column![
                text(ui_text(language, title)).size(22).font(lpm_bold_font()),
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

fn lpm_nav_disabled_button_style(_theme: &Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(206, 208, 218))),
        text_color: Color::from_rgb8(112, 116, 130),
        border: iced::Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
        ..button::Style::default()
    }
}

fn lpm_nav_additional_option_button_style(_theme: &Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Color::from_rgb8(32, 35, 47),
        border: iced::Border { radius: 14.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        shadow: iced::Shadow { color: Color::TRANSPARENT, offset: iced::Vector::new(0.0, 0.0), blur_radius: 0.0 },
        ..button::Style::default()
    }
}

fn lpm_nav_settings_move_button_style(_theme: &Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(84, 91, 241))),
        text_color: Color::from_rgb8(255, 255, 255),
        border: iced::Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
        shadow: iced::Shadow { color: Color::TRANSPARENT, offset: iced::Vector::new(0.0, 0.0), blur_radius: 0.0 },
        ..button::Style::default()
    }
}

fn lpm_nav_language_pick_list_style(_theme: &Theme, status: iced::widget::pick_list::Status) -> iced::widget::pick_list::Style {
    let active = iced::widget::pick_list::Style {
        text_color: Color::from_rgb8(32, 35, 47),
        placeholder_color: Color::from_rgb8(92, 96, 112),
        handle_color: Color::from_rgb8(32, 35, 47),
        background: Background::Color(Color::from_rgb8(227, 227, 227)),
        border: iced::Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
    };

    match status {
        iced::widget::pick_list::Status::Active => active,
        iced::widget::pick_list::Status::Hovered | iced::widget::pick_list::Status::Opened { .. } => iced::widget::pick_list::Style { background: Background::Color(Color::from_rgb8(227, 227, 227)), ..active },
    }
}

fn lpm_nav_language_pick_list_menu_style(_theme: &Theme) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: Background::Color(Color::from_rgb8(255, 255, 255)),
        border: iced::Border { radius: 0.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        text_color: Color::from_rgb8(32, 35, 47),
        selected_text_color: Color::from_rgb8(32, 35, 47),
        selected_background: Background::Color(Color::from_rgb8(238, 240, 250)),
        shadow: iced::Shadow::default(),
    }
}

fn lpm_nav_dashboard_black_screen_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0, 0, 0))),
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        border: iced::Border { radius: 8.0.into(), ..iced::Border::default() },
        ..container::Style::default()
    }
}

fn lpm_nav_app_background_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(248, 248, 252))), text_color: Some(Color::from_rgb8(32, 35, 47)), ..container::Style::default() }
}

fn lpm_nav_menu_panel_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(246, 246, 251))), text_color: Some(Color::from_rgb8(32, 35, 47)), ..container::Style::default() }
}

fn lpm_nav_section_label_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(239, 239, 246))), text_color: Some(Color::from_rgb8(113, 116, 130)), ..container::Style::default() }
}

fn lpm_nav_divider_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(210, 212, 224))), ..container::Style::default() }
}

fn lpm_nav_footer_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(246, 246, 251))), text_color: Some(Color::from_rgb8(45, 48, 62)), ..container::Style::default() }
}

fn lpm_nav_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(28, 31, 45)),
        border: iced::Border { radius: 0.0.into(), width: 1.2, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_settings_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 12.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_panel_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(255, 255, 255))), text_color: Some(Color::from_rgb8(32, 35, 47)), ..container::Style::default() }
}

fn lpm_nav_extra_option_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 14.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_primary_card_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(232, 238, 255))), text_color: Some(Color::from_rgb8(29, 42, 86)), ..container::Style::default() }
}

fn lpm_nav_status_idle_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(222, 244, 231))), text_color: Some(Color::from_rgb8(23, 104, 59)), ..container::Style::default() }
}

fn lpm_nav_status_busy_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(255, 238, 214))), text_color: Some(Color::from_rgb8(150, 82, 20)), ..container::Style::default() }
}

fn lpm_nav_dashboard_inner_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 12.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_action_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 12.0.into(), width: 1.0, color: Color::from_rgb8(210, 214, 228) },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_folder_info_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 14.0.into(), width: 1.0, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_step_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 14.0.into(), width: 1.2, color: Color::from_rgb8(222, 225, 236) },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_summary_item_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(248, 248, 252))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 10.0.into(), width: 1.0, color: Color::from_rgb8(232, 234, 242) },
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_apk_download_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(224, 228, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 12.0.into(), width: 1.0, color: Color::from_rgb8(240, 198, 198) },
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_update_popup_scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.16))),
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_update_popup_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border { radius: 16.0.into(), width: 1.0, color: Color::from_rgb8(210, 214, 228) },
        shadow: iced::Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.18), offset: iced::Vector::new(0.0, 8.0), blur_radius: 18.0 },
        ..container::Style::default()
    }
}

fn log_container_style(_theme: &Theme) -> container::Style {
    container::Style { background: Some(Background::Color(Color::from_rgb8(238, 238, 238))), text_color: Some(Color::from_rgb8(20, 20, 20)), ..container::Style::default() }
}

async fn check_program_update_worker(current_version: String) -> Result<ProgramUpdateCheckResult, String> {
    check_program_update_blocking(&current_version)
}

fn check_program_update_blocking(current_version: &str) -> Result<ProgramUpdateCheckResult, String> {
    let response = ureq::get(GBST_RELEASES_API_URL)
        .set("User-Agent", "GBST")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("GitHub 릴리즈 정보를 가져오지 못했습니다: {err}"))?;

    let body = response
        .into_string()
        .map_err(|err| format!("GitHub 응답을 읽지 못했습니다: {err}"))?;

    let releases: serde_json::Value = serde_json::from_str(&body)
        .map_err(|err| format!("GitHub 응답 JSON 파싱 실패: {err}"))?;

    let releases = releases
        .as_array()
        .ok_or_else(|| "GitHub 릴리즈 응답 형식이 예상과 다릅니다.".to_string())?;

    let mut latest_tag: Option<String> = None;
    let mut latest_url: Option<String> = None;
    let mut latest_asset_name: Option<String> = None;
    let mut latest_version: Option<semver::Version> = None;

    for release in releases {
        if release.get("draft").and_then(|value| value.as_bool()).unwrap_or(false) {
            continue;
        }
        if release.get("prerelease").and_then(|value| value.as_bool()).unwrap_or(false) {
            continue;
        }

        let Some(tag) = release.get("tag_name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(parsed) = parse_gbst_version(tag) else {
            continue;
        };

        if latest_version.as_ref().map_or(true, |version| parsed > *version) {
            latest_version = Some(parsed);
            latest_tag = Some(tag.to_string());
            latest_url = release.get("html_url").and_then(|value| value.as_str()).map(ToOwned::to_owned);
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

    let latest_parsed = latest_version
        .ok_or_else(|| "최신 릴리즈 버전을 해석하지 못했습니다.".to_string())?;

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
        for token in ["win", "windows", "x64", "amd64", "x86", "x86-x64", "x86_x64"] {
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
