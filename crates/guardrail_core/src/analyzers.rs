use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::config::GuardrailConfig;
use crate::report::{CheckResult, CheckStatus, GuardrailReport};

pub struct ValidationOptions {
    pub workspace_root: PathBuf,
    pub run_id: String,
}

impl ValidationOptions {
    pub fn new(workspace_root: PathBuf, run_id: impl Into<String>) -> Self {
        Self {
            workspace_root,
            run_id: run_id.into(),
        }
    }
}

pub fn run_validations(
    config: &GuardrailConfig,
    options: &ValidationOptions,
) -> Result<GuardrailReport> {
    let mut checks = Vec::new();
    let toggles = &config.analyzers;

    if toggles.fmt_enabled() {
        checks.push(run_fmt(&options.workspace_root)?);
    }

    if toggles.clippy_enabled() {
        checks.push(run_clippy(&options.workspace_root)?);
    }

    if toggles.deterministic_enabled() {
        checks.push(run_deterministic_scan(&options.workspace_root)?);
    }

    let report = GuardrailReport::new(
        options.run_id.clone(),
        config.source_info(),
        checks,
        "Guardrail CLI MVP",
    );
    Ok(report)
}

fn run_fmt(workspace_root: &Path) -> Result<CheckResult> {
    run_command(
        "fmt",
        workspace_root,
        "cargo",
        ["fmt", "--all", "--", "--check"],
    )
}

fn run_clippy(workspace_root: &Path) -> Result<CheckResult> {
    run_command(
        "clippy",
        workspace_root,
        "cargo",
        [
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )
}

fn run_command(
    name: &str,
    workspace_root: &Path,
    cmd: &str,
    args: impl IntoIterator<Item = &'static str>,
) -> Result<CheckResult> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("{name} command failed to start"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut details = stdout.trim().to_owned();
    if !stderr.trim().is_empty() {
        if !details.is_empty() {
            details.push_str("\n--- stderr ---\n");
        }
        details.push_str(stderr.trim());
    }

    let status = if output.status.success() {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };

    Ok(CheckResult {
        name: name.to_string(),
        status,
        details,
        log_path: None,
    })
}

fn run_deterministic_scan(workspace_root: &Path) -> Result<CheckResult> {
    let mut offenders = Vec::new();
    let guardrail_core_root = workspace_root.join("crates").join("guardrail_core");
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_entry(|e| filter_entry(e.path()))
    {
        let entry = entry?;
        let path = entry.path();
        if path.starts_with(&guardrail_core_root) {
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            let contents = std::fs::read_to_string(path)?;
            if contents.contains("thread_rng()") || contents.contains("thread_rng(") {
                offenders.push(
                    path.strip_prefix(workspace_root)
                        .unwrap()
                        .display()
                        .to_string(),
                );
            }
        }
    }

    if offenders.is_empty() {
        Ok(CheckResult {
            name: "deterministic_seed_scan".into(),
            status: CheckStatus::Pass,
            details: "No non-deterministic RNG usage detected".into(),
            log_path: None,
        })
    } else {
        Ok(CheckResult {
            name: "deterministic_seed_scan".into(),
            status: CheckStatus::Fail,
            details: format!("Found thread_rng usage in:\n{}", offenders.join("\n")),
            log_path: None,
        })
    }
}

fn filter_entry(path: &Path) -> bool {
    let ignored = ["target", ".git", "reports"];
    for part in path.components() {
        if let std::path::Component::Normal(os_str) = part {
            if let Some(part_str) = os_str.to_str() {
                if ignored.contains(&part_str) {
                    return false;
                }
            }
        }
    }
    true
}
