use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AndroidMajor {
    Android13,
    Android14,
    Android15,
    Android16,
    Android17,
    Android18,
}

impl AndroidMajor {
    pub fn parse(value: &str) -> Option<Self> {
        let first = value
            .trim()
            .split(|ch: char| !ch.is_ascii_digit())
            .find(|part| !part.is_empty())?;
        let major = first.parse::<u32>().ok()?;
        Self::from_u32(major)
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            13 => Some(Self::Android13),
            14 => Some(Self::Android14),
            15 => Some(Self::Android15),
            16 => Some(Self::Android16),
            17 => Some(Self::Android17),
            18 => Some(Self::Android18),
            _ => None,
        }
    }

    pub fn value(self) -> u32 {
        match self {
            Self::Android13 => 13,
            Self::Android14 => 14,
            Self::Android15 => 15,
            Self::Android16 => 16,
            Self::Android17 => 17,
            Self::Android18 => 18,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub manufacturer: String,
    pub model: String,
    pub android_version: String,
    pub android_major: AndroidMajor,
}


pub const GOOGLE_REQUIRED_PACKAGES: [&str; 0] = [];

#[derive(Debug, Clone)]
pub struct DashboardInfo {
    pub model_name: String,
    pub android_version: String,
    pub manufacturer: String,
    pub rom_type: String,
    pub google_service_status: String,
    pub is_lenovo: bool,
    pub android_major: Option<AndroidMajor>,
}

impl DashboardInfo {
    pub fn unknown() -> Self {
        Self {
            model_name: "알 수 없음".to_string(),
            android_version: "알 수 없음".to_string(),
            manufacturer: "알 수 없음".to_string(),
            rom_type: "알 수 없음".to_string(),
            google_service_status: "알 수 없음".to_string(),
            is_lenovo: false,
            android_major: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApkEntry {
    pub android_major: u32,
    pub package: String,
    pub url: String,
    pub line_no: usize,
}

#[derive(Debug, Clone)]
pub struct LocalApk {
    pub package: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePolicy {
    Stop,
    Continue,
}

#[derive(Debug, Clone)]
pub enum PlanStep {
    Shell {
        label: String,
        command: String,
        policy: FailurePolicy,
    },
    InstallApk {
        package: String,
        path: PathBuf,
        policy: FailurePolicy,
    },
    RebootAndWait {
        label: String,
        policy: FailurePolicy,
    },
    Delay {
        label: String,
        seconds: u64,
        policy: FailurePolicy,
    },
    GrantRequestedPermissions {
        label: String,
        package: String,
        permissions: Vec<String>,
        policy: FailurePolicy,
    },
    AppOpsAllowExisting {
        label: String,
        package: String,
        ops: Vec<String>,
        policy: FailurePolicy,
    },
    HealthCheck {
        label: String,
        packages: Vec<String>,
        policy: FailurePolicy,
    },
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub android_major: AndroidMajor,
    pub steps: Vec<PlanStep>,
}
