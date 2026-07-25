use crate::paths;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageOption {
    Korean,
    English,
    Russian,
    Japanese,
    TraditionalChinese,
    Vietnamese,
    Greek,
    Hindi,
    Georgian,
    Dutch,
    Arabic,
    Spanish,
}

impl LanguageOption {
    pub const ALL: [LanguageOption; 12] = [
        LanguageOption::Korean,
        LanguageOption::English,
        LanguageOption::Russian,
        LanguageOption::Japanese,
        LanguageOption::TraditionalChinese,
        LanguageOption::Vietnamese,
        LanguageOption::Greek,
        LanguageOption::Hindi,
        LanguageOption::Georgian,
        LanguageOption::Dutch,
        LanguageOption::Arabic,
        LanguageOption::Spanish,
    ];

    pub fn code(self) -> &'static str {
        match self {
            LanguageOption::Korean => "ko",
            LanguageOption::English => "en",
            LanguageOption::Russian => "ru",
            LanguageOption::Japanese => "ja",
            LanguageOption::TraditionalChinese => "zh-TW",
            LanguageOption::Vietnamese => "vi",
            LanguageOption::Greek => "el",
            LanguageOption::Hindi => "hi",
            LanguageOption::Georgian => "ka",
            LanguageOption::Dutch => "nl",
            LanguageOption::Arabic => "ar",
            LanguageOption::Spanish => "es",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            LanguageOption::Korean => "한국어",
            LanguageOption::English => "English",
            LanguageOption::Russian => "Русский",
            LanguageOption::Japanese => "日本語",
            LanguageOption::TraditionalChinese => "繁體中文",
            LanguageOption::Vietnamese => "Tiếng Việt",
            LanguageOption::Greek => "Ελληνικά",
            LanguageOption::Hindi => "हिन्दी",
            LanguageOption::Georgian => "ქართული",
            LanguageOption::Dutch => "Nederlands",
            LanguageOption::Arabic => "العربية",
            LanguageOption::Spanish => "Español",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let normalized = code.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "ko" | "ko-kr" => Some(LanguageOption::Korean),
            "en" | "en-us" | "en-gb" => Some(LanguageOption::English),
            "ru" | "ru-ru" => Some(LanguageOption::Russian),
            "ja" | "jp" | "ja-jp" => Some(LanguageOption::Japanese),
            "zh" | "zh-tw" | "zh-hant" | "zh-hk" => Some(LanguageOption::TraditionalChinese),
            "vi" | "vi-vn" => Some(LanguageOption::Vietnamese),
            "el" | "el-gr" => Some(LanguageOption::Greek),
            "hi" | "hi-in" => Some(LanguageOption::Hindi),
            "ka" | "ka-ge" => Some(LanguageOption::Georgian),
            "nl" | "nl-nl" => Some(LanguageOption::Dutch),
            "ar" | "ar-sa" | "ar-ae" => Some(LanguageOption::Arabic),
            "es" | "es-es" | "es-mx" => Some(LanguageOption::Spanish),
            _ => None,
        }
    }

    pub fn from_locale(locale: &str) -> Option<Self> {
        Self::from_code(locale)
    }

    pub fn is_rtl(self) -> bool {
        matches!(self, LanguageOption::Arabic)
    }
}

impl fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub fn load_saved_language() -> Option<LanguageOption> {
    std::fs::read_to_string(paths::language_config_path())
        .ok()
        .and_then(|text| LanguageOption::from_code(text.trim()))
}

pub fn save_language(language: LanguageOption) -> std::io::Result<()> {
    let path = paths::language_config_path();
    paths::ensure_parent(&path)?;
    std::fs::write(path, language.code())
}

pub fn detect_initial_language() -> LanguageOption {
    if let Some(saved) = load_saved_language() {
        return saved;
    }

    if let Some(os_language) = sys_locale::get_locale() {
        if let Some(language) = LanguageOption::from_locale(&os_language) {
            return language;
        }
    }

    LanguageOption::English
}

pub fn t(language: LanguageOption, key: &str) -> &'static str {
    let ko = language == LanguageOption::Korean;
    match key {
        "app_title" => "Google Basic Service Tool",
        "dashboard" => if ko { "대시보드" } else { "Dashboard" },
        "install" => if ko { "Google 설치" } else { "Google Install" },
        "apk_links" => if ko { "APK 링크" } else { "APK Links" },
        "log" => if ko { "로그 관리" } else { "Logs" },
        "settings" => if ko { "설정" } else { "Settings" },
        "device_manage" => if ko { "기기 관리" } else { "Device" },
        "program" => if ko { "프로그램" } else { "Program" },
        "idle" => if ko { "대기 중" } else { "Idle" },
        "busy" => if ko { "작업 중" } else { "Working" },
        "detect_device" => if ko { "기기 감지" } else { "Detect device" },
        "start_install" => if ko { "설치 시작" } else { "Start install" },
        "open_apk_links" => if ko { "GitHub Base64 링크 열기" } else { "Open GitHub Base64 Links" },
        "open_apk_folder" => if ko { "APK 폴더 열기" } else { "Open APK folder" },
        "clear_log" => if ko { "로그 지우기" } else { "Clear log" },
        "export_log" => if ko { "로그 저장" } else { "Export log" },
        "language" => if ko { "프로그램 언어" } else { "Program language" },
        "legal_notice" => if ko { "법적 고지" } else { "Legal Notice" },
        "youtube" => if ko { "개발자 YouTube" } else { "Developer YouTube" },
        _ => "",
    }
}

pub fn translate_owned(language: LanguageOption, content: &str) -> String {
    if language == LanguageOption::Korean || content.trim().is_empty() {
        return content.to_string();
    }

    translate_exact(language, content)
        .unwrap_or(content)
        .to_string()
}

