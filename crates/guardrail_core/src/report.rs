use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailReport {
    pub id: String,
    pub timestamp: String,
    pub source: SourceInfo,
    pub summary: ReportSummary,
    pub risks: Vec<RiskEntry>,
    pub checks: Vec<CheckResult>,
    pub next_actions: Vec<NextAction>,
}

impl GuardrailReport {
    pub fn new(
        id: impl Into<String>,
        source: SourceInfo,
        checks: Vec<CheckResult>,
        notes: impl Into<String>,
    ) -> Self {
        let (status, score) = summarize_checks(&checks);
        Self {
            id: id.into(),
            timestamp: Utc::now().to_rfc3339(),
            source,
            summary: ReportSummary {
                status,
                score,
                notes: notes.into(),
            },
            risks: Vec::new(),
            checks,
            next_actions: Vec::new(),
        }
    }
}

fn summarize_checks(checks: &[CheckResult]) -> (ReportStatus, f32) {
    if checks.iter().any(|c| c.status == CheckStatus::Fail) {
        (ReportStatus::Fail, 0.0)
    } else if checks.iter().any(|c| c.status == CheckStatus::Warn) {
        (ReportStatus::Warn, 0.7)
    } else {
        (ReportStatus::Pass, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub prompt_path: std::path::PathBuf,
    pub response_path: std::path::PathBuf,
    pub diff_path: std::path::PathBuf,
    #[serde(default)]
    pub spec_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub status: ReportStatus,
    pub score: f32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntry {
    pub category: String,
    pub description: String,
    pub severity: String,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub details: String,
    pub log_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
    Warn,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    pub description: String,
    pub owner: Option<String>,
    pub linked_checklist: Option<String>,
}
