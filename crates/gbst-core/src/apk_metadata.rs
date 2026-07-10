use crate::model::LocalApk;
use std::collections::BTreeMap;
use flate2::read::DeflateDecoder;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const UTF8_FLAG: u32 = 0x0000_0100;
const NO_INDEX: u32 = 0xFFFF_FFFF;
const TYPE_STRING: u8 = 0x03;
const TYPE_INT_DEC: u8 = 0x10;
const TYPE_INT_HEX: u8 = 0x11;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkManifestInfo {
    pub package: String,
    pub version_code: Option<u64>,
}

pub fn apk_version_codes_from_local_apks(apks: &[LocalApk]) -> BTreeMap<String, u64> {
    let mut versions = BTreeMap::new();
    for apk in apks {
        if let Ok(info) = read_apk_manifest_info(&apk.path) {
            let package = if info.package.trim().is_empty() {
                apk.package.clone()
            } else {
                info.package
            };

            if let Some(version_code) = info.version_code {
                versions
                    .entry(package)
                    .and_modify(|existing| {
                        if version_code > *existing {
                            *existing = version_code;
                        }
                    })
                    .or_insert(version_code);
            }
        }
    }
    versions
}

pub fn apk_version_codes_from_dir(dir: &Path) -> BTreeMap<String, u64> {
    let mut versions = BTreeMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return versions;
    };

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

        if let Ok(info) = read_apk_manifest_info(&path) {
            let package = info.package.trim();
            let Some(version_code) = info.version_code else {
                continue;
            };
            if package.is_empty() {
                continue;
            }

            versions
                .entry(package.to_string())
                .and_modify(|existing| {
                    if version_code > *existing {
                        *existing = version_code;
                    }
                })
                .or_insert(version_code);
        }
    }

    versions
}

pub fn read_apk_manifest_info(path: &Path) -> std::io::Result<ApkManifestInfo> {
    let mut file = File::open(path)?;
    let mut apk_bytes = Vec::new();
    file.read_to_end(&mut apk_bytes)?;

    let manifest_bytes = extract_zip_entry(&apk_bytes, "AndroidManifest.xml").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("AndroidManifest.xml을 찾지 못했습니다: {}", path.display()),
        )
    })?;

    parse_binary_android_manifest(&manifest_bytes).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("AndroidManifest.xml 파싱 실패: {}", path.display()),
        )
    })
}

fn extract_zip_entry(bytes: &[u8], entry_name: &str) -> Option<Vec<u8>> {
    let eocd_offset = find_end_of_central_directory(bytes)?;
    let entry_count = read_u16(bytes, eocd_offset + 10)? as usize;
    let central_offset = read_u32(bytes, eocd_offset + 16)? as usize;
    let mut offset = central_offset;

    for _ in 0..entry_count {
        if offset + 46 > bytes.len() || read_u32(bytes, offset)? != 0x0201_4B50 {
            return None;
        }

        let method = read_u16(bytes, offset + 10)?;
        let compressed_size = read_u32(bytes, offset + 20)? as usize;
        let file_name_len = read_u16(bytes, offset + 28)? as usize;
        let extra_len = read_u16(bytes, offset + 30)? as usize;
        let comment_len = read_u16(bytes, offset + 32)? as usize;
        let local_header_offset = read_u32(bytes, offset + 42)? as usize;

        let name_start = offset + 46;
        let name_end = name_start.checked_add(file_name_len)?;
        if name_end > bytes.len() {
            return None;
        }
        let name = String::from_utf8_lossy(&bytes[name_start..name_end]);

        if name == entry_name {
            return extract_zip_local_entry(bytes, local_header_offset, compressed_size, method);
        }

        offset = name_end.checked_add(extra_len)?.checked_add(comment_len)?;
    }

    None
}

fn extract_zip_local_entry(
    bytes: &[u8],
    local_header_offset: usize,
    compressed_size: usize,
    method: u16,
) -> Option<Vec<u8>> {
    if local_header_offset + 30 > bytes.len() || read_u32(bytes, local_header_offset)? != 0x0403_4B50 {
        return None;
    }

    let file_name_len = read_u16(bytes, local_header_offset + 26)? as usize;
    let extra_len = read_u16(bytes, local_header_offset + 28)? as usize;
    let data_start = local_header_offset
        .checked_add(30)?
        .checked_add(file_name_len)?
        .checked_add(extra_len)?;
    let data_end = data_start.checked_add(compressed_size)?;
    if data_end > bytes.len() {
        return None;
    }

    let data = &bytes[data_start..data_end];
    match method {
        0 => Some(data.to_vec()),
        8 => {
            let mut decoder = DeflateDecoder::new(data);
            let mut output = Vec::new();
            decoder.read_to_end(&mut output).ok()?;
            Some(output)
        }
        _ => None,
    }
}

