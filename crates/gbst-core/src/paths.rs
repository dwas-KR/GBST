use std::fs;
use std::path::{Path, PathBuf};

const COMPANY_DIR: &str = "dwas_kr";
const APP_DIR: &str = "GBST";
const LANGUAGE_FILE: &str = "language.txt";
const ADB_KEY_FILE: &str = "adbkey";
const ADB_PUBLIC_KEY_FILE: &str = "adbkey.pub";

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

pub fn company_root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(runtime_root)
        .join(COMPANY_DIR)
}

pub fn config_root() -> PathBuf {
    company_root().join(APP_DIR)
}

pub fn apk_cache_dir() -> PathBuf {
    config_root().join("apk")
}

pub fn language_config_path() -> PathBuf {
    company_root().join(LANGUAGE_FILE)
}

fn legacy_language_config_path() -> PathBuf {
    config_root().join(LANGUAGE_FILE)
}

pub fn stable_adb_key_path() -> PathBuf {
    stable_adb_key_dir().join(ADB_KEY_FILE)
}

pub fn stable_adb_key_dir() -> PathBuf {
    company_root().join("adb")
}

fn legacy_stable_adb_key_dir() -> PathBuf {
    config_root().join("adb")
}

pub fn lpmbox_adb_key_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(runtime_root)
        .join("LPMBox")
        .join("adb")
        .join(ADB_KEY_FILE)
}

pub fn adb_key_path() -> PathBuf {
    stable_adb_key_path()
}

pub fn apk_download_dir(android_major: u32) -> PathBuf {
    apk_cache_dir().join(format!("Android {android_major}"))
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
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn move_or_remove_legacy_file(source: &Path, target: &Path) -> std::io::Result<()> {
    if !source.is_file() {
        return Ok(());
    }

    if target.is_file() {
        fs::remove_file(source)?;
        return Ok(());
    }

    ensure_parent(target)?;

    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, target)?;
            fs::remove_file(source)
        }
    }
}

fn migrate_legacy_adb_path() -> std::io::Result<()> {
    let legacy_dir = legacy_stable_adb_key_dir();
    let stable_dir = stable_adb_key_dir();
    fs::create_dir_all(&stable_dir)?;

    move_or_remove_legacy_file(
        &legacy_dir.join(ADB_KEY_FILE),
        &stable_dir.join(ADB_KEY_FILE),
    )?;
    move_or_remove_legacy_file(
        &legacy_dir.join(ADB_PUBLIC_KEY_FILE),
        &stable_dir.join(ADB_PUBLIC_KEY_FILE),
    )?;

    if legacy_dir.exists() {
        fs::remove_dir_all(legacy_dir)?;
    }

    Ok(())
}

fn migrate_legacy_language_path() -> std::io::Result<()> {
    move_or_remove_legacy_file(
        &legacy_language_config_path(),
        &language_config_path(),
    )
}

pub fn migrate_legacy_paths() -> std::io::Result<()> {
    fs::create_dir_all(company_root())?;
    fs::create_dir_all(config_root())?;
    migrate_legacy_adb_path()?;
    migrate_legacy_language_path()?;
    Ok(())
}

pub fn ensure_runtime_directories() -> std::io::Result<()> {
    migrate_legacy_paths()?;
    fs::create_dir_all(apk_cache_dir())?;
    fs::create_dir_all(stable_adb_key_dir())?;
    fs::create_dir_all(logs_dir())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apk_download_dir_uses_local_appdata_cache_root() {
        assert_eq!(
            apk_download_dir(13),
            config_root().join("apk").join("Android 13")
        );
    }
}
