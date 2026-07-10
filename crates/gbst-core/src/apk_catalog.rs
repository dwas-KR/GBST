use crate::error::{GbstError, Result};
use crate::model::ApkEntry;
use std::collections::BTreeMap;

pub const REMOTE_APK_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/dwas-KR/GBST/refs/heads/Download/GBST_apk.txt";

const REDACTED_NOTICE: &str = "This code is part of the program's core implementation and has been commented out.";

#[derive(Debug, Clone, Default)]
pub struct ApkCatalog {
    by_android: BTreeMap<u32, Vec<ApkEntry>>,
}

impl ApkCatalog {
    pub fn load_from_remote_catalog<F>(_url: &str, mut on_log: F) -> Result<Self>
    where
        F: FnMut(String),
    {
        on_log(REDACTED_NOTICE.to_string());
        Err(GbstError::Catalog(REDACTED_NOTICE.to_string()))
    }

    pub fn parse(_text: &str) -> Result<Self> {
        Err(GbstError::Catalog(REDACTED_NOTICE.to_string()))
    }

    pub fn entries_for_android(&self, android_major: u32) -> Result<Vec<ApkEntry>> {
        let entries = self.by_android.get(&android_major).cloned().unwrap_or_default();

        if entries.is_empty() {
            return Err(GbstError::MissingAndroidCatalog(android_major));
        }

        Ok(entries)
    }

    pub fn android_versions(&self) -> Vec<u32> {
        self.by_android.keys().copied().collect()
    }
}
