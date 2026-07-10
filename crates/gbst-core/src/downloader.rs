use crate::error::{GbstError, Result};
use crate::apk_metadata::read_apk_manifest_info;
use crate::model::{ApkEntry, LocalApk};
use crate::paths;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn download_apks_for_android<F>(
    android_major: u32,
    entries: &[ApkEntry],
    mut on_log: F,
) -> Result<Vec<LocalApk>>
where
    F: FnMut(String),
{
    let dir = paths::apk_download_dir(android_major);
    std::fs::create_dir_all(&dir)?;

    let tupdatel = entries.len().max(1);
    let mut result = Vec::new();

    on_log(format!(
        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일 다운로드 시작",
        apk_download_progress_bar(0, tupdatel),
        android_major,
    ));

    for (index, entry) in entries.iter().enumerate() {
        on_log(format!(
            "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일을 다운로드 중...",
            apk_download_progress_bar(index, tupdatel),
            android_major,
        ));

        let file_name = file_name_from_url(&entry.url)
            .unwrap_or_else(|| format!("{}_line_{}.apk", entry.package, entry.line_no));
        let local_path = dir.join(sanitize_file_name(&file_name));

        if !(local_path.is_file() && local_path.metadata()?.len() > 0) {
            download_to_file(&entry.url, &local_path)?;
        }

        result.push(LocalApk {
            package: entry.package.clone(),
            path: local_path,
        });
    }

    on_log(format!(
        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일 다운로드 완료",
        apk_download_progress_bar(tupdatel, tupdatel),
        android_major,
    ));

    Ok(result)
}

pub fn local_apks_from_download_dir(android_major: u32) -> Result<Vec<LocalApk>> {
    let dir = paths::apk_download_dir(android_major);
    let mut result = Vec::new();

    let entries = std::fs::read_dir(&dir).map_err(|err| {
        GbstError::Download(format!(
            "다운로드된 APK 폴더를 읽지 못했습니다: {}: {err}",
            dir.display()
        ))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("apk") {
            continue;
        }

        let Ok(info) = read_apk_manifest_info(&path) else {
            continue;
        };

        let package = info.package.trim();
        if package.is_empty() {
            continue;
        }

        result.push(LocalApk {
            package: package.to_string(),
            path,
        });
    }

    if result.is_empty() {
        return Err(GbstError::Download(format!(
            "다운로드된 APK 파일을 찾지 못했습니다: {}",
            dir.display()
        )));
    }

    Ok(result)
}

pub fn group_by_package(apks: &[LocalApk]) -> BTreeMap<String, Vec<PathBuf>> {
    let mut map: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for apk in apks {
        map.entry(apk.package.clone())
            .or_default()
            .push(apk.path.clone());
    }
    map
}

fn download_to_file(url: &str, path: &Path) -> Result<()> {
    paths::ensure_parent(path)?;
    let response = ureq::get(url)
        .call()
        .map_err(|err| GbstError::Download(format!("다운로드 실패: {err}")))?;

    let mut reader = response.into_reader();
    let mut file = File::create(path)?;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| GbstError::Download(format!("다운로드 실패: {err}")))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
    }

    Ok(())
}

fn file_name_from_url(url: &str) -> Option<String> {
    let without_query = url.split('?').next().unwrap_or(url);
    let last = without_query.rsplit('/').next()?.trim();
    if last.is_empty() || !last.to_ascii_lowercase().ends_with(".apk") {
        None
    } else {
        Some(last.to_string())
    }
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '~') { ch } else { '_' })
        .collect()
}

fn apk_download_progress_bar(current: usize, tupdatel: usize) -> String {
    let tupdatel = tupdatel.max(1);
    let current = current.min(tupdatel);
    let percent = if current >= tupdatel {
        100
    } else {
        ((current as f32 / tupdatel as f32) * 100.0).round() as usize
    };
    let tupdatel_blocks = 20usize;
    let filled = ((percent.min(100) * tupdatel_blocks) + 50) / 100;
    let empty = tupdatel_blocks.saturating_sub(filled);

    format!("[{}{}] {}%", "█".repeat(filled), "·".repeat(empty), percent.min(100))
}
