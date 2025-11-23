# LLM Guardrail CLI (Draft)

The guardrail CLI is a lightweight Rust binary that inspects LLM-generated change sets before humans review them. It standardizes data capture, automated analysis, and reporting.

## High-level Flow

1. **Ingest** an artifact (prompt, response, proposed diff).
2. **Validate** against guardrails:
   - `cargo fmt` / `cargo clippy` dry-runs on patched tree.
   - Static rules (deterministic RNG usage, unsafe audit notes).
   - Checklist linkage to `docs/validation-matrix.md`.
3. **Augment**: create TODO/test stubs inside `tests/llm_regression/` when gaps are detected.
4. **Emit** a JSON report consumed by humans + CI.

## Proposed Commands

| Command | Description |
| --- | --- |
| `guardrail ingest --prompt prompt.md --response response.md --diff diff.patch` | Stores artifacts under `.llm_logs/` with metadata. |
| `guardrail validate --spec docs/feature.md --targets native,wasm` | Runs analyzers (formatter, lint, deterministic seed scan, unsafe checklist). |
| `guardrail report --out reports/<id>.json` | Serializes summary per `report_schema.json`, including risk level and required follow-ups. |
| `guardrail annotate --pr 123` | (Future) Pushes GitHub/GitLab discussion comments from the report. |

## Extensibility

- **Analyzers** are trait objects keyed by config. Examples: Bevy schedule inspector, Macroquad render pass checker. They can be enabled via `guardrail.toml`.
- **Outputs** currently include JSON and markdown. Adding SARIF or Octopus Deploy release notes merely requires a new serializer.
- **Telemetry** hooks into `tracing` and writes span data to `reports/<timestamp>.log`, enabling offline diagnosis.

## Implementation Sketch

```
crates/
  guardrail_core/      # data model, analyzers, report pipeline
  guardrail_cli/       # clap-based CLI binary (declared here under tools/)
tools/
  llm_guardrail_cli/
    README.md
    report_schema.json
    report.example.json
```

The CLI will eventually join the workspace via a `[workspace]` root `Cargo.toml`. Until then, keep the design documents in this folder to guide implementation.

## Next Steps

1. Finalize the JSON schema so humans + CI parse the same report format.
2. Implement the ingest + validate commands backed by `guardrail_core`.
3. Wire the CLI into CI (e.g., `just guardrails`) and make merge gating conditional on a green report.

