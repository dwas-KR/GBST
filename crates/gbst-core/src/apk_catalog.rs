use crate::error::{GbstError, Result};
use crate::model::ApkEntry;
use base64::{engine::general_purpose, Engine as _};
use std::collections::BTreeMap;

pub const REMOTE_APK_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/dwas-KR/GBST/refs/heads/Download/GBST_apk.txt";

#[derive(Debug, Clone, Default)]
pub struct ApkCatalog {
    by_android: BTreeMap<u32, Vec<ApkEntry>>,
}

impl ApkCatalog {
    pub fn load_from_github_base64<F>(url: &str, mut on_log: F) -> Result<Self>
    where
        F: FnMut(String),
    {
        let url = url.trim();
        if !(url.starts_with("https://raw.githubusercontent.com/")
            || url.starts_with("https://gist.githubusercontent.com/"))
        {
            return Err(GbstError::Catalog(
                "GitHub raw URL만 사용할 수 있습니다. raw.githubusercontent.com 주소를 입력해주세요."
                    .to_string(),
            ));
        }

        on_log("__SPINNER__|apk_catalog|[APK] GitHub 텍스트 읽는 중... │".to_string());

        let encoded = ureq::get(url)
            .call()
            .map_err(|err| GbstError::Download(format!("GitHub Base64 링크 파일 다운로드 실패: {err}")))?
            .into_string()
            .map_err(|err| GbstError::Download(format!("GitHub Base64 링크 파일 읽기 실패: {err}")))?;

        let decoded = decode_base64_document(&encoded)?;
        let catalog = Self::parse(&decoded)?;

        on_log("__SPINNER__|apk_catalog|[APK] GitHub 텍스트를 확인했습니다.".to_string());
        Ok(catalog)
    }

    pub fn parse(text: &str) -> Result<Self> {
        let mut by_android: BTreeMap<u32, Vec<ApkEntry>> = BTreeMap::new();
        let mut current_android: Option<u32> = None;

        for (idx, raw_line) in text.lines().enumerate() {
            let line_no = idx + 1;
            let line = raw_line.trim().trim_start_matches('\u{feff}').trim();

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with('#') {
                if let Some(android) = parse_android_header(line) {
                    current_android = Some(android);
                    by_android.entry(android).or_default();
                }
                continue;
            }

            let Some(android_major) = current_android else {
                return Err(GbstError::Catalog(format!(
                    "{line_no}번째 줄이 Android 섹션 밖에 있습니다: {line}"
                )));
            };

            let Some((package, url)) = line.split_once(':') else {
                return Err(GbstError::Catalog(format!(
                    "{line_no}번째 줄은 `패키지: 링크` 형식이어야 합니다: {line}"
                )));
            };

            let package = package.trim();
            let url = url.trim();

            if package.is_empty() || url.is_empty() {
                return Err(GbstError::Catalog(format!(
                    "{line_no}번째 줄의 패키지명 또는 링크가 비어 있습니다: {line}"
                )));
            }

            if url.eq_ignore_ascii_case("O") {
                continue;
            }

            if !package
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
            {
                return Err(GbstError::Catalog(format!(
                    "{line_no}번째 줄의 패키지명이 올바르지 않습니다: {package}"
                )));
            }

            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err(GbstError::Catalog(format!(
                    "{line_no}번째 줄의 링크는 http/https로 시작해야 합니다: {url}"
                )));
            }

            by_android.entry(android_major).or_default().push(ApkEntry {
                android_major,
                package: package.to_string(),
                url: url.to_string(),
                line_no,
            });
        }

        Ok(Self { by_android })
    }

    pub fn entries_for_android(&self, android_major: u32) -> Result<Vec<ApkEntry>> {
        let entries = self
            .by_android
            .get(&android_major)
            .cloned()
            .unwrap_or_default();

        if entries.is_empty() {
            return Err(GbstError::MissingAndroidCatalog(android_major));
        }

        Ok(entries)
    }

    pub fn android_versions(&self) -> Vec<u32> {
        self.by_android.keys().copied().collect()
    }
}

fn decode_base64_document(encoded_text: &str) -> Result<String> {
    let mut compact = encoded_text
        .trim()
        .trim_start_matches('\u{feff}')
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("");

    if let Some((_, right)) = compact.split_once("base64,") {
        compact = right.trim().to_string();
    }

    compact.retain(|ch| !ch.is_whitespace());

    if compact.is_empty() {
        return Err(GbstError::Catalog(
            "GitHub Base64 APK 링크 파일이 비어 있습니다.".to_string(),
        ));
    }

    let mut padded = compact.clone();
    match padded.len() % 4 {
        0 => {}
        2 => padded.push_str("=="),
        3 => padded.push('='),
        _ => {
            return Err(GbstError::Catalog(
                "Base64 길이가 올바르지 않습니다. Encoder 결과 전체를 업로드했는지 확인해주세요."
                    .to_string(),
            ))
        }
    }

    let bytes = general_purpose::STANDARD
        .decode(padded.as_bytes())
        .or_else(|_| general_purpose::URL_SAFE.decode(padded.as_bytes()))
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(compact.as_bytes()))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(compact.as_bytes()))
        .map_err(|err| GbstError::Catalog(format!("Base64 해독 실패: {err}")))?;

    String::from_utf8(bytes).map_err(|err| {
        GbstError::Catalog(format!(
            "Base64 해독 결과가 UTF-8 텍스트가 아닙니다: {err}"
        ))
    })
}

fn parse_android_header(line: &str) -> Option<u32> {
    let lowered = line.to_ascii_lowercase();
    let android_pos = lowered.find("android")?;
    let after = &lowered[android_pos + "android".len()..];
    let digits: String = after
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();

    let value = digits.parse::<u32>().ok()?;
    if (13..=18).contains(&value) {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_android_headers() {
        let parsed = ApkCatalog::parse(
            r#"
#Android 13
com.android.vending: https://example.com/vending.apk
com.google.android.gsf: https://example.com/gsf.apk
# Android 18 (미구현)
# Android 14 설명을 붙여도 됩니다
com.google.android.gms: https://example.com/gms.apk
"#,
        )
        .unwrap();

        assert_eq!(parsed.entries_for_android(13).unwrap().len(), 2);
        assert!(parsed.entries_for_android(18).is_err());
        assert_eq!(
            parsed.entries_for_android(14).unwrap()[0].package,
            "com.google.android.gms"
        );
    }

    #[test]
    fn decodes_standard_base64_without_padding() {
        let decoded = decode_base64_document("7JWI64WV7ZWY7IS47JqU").unwrap();
        assert_eq!(decoded, "안녕하세요");
    }
    #[test]
    fn accepts_signature_variant_keys_and_skips_placeholders() {
        let parsed = ApkCatalog::parse(
            r#"
# Android 16
com.google.android.configupdater_1: https://example.com/config_1.apk
com.google.android.configupdater_2: https://example.com/config_2.apk
com.google.android.onetimeinitializer: O
"#,
        )
        .unwrap();

        let entries = parsed.entries_for_android(16).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].package, "com.google.android.configupdater_1");
        assert_eq!(entries[1].package, "com.google.android.configupdater_2");
    }

}
