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
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

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

    on_log(format!(
        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일 다운로드 시작",
        apk_download_progress_bar(0, total),
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

        handles.push(thread::spawn(move || loop {
            let index = next_index.fetch_add(1, Ordering::Relaxed);
            if index >= tasks.len() {
                break;
            }

            let result = prepare_local_apk(&tasks[index]);
            if tx.send((index, result)).is_err() {
                break;
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
                        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일을 다운로드 중...",
                        apk_download_progress_bar(prepared_count, total),
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
                        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일을 다운로드 중...",
                        apk_download_progress_bar(prepared_count, total),
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

    on_log(format!(
        "__SPINNER__|apk_download|[APK] {} Android {} Google Service APK 파일 다운로드 완료",
        apk_download_progress_bar(total, total),
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
        if let Ok(apk) = validate_local_apk(&task.entry, &task.local_path) {
            return Ok(apk);
        }
        let _ = std::fs::remove_file(&task.local_path);
    }

    let mut failures = Vec::new();
    let urls = download_url_candidates(&task.entry.url);

    for url in urls {
        for attempt in 0..3 {
            match download_to_file(&url, &task.local_path) {
                Ok(()) => match validate_local_apk(&task.entry, &task.local_path) {
                    Ok(apk) => return Ok(apk),
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
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(90))
        .timeout_write(Duration::from_secs(30))
        .build()
}

fn remote_content_length(url: &str) -> u64 {
    let agent = download_http_agent();
    for candidate in download_url_candidates(url) {
        let response = agent.head(&candidate)
            .set("User-Agent", concat!("GBST/", env!("CARGO_PKG_VERSION")))
            .set("Accept-Encoding", "identity")
            .call();

        if let Ok(response) = response {
            if let Some(size) = response
                .header("Content-Length")
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
    Ok(read == signature.len() && matches!(signature, [b'P', b'K', 3, 4] | [b'P', b'K', 5, 6] | [b'P', b'K', 7, 8]))
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
        .set("User-Agent", concat!("GBST/", env!("CARGO_PKG_VERSION")))
        .set(
            "Accept",
            "application/vnd.android.package-archive, application/octet-stream, */*",
        )
        .set("Accept-Encoding", "identity")
        .call()
        .map_err(|err| GbstError::Download(format!("다운로드 실패: {err}")))?;

    let expected_content_length = response
        .header("Content-Length")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0);

    let content_type = response
        .header("Content-Type")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if content_type.contains("text/html") {
        return Err(GbstError::Download(format!(
            "APK 대신 HTML 응답을 받았습니다: {content_type}"
        )));
    }

    let mut reader = response.into_reader();
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
        let is_apk = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("apk"));

        if is_temp || (is_apk && !expected_paths.contains(&path)) {
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

fn apk_download_progress_bar(current: usize, total: usize) -> String {
    let total = total.max(1);
    let current = current.min(total);
    let percent = if current >= total {
        100
    } else {
        ((current as f32 / total as f32) * 100.0).round() as usize
    };
    let total_blocks = 20usize;
    let filled = ((percent.min(100) * total_blocks) + 50) / 100;
    let empty = total_blocks.saturating_sub(filled);

    format!(
        "[{}{}] {}%",
        "█".repeat(filled),
        "·".repeat(empty),
        percent.min(100)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_download_tasks_largest_first() {
        let mut tasks = vec![
            DownloadTask {
                entry: ApkEntry { android_major: 16, package: "small".into(), url: "https://example.com/small.apk".into(), line_no: 1 },
                local_path: PathBuf::from("small.apk"),
                expected_size: 10,
            },
            DownloadTask {
                entry: ApkEntry { android_major: 16, package: "large".into(), url: "https://example.com/large.apk".into(), line_no: 2 },
                local_path: PathBuf::from("large.apk"),
                expected_size: 300,
            },
            DownloadTask {
                entry: ApkEntry { android_major: 16, package: "medium".into(), url: "https://example.com/medium.apk".into(), line_no: 3 },
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
