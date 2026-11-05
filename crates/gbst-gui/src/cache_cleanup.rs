// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, io, path::Path};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CleanupReport {
    pub files_removed: usize,
    pub bytes_freed: u64,
}

fn is_cache_file(filename: &str) -> bool {
    let file = filename.to_ascii_lowercase();
    [".apk", ".apk.gbst-cache.json", ".part", ".tmp", ".download"]
        .iter()
        .any(|suffix| file.ends_with(suffix))
}
fn is_version_dir(name: &str) -> bool {
    name.strip_prefix("Android ")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some_and(|v| (13..=99).contains(&v))
}

// Deletes only GBST-owned cache artifacts directly inside apk/Android N.
// Preserves all unknown files, configuration, logs, ADB keys and links.
pub fn clean_owned_apk_cache(root: &Path) -> io::Result<CleanupReport> {
    let meta = match fs::symlink_metadata(root) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(CleanupReport::default());
        }
        Err(err) => return Err(err),
    };
    if !meta.file_type().is_dir() || meta.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "GBST cache root must be a real directory",
        ));
    }
    let mut result = CleanupReport::default();
    for item in fs::read_dir(root)? {
        let item = item?;
        let Some(name) = item.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !is_version_dir(&name) {
            continue;
        }
        let folder = item.path();
        let meta = fs::symlink_metadata(&folder)?;
        if !meta.file_type().is_dir() || meta.file_type().is_symlink() {
            continue;
        }
        for item in fs::read_dir(&folder)? {
            let item = item?;
            let path = item.path();
            let Some(name) = item.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !is_cache_file(&name) {
                continue;
            }
            let meta = fs::symlink_metadata(&path)?;
            if !meta.file_type().is_file() || meta.file_type().is_symlink() {
                continue;
            }
            fs::remove_file(&path)?;
            result.files_removed += 1;
            result.bytes_freed += meta.len();
        }
        if fs::read_dir(&folder)?.next().is_none() {
            fs::remove_dir(&folder)?;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static ID: AtomicU64 = AtomicU64::new(0);
    fn test_dir() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "gbst_cache_cleanup_{}_{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn cleanup_removes_only_own_apks_and_hash_manifests() {
        let base = test_dir();
        let root = base.join("dwas_kr").join("GBST").join("apk");
        let dir = root.join("Android 13");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("com.google.android.gms.apk"), [1u8, 2, 3]).unwrap();
        fs::write(
            dir.join("com.google.android.gms.apk.gbst-cache.json"),
            b"{}",
        )
        .unwrap();
        fs::write(dir.join("pending.part"), b"x").unwrap();
        fs::write(dir.join("unrelated.txt"), b"retain").unwrap();
        fs::create_dir_all(base.join("adb")).unwrap();
        fs::write(base.join("adb").join("adbkey"), b"secret").unwrap();
        fs::create_dir_all(base.join("logs")).unwrap();
        fs::write(base.join("logs").join("GBST.txt"), b"retain").unwrap();
        fs::write(base.join("language.txt"), b"ko").unwrap();
        let result = clean_owned_apk_cache(&root).unwrap();
        assert_eq!(result.files_removed, 3);
        assert_eq!(result.bytes_freed, 6);
        assert!(dir.join("unrelated.txt").exists());
        assert!(base.join("adb").join("adbkey").exists());
        assert!(base.join("logs").join("GBST.txt").exists());
        assert!(base.join("language.txt").exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn missing_root_and_repeat_cleanup_are_safe() {
        let base = test_dir();
        let missing = base.join("unknown");
        assert_eq!(
            clean_owned_apk_cache(&missing).unwrap(),
            CleanupReport::default()
        );
        let root = base.join("apk");
        fs::create_dir_all(root.join("Android 17")).unwrap();
        fs::create_dir_all(root.join("Android 12")).unwrap();
        fs::write(root.join("Android 17").join("test.apk"), b"ok").unwrap();
        fs::write(root.join("Android 12").join("old.apk"), b"retain").unwrap();
        assert_eq!(clean_owned_apk_cache(&root).unwrap().files_removed, 1);
        assert_eq!(clean_owned_apk_cache(&root).unwrap().files_removed, 0);
        assert!(root.join("Android 12").join("old.apk").exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn linked_directories_are_not_followed() {
        use std::os::unix::fs::symlink;
        let base = test_dir();
        let root = base.join("apk");
        let protected = base.join("protected");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&protected).unwrap();
        fs::write(protected.join("important.apk"), b"retain").unwrap();
        symlink(&protected, root.join("Android 13")).unwrap();
        assert_eq!(clean_owned_apk_cache(&root).unwrap().files_removed, 0);
        assert!(protected.join("important.apk").exists());
        fs::remove_dir_all(base).unwrap();
    }
}
