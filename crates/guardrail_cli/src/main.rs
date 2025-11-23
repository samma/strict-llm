use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Args, Parser, Subcommand};
use guardrail_core::{run_validations, GuardrailConfig, GuardrailReport, ValidationOptions};

#[derive(Parser)]
#[command(version, about = "Validate LLM-generated changes against guardrails")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Copy prompt/response/diff artifacts into a structured log directory.
    Ingest(IngestArgs),
    /// Run analyzers defined in a config file and emit a JSON report.
    Validate(ValidateArgs),
    /// Pretty-print an existing report.
    Report(ReportArgs),
}

#[derive(Args)]
struct IngestArgs {
    #[arg(long)]
    prompt: PathBuf,
    #[arg(long)]
    response: PathBuf,
    #[arg(long)]
    diff: PathBuf,
    #[arg(long, default_value = ".llm_logs/latest")]
    out_dir: PathBuf,
}

#[derive(Args)]
struct ValidateArgs {
    #[arg(long, default_value = "tools/llm_guardrail_cli/guardrail.example.toml")]
    config: PathBuf,
    #[arg(long)]
    id: Option<String>,
}

#[derive(Args)]
struct ReportArgs {
    #[arg(long)]
    input: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest(args) => handle_ingest(args),
        Commands::Validate(args) => handle_validate(args),
        Commands::Report(args) => handle_report(args),
    }
}

fn handle_ingest(args: IngestArgs) -> Result<()> {
    fs::create_dir_all(&args.out_dir)?;
    copy_into(&args.prompt, &args.out_dir.join("prompt.md"))?;
    copy_into(&args.response, &args.out_dir.join("response.md"))?;
    copy_into(&args.diff, &args.out_dir.join("patch.diff"))?;

    let metadata = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "prompt": args.prompt,
        "response": args.response,
        "diff": args.diff,
    });
    fs::write(
        args.out_dir.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )?;
    println!("Artifacts stored in {}", args.out_dir.display());
    Ok(())
}

fn handle_validate(args: ValidateArgs) -> Result<()> {
    let config_path = args.config;
    let config = GuardrailConfig::from_path(&config_path)?;
    config.validate_sources()?;

    let run_id = args
        .id
        .unwrap_or_else(|| format!("run-{}", Utc::now().format("%Y%m%dT%H%M%S")));
    let workspace_root = std::env::current_dir()?;
    let options = ValidationOptions::new(workspace_root, run_id.clone());

    let report = run_validations(&config, &options)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    if let Some(report_cfg) = config.report.as_ref() {
        if let Some(parent) = report_cfg.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&report_cfg.path, serde_json::to_string_pretty(&report)?)?;
        println!("Report written to {}", report_cfg.path.display());
    }

    Ok(())
}

fn handle_report(args: ReportArgs) -> Result<()> {
    let data = fs::read_to_string(&args.input)?;
    let report: GuardrailReport = serde_json::from_str(&data)?;
    println!(
        "Report {} -> {:?} ({:.2})",
        report.id, report.summary.status, report.summary.score
    );
    Ok(())
}

fn copy_into(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}