fn find_end_of_central_directory(bytes: &[u8]) -> Option<usize> {
    let min = bytes.len().saturating_sub(65_557);
    let max = bytes.len().saturating_sub(22);
    for offset in (min..=max).rev() {
        if read_u32(bytes, offset) == Some(0x0605_4B50) {
            return Some(offset);
        }
    }
    None
}

fn parse_binary_android_manifest(bytes: &[u8]) -> Option<ApkManifestInfo> {
    if bytes.len() < 8 {
        return None;
    }

    let mut offset = read_u16(bytes, 2)? as usize;
    if offset < 8 || offset > bytes.len() {
        offset = 8;
    }

    let mut strings: Vec<String> = Vec::new();

    while offset + 8 <= bytes.len() {
        let chunk_type = read_u16(bytes, offset)?;
        let header_size = read_u16(bytes, offset + 2)? as usize;
        let chunk_size = read_u32(bytes, offset + 4)? as usize;
        if chunk_size < 8 || offset.checked_add(chunk_size)? > bytes.len() {
            break;
        }

        match chunk_type {
            RES_STRING_POOL_TYPE => {
                strings = parse_string_pool(bytes, offset)?;
            }
            RES_XML_START_ELEMENT_TYPE => {
                if let Some(info) = parse_start_element(bytes, offset, header_size, &strings) {
                    return Some(info);
                }
            }
            _ => {}
        }

        offset += chunk_size;
    }

    None
}

fn parse_string_pool(bytes: &[u8], chunk_offset: usize) -> Option<Vec<String>> {
    let header_size = read_u16(bytes, chunk_offset + 2)? as usize;
    let chunk_size = read_u32(bytes, chunk_offset + 4)? as usize;
    let string_count = read_u32(bytes, chunk_offset + 8)? as usize;
    let flags = read_u32(bytes, chunk_offset + 16)?;
    let strings_start = read_u32(bytes, chunk_offset + 20)? as usize;
    let is_utf8 = flags & UTF8_FLAG != 0;

    if header_size < 28 || chunk_offset.checked_add(chunk_size)? > bytes.len() {
        return None;
    }

    let offsets_start = chunk_offset + header_size;
    let strings_base = chunk_offset + strings_start;
    let chunk_end = chunk_offset + chunk_size;

    let mut strings = Vec::with_capacity(string_count);
    for index in 0..string_count {
        let offset_pos = offsets_start + index * 4;
        let string_offset = read_u32(bytes, offset_pos)? as usize;
        let pos = strings_base.checked_add(string_offset)?;
        if pos >= chunk_end {
            strings.push(String::new());
            continue;
        }

        let value = if is_utf8 {
            read_utf8_string(bytes, pos, chunk_end).unwrap_or_default()
        } else {
            read_utf16_string(bytes, pos, chunk_end).unwrap_or_default()
        };
        strings.push(value);
    }

    Some(strings)
}

fn parse_start_element(
    bytes: &[u8],
    chunk_offset: usize,
    _header_size: usize,
    strings: &[String],
) -> Option<ApkManifestInfo> {
    if chunk_offset + 36 > bytes.len() {
        return None;
    }

    let tag_name_idx = read_u32(bytes, chunk_offset + 20)?;
    let tag_name = string_at(strings, tag_name_idx)?;
    if tag_name != "manifest" {
        return None;
    }

    let attribute_start = read_u16(bytes, chunk_offset + 24)? as usize;
    let attribute_size = read_u16(bytes, chunk_offset + 26)? as usize;
    let attribute_count = read_u16(bytes, chunk_offset + 28)? as usize;
    let attr_size = attribute_size.max(20);
    let attrs_offset = chunk_offset + 16 + attribute_start;

    let mut package = String::new();
    let mut version_code: Option<u64> = None;
    let mut version_code_major: Option<u64> = None;

    for index in 0..attribute_count {
        let attr_offset = attrs_offset + index * attr_size;
        if attr_offset + 20 > bytes.len() {
            continue;
        }

        let attr_name_idx = read_u32(bytes, attr_offset + 4).unwrap_or(NO_INDEX);
        let raw_value_idx = read_u32(bytes, attr_offset + 8).unwrap_or(NO_INDEX);
        let data_type = read_u8(bytes, attr_offset + 15).unwrap_or(0);
        let data = read_u32(bytes, attr_offset + 16).unwrap_or(0);

        let Some(attr_name) = string_at(strings, attr_name_idx) else {
            continue;
        };

        match attr_name.as_str() {
            "package" => {
                package = value_as_string(strings, raw_value_idx, data_type, data)
                    .unwrap_or_default();
            }
            "versionCode" => {
                version_code = value_as_u64(strings, raw_value_idx, data_type, data);
            }
            "versionCodeMajor" => {
                version_code_major = value_as_u64(strings, raw_value_idx, data_type, data);
            }
            _ => {}
        }
    }

    let combined_version = match (version_code, version_code_major) {
        (Some(code), Some(major)) => Some((major << 32) | code),
        (Some(code), None) => Some(code),
        (None, Some(major)) => Some(major << 32),
        (None, None) => None,
    };

    Some(ApkManifestInfo {
        package,
        version_code: combined_version,
    })
}

