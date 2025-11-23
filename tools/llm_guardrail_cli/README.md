# LLM Guardrail CLI

`guardrail_cli` enforces the guardrails defined in `docs/llm-rust-guardrails.md` before any LLM-produced change reaches a PR. It lives in `crates/guardrail_cli` and relies on `guardrail_core` for config parsing, analyzers, and report generation.

## Build & Install

```
cargo build -p guardrail_cli
# or run ad-hoc
cargo run -p guardrail_cli -- <command> ...
```

## Commands

| Command | Example | Description |
| --- | --- | --- |
| `ingest` | `cargo run -p guardrail_cli -- ingest --prompt .llm_logs/incoming/prompt.md --response .llm_logs/incoming/response.md --diff .llm_logs/incoming/patch.diff --out-dir .llm_logs/pr-42` | Copies prompt/response/diff artifacts into a canonical folder and records metadata for later audits. |
| `validate` | `cargo run -p guardrail_cli -- validate --config tools/llm_guardrail_cli/guardrail.example.toml --id pr-42-attempt-1` | Runs analyzers configured in the TOML file (fmt, clippy, deterministic seed scan, Bevy sandbox checks) and prints a JSON report. If the config specifies `report.path`, the report is also written to disk. |
| `report` | `cargo run -p guardrail_cli -- report --input reports/pr-42-attempt-1.json` | Reads an existing report (see `report_schema.json`) and prints a concise summary. Useful for CI log output or quick local checks. |

## Configuration

`tools/llm_guardrail_cli/guardrail.example.toml` demonstrates the available settings:

- `sources.*` — relative paths to the prompt/response/diff that triggered the run.
- `analyzers` — enable/disable `fmt`, `clippy`, `deterministic_seed_scan`, and `bevy_sandbox_checks`.
- `report.path` — optional output path for the generated JSON. Set `include_logs = true` when CI should capture analyzer logs too.

Extend the config as new analyzers land (e.g., Bevy schedule inspector) by adding toggles and hooking them into `guardrail_core::analyzers`.

## Reports

All outputs conform to `report_schema.json`. See `report.example.json` for the artifact produced by `validate`. CI should treat `summary.status = fail` as a hard blocker; `warn` requires a human sign-off referencing the linked validation matrix row.

## Extensibility

- **Analyzers**: each check implements a simple trait and runs inside `guardrail_core`. Add new analyzers (asset validation, unsafe audits) behind config flags so they can be rolled out gradually.
- **Outputs**: the `report` command currently prints JSON; adding SARIF or Markdown writers only requires serializing the `GuardrailReport` struct differently.
- **Telemetry**: all commands emit `tracing` logs. Point `RUST_LOG=guardrail_cli=debug` during CI debugging to capture detailed analyzer traces.

