use std::path::{Path, PathBuf};

const COMPANY_DIR: &str = "dwas_kr";
const APP_DIR: &str = "GBST";
const LANGUAGE_FILE: &str = "language.txt";

pub fn runtime_root() -> PathBuf {
    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.join("Cargo.toml").is_file() && current_dir.join("crates").is_dir() {
            return current_dir;
        }
    }

    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(std::env::temp_dir)
}

pub fn config_root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(runtime_root)
        .join(COMPANY_DIR)
        .join(APP_DIR)
}

pub fn language_config_path() -> PathBuf {
    config_root().join(LANGUAGE_FILE)
}

pub fn runtime_adb_key_path() -> PathBuf {
    runtime_adb_key_dir().join("adbkey")
}

pub fn runtime_adb_key_dir() -> PathBuf {
    runtime_root().join("adb")
}

pub fn stable_adb_key_path() -> PathBuf {
    config_root().join("adb").join("adbkey")
}

pub fn stable_adb_key_dir() -> PathBuf {
    config_root().join("adb")
}


pub fn lpmbox_adb_key_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(runtime_root)
        .join("LPMBox")
        .join("adb")
        .join("adbkey")
}

pub fn adb_key_path() -> PathBuf {
    let stable_path = stable_adb_key_path();
    if stable_path.is_file() {
        return stable_path;
    }

    let legacy_path = runtime_adb_key_path();
    if legacy_path.is_file() {
        if let Some(parent) = stable_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::copy(&legacy_path, &stable_path).is_ok() {
            let legacy_pub = legacy_path.with_extension("pub");
            let stable_pub = stable_path.with_extension("pub");
            if legacy_pub.is_file() {
                let _ = std::fs::copy(legacy_pub, stable_pub);
            }
            return stable_path;
        }
        return legacy_path;
    }

    stable_path
}

pub fn apk_download_dir(android_major: u32) -> PathBuf {
    runtime_root()
        .join("APK")
        .join(format!("Android {android_major}"))
}

pub fn logs_dir() -> PathBuf {
    runtime_root().join("logs")
}

pub fn log_file_path() -> PathBuf {
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M");
    logs_dir().join(format!("GBST_{stamp}.txt"))
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn ensure_runtime_directories() -> std::io::Result<()> {
    std::fs::create_dir_all(config_root())?;
    std::fs::create_dir_all(stable_adb_key_dir())?;
    std::fs::create_dir_all(runtime_adb_key_dir())?;
    std::fs::create_dir_all(runtime_root().join("APK"))?;
    std::fs::create_dir_all(logs_dir())?;
    Ok(())
}