fn value_as_string(strings: &[String], raw_value_idx: u32, data_type: u8, data: u32) -> Option<String> {
    if raw_value_idx != NO_INDEX {
        return string_at(strings, raw_value_idx);
    }
    if data_type == TYPE_STRING {
        return string_at(strings, data);
    }
    Some(data.to_string())
}

fn value_as_u64(strings: &[String], raw_value_idx: u32, data_type: u8, data: u32) -> Option<u64> {
    if raw_value_idx != NO_INDEX {
        if let Some(raw) = string_at(strings, raw_value_idx) {
            return parse_u64_from_text(&raw);
        }
    }

    match data_type {
        TYPE_INT_DEC | TYPE_INT_HEX => Some(data as u64),
        TYPE_STRING => string_at(strings, data).and_then(|value| parse_u64_from_text(&value)),
        _ => Some(data as u64),
    }
}

fn parse_u64_from_text(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u64>().ok()
    }
}

fn string_at(strings: &[String], index: u32) -> Option<String> {
    if index == NO_INDEX {
        return None;
    }
    strings.get(index as usize).cloned()
}

fn read_utf8_string(bytes: &[u8], mut pos: usize, end: usize) -> Option<String> {
    let (_, used_utf16_len) = read_length8(bytes, pos, end)?;
    pos += used_utf16_len;
    let (byte_len, used_byte_len) = read_length8(bytes, pos, end)?;
    pos += used_byte_len;

    let str_end = pos.checked_add(byte_len)?;
    if str_end > end || str_end > bytes.len() {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes[pos..str_end]).to_string())
}

fn read_utf16_string(bytes: &[u8], mut pos: usize, end: usize) -> Option<String> {
    let (unit_len, used) = read_length16(bytes, pos, end)?;
    pos += used;
    let byte_len = unit_len.checked_mul(2)?;
    let str_end = pos.checked_add(byte_len)?;
    if str_end > end || str_end > bytes.len() {
        return None;
    }

    let mut units = Vec::with_capacity(unit_len);
    let mut current = pos;
    while current + 1 < str_end {
        units.push(u16::from_le_bytes([bytes[current], bytes[current + 1]]));
        current += 2;
    }

    Some(String::from_utf16_lossy(&units))
}

fn read_length8(bytes: &[u8], pos: usize, end: usize) -> Option<(usize, usize)> {
    if pos >= end || pos >= bytes.len() {
        return None;
    }
    let first = bytes[pos];
    if first & 0x80 != 0 {
        if pos + 1 >= end || pos + 1 >= bytes.len() {
            return None;
        }
        Some(((((first & 0x7F) as usize) << 8) | bytes[pos + 1] as usize, 2))
    } else {
        Some((first as usize, 1))
    }
}

fn read_length16(bytes: &[u8], pos: usize, end: usize) -> Option<(usize, usize)> {
    if pos + 1 >= end || pos + 1 >= bytes.len() {
        return None;
    }
    let first = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]);
    if first & 0x8000 != 0 {
        if pos + 3 >= end || pos + 3 >= bytes.len() {
            return None;
        }
        let second = u16::from_le_bytes([bytes[pos + 2], bytes[pos + 3]]);
        Some(((((first & 0x7FFF) as usize) << 16) | second as usize, 4))
    } else {
        Some((first as usize, 2))
    }
}

fn read_u8(bytes: &[u8], offset: usize) -> Option<u8> {
    bytes.get(offset).copied()
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let bytes = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
