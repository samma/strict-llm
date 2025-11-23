use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{self, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GuardrailConfig {
    pub sources: SourceConfig,
    #[serde(default)]
    pub analyzers: AnalyzerToggles,
    #[serde(default)]
    pub report: Option<ReportConfig>,
    #[serde(default)]
    pub targets: Option<TargetConfig>,
    #[serde(default)]
    pub telemetry: Option<TelemetryConfig>,
}

impl GuardrailConfig {
    pub fn from_path(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let cfg: GuardrailConfig = toml::from_str(&data)?;
        Ok(cfg)
    }

    pub fn source_info(&self) -> crate::report::SourceInfo {
        crate::report::SourceInfo {
            prompt_path: self.sources.prompt.clone(),
            response_path: self.sources.response.clone(),
            diff_path: self.sources.diff.clone(),
            spec_refs: self.sources.spec_refs.clone().unwrap_or_default(),
        }
    }

    pub fn validate_sources(&self) -> Result<()> {
        self.sources.ensure_exists()
    }
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub prompt: PathBuf,
    pub response: PathBuf,
    pub diff: PathBuf,
    #[serde(default)]
    pub spec_refs: Option<Vec<String>>,
}

impl SourceConfig {
    fn ensure_exists(&self) -> Result<()> {
        for (label, path) in [
            ("prompt", &self.prompt),
            ("response", &self.response),
            ("diff", &self.diff),
        ] {
            if !path.exists() {
                anyhow::bail!("Source {label} missing at {}", path.display());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnalyzerToggles {
    #[serde(default)]
    pub fmt: Option<bool>,
    #[serde(default)]
    pub clippy: Option<bool>,
    #[serde(default)]
    pub deterministic: Option<bool>,
}

impl AnalyzerToggles {
    pub fn fmt_enabled(&self) -> bool {
        self.fmt.unwrap_or(true)
    }
    pub fn clippy_enabled(&self) -> bool {
        self.clippy.unwrap_or(true)
    }
    pub fn deterministic_enabled(&self) -> bool {
        self.deterministic.unwrap_or(true)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReportConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub include_logs: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TargetConfig {
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
    #[serde(default)]
    pub checklist_refs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enable_trace: Option<bool>,
    #[serde(default)]
    pub trace_filter: Option<String>,
}
