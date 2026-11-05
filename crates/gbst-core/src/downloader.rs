// SPDX-License-Identifier: GPL-3.0-only
use crate::apk_metadata::read_apk_manifest_info;
use crate::error::{GbstError, Result};
use crate::model::{ApkEntry, LocalApk};
use crate::paths;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

const PINNED_APK_SHA256: &str = include_str!("../../../GBST_APK_SHA256SUMS.txt");

#[derive(Clone)]
struct DownloadTask {
    entry: ApkEntry,
    local_path: PathBuf,
    expected_size: u64,
}

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

    let total = entries.len().max(1);
    let mut tasks = entries
        .iter()
        .cloned()
        .map(|entry| {
            let file_name = download_file_name(&entry);
            let local_path = dir.join(file_name);
            let cached_size = local_path
                .metadata()
                .ok()
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            let expected_size = if cached_size > 0 {
                cached_size
            } else {
                remote_content_length(&entry.url)
            };

            DownloadTask {
                entry,
                local_path,
                expected_size,
            }
        })
        .collect::<Vec<_>>();
    sort_download_tasks_largest_first(&mut tasks);

    // Typed progress is rendered by the LTBox-style Iced progress bar,
    // not embedded as Unicode blocks in the human-readable APK log.
    on_log("__GBST_DOWNLOAD_PROGRESS__|0".to_string());
    on_log(format!(
        "__SPINNER__|apk_download|[APK] Android {} Google Service APK 파일 다운로드 시작",
        android_major,
    ));

    let worker_count = tasks.len().min(4).max(1);
    let shared_tasks = Arc::new(tasks);
    let next_index = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(worker_count);

    for _ in 0..worker_count {
        let tasks = Arc::clone(&shared_tasks);
        let next_index = Arc::clone(&next_index);
        let tx = tx.clone();

        handles.push(thread::spawn(move || {
            loop {
                let index = next_index.fetch_add(1, Ordering::Relaxed);
                if index >= tasks.len() {
                    break;
                }

                let result = prepare_local_apk(&tasks[index]);
                if tx.send((index, result)).is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);

    let mut ordered = vec![None; shared_tasks.len()];
    let mut failure_details = vec![None; shared_tasks.len()];
    let mut prepared_count = 0usize;

    for (index, result) in rx {
        match result {
            Ok(apk) => {
                if ordered[index].is_none() {
                    prepared_count = prepared_count.saturating_add(1);
                }
                ordered[index] = Some(apk);
                failure_details[index] = None;

                if prepared_count < total {
                    on_log(format!(
                        "__GBST_DOWNLOAD_PROGRESS__|{}",
                        apk_download_percent(prepared_count, total),
                    ));
                    on_log(format!(
                        "__SPINNER__|apk_download|[APK] Android {} Google Service APK 파일을 다운로드 중...",
                        android_major,
                    ));
                }
            }
            Err(err) => {
                failure_details[index] = Some(err.to_string());
            }
        }
    }

    let mut worker_panicked = false;
    for handle in handles {
        if handle.join().is_err() {
            worker_panicked = true;
        }
    }

    let mut retry_indexes = Vec::new();
    for (index, task) in shared_tasks.iter().enumerate() {
        let valid = ordered[index].as_ref().is_some_and(|apk| {
            apk.path.is_file() && validate_local_apk(&task.entry, &apk.path).is_ok()
        });

        if !valid {
            ordered[index] = None;
            retry_indexes.push(index);
        }
    }

    for index in retry_indexes {
        thread::sleep(Duration::from_millis(1000));

        match prepare_local_apk(&shared_tasks[index]) {
            Ok(apk) => {
                if ordered[index].is_none() {
                    prepared_count = prepared_count.saturating_add(1);
                }
                ordered[index] = Some(apk);
                failure_details[index] = None;

                if prepared_count < total {
                    on_log(format!(
                        "__GBST_DOWNLOAD_PROGRESS__|{}",
                        apk_download_percent(prepared_count, total),
                    ));
                    on_log(format!(
                        "__SPINNER__|apk_download|[APK] Android {} Google Service APK 파일을 다운로드 중...",
                        android_major,
                    ));
                }
            }
            Err(err) => {
                failure_details[index] = Some(err.to_string());
            }
        }
    }

    let unresolved = shared_tasks
        .iter()
        .enumerate()
        .filter_map(|(index, task)| {
            if ordered[index].is_some() {
                return None;
            }

            Some(format!(
                "{} ({})",
                task.entry.package,
                failure_details[index]
                    .as_deref()
                    .unwrap_or("다운로드 결과 없음")
            ))
        })
        .collect::<Vec<_>>();

    if !unresolved.is_empty() {
        let worker_status = if worker_panicked {
            " / 다운로드 작업 스레드 비정상 종료 감지"
        } else {
            ""
        };

        return Err(GbstError::Download(format!(
            "APK 다운로드 및 최종 검증에 실패했습니다: {}{}",
            unresolved.join(" | "),
            worker_status
        )));
    }

    let result = ordered
        .into_iter()
        .enumerate()
        .map(|(index, apk)| {
            apk.ok_or_else(|| {
                GbstError::Download(format!(
                    "APK 다운로드 결과가 누락되었습니다: {}",
                    shared_tasks[index].entry.package
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let expected_paths = result
        .iter()
        .map(|apk| apk.path.clone())
        .collect::<HashSet<_>>();
    prune_download_dir(&dir, &expected_paths)?;

    on_log("__GBST_DOWNLOAD_PROGRESS__|100".to_string());
    on_log(format!(
        "__SPINNER__|apk_download|[APK] Android {} Google Service APK 파일 다운로드 완료",
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
    for paths in map.values_mut() {
        paths.sort();
    }
    map
}

fn prepare_local_apk(task: &DownloadTask) -> Result<LocalApk> {
    migrate_legacy_cached_apk(task)?;

    if task.local_path.is_file() && task.local_path.metadata()?.len() > 0 {
        if cached_file_matches_catalog(&task.entry, &task.local_path)
            && verify_pinned_apk_integrity(&task.entry, &task.local_path).is_ok()
        {
            if let Ok(apk) = validate_local_apk(&task.entry, &task.local_path) {
                return Ok(apk);
            }
        }
        // Missing/invalid metadata forces a one-time refresh of older caches.
        let _ = std::fs::remove_file(&task.local_path);
        let _ = std::fs::remove_file(cache_record_path(&task.local_path));
    }

    let mut failures = Vec::new();
    let urls = download_url_candidates(&task.entry.url);

    for url in urls {
        for attempt in 0..3 {
            match download_to_file(&url, &task.local_path) {
                Ok(()) => match validate_local_apk(&task.entry, &task.local_path) {
                    Ok(apk) => match verify_pinned_apk_integrity(&task.entry, &task.local_path)
                        .and_then(|_| write_cache_record(&task.entry, &task.local_path))
                    {
                        Ok(()) => return Ok(apk),
                        Err(err) => {
                            failures.push(format!(
                                "{}: 캐시 기록 실패: {err}",
                                compact_download_url(&url)
                            ));
                            let _ = std::fs::remove_file(&task.local_path);
                        }
                    },
                    Err(err) => {
                        failures.push(format!("{}: {err}", compact_download_url(&url)));
                        let _ = std::fs::remove_file(&task.local_path);
                    }
                },
                Err(err) => {
                    failures.push(format!("{}: {err}", compact_download_url(&url)));
                    let _ = std::fs::remove_file(&task.local_path);
                }
            }

            remove_partial_download(&task.local_path);
            if attempt < 2 {
                thread::sleep(Duration::from_millis(750 * (attempt as u64 + 1)));
            }
        }
    }

    Err(GbstError::Download(format!(
        "APK 다운로드 및 검증에 실패했습니다: {} / {}",
        task.entry.package,
        failures.join(" | ")
    )))
}

fn migrate_legacy_cached_apk(task: &DownloadTask) -> Result<()> {
    if task.local_path.exists() {
        return Ok(());
    }

    let Some(parent) = task.local_path.parent() else {
        return Ok(());
    };
    let legacy_path = parent.join(legacy_download_file_name(&task.entry));
    if legacy_path == task.local_path || !legacy_path.is_file() {
        return Ok(());
    }

    match validate_local_apk(&task.entry, &legacy_path) {
        Ok(_) => {
            std::fs::rename(legacy_path, &task.local_path)?;
        }
        Err(_) => {
            let _ = std::fs::remove_file(legacy_path);
        }
    }

    Ok(())
}

fn validate_local_apk(entry: &ApkEntry, local_path: &Path) -> Result<LocalApk> {
    let manifest = read_apk_manifest_info(local_path).map_err(|err| {
        GbstError::Download(format!(
            "다운로드한 APK 정보를 읽지 못했습니다: {}: {err}",
            local_path.display()
        ))
    })?;
    let expected_package = canonical_catalog_package(&entry.package);
    let actual_package = manifest.package.trim();

    if actual_package != expected_package {
        return Err(GbstError::Download(format!(
            "APK 패키지명이 목록과 일치하지 않습니다: 목록={expected_package}, APK={actual_package}, 파일={}",
            local_path.display()
        )));
    }

    Ok(LocalApk {
        package: actual_package.to_string(),
        path: local_path.to_path_buf(),
    })
}

/// Local cache integrity only; not a publisher or signature verification.
fn cache_record_path(apk_path: &Path) -> PathBuf {
    apk_path.with_extension("apk.gbst-cache.json")
}

fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut reader = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let len = reader.read(&mut buffer)?;
        if len == 0 {
            break;
        }
        hasher.update(&buffer[..len]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Release-asset digest pinned at build time from the project transparency manifest.
/// A new release asset requires a deliberate source update and a new build.
fn expected_pinned_sha256(url: &str) -> Option<&'static str> {
    let filename = url.split('?').next()?.rsplit('/').next()?;
    PINNED_APK_SHA256.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let recorded_filename = parts.next()?;
        if recorded_filename.replace('~', "-") == filename {
            Some(digest)
        } else {
            None
        }
    })
}

fn verify_pinned_apk_integrity(entry: &ApkEntry, path: &Path) -> Result<()> {
    let expected = expected_pinned_sha256(&entry.url).ok_or_else(|| {
        GbstError::Download(format!(
            "신규 APK는 프로그램 내 신뢰 목록에 등록되어 있지 않습니다: {}",
            entry.url
        ))
    })?;
    let actual = hash_file_sha256(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(GbstError::Download(format!(
            "APK SHA-256 불일치: {}. 공식 배포 목록의 원본과 다릅니다.",
            entry.package
        )));
    }
    Ok(())
}

fn cached_file_matches_catalog(entry: &ApkEntry, apk_path: &Path) -> bool {
    let record = match std::fs::read(cache_record_path(apk_path))
        .ok()
        .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
    {
        Some(record) => record,
        None => return false,
    };
    let Ok(metadata) = apk_path.metadata() else {
        return false;
    };
    if !metadata.is_file() || metadata.len() == 0 {
        return false;
    }
    let expected_url = record.get("url").and_then(|value| value.as_str());
    let expected_size = record.get("size").and_then(|value| value.as_u64());
    let expected_sha = record.get("sha256").and_then(|value| value.as_str());
    if expected_url != Some(entry.url.as_str()) || expected_size != Some(metadata.len()) {
        return false;
    }
    match (expected_sha, hash_file_sha256(apk_path)) {
        (Some(expected), Ok(actual)) => expected == actual,
        _ => false,
    }
}

fn write_cache_record(entry: &ApkEntry, apk_path: &Path) -> Result<()> {
    let record = serde_json::json!({
        "format": 1,
        "url": entry.url,
        "size": apk_path.metadata()?.len(),
        "sha256": hash_file_sha256(apk_path)?,
    });
    let target = cache_record_path(apk_path);
    let tmp = target.with_extension("json.part");
    let data = serde_json::to_vec_pretty(&record)
        .map_err(|e| GbstError::Download(format!("캐시 정보 직렬화 실패: {e}")))?;
    std::fs::write(&tmp, data)?;
    if target.exists() {
        std::fs::remove_file(&target)?;
    }
    std::fs::rename(&tmp, &target)?;
    Ok(())
}

fn canonical_catalog_package(package: &str) -> &str {
    let Some((base, suffix)) = package.rsplit_once('_') else {
        return package;
    };

    if !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()) {
        base
    } else {
        package
    }
}

fn sort_download_tasks_largest_first(tasks: &mut [DownloadTask]) {
    tasks.sort_by(|left, right| {
        right
            .expected_size
            .cmp(&left.expected_size)
            .then_with(|| left.entry.line_no.cmp(&right.entry.line_no))
    });
}

fn download_http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_body(Some(Duration::from_secs(90)))
        .timeout_send_body(Some(Duration::from_secs(30)))
        .build()
        .into()
}

fn remote_content_length(url: &str) -> u64 {
    let agent = download_http_agent();
    for candidate in download_url_candidates(url) {
        let response = agent
            .head(&candidate)
            .header("User-Agent", concat!("GBST/", env!("CARGO_PKG_VERSION")))
            .header("Accept-Encoding", "identity")
            .call();

        if let Ok(response) = response {
            if let Some(size) = response
                .headers()
                .get("Content-Length")
                .and_then(|header| header.to_str().ok())
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
            {
                return size;
            }
        }
    }

    0
}

fn file_has_zip_magic(path: &Path) -> Result<bool> {
    let mut file = std::fs::File::open(path)?;
    let mut signature = [0_u8; 4];
    let read = file.read(&mut signature)?;
    Ok(read == signature.len()
        && matches!(
            signature,
            [b'P', b'K', 3, 4] | [b'P', b'K', 5, 6] | [b'P', b'K', 7, 8]
        ))
}

fn download_to_file(url: &str, path: &Path) -> Result<()> {
    paths::ensure_parent(path)?;

    let mut temp_name = path.as_os_str().to_os_string();
    temp_name.push(".part");
    let temp_path = PathBuf::from(temp_name);
    let _ = std::fs::remove_file(&temp_path);

    let agent = download_http_agent();
    let response = agent
        .get(url)
        .header("User-Agent", concat!("GBST/", env!("CARGO_PKG_VERSION")))
        .header(
            "Accept",
            "application/vnd.android.package-archive, application/octet-stream, */*",
        )
        .header("Accept-Encoding", "identity")
        .call()
        .map_err(|err| GbstError::Download(format!("다운로드 실패: {err}")))?;

    let expected_content_length = response
        .headers()
        .get("Content-Length")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0);

    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|header| header.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if content_type.contains("text/html") {
        return Err(GbstError::Download(format!(
            "APK 대신 HTML 응답을 받았습니다: {content_type}"
        )));
    }

    let mut reader = response.into_body().into_reader();
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    let mut writer = BufWriter::with_capacity(4 * 1024 * 1024, file);
    let mut buffer = vec![0_u8; 4 * 1024 * 1024];

    let copy_result = (|| -> Result<u64> {
        let mut written = 0u64;
        loop {
            let read = reader
                .read(&mut buffer)
                .map_err(|err| GbstError::Download(format!("다운로드 실패: {err}")))?;
            if read == 0 {
                break;
            }
            writer.write_all(&buffer[..read])?;
            written = written.saturating_add(read as u64);
        }
        writer.flush()?;
        Ok(written)
    })();

    let written = match copy_result {
        Ok(written) => written,
        Err(err) => {
            drop(writer);
            let _ = std::fs::remove_file(&temp_path);
            return Err(err);
        }
    };
    drop(writer);

    if written == 0 {
        let _ = std::fs::remove_file(&temp_path);
        return Err(GbstError::Download(
            "다운로드한 APK 파일의 크기가 0바이트입니다.".to_string(),
        ));
    }

    if let Some(expected) = expected_content_length {
        if written != expected {
            let _ = std::fs::remove_file(&temp_path);
            return Err(GbstError::Download(format!(
                "APK 다운로드 크기가 응답과 일치하지 않습니다: 예상={expected}바이트, 실제={written}바이트"
            )));
        }
    }

    if !file_has_zip_magic(&temp_path)? {
        let _ = std::fs::remove_file(&temp_path);
        return Err(GbstError::Download(
            "다운로드한 파일이 APK/ZIP 형식이 아닙니다.".to_string(),
        ));
    }

    if path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(&temp_path, path)?;
    Ok(())
}

fn download_file_name(entry: &ApkEntry) -> String {
    format!("{}.apk", sanitize_file_name(&entry.package))
}

fn legacy_download_file_name(entry: &ApkEntry) -> String {
    let package = sanitize_file_name(&entry.package);
    let identity = download_identity(&entry.url);
    format!("{package}_{identity}.apk")
}

fn download_identity(url: &str) -> String {
    if let Some(query) = url.split_once('?').map(|(_, query)| query) {
        for item in query.split('&') {
            let Some((key, value)) = item.split_once('=') else {
                continue;
            };
            if key.eq_ignore_ascii_case("id") && !value.trim().is_empty() {
                return sanitize_file_name(value.trim());
            }
        }
    }

    let digest = Sha256::digest(url.as_bytes());
    hex::encode(&digest[..12])
}

fn download_url_candidates(url: &str) -> Vec<String> {
    let mut urls = Vec::new();

    if let Some(file_id) = google_drive_file_id(url) {
        push_unique_url(&mut urls, url.to_string());
        push_unique_url(
            &mut urls,
            format!(
                "https://drive.usercontent.google.com/download?id={file_id}&export=download&authuser=0&confirm=t"
            ),
        );
        push_unique_url(
            &mut urls,
            format!("https://drive.google.com/uc?export=download&confirm=t&id={file_id}"),
        );
    } else {
        push_unique_url(&mut urls, url.to_string());
    }

    urls
}

fn google_drive_file_id(url: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    for item in query.split('&') {
        let Some((key, value)) = item.split_once('=') else {
            continue;
        };
        if key.eq_ignore_ascii_case("id") && !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn push_unique_url(urls: &mut Vec<String>, url: String) {
    if !urls.iter().any(|existing| existing == &url) {
        urls.push(url);
    }
}

fn remove_partial_download(path: &Path) {
    let mut temp_name = path.as_os_str().to_os_string();
    temp_name.push(".part");
    let _ = std::fs::remove_file(PathBuf::from(temp_name));
}

fn compact_download_url(url: &str) -> String {
    if let Some(file_id) = google_drive_file_id(url) {
        format!("Google Drive id={file_id}")
    } else {
        url.split('?').next().unwrap_or(url).to_string()
    }
}

fn prune_download_dir(dir: &Path, expected_paths: &HashSet<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let is_temp = file_name.ends_with(".part");
        let is_cache_record = file_name.ends_with(".apk.gbst-cache.json");
        let stale_cache_record = is_cache_record && {
            let apk_filename = file_name.trim_end_matches(".gbst-cache.json");
            !expected_paths.contains(&dir.join(apk_filename))
        };
        let is_apk = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("apk"));

        if is_temp || stale_cache_record || (is_apk && !expected_paths.contains(&path)) {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '~') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn apk_download_percent(current: usize, total: usize) -> u8 {
    let total = total.max(1);
    let current = current.min(total);
    ((current as f64 * 100.0 / total as f64).round() as u8).min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apk_download_progress_uses_numeric_events_instead_of_text_blocks() {
        assert_eq!(apk_download_percent(0, 4), 0);
        assert_eq!(apk_download_percent(1, 4), 25);
        assert_eq!(apk_download_percent(3, 4), 75);
        assert_eq!(apk_download_percent(4, 4), 100);
        assert_eq!(apk_download_percent(10, 4), 100);
        assert_eq!(apk_download_percent(0, 0), 0);
    }

    #[test]
    fn all_bundled_release_assets_have_pinned_sha256() {
        let catalog =
            crate::apk_catalog::ApkCatalog::parse(crate::apk_catalog::BUNDLED_APK_CATALOG_TEXT)
                .unwrap();
        for major in 13..=17 {
            for entry in catalog.entries_for_android(major).unwrap() {
                let hash = expected_pinned_sha256(&entry.url);
                assert!(hash.is_some(), "Missing pinned hash: {}", entry.url);
                assert_eq!(hash.unwrap().len(), 64);
            }
        }
    }

    #[test]
    fn cached_apks_are_invalidated_when_url_or_bytes_change() {
        let temp = std::env::temp_dir().join(format!(
            "gbst-cache-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let apk = temp.join("sample.apk");
        let entry = ApkEntry {
            android_major: 16,
            package: "com.example".into(),
            url: "https://example.com/v1.apk".into(),
            line_no: 1,
        };
        std::fs::write(&apk, b"sample bytes").unwrap();
        assert!(!cached_file_matches_catalog(&entry, &apk));
        write_cache_record(&entry, &apk).unwrap();
        assert!(cached_file_matches_catalog(&entry, &apk));
        let mut newer = entry.clone();
        newer.url = "https://example.com/v2.apk".into();
        assert!(!cached_file_matches_catalog(&newer, &apk));
        std::fs::write(&apk, b"tampered data").unwrap();
        assert!(!cached_file_matches_catalog(&entry, &apk));
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn sorts_download_tasks_largest_first() {
        let mut tasks = vec![
            DownloadTask {
                entry: ApkEntry {
                    android_major: 16,
                    package: "small".into(),
                    url: "https://example.com/small.apk".into(),
                    line_no: 1,
                },
                local_path: PathBuf::from("small.apk"),
                expected_size: 10,
            },
            DownloadTask {
                entry: ApkEntry {
                    android_major: 16,
                    package: "large".into(),
                    url: "https://example.com/large.apk".into(),
                    line_no: 2,
                },
                local_path: PathBuf::from("large.apk"),
                expected_size: 300,
            },
            DownloadTask {
                entry: ApkEntry {
                    android_major: 16,
                    package: "medium".into(),
                    url: "https://example.com/medium.apk".into(),
                    line_no: 3,
                },
                local_path: PathBuf::from("medium.apk"),
                expected_size: 100,
            },
        ];

        sort_download_tasks_largest_first(&mut tasks);

        assert_eq!(tasks[0].entry.package, "large");
        assert_eq!(tasks[1].entry.package, "medium");
        assert_eq!(tasks[2].entry.package, "small");
    }
}