fn translate_exact(language: LanguageOption, key: &str) -> Option<&'static str> {
    if let Some(value) = translate_dashboard_card_text(language, key) {
        return Some(value);
    }

    match language {
        LanguageOption::Korean => None,
        LanguageOption::English => match key {
            "대시보드" => Some("Dashboard"),
            "Google 설치/복구" => Some("Google Install/Repair"),
            "로그 관리" => Some("Logs"),
            "설정" => Some("Settings"),
            "기기 관리" => Some("Device"),
            "프로그램" => Some("Program"),
            "작업 중" => Some("Working"),
            "대기 중" => Some("Idle"),
            "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Manage GBST status and key functions in one place."),
            "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("View the task log and save it as a text file."),
            "GBST 프로그램을 설정합니다." => Some("Configure GBST."),
            "모델명" => Some("Model"),
            "Android 버전" => Some("Android Version"),
            "제조사" => Some("Manufacturer"),
            "롬 유형" => Some("ROM Type"),
            "구글 서비스 상태" => Some("Google Services Status"),
            "후원하기" => Some("Support Developer"),
            "개발자에게 큰 힘과 응원이 됩니다." => Some("Your support helps the developer a lot."),
            "이동" => Some("Go"),
            "시작" => Some("Start"),
            "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Download the APKs for your Android version and install Google Basic Services."),
            "개발자 유튜브" => Some("Developer YouTube"),
            "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Find more Lenovo tablet programs."),
            "작업 로그" => Some("Task Log"),
            "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APK Download / Package Install status"),
            "로그 내보내기" => Some("Export Log"),
            "로그 지우기" => Some("Clear Log"),
            "언어 변경" => Some("Language"),
            "GBST 프로그램 표시 언어를 변경합니다." => Some("Change the display language for GBST."),
            "개발자 YouTube 채널로 이동합니다." => Some("Open the developer's YouTube channel."),
            "프로그램 업데이트" => Some("Program Update"),
            "GBST 최신 릴리즈 버전을 확인합니다." => Some("Check the latest GBST release."),
            "확인" => Some("Check"),
            "확인 중" => Some("Checking"),
            "피드백" => Some("Feedback"),
            "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Your feedback helps improve the program."),
            "새로운 업데이트가 있습니다." => Some("A new update is available."),
            "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("An upgraded file that fixes issues in the current version\nhas been detected."),
            "파일 업데이트 (권장)" => Some("Update File (Recommended)"),
            "다음에 하기" => Some("Later"),
            _ => None,
        },
        LanguageOption::Russian => match key {
            "대시보드" => Some("Панель"),
            "Google 설치/복구" => Some("Установка/восстановление Google"),
            "로그 관리" => Some("Журналы"),
            "설정" => Some("Настройки"),
            "기기 관리" => Some("Устройство"),
            "프로그램" => Some("Программа"),
            "작업 중" => Some("Выполняется"),
            "대기 중" => Some("Ожидание"),
            "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Управляйте состоянием GBST и основными функциями на одном экране."),
            "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("Просматривайте журнал работы и сохраняйте его в текстовый файл."),
            "GBST 프로그램을 설정합니다." => Some("Настройка GBST."),
            "모델명" => Some("Модель"),
            "Android 버전" => Some("Версия Android"),
            "제조사" => Some("Производитель"),
            "롬 유형" => Some("Тип ROM"),
            "구글 서비스 상태" => Some("Состояние Google Services"),
            "후원하기" => Some("Поддержать разработчика"),
            "개발자에게 큰 힘과 응원이 됩니다." => Some("Ваша поддержка очень помогает разработчику."),
            "이동" => Some("Перейти"),
            "시작" => Some("Запуск"),
            "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Загрузить APK для вашей версии Android и установить Google Basic Services."),
            "개발자 유튜브" => Some("YouTube разработчика"),
            "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Найти больше программ для планшетов Lenovo."),
            "작업 로그" => Some("Журнал работы"),
            "ADB / APK Download / Package Install 진행 상태" => Some("Состояние ADB / загрузки APK / установки пакетов"),
            "로그 내보내기" => Some("Экспорт журнала"),
            "로그 지우기" => Some("Очистить журнал"),
            "언어 변경" => Some("Язык"),
            "GBST 프로그램 표시 언어를 변경합니다." => Some("Изменить язык интерфейса GBST."),
            "개발자 YouTube 채널로 이동합니다." => Some("Открыть YouTube-канал разработчика."),
            "프로그램 업데이트" => Some("Обновление программы"),
            "GBST 최신 릴리즈 버전을 확인합니다." => Some("Проверить последнюю версию GBST."),
            "확인" => Some("Проверить"),
            "확인 중" => Some("Проверка"),
            "피드백" => Some("Обратная связь"),
            "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Ваш отзыв поможет улучшить программу."),
            "새로운 업데이트가 있습니다." => Some("Доступно новое обновление."),
            "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("Обнаружен обновлённый файл, исправляющий проблемы текущей версии."),
            "파일 업데이트 (권장)" => Some("Обновить файл (рекомендуется)"),
            "다음에 하기" => Some("Позже"),
            _ => None,
        },
        LanguageOption::Japanese => match key {
            "대시보드" => Some("ダッシュボード"),
            "Google 설치/복구" => Some("Google インストール/修復"),
            "로그 관리" => Some("ログ管理"),
            "설정" => Some("設定"),
            "기기 관리" => Some("デバイス管理"),
            "프로그램" => Some("プログラム"),
            "작업 중" => Some("作業中"),
            "대기 중" => Some("待機中"),
            "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("GBSTの作業状態と主要機能を1つの画面で管理します。"),
            "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("作業ログを確認し、テキストファイルとして保存します。"),
            "GBST 프로그램을 설정합니다." => Some("GBSTを設定します。"),
            "모델명" => Some("モデル名"),
            "Android 버전" => Some("Android バージョン"),
            "제조사" => Some("メーカー"),
            "롬 유형" => Some("ROMタイプ"),
            "구글 서비스 상태" => Some("Googleサービス状態"),
            "후원하기" => Some("開発者を支援"),
            "개발자에게 큰 힘과 응원이 됩니다." => Some("開発者への大きな支援になります。"),
            "이동" => Some("移動"),
            "시작" => Some("開始"),
            "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Androidバージョンに合ったAPKをダウンロードし、Google基本サービスをインストールします。"),
            "개발자 유튜브" => Some("開発者YouTube"),
            "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Lenovoタブレット向けプログラムをもっと見る"),
            "작업 로그" => Some("作業ログ"),
            "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APKダウンロード / パッケージインストールの状態"),
            "로그 내보내기" => Some("ログを書き出す"),
            "로그 지우기" => Some("ログを消去"),
            "언어 변경" => Some("言語変更"),
            "GBST 프로그램 표시 언어를 변경합니다." => Some("GBSTの表示言語を変更します。"),
            "개발자 YouTube 채널로 이동합니다." => Some("開発者のYouTubeチャンネルを開きます。"),
            "프로그램 업데이트" => Some("プログラム更新"),
            "GBST 최신 릴리즈 버전을 확인합니다." => Some("GBSTの最新リリースを確認します。"),
            "확인" => Some("確認"),
            "확인 중" => Some("確認中"),
            "피드백" => Some("フィードバック"),
            "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("ご意見がプログラム改善につながります。"),
            "새로운 업데이트가 있습니다." => Some("新しいアップデートがあります。"),
            "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("現在のバージョンの問題を修正したアップグレードファイルを検出しました。"),
            "파일 업데이트 (권장)" => Some("ファイル更新（推奨）"),
            "다음에 하기" => Some("後で"),
            _ => None,
        },
        LanguageOption::TraditionalChinese => match key {
            "대시보드" => Some("儀表板"), "Google 설치/복구" => Some("Google 安裝/修復"), "로그 관리" => Some("記錄管理"), "설정" => Some("設定"), "기기 관리" => Some("裝置管理"), "프로그램" => Some("程式"), "작업 중" => Some("執行中"), "대기 중" => Some("待命中"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("在同一畫面管理 GBST 狀態與主要功能。"), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("查看工作記錄並儲存為文字檔。"), "GBST 프로그램을 설정합니다." => Some("設定 GBST 程式。"), "모델명" => Some("型號"), "Android 버전" => Some("Android 版本"), "제조사" => Some("製造商"), "롬 유형" => Some("ROM 類型"), "구글 서비스 상태" => Some("Google 服務狀態"), "후원하기" => Some("支持開發者"), "개발자에게 큰 힘과 응원이 됩니다." => Some("您的支持對開發者非常有幫助。"), "이동" => Some("前往"), "시작" => Some("開始"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("下載符合 Android 版本的 APK 並安裝 Google 基本服務。"), "개발자 유튜브" => Some("開發者 YouTube"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("查看更多 Lenovo 平板程式"), "작업 로그" => Some("工作記錄"), "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APK 下載 / 套件安裝狀態"), "로그 내보내기" => Some("匯出記錄"), "로그 지우기" => Some("清除記錄"), "언어 변경" => Some("語言變更"), "GBST 프로그램 표시 언어를 변경합니다." => Some("變更 GBST 的顯示語言。"), "개발자 YouTube 채널로 이동합니다." => Some("開啟開發者 YouTube 頻道。"), "프로그램 업데이트" => Some("程式更新"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("檢查 GBST 最新版本。"), "확인" => Some("檢查"), "확인 중" => Some("檢查中"), "피드백" => Some("意見回饋"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("您的意見能幫助改善程式。"), "새로운 업데이트가 있습니다." => Some("有新的更新。"), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("偵測到修復目前版本問題的升級檔案。"), "파일 업데이트 (권장)" => Some("更新檔案（建議）"), "다음에 하기" => Some("稍後"), _ => None,
        },
        LanguageOption::Vietnamese => match key {
            "대시보드" => Some("Bảng điều khiển"), "Google 설치/복구" => Some("Cài đặt/khôi phục Google"), "로그 관리" => Some("Quản lý nhật ký"), "설정" => Some("Cài đặt"), "기기 관리" => Some("Thiết bị"), "프로그램" => Some("Chương trình"), "작업 중" => Some("Đang xử lý"), "대기 중" => Some("Đang chờ"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Quản lý trạng thái GBST và các chức năng chính trên một màn hình."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("Xem nhật ký công việc và lưu thành tệp văn bản."), "GBST 프로그램을 설정합니다." => Some("Cấu hình GBST."), "모델명" => Some("Tên mẫu"), "Android 버전" => Some("Phiên bản Android"), "제조사" => Some("Nhà sản xuất"), "롬 유형" => Some("Loại ROM"), "구글 서비스 상태" => Some("Trạng thái Google Services"), "후원하기" => Some("Ủng hộ nhà phát triển"), "개발자에게 큰 힘과 응원이 됩니다." => Some("Sự ủng hộ của bạn là động lực lớn cho nhà phát triển."), "이동" => Some("Đi tới"), "시작" => Some("Bắt đầu"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Tải APK phù hợp với phiên bản Android và cài đặt Google Basic Services."), "개발자 유튜브" => Some("YouTube nhà phát triển"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Xem thêm chương trình cho máy tính bảng Lenovo"), "작업 로그" => Some("Nhật ký công việc"), "ADB / APK Download / Package Install 진행 상태" => Some("Trạng thái ADB / tải APK / cài đặt gói"), "로그 내보내기" => Some("Xuất nhật ký"), "로그 지우기" => Some("Xóa nhật ký"), "언어 변경" => Some("Ngôn ngữ"), "GBST 프로그램 표시 언어를 변경합니다." => Some("Thay đổi ngôn ngữ hiển thị của GBST."), "개발자 YouTube 채널로 이동합니다." => Some("Mở kênh YouTube của nhà phát triển."), "프로그램 업데이트" => Some("Cập nhật chương trình"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("Kiểm tra bản phát hành GBST mới nhất."), "확인" => Some("Kiểm tra"), "확인 중" => Some("Đang kiểm tra"), "피드백" => Some("Phản hồi"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Phản hồi của bạn giúp cải thiện chương trình."), "새로운 업데이트가 있습니다." => Some("Có bản cập nhật mới."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("Đã phát hiện tệp nâng cấp khắc phục sự cố của phiên bản hiện tại."), "파일 업데이트 (권장)" => Some("Cập nhật tệp (khuyến nghị)"), "다음에 하기" => Some("Để sau"), _ => None,
        },
        LanguageOption::Greek => match key {
            "대시보드" => Some("Πίνακας"), "Google 설치/복구" => Some("Εγκατάσταση/επισκευή Google"), "로그 관리" => Some("Αρχεία καταγραφής"), "설정" => Some("Ρυθμίσεις"), "기기 관리" => Some("Συσκευή"), "프로그램" => Some("Πρόγραμμα"), "작업 중" => Some("Εκτέλεση"), "대기 중" => Some("Αναμονή"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Διαχειριστείτε την κατάσταση GBST και τις βασικές λειτουργίες σε μία οθόνη."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("Δείτε το αρχείο καταγραφής και αποθηκεύστε το ως κείμενο."), "GBST 프로그램을 설정합니다." => Some("Ρυθμίστε το GBST."), "모델명" => Some("Μοντέλο"), "Android 버전" => Some("Έκδοση Android"), "제조사" => Some("Κατασκευαστής"), "롬 유형" => Some("Τύπος ROM"), "구글 서비스 상태" => Some("Κατάσταση Google Services"), "후원하기" => Some("Υποστήριξη προγραμματιστή"), "개발자에게 큰 힘과 응원이 됩니다." => Some("Η υποστήριξή σας βοηθά πολύ τον προγραμματιστή."), "이동" => Some("Μετάβαση"), "시작" => Some("Έναρξη"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Λήψη APK για την έκδοση Android και εγκατάσταση Google Basic Services."), "개발자 유튜브" => Some("YouTube προγραμματιστή"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Δείτε περισσότερα προγράμματα για tablet Lenovo"), "작업 로그" => Some("Αρχείο εργασίας"), "ADB / APK Download / Package Install 진행 상태" => Some("Κατάσταση ADB / λήψης APK / εγκατάστασης πακέτων"), "로그 내보내기" => Some("Εξαγωγή αρχείου"), "로그 지우기" => Some("Εκκαθάριση"), "언어 변경" => Some("Γλώσσα"), "GBST 프로그램 표시 언어를 변경합니다." => Some("Αλλαγή γλώσσας εμφάνισης GBST."), "개발자 YouTube 채널로 이동합니다." => Some("Άνοιγμα καναλιού YouTube προγραμματιστή."), "프로그램 업데이트" => Some("Ενημέρωση προγράμματος"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("Έλεγχος της τελευταίας έκδοσης GBST."), "확인" => Some("Έλεγχος"), "확인 중" => Some("Έλεγχος"), "피드백" => Some("Σχόλια"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Τα σχόλιά σας βοηθούν στη βελτίωση του προγράμματος."), "새로운 업데이트가 있습니다." => Some("Υπάρχει νέα ενημέρωση."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("Εντοπίστηκε αρχείο αναβάθμισης που διορθώνει την τρέχουσα έκδοση."), "파일 업데이트 (권장)" => Some("Ενημέρωση αρχείου (συνιστάται)"), "다음에 하기" => Some("Αργότερα"), _ => None,
        },
        LanguageOption::Hindi => match key {
            "대시보드" => Some("डैशबोर्ड"), "Google 설치/복구" => Some("Google इंस्टॉल/रिपेयर"), "로그 관리" => Some("लॉग"), "설정" => Some("सेटिंग्स"), "기기 관리" => Some("डिवाइस"), "프로그램" => Some("प्रोग्राम"), "작업 중" => Some("काम जारी"), "대기 중" => Some("प्रतीक्षा"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("GBST स्थिति और मुख्य सुविधाएँ एक ही स्क्रीन पर प्रबंधित करें।"), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("कार्य लॉग देखें और टेक्स्ट फ़ाइल के रूप में सहेजें।"), "GBST 프로그램을 설정합니다." => Some("GBST सेट करें।"), "모델명" => Some("मॉडल"), "Android 버전" => Some("Android संस्करण"), "제조사" => Some("निर्माता"), "롬 유형" => Some("ROM प्रकार"), "구글 서비스 상태" => Some("Google Services स्थिति"), "후원하기" => Some("डेवलपर का समर्थन करें"), "개발자에게 큰 힘과 응원이 됩니다." => Some("आपका समर्थन डेवलपर के लिए बहुत मददगार है।"), "이동" => Some("जाएँ"), "시작" => Some("शुरू"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Android संस्करण के अनुसार APK डाउनलोड कर Google Basic Services इंस्टॉल करें।"), "개발자 유튜브" => Some("डेवलपर YouTube"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Lenovo टैबलेट के और प्रोग्राम देखें"), "작업 로그" => Some("कार्य लॉग"), "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APK डाउनलोड / पैकेज इंस्टॉल स्थिति"), "로그 내보내기" => Some("लॉग निर्यात"), "로그 지우기" => Some("लॉग साफ़ करें"), "언어 변경" => Some("भाषा"), "GBST 프로그램 표시 언어를 변경합니다." => Some("GBST की प्रदर्शन भाषा बदलें।"), "개발자 YouTube 채널로 이동합니다." => Some("डेवलपर का YouTube चैनल खोलें।"), "프로그램 업데이트" => Some("प्रोग्राम अपडेट"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("GBST का नवीनतम रिलीज़ जाँचें।"), "확인" => Some("जाँचें"), "확인 중" => Some("जाँच जारी"), "피드백" => Some("प्रतिक्रिया"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("आपकी प्रतिक्रिया प्रोग्राम को बेहतर बनाती है।"), "새로운 업데이트가 있습니다." => Some("नया अपडेट उपलब्ध है।"), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("मौजूदा संस्करण की समस्याएँ ठीक करने वाली अपग्रेड फ़ाइल मिली है।"), "파일 업데이트 (권장)" => Some("फ़ाइल अपडेट (अनुशंसित)"), "다음에 하기" => Some("बाद में"), _ => None,
        },
        LanguageOption::Georgian => match key {
            "대시보드" => Some("მართვის დაფა"), "Google 설치/복구" => Some("Google-ის დაყენება/აღდგენა"), "로그 관리" => Some("ლოგები"), "설정" => Some("პარამეტრები"), "기기 관리" => Some("მოწყობილობა"), "프로그램" => Some("პროგრამა"), "작업 중" => Some("მიმდინარეობს"), "대기 중" => Some("მოლოდინი"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("მართეთ GBST-ის მდგომარეობა და ძირითადი ფუნქციები ერთ ეკრანზე."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("ნახეთ სამუშაო ლოგი და შეინახეთ ტექსტურ ფაილად."), "GBST 프로그램을 설정합니다." => Some("GBST-ის პარამეტრების შეცვლა."), "모델명" => Some("მოდელი"), "Android 버전" => Some("Android ვერსია"), "제조사" => Some("მწარმოებელი"), "롬 유형" => Some("ROM ტიპი"), "구글 서비스 상태" => Some("Google Services სტატუსი"), "후원하기" => Some("დეველოპერის მხარდაჭერა"), "개발자에게 큰 힘과 응원이 됩니다." => Some("თქვენი მხარდაჭერა დეველოპერს ძალიან ეხმარება."), "이동" => Some("გადასვლა"), "시작" => Some("დაწყება"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("ჩამოტვირთეთ APK Android-ის ვერსიის მიხედვით და დააყენეთ Google Basic Services."), "개발자 유튜브" => Some("დეველოპერის YouTube"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("იხილეთ მეტი Lenovo ტაბლეტის პროგრამა"), "작업 로그" => Some("სამუშაო ლოგი"), "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APK ჩამოტვირთვის / პაკეტის დაყენების სტატუსი"), "로그 내보내기" => Some("ლოგის ექსპორტი"), "로그 지우기" => Some("ლოგის გასუფთავება"), "언어 변경" => Some("ენა"), "GBST 프로그램 표시 언어를 변경합니다." => Some("შეცვალეთ GBST-ის საჩვენებელი ენა."), "개발자 YouTube 채널로 이동합니다." => Some("გახსენით დეველოპერის YouTube არხი."), "프로그램 업데이트" => Some("პროგრამის განახლება"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("შეამოწმეთ GBST-ის უახლესი ვერსია."), "확인" => Some("შემოწმება"), "확인 중" => Some("მოწმდება"), "피드백" => Some("უკუკავშირი"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("თქვენი უკუკავშირი პროგრამის გაუმჯობესებას ეხმარება."), "새로운 업데이트가 있습니다." => Some("ხელმისაწვდომია ახალი განახლება."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("ნაპოვნია განახლებული ფაილი, რომელიც მიმდინარე ვერსიის პრობლემებს აგვარებს."), "파일 업데이트 (권장)" => Some("ფაილის განახლება (რეკომენდებულია)"), "다음에 하기" => Some("მოგვიანებით"), _ => None,
        },
        LanguageOption::Dutch => match key {
            "대시보드" => Some("Dashboard"), "Google 설치/복구" => Some("Google installeren/herstellen"), "로그 관리" => Some("Logboeken"), "설정" => Some("Instellingen"), "기기 관리" => Some("Apparaat"), "프로그램" => Some("Programma"), "작업 중" => Some("Bezig"), "대기 중" => Some("Inactief"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Beheer de GBST-status en hoofdfuncties op één scherm."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("Bekijk het taaklogboek en sla het op als tekstbestand."), "GBST 프로그램을 설정합니다." => Some("GBST configureren."), "모델명" => Some("Model"), "Android 버전" => Some("Android-versie"), "제조사" => Some("Fabrikant"), "롬 유형" => Some("ROM-type"), "구글 서비스 상태" => Some("Google Services-status"), "후원하기" => Some("Ontwikkelaar steunen"), "개발자에게 큰 힘과 응원이 됩니다." => Some("Uw steun helpt de ontwikkelaar enorm."), "이동" => Some("Ga"), "시작" => Some("Start"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Download APK's voor uw Android-versie en installeer Google Basic Services."), "개발자 유튜브" => Some("Ontwikkelaar YouTube"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Bekijk meer Lenovo-tabletprogramma's"), "작업 로그" => Some("Taaklogboek"), "ADB / APK Download / Package Install 진행 상태" => Some("ADB / APK-download / pakketinstallatie status"), "로그 내보내기" => Some("Log exporteren"), "로그 지우기" => Some("Log wissen"), "언어 변경" => Some("Taal"), "GBST 프로그램 표시 언어를 변경합니다." => Some("Wijzig de weergavetaal van GBST."), "개발자 YouTube 채널로 이동합니다." => Some("Open het YouTube-kanaal van de ontwikkelaar."), "프로그램 업데이트" => Some("Programma-update"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("Controleer de nieuwste GBST-release."), "확인" => Some("Controleren"), "확인 중" => Some("Controleren"), "피드백" => Some("Feedback"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Uw feedback helpt het programma te verbeteren."), "새로운 업데이트가 있습니다." => Some("Er is een nieuwe update beschikbaar."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("Er is een upgradebestand gevonden dat problemen in de huidige versie oplost."), "파일 업데이트 (권장)" => Some("Bestand bijwerken (aanbevolen)"), "다음에 하기" => Some("Later"), _ => None,
        },
        LanguageOption::Arabic => match key {
            "대시보드" => Some("لوحة المعلومات"), "Google 설치/복구" => Some("تثبيت/إصلاح Google"), "로그 관리" => Some("السجلات"), "설정" => Some("الإعدادات"), "기기 관리" => Some("الجهاز"), "프로그램" => Some("البرنامج"), "작업 중" => Some("قيد العمل"), "대기 중" => Some("في الانتظار"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("إدارة حالة GBST والوظائف الرئيسية من شاشة واحدة."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("عرض سجل العمل وحفظه كملف نصي."), "GBST 프로그램을 설정합니다." => Some("ضبط GBST."), "모델명" => Some("الطراز"), "Android 버전" => Some("إصدار Android"), "제조사" => Some("الشركة المصنّعة"), "롬 유형" => Some("نوع ROM"), "구글 서비스 상태" => Some("حالة خدمات Google"), "후원하기" => Some("دعم المطوّر"), "개발자에게 큰 힘과 응원이 됩니다." => Some("دعمك يساعد المطوّر كثيرًا."), "이동" => Some("انتقال"), "시작" => Some("بدء"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("تنزيل ملفات APK المناسبة لإصدار Android وتثبيت خدمات Google الأساسية."), "개발자 유튜브" => Some("YouTube المطوّر"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("استعرض المزيد من برامج أجهزة Lenovo اللوحية"), "작업 로그" => Some("سجل العمل"), "ADB / APK Download / Package Install 진행 상태" => Some("حالة ADB / تنزيل APK / تثبيت الحزم"), "로그 내보내기" => Some("تصدير السجل"), "로그 지우기" => Some("مسح السجل"), "언어 변경" => Some("اللغة"), "GBST 프로그램 표시 언어를 변경합니다." => Some("تغيير لغة عرض GBST."), "개발자 YouTube 채널로 이동합니다." => Some("فتح قناة المطوّر على YouTube."), "프로그램 업데이트" => Some("تحديث البرنامج"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("التحقق من أحدث إصدار GBST."), "확인" => Some("تحقق"), "확인 중" => Some("جارٍ التحقق"), "피드백" => Some("ملاحظات"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("ملاحظاتك تساعد على تحسين البرنامج."), "새로운 업데이트가 있습니다." => Some("يتوفر تحديث جديد."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("تم العثور على ملف ترقية يعالج مشاكل الإصدار الحالي."), "파일 업데이트 (권장)" => Some("تحديث الملف (موصى به)"), "다음에 하기" => Some("لاحقًا"), _ => None,
        },
        LanguageOption::Spanish => match key {
            "대시보드" => Some("Panel"), "Google 설치/복구" => Some("Instalar/Reparar Google"), "로그 관리" => Some("Registros"), "설정" => Some("Ajustes"), "기기 관리" => Some("Dispositivo"), "프로그램" => Some("Programa"), "작업 중" => Some("Trabajando"), "대기 중" => Some("En espera"), "GBST 작업 상태와 주요 기능을 한 화면에서 관리합니다." => Some("Gestiona el estado de GBST y sus funciones principales en una sola pantalla."), "작업 로그를 확인하고 텍스트 파일로 저장합니다." => Some("Revisa el registro de trabajo y guárdalo como archivo de texto."), "GBST 프로그램을 설정합니다." => Some("Configura GBST."), "모델명" => Some("Modelo"), "Android 버전" => Some("Versión de Android"), "제조사" => Some("Fabricante"), "롬 유형" => Some("Tipo de ROM"), "구글 서비스 상태" => Some("Estado de Google Services"), "후원하기" => Some("Apoyar al desarrollador"), "개발자에게 큰 힘과 응원이 됩니다." => Some("Tu apoyo ayuda mucho al desarrollador."), "이동" => Some("Ir"), "시작" => Some("Iniciar"), "Android 버전에 맞는 APK를 다운로드하고 Google 기본 서비스를 설치합니다." => Some("Descarga los APK para tu versión de Android e instala Google Basic Services."), "개발자 유튜브" => Some("YouTube del desarrollador"), "더 많은 레노버 태블릿 프로그램 찾아보기" => Some("Ver más programas para tablets Lenovo"), "작업 로그" => Some("Registro de trabajo"), "ADB / APK Download / Package Install 진행 상태" => Some("Estado de ADB / descarga APK / instalación de paquetes"), "로그 내보내기" => Some("Exportar registro"), "로그 지우기" => Some("Borrar registro"), "언어 변경" => Some("Idioma"), "GBST 프로그램 표시 언어를 변경합니다." => Some("Cambia el idioma de visualización de GBST."), "개발자 YouTube 채널로 이동합니다." => Some("Abrir el canal de YouTube del desarrollador."), "프로그램 업데이트" => Some("Actualización del programa"), "GBST 최신 릴리즈 버전을 확인합니다." => Some("Comprobar la última versión de GBST."), "확인" => Some("Comprobar"), "확인 중" => Some("Comprobando"), "피드백" => Some("Comentarios"), "의견을 주시면 프로그램이 완벽해질 수 있습니다." => Some("Tus comentarios ayudan a mejorar el programa."), "새로운 업데이트가 있습니다." => Some("Hay una nueva actualización."), "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => Some("Se detectó un archivo actualizado que corrige problemas de la versión actual."), "파일 업데이트 (권장)" => Some("Actualizar archivo (recomendado)"), "다음에 하기" => Some("Más tarde"), _ => None,
        },
    }
}



fn translate_dashboard_card_text(language: LanguageOption, key: &str) -> Option<&'static str> {
    match (language, key) {
        (LanguageOption::English, "Google 작업 시작") => Some("Start Google Task"),
        (LanguageOption::English, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nInstall/Repair/Update"),
        (LanguageOption::English, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Install, repair, and update\nGoogle Services features\nfor your Android version."),
        (LanguageOption::English, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Find useful programs\nfor Lenovo tablets."),

        (LanguageOption::Russian, "Google 작업 시작") => Some("Запуск задачи Google"),
        (LanguageOption::Russian, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nустановка/восстановление/обновление"),
        (LanguageOption::Russian, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Установите, восстановите и обновите\nфункции Google Services\nдля вашей версии Android."),
        (LanguageOption::Russian, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Найдите полезные программы\nдля планшетов Lenovo."),

        (LanguageOption::Japanese, "Google 작업 시작") => Some("Google作業を開始"),
        (LanguageOption::Japanese, "Google 서비스\n설치/복구/업데이트") => Some("Googleサービス\nインストール/修復/更新"),
        (LanguageOption::Japanese, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Androidバージョンに合わせて\nGoogleサービス機能を\nインストール、修復、更新します。"),
        (LanguageOption::Japanese, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Lenovoタブレットに役立つ\nプログラムを確認できます。"),

        (LanguageOption::TraditionalChinese, "Google 작업 시작") => Some("開始 Google 作業"),
        (LanguageOption::TraditionalChinese, "Google 서비스\n설치/복구/업데이트") => Some("Google 服務\n安裝/修復/更新"),
        (LanguageOption::TraditionalChinese, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("依照 Android 版本\n安裝、修復並更新\nGoogle 服務功能。"),
        (LanguageOption::TraditionalChinese, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("查看適合 Lenovo 平板的\n實用程式。"),

        (LanguageOption::Vietnamese, "Google 작업 시작") => Some("Bắt đầu tác vụ Google"),
        (LanguageOption::Vietnamese, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nCài đặt/khôi phục/cập nhật"),
        (LanguageOption::Vietnamese, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Cài đặt, khôi phục và cập nhật\ncác tính năng Google Services\nphù hợp với phiên bản Android."),
        (LanguageOption::Vietnamese, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Xem các chương trình hữu ích\ncho máy tính bảng Lenovo."),

        (LanguageOption::Greek, "Google 작업 시작") => Some("Έναρξη εργασίας Google"),
        (LanguageOption::Greek, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nΕγκατάσταση/επισκευή/ενημέρωση"),
        (LanguageOption::Greek, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Εγκατάσταση, επισκευή και ενημέρωση\nλειτουργιών Google Services\nγια την έκδοση Android."),
        (LanguageOption::Greek, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Δείτε χρήσιμα προγράμματα\nγια tablet Lenovo."),

        (LanguageOption::Hindi, "Google 작업 시작") => Some("Google कार्य शुरू करें"),
        (LanguageOption::Hindi, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nइंस्टॉल/रिपेयर/अपडेट"),
        (LanguageOption::Hindi, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("अपने Android संस्करण के अनुसार\nGoogle Services सुविधाएँ\nइंस्टॉल, रिपेयर और अपडेट करें।"),
        (LanguageOption::Hindi, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Lenovo टैबलेट के लिए\nउपयोगी प्रोग्राम देखें।"),

        (LanguageOption::Georgian, "Google 작업 시작") => Some("Google სამუშაოს დაწყება"),
        (LanguageOption::Georgian, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nდაყენება/აღდგენა/განახლება"),
        (LanguageOption::Georgian, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("დააყენეთ, აღადგინეთ და განაახლეთ\nGoogle Services-ის ფუნქციები\nAndroid-ის ვერსიის მიხედვით."),
        (LanguageOption::Georgian, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("იხილეთ Lenovo ტაბლეტებისთვის\nსასარგებლო პროგრამები."),

        (LanguageOption::Dutch, "Google 작업 시작") => Some("Google-taak starten"),
        (LanguageOption::Dutch, "Google 서비스\n설치/복구/업데이트") => Some("Google-services\ninstalleren/herstellen/bijwerken"),
        (LanguageOption::Dutch, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Installeer, herstel en werk\nGoogle Services-functies bij\nvoor uw Android-versie."),
        (LanguageOption::Dutch, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Bekijk handige programma's\nvoor Lenovo-tablets."),

        (LanguageOption::Arabic, "Google 작업 시작") => Some("بدء مهمة Google"),
        (LanguageOption::Arabic, "Google 서비스\n설치/복구/업데이트") => Some("خدمات Google\nتثبيت/إصلاح/تحديث"),
        (LanguageOption::Arabic, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("ثبّت ميزات خدمات Google وأصلحها وحدّثها\nبما يناسب إصدار Android\nعلى جهازك."),
        (LanguageOption::Arabic, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("اطّلع على برامج مفيدة\nلأجهزة Lenovo اللوحية."),

        (LanguageOption::Spanish, "Google 작업 시작") => Some("Iniciar tarea de Google"),
        (LanguageOption::Spanish, "Google 서비스\n설치/복구/업데이트") => Some("Google Services\nInstalar/Reparar/Actualizar"),
        (LanguageOption::Spanish, "Android 버전에 맞는\nGoogle 서비스 기능을\n설치, 복구, 업데이트 합니다.") => Some("Instala, repara y actualiza\nlas funciones de Google Services\npara tu versión de Android."),
        (LanguageOption::Spanish, "레노버 태블릿에 유용한\n프로그램을 확인하실 수 있습니다.") => Some("Consulta programas útiles\npara tablets Lenovo."),

        _ => None,
    }
}

pub fn translate_runtime_text(language: LanguageOption, content: &str) -> String {
    if language == LanguageOption::Korean || content.trim().is_empty() {
        return content.to_string();
    }

    let text = content.trim();

    if text == "오류:" {
        return phrase(language, "error_prefix").unwrap_or("Error:").to_string();
    }

    if let Some(value) = translate_status_value(language, text) {
        return value.to_string();
    }

    if let Some(exact) = translate_exact(language, text) {
        return exact.to_string();
    }

    if let Some(dynamic) = translate_dynamic_runtime_text(language, text) {
        return dynamic;
    }

    content.to_string()
}

fn translate_status_value(language: LanguageOption, text: &str) -> Option<&'static str> {
    Some(match text {
        "알 수 없음" => lang_text(language, "Unknown", "Неизвестно", "不明", "未知", "Không rõ", "Άγνωστο", "अज्ञात", "უცნობია", "Onbekend", "غير معروف", "Desconocido"),
        "정상" => lang_text(language, "Normal", "Нормально", "正常", "正常", "Bình thường", "Κανονικό", "सामान्य", "ნორმალური", "Normaal", "طبيعي", "Normal"),
        "복구 필요" => lang_text(language, "Repair needed", "Требуется восстановление", "修復が必要", "需要修復", "Cần sửa chữa", "Απαιτείται επιδιόρθωση", "मरम्मत आवश्यक", "საჭიროა აღდგენა", "Herstel nodig", "يلزم الإصلاح", "Requiere reparación"),
        "업데이트 필요" => lang_text(language, "Update needed", "Требуется обновление", "更新が必要", "需要更新", "Cần cập nhật", "Απαιτείται ενημέρωση", "अपडेट आवश्यक", "საჭიროა განახლება", "Update nodig", "يلزم التحديث", "Requiere actualización"),
        "감지 실패(복구 필요)" => lang_text(language, "Not detected (repair needed)", "Не обнаружено (требуется восстановление)", "検出失敗（修復が必要）", "偵測失敗（需要修復）", "Không phát hiện (cần sửa chữa)", "Δεν εντοπίστηκε (απαιτείται επιδιόρθωση)", "पता नहीं चला (मरम्मत आवश्यक)", "ვერ გამოვლინდა (საჭიროა აღდგენა)", "Niet gedetecteerd (herstel nodig)", "لم يتم الكشف (يلزم الإصلاح)", "No detectado (requiere reparación)"),
        "Lenovo 기기가 아닙니다." => lang_text(language, "Not a Lenovo device.", "Это не устройство Lenovo.", "Lenovo端末ではありません。", "不是 Lenovo 裝置。", "Không phải thiết bị Lenovo.", "Δεν είναι συσκευή Lenovo.", "यह Lenovo डिवाइस नहीं है।", "ეს Lenovo მოწყობილობა არ არის.", "Geen Lenovo-apparaat.", "ليس جهاز Lenovo.", "No es un dispositivo Lenovo."),
        "PRC(중국 내수롬)" => lang_text(language, "PRC (China ROM)", "PRC (китайская ROM)", "PRC（中国版ROM）", "PRC（中國版 ROM）", "PRC (ROM Trung Quốc)", "PRC (κινεζική ROM)", "PRC (चीन ROM)", "PRC (ჩინური ROM)", "PRC (China-ROM)", "PRC (روم الصين)", "PRC (ROM china)"),
        "ROW(글로벌롬)" => lang_text(language, "ROW (Global ROM)", "ROW (глобальная ROM)", "ROW（グローバルROM）", "ROW（全球版 ROM）", "ROW (ROM toàn cầu)", "ROW (παγκόσμια ROM)", "ROW (ग्लोबल ROM)", "ROW (გლობალური ROM)", "ROW (Global ROM)", "ROW (الروم العالمي)", "ROW (ROM global)"),
        _ => return None,
    })
}

fn translate_dynamic_runtime_text(language: LanguageOption, text: &str) -> Option<String> {
    if let Some(rest) = text.strip_prefix("[GBST] GUI 초기화 완료 / 언어 설정: ") {
        return Some(format!("{}{}", phrase(language, "gui_init")?, rest.trim()));
    }
    if text == "[GBST] 기기에 Google Service 설치, 복구, 업데이트를 시작합니다." {
        return Some(phrase(language, "gbst_start")?.to_string());
    }
    if text == "[APK] 다운로드한 파일 및 폴더 제거 완료" {
        return Some(phrase(language, "apk_cleanup_done")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[APK] 다운로드한 파일 및 폴더 제거...") {
        return Some(format!("{}{}", phrase(language, "apk_cleanup_working")?, rest));
    }
    if text == "[안내] PC(노트북)와 연결한 태블릿에 잠금 해제 → 메세지 창 왼쪽 중간 체크 박스 체크 → 오른쪽 하단 Allow(허용)를 터치해주세요." {
        return Some(phrase(language, "adb_guide")?.to_string());
    }
    if text == "[ADB] 기기 감지를 시작합니다." {
        return Some(phrase(language, "adb_detect_start")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[ADB] 기기 감지 중...") {
        return Some(format!("{}{}", phrase(language, "adb_detecting")?, rest));
    }
    if text == "[ADB] 기기 감지 완료" {
        return Some(phrase(language, "adb_detect_done")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[Device] 감지된 기기에 정보, 제조사: ") {
        return Some(format_device_info_log(language, rest));
    }
    if let Some(rest) = text.strip_prefix("[Device] 사용자가 설정한 화면 꺼짐 값: ") {
        return Some(format!("{}{}", phrase(language, "screen_timeout_value")?, rest.trim()));
    }
    if text == "[Device] 사용자가 설정한 값으로 복원합니다." {
        return Some(phrase(language, "restore_screen_timeout")?.to_string());
    }
    if text == "파일 다운로드" {
        return Some(phrase(language, "file_download_title")?.to_string());
    }
    if text == "Google Service APK 파일을 준비하고 있습니다." {
        return Some(phrase(language, "file_download_desc")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[APK] GitHub 텍스트 읽는 중...") {
        return Some(format!("{}{}", phrase(language, "github_reading")?, rest));
    }
    if text == "[APK] GitHub 텍스트를 확인했습니다." {
        return Some(phrase(language, "github_done")?.to_string());
    }
    if let Some(translated) = translate_apk_download_log(language, text) {
        return Some(translated);
    }
    if text == "[GBST] PC(노트북)에 파일 및 구성 요소 환경과 구글 서비스 설치, 복구, 업데이트를 준비합니다." {
        return Some(phrase(language, "prep")?.to_string());
    }
    if text == "[ADB] 기기에 연결할 준비합니다." {
        return Some(phrase(language, "connect_prep")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[ADB] 기기 재부팅 중...") {
        return Some(format!("{}{}", phrase(language, "rebooting")?, rest));
    }
    if text == "[ADB] 기기 재부팅 완료" {
        return Some(phrase(language, "reboot_done")?.to_string());
    }
    if text == "[ADB] OTA(업데이트) 알림 및 권한을 비활성화 합니다." {
        return Some(phrase(language, "ota_disable")?.to_string());
    }
    if text == "[안내] 동일한 질문을 주시는 분들이 많아 안내 드립니다." {
        return Some(phrase(language, "wait_guide_1")?.to_string());
    }
    if text == "[안내] 프로그램이 작업 중이므로 작업이 멈추거나, 중단되거나,"
        || text == "[안내] 프로그램 작업이 멈추거나, 중단되거나, 에러가 발생하거나,"
    {
        return Some(phrase(language, "wait_guide_2")?.to_string());
    }
    if text == "[안내] 연결이 끊긴 것이 아니니 안심하시고 작업이 완료 될 때까지" {
        return Some(phrase(language, "wait_guide_3")?.to_string());
    }
    if text == "[안내] PC(노트북), 케이블, 기기를 가만히 둔 상태로 1분~5분 대기해 주세요." {
        return Some(phrase(language, "wait_guide_4")?.to_string());
    }
    if let Some(translated) = translate_google_stage_log(language, text) {
        return Some(translated);
    }
    if text == "[Device] 설치된 Google Services APK 버전과 GBST 정보를 다시 조회합니다." {
        return Some(phrase(language, "post_install_refresh_start")?.to_string());
    }
    if let Some(status) = text.strip_prefix("[Device] GBST 기기 정보 재조회 완료, Google Services 상태: ") {
        return Some(format!(
            "{}{}",
            phrase(language, "post_install_refresh_done")?,
            translate_status_value(language, status.trim()).unwrap_or(status.trim())
        ));
    }
    if text == "[GBST] 작업을 마무리 합니다." {
        return Some(phrase(language, "finalize")?.to_string());
    }
    if text == "[GBST] Google Services 복구 및 업데이트가 완료 되었습니다." {
        return Some(phrase(language, "google_done")?.to_string());
    }
    if text == "[GBST] 기기에 Google Services APK 버전이 낮으므로 업데이트를 진행합니다." {
        return Some(phrase(language, "google_update_required_log")?.to_string());
    }
    if text == "[GBST] 기기에 Google Services가 정상적이지 않으므로 복구를 진행합니다." {
        return Some(phrase(language, "google_repair_required_log")?.to_string());
    }
    if text == "[GBST] 작업이 완료되었습니다." {
        return Some(phrase(language, "task_done")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[GBST] APK 사전 다운로드 준비 실패: ") {
        return Some(format!("{}{}", phrase(language, "startup_apk_prepare_failed")?, translate_runtime_text(language, rest)));
    }
    if let Some(rest) = text.strip_prefix("[GBST] 작업 실패: ") {
        return Some(format!("{}{}", phrase(language, "task_failed")?, translate_runtime_text(language, rest)));
    }
    if text == "[GBST] 작업 스레드 연결이 종료되었습니다." {
        return Some(phrase(language, "worker_closed")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[Log] GBST ") {
        if let Some((flow, path_part)) = rest.split_once(" 작업 로그를 ") {
            if let Some(path) = path_part.strip_suffix("에 저장합니다.") {
                return Some(format!("{} {} {} {}", phrase(language, "log_prefix")?, translate_flow_label(language, flow.trim()), phrase(language, "log_saved_to")?, path.trim()));
            }
        }
    }
    if let Some(rest) = text.strip_prefix("[Log] 작업 로그 자동 저장 실패: ") {
        return Some(format!("{}{}", phrase(language, "log_auto_save_failed")?, rest.trim()));
    }
    if let Some(rest) = text.strip_prefix("[Log] 저장 실패: ") {
        return Some(format!("{}{}", phrase(language, "log_save_failed")?, rest.trim()));
    }
    if let Some(rest) = text.strip_prefix("[Dashboard] 감지 실패: ") {
        return Some(format!("{}{}", phrase(language, "dashboard_detect_failed")?, rest.trim()));
    }
    if let Some(rest) = text.strip_prefix("[Update] ") {
        return translate_update_log(language, rest);
    }
    if let Some(rest) = text.strip_prefix("[설정] ") {
        return translate_settings_log(language, rest);
    }
    if text == "로그가 없습니다." {
        return Some(phrase(language, "no_logs")?.to_string());
    }
    if text == "본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다." {
        return Some(phrase(language, "lenovo_only")?.to_string());
    }
    if text == "APK 다운로드에 실패했습니다. GitHub Base64 링크 파일 또는 해독된 APK 다운로드 링크를 다시 확인해주세요." {
        return Some(phrase(language, "apk_download_failed")?.to_string());
    }
    if let Some(rest) = text.strip_prefix("[Error] 본 프로그램은 레노버 태블릿만 지원합니다. 작업을 중지합니다.") {
        return Some(format!("{}{}", phrase(language, "error_lenovo_only")?, rest));
    }

    if let Some(translated) = translate_common_error_text(language, text) {
        return Some(translated);
    }

    if let Some(translated) = translate_korean_log_fragments(language, text) {
        return Some(translated);
    }

    None
}

fn format_device_info_log(language: LanguageOption, rest: &str) -> String {
    let mut manufacturer = "";
    let mut model = "";
    let mut android = "";
    for part in rest.split(" / ") {
        let part = part.trim();
        if manufacturer.is_empty() {
            manufacturer = part.trim();
        } else if let Some(value) = part.strip_prefix("모델명: ") {
            model = value.trim();
        } else if let Some(value) = part.strip_prefix("Android 버전: ") {
            android = value.trim();
        }
    }
    format!(
        "{}{} / {}{} / {}{}",
        phrase(language, "device_info_prefix").unwrap_or("[Device] Detected device info, Manufacturer: "),
        manufacturer,
        phrase(language, "device_model_label").unwrap_or("Model: "),
        model,
        phrase(language, "device_android_label").unwrap_or("Android version: "),
        android,
    )
}

fn translate_apk_download_log(language: LanguageOption, text: &str) -> Option<String> {
    let rest = text.strip_prefix("[APK] ")?;
    if let Some((bar, after_bar)) = rest.split_once(" Android ") {
        if let Some((android, _)) = after_bar.split_once(" Google Service APK 파일 다운로드 시작") {
            return Some(format!("[APK] {} Android {} {}", bar.trim(), android.trim(), phrase(language, "apk_download_start")?));
        }
        if let Some((android, _)) = after_bar.split_once(" Google Service APK 파일 다운로드 완료") {
            return Some(format!("[APK] {} Android {} {}", bar.trim(), android.trim(), phrase(language, "apk_download_done")?));
        }
        if let Some((android, suffix)) = after_bar.split_once(" Google Service APK 파일을 다운로드 중...") {
            return Some(format!("[APK] {} Android {} {}{}", bar.trim(), android.trim(), phrase(language, "apk_downloading")?, suffix));
        }
    }
    None
}

fn translate_google_stage_log(language: LanguageOption, text: &str) -> Option<String> {
    let rest = text.strip_prefix("[APK] ")?;
    if let Some((bar, after_bar)) = rest.split_once(" 구글 서비스 작업 ") {
        if let Some((stage, _)) = after_bar.split_once("단계 완료") {
            return Some(format!("[APK] {} {} {}", bar.trim(), phrase(language, "google_stage_done")?, stage.trim()));
        }
    }
    if let Some((bar, after_bar)) = rest.split_once(" 구글 서비스 작업 중... (") {
        if let Some((stage, suffix)) = after_bar.split_once(")") {
            return Some(format!("[APK] {} {} ({}){}", bar.trim(), phrase(language, "google_stage_working")?, stage.trim(), suffix));
        }
    }
    None
}

fn translate_update_log(language: LanguageOption, rest: &str) -> Option<String> {
    if rest == "이미 최신 릴리즈 확인이 진행 중입니다." {
        return Some(phrase(language, "update_already_checking")?.to_string());
    }
    if rest == "최신 GBST 릴리즈를 확인합니다." {
        return Some(phrase(language, "update_checking")?.to_string());
    }
    if let Some(value) = rest.strip_prefix("새 GBST 버전을 찾았습니다: 현재 ") {
        return Some(format!("{}{}", phrase(language, "update_found")?, value));
    }
    if let Some(value) = rest.strip_prefix("릴리즈 ZIP 파일: ") {
        return Some(format!("{}{}", phrase(language, "update_asset")?, value));
    }
    if rest == "대시보드에 업데이트 안내 창을 표시합니다." {
        return Some(phrase(language, "update_notice_dashboard")?.to_string());
    }
    if let Some(value) = rest.strip_prefix("이미 최신 버전을 사용 중입니다: GBST ") {
        return Some(format!("{}GBST {}", phrase(language, "update_latest")?, value));
    }
    if let Some(value) = rest.strip_prefix("업데이트 확인 실패: ") {
        return Some(format!("{}{}", phrase(language, "update_failed")?, translate_runtime_text(language, value)));
    }
    if rest == "수동 확인을 위해 GitHub Releases 페이지를 엽니다." {
        return Some(phrase(language, "update_open_manual")?.to_string());
    }
    if let Some(value) = rest.strip_prefix("GitHub Releases 페이지 열기 실패: ") {
        return Some(format!("{}{}", phrase(language, "update_open_failed")?, value));
    }
    if rest == "새로운 업데이트 파일을 감지했습니다." {
        return Some(phrase(language, "update_detected")?.to_string());
    }
    if rest == "이번 업데이트 안내를 다음에 다시 확인합니다." {
        return Some(phrase(language, "update_later")?.to_string());
    }
    None
}


fn translate_common_error_text(language: LanguageOption, text: &str) -> Option<String> {
    let mut out = text.to_string();
    let replacements = [
        ("GitHub 릴리즈 정보를 가져오지 못했습니다", phrase(language, "err_github_release_info")?),
        ("GitHub 응답을 읽지 못했습니다", phrase(language, "err_github_response_read")?),
        ("GitHub 응답 JSON 파싱 실패", phrase(language, "err_github_json")?),
        ("GitHub 릴리즈 응답 형식이 예상과 다릅니다.", phrase(language, "err_github_format")?),
        ("확인 가능한 GBST 릴리즈 버전을 찾지 못했습니다.", phrase(language, "err_gbst_release_missing")?),
        ("태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요.", phrase(language, "err_no_usb_device")?),
        ("ADB 기기 감지 시간 초과. 마지막 상태:", phrase(language, "err_adb_timeout")?),
        ("USB ADB 기기를 찾지 못했습니다.", phrase(language, "err_usb_adb_missing")?),
        ("ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요.", phrase(language, "err_multiple_adb")?),
        ("ADB shell 실패", phrase(language, "err_adb_shell_failed")?),
        ("Lenovo 기기가 아닙니다. 감지된 제조사:", phrase(language, "err_not_lenovo_maker")?),
        ("Android 13~18만 지원합니다. 감지된 버전:", phrase(language, "err_android_supported")?),
        ("screen_off_timeout 복원 값이 올바르지 않습니다:", phrase(language, "err_screen_timeout_value")?),
    ];

    let original = out.clone();
    for (from, to) in replacements {
        out = out.replace(from, to);
    }

    if out != original { Some(out) } else { None }
}


fn translate_korean_log_fragments(language: LanguageOption, text: &str) -> Option<String> {
    let original = text.to_string();
    let replacements = [
        (
            "[안내] 프로그램 작업이 멈추거나, 중단되거나, 에러가 발생하거나,",
            phrase(language, "wait_guide_2")?,
        ),
        (
            "[안내] PC(노트북), 케이블, 기기를 가만히 둔 상태로 1분~5분 대기해 주세요.",
            phrase(language, "wait_guide_4")?,
        ),
        ("[경고]", lang_text(language, "[Warning]", "[Предупреждение]", "[警告]", "[警告]", "[Cảnh báo]", "[Προειδοποίηση]", "[चेतावनी]", "[გაფრთხილება]", "[Waarschuwing]", "[تحذير]", "[Advertencia]")),
        ("계속 진행", lang_text(language, "continuing", "продолжаем", "続行", "繼續執行", "tiếp tục", "συνέχεια", "जारी रखा जा रहा है", "გაგრძელება", "doorgaan", "متابعة", "continuando")),
        ("APK 설치 실패", lang_text(language, "APK install failed", "сбой установки APK", "APKインストール失敗", "APK 安裝失敗", "cài đặt APK thất bại", "αποτυχία εγκατάστασης APK", "APK इंस्टॉल विफल", "APK-ის დაყენება ვერ მოხერხდა", "APK-installatie mislukt", "فشل تثبيت APK", "falló la instalación del APK")),
        ("APK 설치 중", lang_text(language, "installing APK", "установка APK", "APKインストール中", "正在安裝 APK", "đang cài đặt APK", "εγκατάσταση APK", "APK इंस्टॉल हो रहा है", "APK-ის დაყენება მიმდინარეობს", "APK installeren", "جارٍ تثبيت APK", "instalando APK")),
        ("대기 실패", lang_text(language, "wait failed", "сбой ожидания", "待機失敗", "等待失敗", "chờ thất bại", "αποτυχία αναμονής", "प्रतीक्षा विफल", "ლოდინი ვერ მოხერხდა", "wachten mislukt", "فشل الانتظار", "falló la espera")),
        ("재부팅 실패", lang_text(language, "reboot failed", "сбой перезагрузки", "再起動失敗", "重新啟動失敗", "khởi động lại thất bại", "αποτυχία επανεκκίνησης", "रीबूट विफल", "გადატვირთვა ვერ მოხერხდა", "opnieuw opstarten mislukt", "فشل إعادة التشغيل", "falló el reinicio")),
        ("권한 적용 실패", lang_text(language, "permission apply failed", "сбой применения разрешений", "権限適用失敗", "權限套用失敗", "áp dụng quyền thất bại", "αποτυχία εφαρμογής δικαιωμάτων", "अनुमति लागू करने में विफल", "ნებართვების გამოყენება ვერ მოხერხდა", "machtigingen toepassen mislukt", "فشل تطبيق الأذونات", "falló la aplicación de permisos")),
        ("AppOps 적용 실패", lang_text(language, "AppOps apply failed", "сбой применения AppOps", "AppOps適用失敗", "AppOps 套用失敗", "áp dụng AppOps thất bại", "αποτυχία εφαρμογής AppOps", "AppOps लागू करने में विफल", "AppOps-ის გამოყენება ვერ მოხერხდა", "AppOps toepassen mislukt", "فشل تطبيق AppOps", "falló la aplicación de AppOps")),
        ("Android 13 추가 재부팅을 진행합니다.", lang_text(language, "Performing the additional Android 13 reboot.", "Выполняется дополнительная перезагрузка Android 13.", "Android 13の追加再起動を実行します。", "正在執行 Android 13 額外重新啟動。", "Đang thực hiện khởi động lại bổ sung cho Android 13.", "Εκτελείται πρόσθετη επανεκκίνηση Android 13.", "Android 13 के लिए अतिरिक्त रीबूट किया जा रहा है।", "Android 13-ის დამატებითი გადატვირთვა მიმდინარეობს.", "Extra Android 13-herstart uitvoeren.", "جارٍ تنفيذ إعادة التشغيل الإضافية لنظام Android 13.", "Realizando el reinicio adicional de Android 13.")),
        ("Android 13 추가 재부팅 실패", lang_text(language, "additional Android 13 reboot failed", "сбой дополнительной перезагрузки Android 13", "Android 13追加再起動失敗", "Android 13 額外重新啟動失敗", "khởi động lại bổ sung Android 13 thất bại", "αποτυχία πρόσθετης επανεκκίνησης Android 13", "Android 13 अतिरिक्त रीबूट विफल", "Android 13-ის დამატებითი გადატვირთვა ვერ მოხერხდა", "extra Android 13-herstart mislukt", "فشلت إعادة التشغيل الإضافية لنظام Android 13", "falló el reinicio adicional de Android 13")),
        ("부여할 런타임 권한 없음", lang_text(language, "no runtime permissions to grant", "нет разрешений времени выполнения для выдачи", "付与するランタイム権限はありません", "沒有可授予的執行階段權限", "không có quyền runtime để cấp", "δεν υπάρχουν δικαιώματα χρόνου εκτέλεσης για εκχώρηση", "देने के लिए कोई रनटाइम अनुमति नहीं", "მისანიჭებელი runtime ნებართვები არ არის", "geen runtime-machtigingen om toe te kennen", "لا توجد أذونات تشغيل لمنحها", "no hay permisos de tiempo de ejecución para conceder")),
        ("적용할 AppOps 없음", lang_text(language, "no AppOps to apply", "нет AppOps для применения", "適用するAppOpsはありません", "沒有可套用的 AppOps", "không có AppOps để áp dụng", "δεν υπάρχουν AppOps για εφαρμογή", "लागू करने के लिए कोई AppOps नहीं", "გამოსაყენებელი AppOps არ არის", "geen AppOps om toe te passen", "لا توجد AppOps لتطبيقها", "no hay AppOps para aplicar")),
        ("ADB USB 직접 연결 실패", lang_text(language, "ADB USB direct connection failed", "сбой прямого USB-подключения ADB", "ADB USB直接接続に失敗しました", "ADB USB 直接連線失敗", "kết nối trực tiếp ADB USB thất bại", "αποτυχία άμεσης σύνδεσης ADB USB", "ADB USB सीधा कनेक्शन विफल", "ADB USB პირდაპირი კავშირი ვერ მოხერხდა", "directe ADB USB-verbinding mislukt", "فشل الاتصال المباشر عبر ADB USB", "falló la conexión directa ADB USB")),
        ("ADB USB 기기 검색 실패", lang_text(language, "ADB USB device search failed", "сбой поиска USB-устройства ADB", "ADB USB端末の検索に失敗しました", "ADB USB 裝置搜尋失敗", "tìm thiết bị ADB USB thất bại", "αποτυχία αναζήτησης συσκευής ADB USB", "ADB USB डिवाइस खोज विफल", "ADB USB მოწყობილობის ძებნა ვერ მოხერხდა", "zoeken naar ADB USB-apparaat mislukt", "فشل البحث عن جهاز ADB USB", "falló la búsqueda del dispositivo ADB USB")),
        ("ADB USB 기기 감지 대기 중입니다.", lang_text(language, "Waiting for an ADB USB device.", "Ожидание USB-устройства ADB.", "ADB USB端末を待機中です。", "正在等待 ADB USB 裝置。", "Đang chờ thiết bị ADB USB.", "Αναμονή για συσκευή ADB USB.", "ADB USB डिवाइस की प्रतीक्षा हो रही है।", "ADB USB მოწყობილობას ელოდება.", "Wachten op een ADB USB-apparaat.", "في انتظار جهاز ADB USB.", "Esperando un dispositivo ADB USB.")),
        ("ADB unauthorized 상태입니다.", lang_text(language, "ADB is unauthorized.", "ADB не авторизован.", "ADBが未承認状態です。", "ADB 未授權。", "ADB chưa được ủy quyền.", "Το ADB δεν είναι εξουσιοδοτημένο.", "ADB अनधिकृत है।", "ADB არაავტორიზებულია.", "ADB is niet geautoriseerd.", "ADB غير مصرح به.", "ADB no está autorizado.")),
        ("외부 adb server가 USB ADB 인터페이스를 점유 중입니다.", lang_text(language, "An external adb server is occupying the USB ADB interface.", "Внешний adb server занимает интерфейс USB ADB.", "外部adb serverがUSB ADBインターフェースを使用中です。", "外部 adb server 正在占用 USB ADB 介面。", "Máy chủ adb bên ngoài đang chiếm giao diện USB ADB.", "Ένας εξωτερικός adb server χρησιμοποιεί τη διεπαφή USB ADB.", "एक बाहरी adb server USB ADB इंटरफ़ेस का उपयोग कर रहा है।", "გარე adb server USB ADB ინტერფეისს იკავებს.", "Een externe adb-server gebruikt de USB ADB-interface.", "يشغل خادم adb خارجي واجهة USB ADB.", "Un servidor adb externo está ocupando la interfaz USB ADB.")),
        ("USB ADB 기기를 찾지 못했습니다.", phrase(language, "err_usb_adb_missing")?),
        ("ADB 기기가 2대 이상 연결되어 있습니다. 하나만 연결해주세요.", phrase(language, "err_multiple_adb")?),
        ("태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요.", phrase(language, "err_no_usb_device")?),
        ("Lenovo 기기가 아닙니다. 감지된 제조사:", phrase(language, "err_not_lenovo_maker")?),
        ("Android 13~18만 지원합니다. 감지된 버전:", phrase(language, "err_android_supported")?),
        ("ADB shell 실패", phrase(language, "err_adb_shell_failed")?),
        ("ADB 오류", lang_text(language, "ADB error", "ошибка ADB", "ADBエラー", "ADB 錯誤", "lỗi ADB", "σφάλμα ADB", "ADB त्रुटि", "ADB შეცდომა", "ADB-fout", "خطأ ADB", "error de ADB")),
        ("다운로드 실패", lang_text(language, "download failed", "сбой загрузки", "ダウンロード失敗", "下載失敗", "tải xuống thất bại", "αποτυχία λήψης", "डाउनलोड विफल", "ჩამოტვირთვა ვერ მოხერხდა", "download mislukt", "فشل التنزيل", "falló la descarga")),
        ("I/O 오류", lang_text(language, "I/O error", "ошибка I/O", "I/Oエラー", "I/O 錯誤", "lỗi I/O", "σφάλμα I/O", "I/O त्रुटि", "I/O შეცდომა", "I/O-fout", "خطأ I/O", "error de E/S")),
        ("APK 링크 파일 파싱 오류", lang_text(language, "APK link file parsing error", "ошибка разбора файла ссылок APK", "APKリンクファイル解析エラー", "APK 連結檔案解析錯誤", "lỗi phân tích tệp liên kết APK", "σφάλμα ανάλυσης αρχείου συνδέσμων APK", "APK लिंक फ़ाइल पार्सिंग त्रुटि", "APK ბმულების ფაილის დამუშავების შეცდომა", "APK-linkbestand parseren mislukt", "خطأ في تحليل ملف روابط APK", "error al analizar el archivo de enlaces APK")),
        ("지원하지 않는 Android 버전입니다", lang_text(language, "unsupported Android version", "неподдерживаемая версия Android", "対応していないAndroidバージョンです", "不支援的 Android 版本", "phiên bản Android không được hỗ trợ", "μη υποστηριζόμενη έκδοση Android", "असमर्थित Android संस्करण", "Android-ის მხარდაუჭერელი ვერსია", "niet-ondersteunde Android-versie", "إصدار Android غير مدعوم", "versión de Android no compatible")),
        ("기기 검증 실패", lang_text(language, "device verification failed", "сбой проверки устройства", "端末検証失敗", "裝置驗證失敗", "xác minh thiết bị thất bại", "αποτυχία επαλήθευσης συσκευής", "डिवाइस सत्यापन विफल", "მოწყობილობის შემოწმება ვერ მოხერხდა", "apparaatverificatie mislukt", "فشل التحقق من الجهاز", "falló la verificación del dispositivo")),
        ("필수 파일이 없습니다", lang_text(language, "required file is missing", "отсутствует обязательный файл", "必須ファイルがありません", "缺少必要檔案", "thiếu tệp bắt buộc", "λείπει απαιτούμενο αρχείο", "आवश्यक फ़ाइल गायब है", "სავალდებულო ფაილი აკლია", "vereist bestand ontbreekt", "الملف المطلوب مفقود", "falta un archivo requerido")),
        ("필수 패키지 APK 링크가 없습니다", lang_text(language, "required package APK link is missing", "отсутствует ссылка APK обязательного пакета", "必須パッケージのAPKリンクがありません", "缺少必要套件 APK 連結", "thiếu liên kết APK gói bắt buộc", "λείπει σύνδεσμος APK απαιτούμενου πακέτου", "आवश्यक पैकेज APK लिंक गायब है", "სავალდებულო პაკეტის APK ბმული აკლია", "APK-link van vereist pakket ontbreekt", "رابط APK للحزمة المطلوبة مفقود", "falta el enlace APK del paquete requerido")),
        ("GitHub Base64 링크 파일 다운로드 실패", lang_text(language, "failed to download the GitHub Base64 link file", "не удалось загрузить файл ссылок GitHub Base64", "GitHub Base64リンクファイルのダウンロードに失敗しました", "GitHub Base64 連結檔案下載失敗", "tải tệp liên kết GitHub Base64 thất bại", "αποτυχία λήψης αρχείου συνδέσμων GitHub Base64", "GitHub Base64 लिंक फ़ाइल डाउनलोड विफल", "GitHub Base64 ბმულის ფაილის ჩამოტვირთვა ვერ მოხერხდა", "GitHub Base64-linkbestand downloaden mislukt", "فشل تنزيل ملف روابط GitHub Base64", "falló la descarga del archivo de enlaces GitHub Base64")),
        ("GitHub Base64 링크 파일 읽기 실패", lang_text(language, "failed to read the GitHub Base64 link file", "не удалось прочитать файл ссылок GitHub Base64", "GitHub Base64リンクファイルの読み取りに失敗しました", "GitHub Base64 連結檔案讀取失敗", "đọc tệp liên kết GitHub Base64 thất bại", "αποτυχία ανάγνωσης αρχείου συνδέσμων GitHub Base64", "GitHub Base64 लिंक फ़ाइल पढ़ना विफल", "GitHub Base64 ბმულის ფაილის წაკითხვა ვერ მოხერხდა", "GitHub Base64-linkbestand lezen mislukt", "فشل قراءة ملف روابط GitHub Base64", "falló la lectura del archivo de enlaces GitHub Base64")),
        ("Base64 해독 실패", lang_text(language, "Base64 decoding failed", "сбой декодирования Base64", "Base64デコード失敗", "Base64 解碼失敗", "giải mã Base64 thất bại", "αποτυχία αποκωδικοποίησης Base64", "Base64 डिकोडिंग विफल", "Base64-ის დეკოდირება ვერ მოხერხდა", "Base64 decoderen mislukt", "فشل فك ترميز Base64", "falló la decodificación Base64")),
        ("Base64 해독 결과가 UTF-8 텍스트가 아닙니다", lang_text(language, "Base64 decoded result is not UTF-8 text", "результат декодирования Base64 не является текстом UTF-8", "Base64デコード結果がUTF-8テキストではありません", "Base64 解碼結果不是 UTF-8 文字", "kết quả giải mã Base64 không phải văn bản UTF-8", "το αποτέλεσμα αποκωδικοποίησης Base64 δεν είναι κείμενο UTF-8", "Base64 डिकोड परिणाम UTF-8 टेक्स्ट नहीं है", "Base64 დეკოდირებული შედეგი UTF-8 ტექსტი არ არის", "Base64-gedecodeerd resultaat is geen UTF-8-tekst", "نتيجة فك ترميز Base64 ليست نص UTF-8", "el resultado decodificado de Base64 no es texto UTF-8")),
        ("링크는 http/https로 시작해야 합니다", lang_text(language, "the link must start with http/https", "ссылка должна начинаться с http/https", "リンクはhttp/httpsで始まる必要があります", "連結必須以 http/https 開頭", "liên kết phải bắt đầu bằng http/https", "ο σύνδεσμος πρέπει να ξεκινά με http/https", "लिंक http/https से शुरू होना चाहिए", "ბმული უნდა იწყებოდეს http/https-ით", "de link moet beginnen met http/https", "يجب أن يبدأ الرابط بـ http/https", "el enlace debe comenzar con http/https")),
        ("패키지명 또는 링크가 비어 있습니다", lang_text(language, "the package name or link is empty", "имя пакета или ссылка пустые", "パッケージ名またはリンクが空です", "套件名稱或連結為空", "tên gói hoặc liên kết đang trống", "το όνομα πακέτου ή ο σύνδεσμος είναι κενό", "पैकेज नाम या लिंक खाली है", "პაკეტის სახელი ან ბმული ცარიელია", "pakketnaam of link is leeg", "اسم الحزمة أو الرابط فارغ", "el nombre del paquete o el enlace está vacío")),
        ("패키지명이 올바르지 않습니다", lang_text(language, "the package name is invalid", "имя пакета недопустимо", "パッケージ名が正しくありません", "套件名稱無效", "tên gói không hợp lệ", "το όνομα πακέτου δεν είναι έγκυρο", "पैकेज नाम अमान्य है", "პაკეტის სახელი არასწორია", "pakketnaam is ongeldig", "اسم الحزمة غير صالح", "el nombre del paquete no es válido")),
    ];

    let mut out = original.clone();
    for (from, to) in replacements {
        out = out.replace(from, to);
    }

    if out != original { Some(out) } else { None }
}

fn translate_settings_log(language: LanguageOption, rest: &str) -> Option<String> {
    if let Some(value) = rest.strip_prefix("YouTube 링크 열기 실패: ") {
        return Some(format!("{}{}", phrase(language, "youtube_open_failed")?, value));
    }
    if let Some(value) = rest.strip_prefix("후원 링크 열기 실패: ") {
        return Some(format!("{}{}", phrase(language, "donate_open_failed")?, value));
    }
    if let Some(value) = rest.strip_prefix("피드백 링크 열기 실패: ") {
        return Some(format!("{}{}", phrase(language, "feedback_open_failed")?, value));
    }
    if let Some(value) = rest.strip_prefix("언어 저장 실패: ") {
        return Some(format!("{}{}", phrase(language, "language_save_failed")?, value));
    }
    None
}

fn translate_flow_label(language: LanguageOption, flow: &str) -> &'static str {
    match flow {
        "완료" => phrase(language, "flow_complete").unwrap_or("completed"),
        "실패" => phrase(language, "flow_failed").unwrap_or("failed"),
        "종료" => phrase(language, "flow_closed").unwrap_or("closed"),
        "수동" => phrase(language, "flow_manual").unwrap_or("manual"),
        _ => "task",
    }
}

fn lang_text(
    language: LanguageOption,
    en: &'static str,
    ru: &'static str,
    ja: &'static str,
    zh: &'static str,
    vi: &'static str,
    el: &'static str,
    hi: &'static str,
    ka: &'static str,
    nl: &'static str,
    ar: &'static str,
    es: &'static str,
) -> &'static str {
    match language {
        LanguageOption::Korean => en,
        LanguageOption::English => en,
        LanguageOption::Russian => ru,
        LanguageOption::Japanese => ja,
        LanguageOption::TraditionalChinese => zh,
        LanguageOption::Vietnamese => vi,
        LanguageOption::Greek => el,
        LanguageOption::Hindi => hi,
        LanguageOption::Georgian => ka,
        LanguageOption::Dutch => nl,
        LanguageOption::Arabic => ar,
        LanguageOption::Spanish => es,
    }
}

fn phrase(language: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "gui_init" => lang_text(language, "[GBST] GUI initialization complete / Language: ", "[GBST] Инициализация GUI завершена / Язык: ", "[GBST] GUI初期化完了 / 言語: ", "[GBST] GUI 初始化完成 / 語言: ", "[GBST] Đã khởi tạo GUI / Ngôn ngữ: ", "[GBST] Η αρχικοποίηση GUI ολοκληρώθηκε / Γλώσσα: ", "[GBST] GUI आरंभ पूर्ण / भाषा: ", "[GBST] GUI ინიციალიზაცია დასრულდა / ენა: ", "[GBST] GUI geïnitialiseerd / Taal: ", "[GBST] اكتمل تهيئة الواجهة / اللغة: ", "[GBST] GUI inicializada / Idioma: "),
        "gbst_start" => lang_text(language, "[GBST] Starting Google Service installation, repair, and update on the device.", "[GBST] Запуск установки, восстановления и обновления Google Service на устройстве.", "[GBST] 端末でGoogle Serviceのインストール、修復、更新を開始します。", "[GBST] 開始在裝置上安裝、修復與更新 Google Service。", "[GBST] Bắt đầu cài đặt, sửa chữa và cập nhật Google Service trên thiết bị.", "[GBST] Ξεκινά η εγκατάσταση, επιδιόρθωση και ενημέρωση του Google Service στη συσκευή.", "[GBST] डिवाइस पर Google Service इंस्टॉल, मरम्मत और अपडेट शुरू हो रहा है।", "[GBST] მოწყობილობაზე Google Service-ის დაყენება, აღდგენა და განახლება იწყება.", "[GBST] Google Service installeren, herstellen en bijwerken op het apparaat wordt gestart.", "[GBST] بدء تثبيت وإصلاح وتحديث Google Service على الجهاز.", "[GBST] Iniciando instalación, reparación y actualización de Google Service en el dispositivo."),
        "apk_cleanup_working" => lang_text(language, "[APK] Removing downloaded files and folders...", "[APK] Удаление загруженных файлов и папок...", "[APK] ダウンロード済みファイルとフォルダーを削除中...", "[APK] 正在移除已下載的檔案與資料夾...", "[APK] Đang xóa tệp và thư mục đã tải...", "[APK] Αφαίρεση ληφθέντων αρχείων και φακέλων...", "[APK] डाउनलोड की गई फ़ाइलें और फ़ोल्डर हटाए जा रहे हैं...", "[APK] ჩამოტვირთული ფაილები და საქაღალდეები იშლება...", "[APK] Gedownloade bestanden en mappen verwijderen...", "[APK] جارٍ إزالة الملفات والمجلدات التي تم تنزيلها...", "[APK] Eliminando archivos y carpetas descargados..."),
        "apk_cleanup_done" => lang_text(language, "[APK] Downloaded files and folders removed.", "[APK] Загруженные файлы и папки удалены.", "[APK] ダウンロード済みファイルとフォルダーを削除しました。", "[APK] 已移除下載的檔案與資料夾。", "[APK] Đã xóa tệp và thư mục đã tải.", "[APK] Τα ληφθέντα αρχεία και οι φάκελοι αφαιρέθηκαν.", "[APK] डाउनलोड की गई फ़ाइलें और फ़ोल्डर हटा दिए गए।", "[APK] ჩამოტვირთული ფაილები და საქაღალდეები წაიშალა.", "[APK] Gedownloade bestanden en mappen verwijderd.", "[APK] تمت إزالة الملفات والمجلدات التي تم تنزيلها.", "[APK] Archivos y carpetas descargados eliminados."),
        "adb_guide" => lang_text(language, "[Guide] Unlock the tablet connected to the PC, check the box in the middle-left of the prompt, then tap Allow at the bottom-right.", "[Инфо] Разблокируйте планшет, подключённый к ПК, отметьте флажок слева в окне запроса и нажмите Allow справа внизу.", "[案内] PCに接続したタブレットのロックを解除し、メッセージ左中央のチェックボックスをオンにして、右下のAllowをタップしてください。", "[指南] 請解鎖連接到 PC 的平板，勾選提示視窗左側中間的核取方塊，然後點選右下角 Allow。", "[Hướng dẫn] Mở khóa máy tính bảng đang kết nối với PC, chọn ô ở giữa bên trái của thông báo rồi nhấn Allow ở góc dưới bên phải.", "[Οδηγός] Ξεκλειδώστε το tablet που είναι συνδεδεμένο στον υπολογιστή, επιλέξτε το πλαίσιο αριστερά στο μήνυμα και πατήστε Allow κάτω δεξιά.", "[मार्गदर्शन] PC से जुड़े टैबलेट को अनलॉक करें, संदेश विंडो में बाएँ बीच का चेक बॉक्स चुनें, फिर नीचे दाईं ओर Allow टैप करें।", "[გზამკვლევი] განბლოკეთ PC-სთან დაკავშირებული ტაბლეტი, მონიშნეთ მოთხოვნის მარცხენა შუა მხარეს ჩექბოქსი და ქვედა მარჯვენა მხარეს დააჭირეთ Allow-ს.", "[Gids] Ontgrendel de tablet die met de pc is verbonden, vink het vakje links in de melding aan en tik rechtsonder op Allow.", "[إرشاد] افتح قفل الجهاز اللوحي المتصل بالكمبيوتر، وحدد المربع في منتصف الجهة اليسرى من الرسالة، ثم اضغط Allow في أسفل اليمين.", "[Guía] Desbloquea la tablet conectada al PC, marca la casilla en la parte central izquierda del mensaje y toca Allow abajo a la derecha."),
        "adb_detect_start" => lang_text(language, "[ADB] Starting device detection.", "[ADB] Запуск обнаружения устройства.", "[ADB] 端末検出を開始します。", "[ADB] 開始偵測裝置。", "[ADB] Bắt đầu phát hiện thiết bị.", "[ADB] Έναρξη εντοπισμού συσκευής.", "[ADB] डिवाइस पहचान शुरू हो रही है।", "[ADB] მოწყობილობის აღმოჩენა იწყება.", "[ADB] Apparaatdetectie starten.", "[ADB] بدء اكتشاف الجهاز.", "[ADB] Iniciando detección del dispositivo."),
        "adb_detecting" => lang_text(language, "[ADB] Detecting device...", "[ADB] Обнаружение устройства...", "[ADB] 端末を検出中...", "[ADB] 正在偵測裝置...", "[ADB] Đang phát hiện thiết bị...", "[ADB] Εντοπισμός συσκευής...", "[ADB] डिवाइस पहचाना जा रहा है...", "[ADB] მოწყობილობა იძებნება...", "[ADB] Apparaat detecteren...", "[ADB] جارٍ اكتشاف الجهاز...", "[ADB] Detectando dispositivo..."),
        "adb_detect_done" => lang_text(language, "[ADB] Device detected.", "[ADB] Устройство обнаружено.", "[ADB] 端末検出完了。", "[ADB] 已偵測到裝置。", "[ADB] Đã phát hiện thiết bị.", "[ADB] Η συσκευή εντοπίστηκε.", "[ADB] डिवाइस मिल गया।", "[ADB] მოწყობილობა აღმოჩენილია.", "[ADB] Apparaat gedetecteerd.", "[ADB] تم اكتشاف الجهاز.", "[ADB] Dispositivo detectado."),
        "device_info_prefix" => lang_text(language, "[Device] Detected device info, Manufacturer: ", "[Device] Обнаружено устройство, производитель: ", "[Device] 検出された端末情報、メーカー: ", "[Device] 偵測到的裝置資訊，製造商: ", "[Device] Thông tin thiết bị đã phát hiện, nhà sản xuất: ", "[Device] Πληροφορίες εντοπισμένης συσκευής, κατασκευαστής: ", "[Device] पहचाने गए डिवाइस की जानकारी, निर्माता: ", "[Device] აღმოჩენილი მოწყობილობის ინფორმაცია, მწარმოებელი: ", "[Device] Gedetecteerde apparaatinformatie, fabrikant: ", "[Device] معلومات الجهاز المكتشف، الشركة المصنّعة: ", "[Device] Información del dispositivo detectado, fabricante: "),
        "device_model_label" => lang_text(language, "Model: ", "Модель: ", "モデル名: ", "型號: ", "Mẫu: ", "Μοντέλο: ", "मॉडल: ", "მოდელი: ", "Model: ", "الطراز: ", "Modelo: "),
        "device_android_label" => lang_text(language, "Android version: ", "Версия Android: ", "Androidバージョン: ", "Android 版本: ", "Phiên bản Android: ", "Έκδοση Android: ", "Android संस्करण: ", "Android ვერსია: ", "Android-versie: ", "إصدار Android: ", "Versión de Android: "),
        "screen_timeout_value" => lang_text(language, "[Device] User's screen timeout value: ", "[Device] Значение тайм-аута экрана пользователя: ", "[Device] ユーザー設定の画面消灯時間: ", "[Device] 使用者設定的螢幕逾時值: ", "[Device] Giá trị thời gian tắt màn hình của người dùng: ", "[Device] Τιμή χρονικού ορίου οθόνης του χρήστη: ", "[Device] उपयोगकर्ता का स्क्रीन टाइमआउट मान: ", "[Device] მომხმარებლის ეკრანის გამორთვის მნიშვნელობა: ", "[Device] Schermtime-outwaarde van gebruiker: ", "[Device] قيمة مهلة إيقاف الشاشة للمستخدم: ", "[Device] Valor de tiempo de espera de pantalla del usuario: "),
        "restore_screen_timeout" => lang_text(language, "[Device] Restoring the user's original setting.", "[Device] Восстановление исходной настройки пользователя.", "[Device] ユーザーの元の設定に復元します。", "[Device] 正在還原使用者原本的設定。", "[Device] Đang khôi phục cài đặt ban đầu của người dùng.", "[Device] Επαναφορά της αρχικής ρύθμισης του χρήστη.", "[Device] उपयोगकर्ता की मूल सेटिंग बहाल की जा रही है।", "[Device] მომხმარებლის საწყისი პარამეტრი აღდგება.", "[Device] Oorspronkelijke gebruikersinstelling herstellen.", "[Device] جارٍ استعادة إعداد المستخدم الأصلي.", "[Device] Restaurando la configuración original del usuario."),
        "github_reading" => lang_text(language, "[APK] Reading GitHub text...", "[APK] Чтение текста GitHub...", "[APK] GitHubテキストを読み込み中...", "[APK] 正在讀取 GitHub 文字...", "[APK] Đang đọc văn bản GitHub...", "[APK] Ανάγνωση κειμένου GitHub...", "[APK] GitHub टेक्स्ट पढ़ा जा रहा है...", "[APK] GitHub ტექსტი იკითხება...", "[APK] GitHub-tekst lezen...", "[APK] جارٍ قراءة نص GitHub...", "[APK] Leyendo texto de GitHub..."),
        "github_done" => lang_text(language, "[APK] GitHub text has been verified.", "[APK] Текст GitHub проверен.", "[APK] GitHubテキストを確認しました。", "[APK] 已確認 GitHub 文字。", "[APK] Đã xác nhận văn bản GitHub.", "[APK] Το κείμενο GitHub επιβεβαιώθηκε.", "[APK] GitHub टेक्स्ट की पुष्टि हो गई।", "[APK] GitHub ტექსტი დადასტურდა.", "[APK] GitHub-tekst is gecontroleerd.", "[APK] تم التحقق من نص GitHub.", "[APK] Texto de GitHub verificado."),
        "file_download_title" => lang_text(language, "File download", "Загрузка файла", "ファイルのダウンロード", "檔案下載", "Tải tệp", "Λήψη αρχείου", "फ़ाइल डाउनलोड", "ფაილის ჩამოტვირთვა", "Bestand downloaden", "تنزيل الملف", "Descarga de archivo"),
        "file_download_desc" => lang_text(language, "Preparing the Google Service APK file.", "Подготовка APK-файла Google Service.", "Google Service APKファイルを準備しています。", "正在準備 Google Service APK 檔案。", "Đang chuẩn bị tệp APK Google Service.", "Προετοιμασία αρχείου APK Google Service.", "Google Service APK फ़ाइल तैयार की जा रही है।", "Google Service APK ფაილი მზადდება.", "Google Service APK-bestand voorbereiden.", "جارٍ تحضير ملف APK الخاص بـ Google Service.", "Preparando el archivo APK de Google Service."),
        "apk_download_start" => lang_text(language, "starting Google Service APK file download", "начинается загрузка APK-файла Google Service", "Google Service APKファイルのダウンロードを開始", "開始下載 Google Service APK 檔案", "bắt đầu tải tệp APK Google Service", "έναρξη λήψης αρχείου APK Google Service", "Google Service APK फ़ाइल डाउनलोड शुरू", "Google Service APK ფაილის ჩამოტვირთვა იწყება", "download van Google Service APK-bestand starten", "بدء تنزيل ملف APK الخاص بـ Google Service", "iniciando descarga del archivo APK de Google Service"),
        "apk_download_done" => lang_text(language, "Google Service APK file download complete", "загрузка APK-файлов Google Service завершена", "Google Service APKファイルのダウンロード完了", "Google Service APK 檔案下載完成", "đã tải xong tệp APK Google Service", "ολοκληρώθηκε η λήψη αρχείων APK Google Service", "Google Service APK फ़ाइल डाउनलोड पूर्ण", "Google Service APK ფაილის ჩამოტვირთვა დასრულდა", "download van Google Service APK-bestand voltooid", "اكتمل تنزيل ملف APK الخاص بـ Google Service", "descarga de archivos APK de Google Service completa"),
        "apk_downloading" => lang_text(language, "downloading Google Service APK file...", "загрузка APK-файла Google Service...", "Google Service APKファイルをダウンロード中...", "正在下載 Google Service APK 檔案...", "đang tải tệp APK Google Service...", "γίνεται λήψη αρχείου APK Google Service...", "Google Service APK फ़ाइल डाउनलोड हो रही है...", "Google Service APK ფაილი იტვირთება...", "Google Service APK-bestand downloaden...", "جارٍ تنزيل ملف APK الخاص بـ Google Service...", "descargando archivo APK de Google Service..."),
        "prep" => lang_text(language, "[GBST] Preparing PC files, components, and the Google service installation, repair, and update environment.", "[GBST] Подготовка файлов ПК, компонентов и среды установки, восстановления и обновления Google service.", "[GBST] PC側のファイル、構成要素、Googleサービスのインストール・修復・更新環境を準備します。", "[GBST] 正在準備 PC 檔案、元件，以及 Google 服務安裝、修復與更新環境。", "[GBST] Đang chuẩn bị tệp, thành phần trên PC và môi trường cài đặt, sửa chữa, cập nhật Google service.", "[GBST] Προετοιμασία αρχείων υπολογιστή, στοιχείων και περιβάλλοντος εγκατάστασης, επιδιόρθωσης και ενημέρωσης Google service.", "[GBST] PC फ़ाइलें, घटक और Google service इंस्टॉल/मरम्मत/अपडेट वातावरण तैयार किया जा रहा है।", "[GBST] მზადდება PC ფაილები, კომპონენტები და Google service-ის დაყენება/აღდგენა/განახლების გარემო.", "[GBST] Pc-bestanden, onderdelen en de omgeving voor Google service installeren, herstellen en bijwerken voorbereiden.", "[GBST] جارٍ تحضير ملفات الكمبيوتر والمكوّنات وبيئة تثبيت وإصلاح وتحديث Google service.", "[GBST] Preparando archivos, componentes y entorno de instalación, reparación y actualización de Google service en el PC."),
        "connect_prep" => lang_text(language, "[ADB] Preparing to connect to the device.", "[ADB] Подготовка к подключению к устройству.", "[ADB] 端末へ接続する準備をしています。", "[ADB] 正在準備連接裝置。", "[ADB] Đang chuẩn bị kết nối với thiết bị.", "[ADB] Προετοιμασία σύνδεσης στη συσκευή.", "[ADB] डिवाइस से कनेक्ट करने की तैयारी हो रही है।", "[ADB] მოწყობილობასთან დასაკავშირებლად მზადება.", "[ADB] Verbinding met het apparaat voorbereiden.", "[ADB] جارٍ التحضير للاتصال بالجهاز.", "[ADB] Preparando la conexión con el dispositivo."),
        "rebooting" => lang_text(language, "[ADB] Rebooting device...", "[ADB] Перезагрузка устройства...", "[ADB] 端末を再起動中...", "[ADB] 正在重新啟動裝置...", "[ADB] Đang khởi động lại thiết bị...", "[ADB] Επανεκκίνηση συσκευής...", "[ADB] डिवाइस रीबूट हो रहा है...", "[ADB] მოწყობილობა გადაიტვირთება...", "[ADB] Apparaat opnieuw opstarten...", "[ADB] جارٍ إعادة تشغيل الجهاز...", "[ADB] Reiniciando dispositivo..."),
        "reboot_done" => lang_text(language, "[ADB] Device reboot complete.", "[ADB] Перезагрузка устройства завершена.", "[ADB] 端末の再起動が完了しました。", "[ADB] 裝置重新啟動完成。", "[ADB] Thiết bị đã khởi động lại xong.", "[ADB] Η επανεκκίνηση της συσκευής ολοκληρώθηκε.", "[ADB] डिवाइस रीबूट पूरा हुआ।", "[ADB] მოწყობილობის გადატვირთვა დასრულდა.", "[ADB] Apparaat opnieuw opgestart.", "[ADB] اكتملت إعادة تشغيل الجهاز.", "[ADB] Reinicio del dispositivo completado."),
        "ota_disable" => lang_text(language, "[ADB] Disabling OTA (update) notifications and permissions.", "[ADB] Отключение уведомлений и разрешений OTA (обновления).", "[ADB] OTA（更新）通知と権限を無効化します。", "[ADB] 正在停用 OTA（更新）通知與權限。", "[ADB] Đang tắt thông báo và quyền OTA (cập nhật).", "[ADB] Απενεργοποίηση ειδοποιήσεων και δικαιωμάτων OTA (ενημέρωση).", "[ADB] OTA (अपडेट) सूचनाएँ और अनुमतियाँ बंद की जा रही हैं।", "[ADB] OTA (განახლება) შეტყობინებები და ნებართვები ითიშება.", "[ADB] OTA-meldingen (updates) en machtigingen uitschakelen.", "[ADB] جارٍ تعطيل إشعارات وأذونات OTA (التحديث).", "[ADB] Desactivando notificaciones y permisos OTA (actualización)."),
        "wait_guide_1" => lang_text(language, "[Guide] Many users ask the same question, so please note:", "[Инфо] Многие пользователи задают тот же вопрос, поэтому обратите внимание:", "[案内] 同じ質問が多いため、次の点をご案内します。", "[指南] 許多使用者會詢問相同問題，請注意：", "[Hướng dẫn] Nhiều người dùng hỏi cùng một câu hỏi, xin lưu ý:", "[Οδηγός] Πολλοί χρήστες κάνουν την ίδια ερώτηση, οπότε σημειώστε:", "[मार्गदर्शन] कई उपयोगकर्ता यही प्रश्न पूछते हैं, कृपया ध्यान दें:", "[გზამკვლევი] ბევრი მომხმარებელი იმავე კითხვას სვამს, ამიტომ გაითვალისწინეთ:", "[Gids] Veel gebruikers stellen dezelfde vraag, let daarom op:", "[إرشاد] يسأل كثير من المستخدمين نفس السؤال، لذا يرجى ملاحظة ما يلي:", "[Guía] Muchos usuarios hacen la misma pregunta, ten en cuenta lo siguiente:"),
        "wait_guide_2" => lang_text(language, "[Guide] The program is working, so the task has not stopped or been interrupted,", "[Инфо] Программа работает, поэтому задача не зависла и не была прервана,", "[案内] プログラムは作業中のため、停止・中断しているわけではありません。", "[指南] 程式正在作業，因此不是停止或中斷，", "[Hướng dẫn] Chương trình đang làm việc, vì vậy tác vụ không bị dừng hoặc gián đoạn,", "[Οδηγός] Το πρόγραμμα εκτελείται, επομένως η εργασία δεν έχει σταματήσει ή διακοπεί,", "[मार्गदर्शन] प्रोग्राम काम कर रहा है, इसलिए कार्य रुका या बाधित नहीं हुआ है,", "[გზამკვლევი] პროგრამა მუშაობს, ამიტომ პროცესი არ გაჩერებულა ან შეწყვეტილა,", "[Gids] Het programma is bezig, dus de taak is niet gestopt of onderbroken,", "[إرشاد] البرنامج يعمل، لذا لم تتوقف المهمة أو تنقطع،", "[Guía] El programa está trabajando, por lo que la tarea no se ha detenido ni interrumpido,"),
        "wait_guide_3" => lang_text(language, "[Guide] and the connection has not been lost. Please wait safely until the task completes.", "[Инфо] и соединение не потеряно. Спокойно дождитесь завершения задачи.", "[案内] 接続が切れたわけでもありません。完了まで安心してお待ちください。", "[指南] 連線也沒有中斷。請放心等待直到作業完成。", "[Hướng dẫn] và kết nối không bị ngắt. Hãy yên tâm chờ đến khi tác vụ hoàn tất.", "[Οδηγός] και η σύνδεση δεν χάθηκε. Περιμένετε με ασφάλεια μέχρι να ολοκληρωθεί η εργασία.", "[मार्गदर्शन] और कनेक्शन नहीं टूटा है। कार्य पूरा होने तक निश्चिंत होकर प्रतीक्षा करें।", "[გზამკვლევი] და კავშირი არ დაკარგულა. დაელოდეთ უსაფრთხოდ დასრულებამდე.", "[Gids] en de verbinding is niet verbroken. Wacht rustig tot de taak klaar is.", "[إرشاد] ولم ينقطع الاتصال. يرجى الانتظار باطمئنان حتى تكتمل المهمة.", "[Guía] y la conexión no se ha perdido. Espera con tranquilidad hasta que termine la tarea."),
        "wait_guide_4" => lang_text(language, "[Guide] Leave the PC, cable, and device untouched and wait 1–5 minutes.", "[Инфо] Не трогайте ПК, кабель и устройство, подождите 1–5 минут.", "[案内] PC、ケーブル、端末には触れずに1〜5分お待ちください。", "[指南] 請保持 PC、線材與裝置不動，等待 1～5 分鐘。", "[Hướng dẫn] Giữ nguyên PC, cáp và thiết bị, chờ 1–5 phút.", "[Οδηγός] Αφήστε τον υπολογιστή, το καλώδιο και τη συσκευή ακίνητα και περιμένετε 1–5 λεπτά.", "[मार्गदर्शन] PC, केबल और डिवाइस को न छुएँ और 1–5 मिनट प्रतीक्षा करें।", "[გზამკვლევი] PC, კაბელი და მოწყობილობა არ შეეხოთ და დაელოდეთ 1–5 წუთი.", "[Gids] Laat pc, kabel en apparaat met rust en wacht 1–5 minuten.", "[إرشاد] اترك الكمبيوتر والكابل والجهاز دون لمس وانتظر من 1 إلى 5 دقائق.", "[Guía] Deja quietos el PC, el cable y el dispositivo y espera de 1 a 5 minutos."),
        "google_stage_working" => lang_text(language, "Google service task in progress...", "выполняется задача Google service...", "Googleサービス作業中...", "Google 服務作業中...", "đang thực hiện tác vụ Google service...", "εργασία Google service σε εξέλιξη...", "Google service कार्य जारी है...", "Google service ამოცანა მიმდინარეობს...", "Google service-taak bezig...", "مهمة Google service قيد التنفيذ...", "tarea de Google service en curso..."),
        "google_stage_done" => lang_text(language, "Google service task step complete:", "этап задачи Google service завершён:", "Googleサービス作業ステップ完了:", "Google 服務作業階段完成:", "hoàn tất bước tác vụ Google service:", "ολοκληρώθηκε βήμα εργασίας Google service:", "Google service कार्य चरण पूरा:", "Google service ამოცანის ეტაპი დასრულდა:", "Google service-taakstap voltooid:", "اكتملت خطوة مهمة Google service:", "paso de tarea de Google service completado:"),
        "finalize" => lang_text(language, "[GBST] Finishing the task.", "[GBST] Завершение задачи.", "[GBST] 作業を完了しています。", "[GBST] 正在完成作業。", "[GBST] Đang hoàn tất tác vụ.", "[GBST] Ολοκλήρωση εργασίας.", "[GBST] कार्य समाप्त किया जा रहा है।", "[GBST] ამოცანა სრულდება.", "[GBST] Taak afronden.", "[GBST] جارٍ إنهاء المهمة.", "[GBST] Finalizando la tarea."),
        "google_done" => lang_text(language, "[GBST] Google Services repair and update completed.", "[GBST] Восстановление и обновление Google Services завершены.", "[GBST] Google Servicesの修復と更新が完了しました。", "[GBST] Google Services 修復與更新已完成。", "[GBST] Đã hoàn tất sửa chữa và cập nhật Google Services.", "[GBST] Η επιδιόρθωση και ενημέρωση των Google Services ολοκληρώθηκε.", "[GBST] Google Services मरम्मत और अपडेट पूरा हुआ।", "[GBST] Google Services-ის აღდგენა და განახლება დასრულდა.", "[GBST] Google Services herstellen en bijwerken voltooid.", "[GBST] اكتمل إصلاح وتحديث Google Services.", "[GBST] Reparación y actualización de Google Services completadas."),
        "task_done" => lang_text(language, "[GBST] Task completed.", "[GBST] Задача завершена.", "[GBST] 作業が完了しました。", "[GBST] 作業已完成。", "[GBST] Tác vụ đã hoàn tất.", "[GBST] Η εργασία ολοκληρώθηκε.", "[GBST] कार्य पूरा हुआ।", "[GBST] ამოცანა დასრულდა.", "[GBST] Taak voltooid.", "[GBST] اكتملت المهمة.", "[GBST] Tarea completada."),
        "startup_apk_prepare_failed" => lang_text(language, "[GBST] Startup APK download preparation failed: ", "[GBST] Ошибка подготовки загрузки APK при запуске: ", "[GBST] 起動時のAPKダウンロード準備に失敗: ", "[GBST] 啟動時 APK 下載準備失敗：", "[GBST] Chuẩn bị tải APK khi khởi động thất bại: ", "[GBST] Η προετοιμασία λήψης APK κατά την εκκίνηση απέτυχε: ", "[GBST] स्टार्टअप APK डाउनलोड तैयारी विफल: ", "[GBST] გაშვებისას APK-ის ჩამოტვირთვის მომზადება ვერ მოხერხდა: ", "[GBST] Voorbereiding APK-download bij opstarten mislukt: ", "[GBST] فشل تحضير تنزيل APK عند بدء التشغيل: ", "[GBST] Falló la preparación de descarga de APK al iniciar: "),
        "task_failed" => lang_text(language, "[GBST] Task failed: ", "[GBST] Ошибка задачи: ", "[GBST] 作業失敗: ", "[GBST] 作業失敗：", "[GBST] Tác vụ thất bại: ", "[GBST] Η εργασία απέτυχε: ", "[GBST] कार्य विफल: ", "[GBST] ამოცანა ვერ შესრულდა: ", "[GBST] Taak mislukt: ", "[GBST] فشلت المهمة: ", "[GBST] Error de tarea: "),
        "worker_closed" => lang_text(language, "[GBST] Worker thread connection closed.", "[GBST] Соединение рабочего потока закрыто.", "[GBST] ワーカースレッド接続が終了しました。", "[GBST] 工作執行緒連線已結束。", "[GBST] Kết nối luồng tác vụ đã kết thúc.", "[GBST] Η σύνδεση του νήματος εργασίας έκλεισε.", "[GBST] कार्य थ्रेड कनेक्शन बंद हो गया।", "[GBST] სამუშაო ნაკადის კავშირი დაიხურა.", "[GBST] Workerthread-verbinding gesloten.", "[GBST] تم إغلاق اتصال مؤشر ترابط العمل.", "[GBST] Conexión del hilo de trabajo cerrada."),
        "log_prefix" => lang_text(language, "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST", "[Log] GBST"),
        "log_saved_to" => lang_text(language, "task log saved to", "журнал задачи сохранён в", "作業ログの保存先", "作業記錄已儲存至", "nhật ký tác vụ đã lưu vào", "το αρχείο καταγραφής αποθηκεύτηκε στο", "कार्य लॉग यहाँ सहेजा गया", "ამოცანის ლოგი შეინახა", "taaklog opgeslagen in", "تم حفظ سجل المهمة في", "registro de tarea guardado en"),
        "flow_complete" => lang_text(language, "completed", "завершённой", "完了", "完成", "hoàn tất", "ολοκληρωμένης", "पूर्ण", "დასრულებული", "voltooide", "المكتملة", "completada"),
        "flow_failed" => lang_text(language, "failed", "с ошибкой", "失敗", "失敗", "thất bại", "αποτυχημένης", "विफल", "ვერ შესრულებული", "mislukte", "الفاشلة", "fallida"),
        "flow_closed" => lang_text(language, "closed", "закрытой", "終了", "結束", "đã đóng", "κλειστής", "बंद", "დახურული", "gesloten", "المغلقة", "cerrada"),
        "flow_manual" => lang_text(language, "manual", "ручной", "手動", "手動", "thủ công", "χειροκίνητης", "मैनुअल", "ხელით", "handmatige", "اليدوية", "manual"),
        "log_auto_save_failed" => lang_text(language, "[Log] Failed to auto-save task log: ", "[Log] Не удалось автоматически сохранить журнал: ", "[Log] 作業ログの自動保存に失敗: ", "[Log] 自動儲存作業記錄失敗：", "[Log] Không thể tự động lưu nhật ký tác vụ: ", "[Log] Αποτυχία αυτόματης αποθήκευσης καταγραφής: ", "[Log] कार्य लॉग स्वतः सहेजने में विफल: ", "[Log] ამოცანის ლოგის ავტომატური შენახვა ვერ მოხერხდა: ", "[Log] Taaklog automatisch opslaan mislukt: ", "[Log] فشل الحفظ التلقائي لسجل المهمة: ", "[Log] Error al guardar automáticamente el registro: "),
        "log_save_failed" => lang_text(language, "[Log] Save failed: ", "[Log] Ошибка сохранения: ", "[Log] 保存失敗: ", "[Log] 儲存失敗：", "[Log] Lưu thất bại: ", "[Log] Αποτυχία αποθήκευσης: ", "[Log] सहेजना विफल: ", "[Log] შენახვა ვერ მოხერხდა: ", "[Log] Opslaan mislukt: ", "[Log] فشل الحفظ: ", "[Log] Error al guardar: "),
        "dashboard_detect_failed" => lang_text(language, "[Dashboard] Detection failed: ", "[Dashboard] Ошибка обнаружения: ", "[Dashboard] 検出失敗: ", "[Dashboard] 偵測失敗：", "[Dashboard] Phát hiện thất bại: ", "[Dashboard] Αποτυχία εντοπισμού: ", "[Dashboard] पहचान विफल: ", "[Dashboard] აღმოჩენა ვერ მოხერხდა: ", "[Dashboard] Detectie mislukt: ", "[Dashboard] فشل الاكتشاف: ", "[Dashboard] Detección fallida: "),
        "no_logs" => lang_text(language, "No logs.", "Журнал пуст.", "ログがありません。", "沒有記錄。", "Không có nhật ký.", "Δεν υπάρχουν καταγραφές.", "कोई लॉग नहीं है।", "ლოგები არ არის.", "Geen logs.", "لا توجد سجلات.", "No hay registros."),
        "lenovo_only" => lang_text(language, "This program supports Lenovo tablets only. Stopping the task.", "Эта программа поддерживает только планшеты Lenovo. Задача остановлена.", "このプログラムはLenovoタブレットのみ対応しています。作業を停止します。", "本程式僅支援 Lenovo 平板。正在停止作業。", "Chương trình này chỉ hỗ trợ máy tính bảng Lenovo. Đang dừng tác vụ.", "Αυτό το πρόγραμμα υποστηρίζει μόνο tablet Lenovo. Η εργασία διακόπτεται.", "यह प्रोग्राम केवल Lenovo टैबलेट का समर्थन करता है। कार्य रोका जा रहा है।", "ეს პროგრამა მხოლოდ Lenovo ტაბლეტებს უჭერს მხარს. ამოცანა ჩერდება.", "Dit programma ondersteunt alleen Lenovo-tablets. De taak wordt gestopt.", "يدعم هذا البرنامج أجهزة Lenovo اللوحية فقط. سيتم إيقاف المهمة.", "Este programa solo admite tablets Lenovo. Deteniendo la tarea."),
        "error_lenovo_only" => lang_text(language, "[Error] This program supports Lenovo tablets only. Stopping the task.", "[Error] Эта программа поддерживает только планшеты Lenovo. Задача остановлена.", "[Error] このプログラムはLenovoタブレットのみ対応しています。作業を停止します。", "[Error] 本程式僅支援 Lenovo 平板。正在停止作業。", "[Error] Chương trình này chỉ hỗ trợ máy tính bảng Lenovo. Đang dừng tác vụ.", "[Error] Αυτό το πρόγραμμα υποστηρίζει μόνο tablet Lenovo. Η εργασία διακόπτεται.", "[Error] यह प्रोग्राम केवल Lenovo टैबलेट का समर्थन करता है। कार्य रोका जा रहा है।", "[Error] ეს პროგრამა მხოლოდ Lenovo ტაბლეტებს უჭერს მხარს. ამოცანა ჩერდება.", "[Error] Dit programma ondersteunt alleen Lenovo-tablets. De taak wordt gestopt.", "[Error] يدعم هذا البرنامج أجهزة Lenovo اللوحية فقط. سيتم إيقاف المهمة.", "[Error] Este programa solo admite tablets Lenovo. Deteniendo la tarea."),
        "apk_download_failed" => lang_text(language, "APK download failed. Please check the GitHub Base64 link file or the decoded APK download links.", "Не удалось загрузить APK. Проверьте файл ссылок GitHub Base64 или расшифрованные ссылки APK.", "APKのダウンロードに失敗しました。GitHub Base64リンクファイルまたは復号されたAPKダウンロードリンクを確認してください。", "APK 下載失敗。請確認 GitHub Base64 連結檔案或解碼後的 APK 下載連結。", "Tải APK thất bại. Vui lòng kiểm tra tệp liên kết GitHub Base64 hoặc các liên kết APK đã giải mã.", "Η λήψη APK απέτυχε. Ελέγξτε το αρχείο συνδέσμων GitHub Base64 ή τους αποκωδικοποιημένους συνδέσμους APK.", "APK डाउनलोड विफल। कृपया GitHub Base64 लिंक फ़ाइल या डिकोड किए गए APK डाउनलोड लिंक जाँचें।", "APK ჩამოტვირთვა ვერ მოხერხდა. შეამოწმეთ GitHub Base64 ბმულების ფაილი ან დეკოდირებული APK ბმულები.", "APK-download mislukt. Controleer het GitHub Base64-linkbestand of de gedecodeerde APK-downloadlinks.", "فشل تنزيل APK. تحقق من ملف روابط GitHub Base64 أو روابط APK بعد فك الترميز.", "Error al descargar APK. Revisa el archivo de enlaces GitHub Base64 o los enlaces APK decodificados."),
        "update_already_checking" => lang_text(language, "[Update] Latest release check is already running.", "[Update] Проверка последнего релиза уже выполняется.", "[Update] 最新リリースの確認はすでに実行中です。", "[Update] 已在檢查最新版本。", "[Update] Đang kiểm tra bản phát hành mới nhất.", "[Update] Ο έλεγχος τελευταίας έκδοσης εκτελείται ήδη.", "[Update] नवीनतम रिलीज़ जाँच पहले से चल रही है।", "[Update] უახლესი ვერსიის შემოწმება უკვე მიმდინარეობს.", "[Update] Controle op nieuwste release is al bezig.", "[Update] فحص أحدث إصدار قيد التشغيل بالفعل.", "[Update] La comprobación de la última versión ya está en curso."),
        "update_checking" => lang_text(language, "[Update] Checking the latest GBST release.", "[Update] Проверка последнего релиза GBST.", "[Update] 最新のGBSTリリースを確認しています。", "[Update] 正在檢查最新 GBST 版本。", "[Update] Đang kiểm tra bản phát hành GBST mới nhất.", "[Update] Έλεγχος τελευταίας έκδοσης GBST.", "[Update] नवीनतम GBST रिलीज़ जाँची जा रही है।", "[Update] მოწმდება GBST-ის უახლესი ვერსია.", "[Update] Nieuwste GBST-release controleren.", "[Update] جارٍ التحقق من أحدث إصدار GBST.", "[Update] Comprobando la última versión de GBST."),
        "update_found" => lang_text(language, "[Update] New GBST version found: current ", "[Update] Найдена новая версия GBST: текущая ", "[Update] 新しいGBSTバージョンを検出: 現在 ", "[Update] 發現新的 GBST 版本：目前 ", "[Update] Tìm thấy phiên bản GBST mới: hiện tại ", "[Update] Βρέθηκε νέα έκδοση GBST: τρέχουσα ", "[Update] नया GBST संस्करण मिला: वर्तमान ", "[Update] ნაპოვნია GBST-ის ახალი ვერსია: მიმდინარე ", "[Update] Nieuwe GBST-versie gevonden: huidig ", "[Update] تم العثور على إصدار GBST جديد: الحالي ", "[Update] Nueva versión de GBST encontrada: actual "),
        "update_asset" => lang_text(language, "[Update] Release ZIP file: ", "[Update] ZIP-файл релиза: ", "[Update] リリースZIPファイル: ", "[Update] 發行 ZIP 檔案：", "[Update] Tệp ZIP phát hành: ", "[Update] Αρχείο ZIP έκδοσης: ", "[Update] रिलीज़ ZIP फ़ाइल: ", "[Update] გამოშვების ZIP ფაილი: ", "[Update] Release-ZIP-bestand: ", "[Update] ملف ZIP للإصدار: ", "[Update] Archivo ZIP de versión: "),
        "update_notice_dashboard" => lang_text(language, "[Update] Showing the update notice on the dashboard.", "[Update] Показ уведомления об обновлении на панели.", "[Update] ダッシュボードに更新案内を表示します。", "[Update] 在儀表板顯示更新通知。", "[Update] Hiển thị thông báo cập nhật trên bảng điều khiển.", "[Update] Εμφάνιση ειδοποίησης ενημέρωσης στον πίνακα.", "[Update] डैशबोर्ड पर अपडेट सूचना दिखाई जा रही है।", "[Update] განახლების შეტყობინება გამოჩნდება დაფაზე.", "[Update] Updatebericht op het dashboard tonen.", "[Update] عرض إشعار التحديث على لوحة المعلومات.", "[Update] Mostrando aviso de actualización en el panel."),
        "update_latest" => lang_text(language, "[Update] You are already using the latest version: ", "[Update] Вы уже используете последнюю версию: ", "[Update] すでに最新バージョンを使用中です: ", "[Update] 您已使用最新版本：", "[Update] Bạn đang dùng phiên bản mới nhất: ", "[Update] Χρησιμοποιείτε ήδη την τελευταία έκδοση: ", "[Update] आप पहले से नवीनतम संस्करण उपयोग कर रहे हैं: ", "[Update] უკვე იყენებთ უახლეს ვერსიას: ", "[Update] U gebruikt al de nieuwste versie: ", "[Update] أنت تستخدم أحدث إصدار بالفعل: ", "[Update] Ya estás usando la última versión: "),
        "update_failed" => lang_text(language, "[Update] Update check failed: ", "[Update] Ошибка проверки обновлений: ", "[Update] 更新確認失敗: ", "[Update] 更新檢查失敗：", "[Update] Kiểm tra cập nhật thất bại: ", "[Update] Ο έλεγχος ενημέρωσης απέτυχε: ", "[Update] अपडेट जाँच विफल: ", "[Update] განახლების შემოწმება ვერ მოხერხდა: ", "[Update] Updatecontrole mislukt: ", "[Update] فشل فحص التحديث: ", "[Update] Error al comprobar actualización: "),
        "update_open_manual" => lang_text(language, "[Update] Opening GitHub Releases for manual check.", "[Update] Открытие GitHub Releases для ручной проверки.", "[Update] 手動確認のためGitHub Releasesページを開きます。", "[Update] 開啟 GitHub Releases 以手動確認。", "[Update] Mở GitHub Releases để kiểm tra thủ công.", "[Update] Άνοιγμα GitHub Releases για χειροκίνητο έλεγχο.", "[Update] मैनुअल जाँच के लिए GitHub Releases खोला जा रहा है।", "[Update] ხელით შესამოწმებლად იხსნება GitHub Releases.", "[Update] GitHub Releases openen voor handmatige controle.", "[Update] فتح GitHub Releases للتحقق اليدوي.", "[Update] Abriendo GitHub Releases para comprobación manual."),
        "update_open_failed" => lang_text(language, "[Update] Failed to open GitHub Releases: ", "[Update] Не удалось открыть GitHub Releases: ", "[Update] GitHub Releasesを開けませんでした: ", "[Update] 無法開啟 GitHub Releases：", "[Update] Không mở được GitHub Releases: ", "[Update] Αποτυχία ανοίγματος GitHub Releases: ", "[Update] GitHub Releases खोलने में विफल: ", "[Update] GitHub Releases ვერ გაიხსნა: ", "[Update] GitHub Releases openen mislukt: ", "[Update] فشل فتح GitHub Releases: ", "[Update] No se pudo abrir GitHub Releases: "),
        "update_detected" => lang_text(language, "[Update] New update file detected.", "[Update] Обнаружен новый файл обновления.", "[Update] 新しい更新ファイルを検出しました。", "[Update] 偵測到新的更新檔案。", "[Update] Đã phát hiện tệp cập nhật mới.", "[Update] Εντοπίστηκε νέο αρχείο ενημέρωσης.", "[Update] नई अपडेट फ़ाइल मिली।", "[Update] ნაპოვნია ახალი განახლების ფაილი.", "[Update] Nieuw updatebestand gedetecteerd.", "[Update] تم اكتشاف ملف تحديث جديد.", "[Update] Nuevo archivo de actualización detectado."),
        "update_later" => lang_text(language, "[Update] This update notice will be checked again later.", "[Update] Это уведомление об обновлении будет проверено позже.", "[Update] この更新案内は後でもう一度確認します。", "[Update] 此更新通知稍後會再次確認。", "[Update] Thông báo cập nhật này sẽ được kiểm tra lại sau.", "[Update] Αυτή η ειδοποίηση ενημέρωσης θα ελεγχθεί ξανά αργότερα.", "[Update] यह अपडेट सूचना बाद में फिर जाँची जाएगी।", "[Update] ეს განახლების შეტყობინება მოგვიანებით კვლავ შემოწმდება.", "[Update] Deze updatemelding wordt later opnieuw gecontroleerd.", "[Update] سيتم التحقق من إشعار التحديث هذا لاحقًا.", "[Update] Este aviso de actualización se comprobará de nuevo más tarde."),
        "youtube_open_failed" => lang_text(language, "[Settings] Failed to open YouTube link: ", "[Settings] Не удалось открыть ссылку YouTube: ", "[Settings] YouTubeリンクを開けませんでした: ", "[Settings] 無法開啟 YouTube 連結：", "[Settings] Không mở được liên kết YouTube: ", "[Settings] Αποτυχία ανοίγματος συνδέσμου YouTube: ", "[Settings] YouTube लिंक खोलने में विफल: ", "[Settings] YouTube ბმული ვერ გაიხსნა: ", "[Settings] YouTube-link openen mislukt: ", "[Settings] فشل فتح رابط YouTube: ", "[Settings] No se pudo abrir el enlace de YouTube: "),
        "donate_open_failed" => lang_text(language, "[Settings] Failed to open support link: ", "[Settings] Не удалось открыть ссылку поддержки: ", "[Settings] 支援リンクを開けませんでした: ", "[Settings] 無法開啟支援連結：", "[Settings] Không mở được liên kết ủng hộ: ", "[Settings] Αποτυχία ανοίγματος συνδέσμου υποστήριξης: ", "[Settings] समर्थन लिंक खोलने में विफल: ", "[Settings] მხარდაჭერის ბმული ვერ გაიხსნა: ", "[Settings] Supportlink openen mislukt: ", "[Settings] فشل فتح رابط الدعم: ", "[Settings] No se pudo abrir el enlace de apoyo: "),
        "feedback_open_failed" => lang_text(language, "[Settings] Failed to open feedback link: ", "[Settings] Не удалось открыть ссылку обратной связи: ", "[Settings] フィードバックリンクを開けませんでした: ", "[Settings] 無法開啟意見回饋連結：", "[Settings] Không mở được liên kết phản hồi: ", "[Settings] Αποτυχία ανοίγματος συνδέσμου σχολίων: ", "[Settings] फ़ीडबैक लिंक खोलने में विफल: ", "[Settings] უკუკავშირის ბმული ვერ გაიხსნა: ", "[Settings] Feedbacklink openen mislukt: ", "[Settings] فشل فتح رابط الملاحظات: ", "[Settings] No se pudo abrir el enlace de comentarios: "),
        "language_save_failed" => lang_text(language, "[Settings] Failed to save language: ", "[Settings] Не удалось сохранить язык: ", "[Settings] 言語の保存に失敗: ", "[Settings] 儲存語言失敗：", "[Settings] Lưu ngôn ngữ thất bại: ", "[Settings] Αποτυχία αποθήκευσης γλώσσας: ", "[Settings] भाषा सहेजने में विफल: ", "[Settings] ენის შენახვა ვერ მოხერხდა: ", "[Settings] Taal opslaan mislukt: ", "[Settings] فشل حفظ اللغة: ", "[Settings] Error al guardar idioma: "),
        "err_github_release_info" => lang_text(language, "Could not retrieve GitHub release information", "Не удалось получить сведения о релизе GitHub", "GitHubリリース情報を取得できませんでした", "無法取得 GitHub 發行資訊", "Không thể lấy thông tin phát hành GitHub", "Δεν ήταν δυνατή η λήψη πληροφοριών έκδοσης GitHub", "GitHub रिलीज़ जानकारी प्राप्त नहीं हो सकी", "GitHub გამოშვების ინფორმაცია ვერ მიიღეს", "GitHub-release-informatie ophalen mislukt", "تعذر جلب معلومات إصدار GitHub", "No se pudo obtener información de la versión de GitHub"),
        "err_github_response_read" => lang_text(language, "Could not read the GitHub response", "Не удалось прочитать ответ GitHub", "GitHub応答を読み取れませんでした", "無法讀取 GitHub 回應", "Không thể đọc phản hồi GitHub", "Δεν ήταν δυνατή η ανάγνωση της απόκρισης GitHub", "GitHub प्रतिक्रिया पढ़ी नहीं जा सकी", "GitHub პასუხი ვერ წაიკითხა", "GitHub-reactie lezen mislukt", "تعذرت قراءة استجابة GitHub", "No se pudo leer la respuesta de GitHub"),
        "err_github_json" => lang_text(language, "GitHub response JSON parsing failed", "Ошибка разбора JSON ответа GitHub", "GitHub応答JSONの解析に失敗しました", "GitHub 回應 JSON 解析失敗", "Phân tích JSON phản hồi GitHub thất bại", "Αποτυχία ανάλυσης JSON απόκρισης GitHub", "GitHub प्रतिक्रिया JSON पार्स विफल", "GitHub პასუხის JSON დამუშავება ვერ მოხერხდა", "JSON van GitHub-reactie parseren mislukt", "فشل تحليل JSON لاستجابة GitHub", "Error al analizar JSON de la respuesta de GitHub"),
        "err_github_format" => lang_text(language, "The GitHub release response format is not as expected.", "Формат ответа релизов GitHub отличается от ожидаемого.", "GitHubリリース応答の形式が想定と異なります。", "GitHub 發行回應格式不符合預期。", "Định dạng phản hồi phát hành GitHub không đúng như mong đợi.", "Η μορφή απόκρισης εκδόσεων GitHub δεν είναι η αναμενόμενη.", "GitHub रिलीज़ प्रतिक्रिया प्रारूप अपेक्षित नहीं है।", "GitHub გამოშვების პასუხის ფორმატი მოსალოდნელი არ არის.", "De GitHub-release-reactie heeft niet het verwachte formaat.", "تنسيق استجابة إصدارات GitHub غير متوقع.", "El formato de respuesta de versiones de GitHub no es el esperado."),
        "err_gbst_release_missing" => lang_text(language, "No available GBST release version was found.", "Доступная версия релиза GBST не найдена.", "確認可能なGBSTリリースバージョンが見つかりませんでした。", "找不到可確認的 GBST 發行版本。", "Không tìm thấy phiên bản phát hành GBST có thể kiểm tra.", "Δεν βρέθηκε διαθέσιμη έκδοση GBST.", "कोई उपलब्ध GBST रिलीज़ संस्करण नहीं मिला।", "ხელმისაწვდომი GBST ვერსია ვერ მოიძებნა.", "Er is geen beschikbare GBST-releaseversie gevonden.", "لم يتم العثور على إصدار GBST متاح.", "No se encontró una versión disponible de GBST."),
        "err_no_usb_device" => lang_text(language, "Tablet detection failed. Please use a proper data cable.", "Не удалось обнаружить планшет. Используйте подходящий кабель передачи данных.", "タブレットの確認に失敗しました。適切なデータケーブルを使用してください。", "平板偵測失敗。請使用正確的資料傳輸線。", "Phát hiện máy tính bảng thất bại. Vui lòng dùng cáp dữ liệu phù hợp.", "Απέτυχε ο εντοπισμός tablet. Χρησιμοποιήστε σωστό καλώδιο δεδομένων.", "टैबलेट पहचान विफल। सही डेटा केबल का उपयोग करें।", "ტაბლეტის აღმოჩენა ვერ მოხერხდა. გამოიყენეთ სწორი მონაცემთა კაბელი.", "Tabletdetectie mislukt. Gebruik een geschikte datakabel.", "فشل اكتشاف الجهاز اللوحي. استخدم كابل بيانات مناسبًا.", "No se detectó la tablet. Usa un cable de datos adecuado."),
        "err_adb_timeout" => lang_text(language, "ADB device detection timed out. Last state:", "Время обнаружения ADB-устройства истекло. Последнее состояние:", "ADB端末検出がタイムアウトしました。最後の状態:", "ADB 裝置偵測逾時。最後狀態：", "Phát hiện thiết bị ADB đã hết thời gian. Trạng thái cuối:", "Έληξε ο χρόνος εντοπισμού συσκευής ADB. Τελευταία κατάσταση:", "ADB डिवाइस पहचान समय समाप्त। अंतिम स्थिति:", "ADB მოწყობილობის აღმოჩენის დრო ამოიწურა. ბოლო მდგომარეობა:", "Time-out bij ADB-apparaatdetectie. Laatste status:", "انتهت مهلة اكتشاف جهاز ADB. آخر حالة:", "Tiempo agotado al detectar dispositivo ADB. Último estado:"),
        "err_usb_adb_missing" => lang_text(language, "USB ADB device was not found.", "USB ADB-устройство не найдено.", "USB ADB端末が見つかりません。", "找不到 USB ADB 裝置。", "Không tìm thấy thiết bị USB ADB.", "Δεν βρέθηκε συσκευή USB ADB.", "USB ADB डिवाइस नहीं मिला।", "USB ADB მოწყობილობა ვერ მოიძებნა.", "USB ADB-apparaat niet gevonden.", "لم يتم العثور على جهاز USB ADB.", "No se encontró dispositivo USB ADB."),
        "err_multiple_adb" => lang_text(language, "Two or more ADB devices are connected. Connect only one device.", "Подключено два или более ADB-устройства. Оставьте только одно.", "ADB端末が2台以上接続されています。1台だけ接続してください。", "已連接兩個以上 ADB 裝置。請只連接一台。", "Có từ hai thiết bị ADB trở lên đang kết nối. Chỉ kết nối một thiết bị.", "Έχουν συνδεθεί δύο ή περισσότερες συσκευές ADB. Συνδέστε μόνο μία.", "दो या अधिक ADB डिवाइस जुड़े हैं। केवल एक डिवाइस कनेक्ट करें।", "დაკავშირებულია ორი ან მეტი ADB მოწყობილობა. დატოვეთ მხოლოდ ერთი.", "Er zijn twee of meer ADB-apparaten verbonden. Sluit slechts één apparaat aan.", "تم توصيل جهازين ADB أو أكثر. وصّل جهازًا واحدًا فقط.", "Hay dos o más dispositivos ADB conectados. Conecta solo uno."),
        "err_adb_shell_failed" => lang_text(language, "ADB shell failed", "Ошибка ADB shell", "ADB shell失敗", "ADB shell 失敗", "ADB shell thất bại", "Αποτυχία ADB shell", "ADB shell विफल", "ADB shell ვერ შესრულდა", "ADB shell mislukt", "فشل ADB shell", "ADB shell falló"),
        "err_not_lenovo_maker" => lang_text(language, "Not a Lenovo device. Detected manufacturer:", "Это не устройство Lenovo. Обнаруженный производитель:", "Lenovo端末ではありません。検出メーカー:", "不是 Lenovo 裝置。偵測到的製造商：", "Không phải thiết bị Lenovo. Nhà sản xuất đã phát hiện:", "Δεν είναι συσκευή Lenovo. Κατασκευαστής που εντοπίστηκε:", "यह Lenovo डिवाइस नहीं है। पहचाना गया निर्माता:", "ეს Lenovo მოწყობილობა არ არის. აღმოჩენილი მწარმოებელი:", "Geen Lenovo-apparaat. Gedetecteerde fabrikant:", "ليس جهاز Lenovo. الشركة المصنّعة المكتشفة:", "No es un dispositivo Lenovo. Fabricante detectado:"),
        "err_android_supported" => lang_text(language, "Only Android 13–18 is supported. Detected version:", "Поддерживается только Android 13–18. Обнаруженная версия:", "Android 13〜18のみ対応しています。検出バージョン:", "僅支援 Android 13–18。偵測到的版本：", "Chỉ hỗ trợ Android 13–18. Phiên bản đã phát hiện:", "Υποστηρίζεται μόνο Android 13–18. Έκδοση που εντοπίστηκε:", "केवल Android 13–18 समर्थित है। पहचाना गया संस्करण:", "მხარდაჭერილია მხოლოდ Android 13–18. აღმოჩენილი ვერსია:", "Alleen Android 13–18 wordt ondersteund. Gedetecteerde versie:", "يتم دعم Android 13–18 فقط. الإصدار المكتشف:", "Solo se admite Android 13–18. Versión detectada:"),
        "err_screen_timeout_value" => lang_text(language, "Invalid screen_off_timeout restore value:", "Недопустимое значение восстановления screen_off_timeout:", "screen_off_timeoutの復元値が正しくありません:", "screen_off_timeout 還原值無效：", "Giá trị khôi phục screen_off_timeout không hợp lệ:", "Μη έγκυρη τιμή επαναφοράς screen_off_timeout:", "screen_off_timeout बहाली मान अमान्य है:", "screen_off_timeout აღდგენის მნიშვნელობა არასწორია:", "Ongeldige herstelwaarde voor screen_off_timeout:", "قيمة استعادة screen_off_timeout غير صالحة:", "Valor de restauración de screen_off_timeout no válido:"),
        "google_update_required_log" => lang_text(language, "[GBST] The Google Services APK version on the device is lower, so the update will proceed.", "[GBST] Версия APK Google Services на устройстве ниже, поэтому будет выполнено обновление.", "[GBST] 端末上のGoogle Services APKのバージョンが低いため、更新を実行します。", "[GBST] 裝置上的 Google Services APK 版本較低，因此將進行更新。", "[GBST] Phiên bản APK Google Services trên thiết bị thấp hơn nên sẽ tiến hành cập nhật.", "[GBST] Η έκδοση του Google Services APK στη συσκευή είναι χαμηλότερη, επομένως θα γίνει ενημέρωση.", "[GBST] डिवाइस पर Google Services APK संस्करण कम है, इसलिए अपडेट किया जाएगा।", "[GBST] მოწყობილობაზე Google Services APK-ის ვერსია დაბალია, ამიტომ განახლება გაგრძელდება.", "[GBST] De Google Services APK-versie op het apparaat is lager, daarom wordt de update uitgevoerd.", "[GBST] إصدار APK لخدمات Google على الجهاز أقل، لذلك سيتم إجراء التحديث.", "[GBST] La versión del APK de Google Services en el dispositivo es inferior, por lo que se realizará la actualización."),
        "google_repair_required_log" => lang_text(language, "[GBST] Google Services on the device is not in a normal state, so repair will proceed.", "[GBST] Google Services на устройстве работает некорректно, поэтому будет выполнено восстановление.", "[GBST] 端末のGoogle Servicesが正常な状態ではないため、修復を実行します。", "[GBST] 裝置上的 Google Services 狀態不正常，因此將進行修復。", "[GBST] Google Services trên thiết bị không ở trạng thái bình thường nên sẽ tiến hành sửa chữa.", "[GBST] Οι υπηρεσίες Google στη συσκευή δεν είναι σε κανονική κατάσταση, επομένως θα γίνει επιδιόρθωση.", "[GBST] डिवाइस पर Google Services सामान्य स्थिति में नहीं है, इसलिए मरम्मत की जाएगी।", "[GBST] მოწყობილობაზე Google Services ნორმალურ მდგომარეობაში არ არის, ამიტომ აღდგენა გაგრძელდება.", "[GBST] Google Services op het apparaat is niet in normale staat, daarom wordt herstel uitgevoerd.", "[GBST] خدمات Google على الجهاز ليست في حالة طبيعية، لذلك سيتم الإصلاح.", "[GBST] Google Services en el dispositivo no está en un estado normal, por lo que se realizará la reparación."),
        "post_install_refresh_start" => lang_text(language, "[Device] Rechecking the installed Google Services APK versions and GBST device information.", "[Device] Повторная проверка установленных версий APK Google Services и сведений устройства GBST.", "[Device] インストール済みGoogle Services APKバージョンとGBST端末情報を再確認します。", "[Device] 正在重新檢查已安裝的 Google Services APK 版本與 GBST 裝置資訊。", "[Device] Đang kiểm tra lại phiên bản APK Google Services đã cài đặt và thông tin thiết bị GBST.", "[Device] Επανέλεγχος των εγκατεστημένων εκδόσεων APK Google Services και των πληροφοριών συσκευής GBST.", "[Device] इंस्टॉल किए गए Google Services APK संस्करण और GBST डिवाइस जानकारी फिर से जाँची जा रही है।", "[Device] დაყენებული Google Services APK ვერსიებისა და GBST მოწყობილობის ინფორმაციის ხელახლა შემოწმება.", "[Device] Geïnstalleerde Google Services APK-versies en GBST-apparaatinformatie opnieuw controleren.", "[Device] جارٍ إعادة التحقق من إصدارات APK لخدمات Google المثبتة ومعلومات جهاز GBST.", "[Device] Volviendo a comprobar las versiones APK de Google Services instaladas y la información del dispositivo GBST."),
        "post_install_refresh_done" => lang_text(language, "[Device] GBST device information refreshed, Google Services status: ", "[Device] Сведения устройства GBST обновлены, состояние Google Services: ", "[Device] GBST端末情報の再取得完了、Google Servicesの状態: ", "[Device] GBST 裝置資訊重新讀取完成，Google Services 狀態：", "[Device] Đã làm mới thông tin thiết bị GBST, trạng thái Google Services: ", "[Device] Οι πληροφορίες συσκευής GBST ανανεώθηκαν, κατάσταση Google Services: ", "[Device] GBST डिवाइस जानकारी रीफ़्रेश हुई, Google Services स्थिति: ", "[Device] GBST მოწყობილობის ინფორმაცია განახლდა, Google Services-ის მდგომარეობა: ", "[Device] GBST-apparaatinformatie vernieuwd, Google Services-status: ", "[Device] تم تحديث معلومات جهاز GBST، حالة خدمات Google: ", "[Device] Información del dispositivo GBST actualizada, estado de Google Services: "),
        "error_prefix" => lang_text(language, "Error:", "Ошибка:", "エラー:", "錯誤：", "Lỗi:", "Σφάλμα:", "त्रुटि:", "შეცდომა:", "Fout:", "خطأ:", "Error:"),
        _ => return None,
    })
}
