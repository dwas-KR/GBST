use crate::error::Result;
use crate::model::{AndroidMajor, FailurePolicy, InstallPlan, LocalApk, PlanStep};

const REDACTED_NOTICE: &str = "This code is part of the program's core implementation and has been commented out.";

pub fn build_google_basic_service_plan(
    android_major: AndroidMajor,
    _local_apks: Vec<LocalApk>,
) -> Result<InstallPlan> {
    Ok(InstallPlan {
        android_major,
        steps: vec![PlanStep::Delay {
            label: REDACTED_NOTICE.to_string(),
            seconds: 1,
            policy: FailurePolicy::Continue,
        }],
    })
}
