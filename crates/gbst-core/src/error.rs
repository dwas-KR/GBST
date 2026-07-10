use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GbstError {
    #[error("I/O 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("APK 링크 파일 파싱 오류: {0}")]
    Catalog(String),

    #[error("Android {0} 버전용 APK 링크가 없습니다")]
    MissingAndroidCatalog(u32),

    #[error("지원하지 않는 Android 버전입니다: {0}")]
    UnsupportedAndroidVersion(u32),

    #[error("다운로드 실패: {0}")]
    Download(String),

    #[error("ADB 오류: {0}")]
    Adb(String),

    #[error("기기 검증 실패: {0}")]
    Device(String),

    #[error("필수 파일이 없습니다: {0}")]
    MissingFile(PathBuf),

    #[error("필수 패키지 APK 링크가 없습니다: Android {android}, {package}")]
    MissingPackage { android: u32, package: String },
}

pub type Result<T> = std::result::Result<T, GbstError>;
