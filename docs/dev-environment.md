# Rust Game Dev Environment & Workflow

This repository targets a predictable, low-friction loop for humans + LLMs. Follow this checklist before sending any code for review.

## 1. Toolchain Pinning

- Install the pinned toolchain with `rustup override set stable` (see `rust-toolchain.toml` once added).
- Components required: `rustfmt`, `clippy`, `rust-analyzer`, `rust-src`.
- Keep `cargo binstall` available for installing auxiliary tools (e.g., `just`, `cargo-nextest`, `cargo-watch`).

## 2. Editor & rust-analyzer

- Enable `rust-analyzer.checkOnSave.command = "clippy"` to surface warnings early.
- Turn on `rust-analyzer.imports.granularity.group = "module"` to keep diffs minimal.
- Use `rust-analyzer.cargo.features = ["native", "wasm"]` to ensure both build flavors are analyzed.
- Regenerate the workspace symbol cache whenever `Cargo.toml` changes to keep the LLM context accurate.

## 3. Formatter, Linter, Tests

The repo standardizes on a `Justfile` (see root) so both humans and LLMs reference the same commands:

| Task | Command | Notes |
| --- | --- | --- |
| Format | `just fmt` | Runs `cargo fmt --all`. |
| Lint | `just lint` | Executes `cargo clippy --all-targets --all-features -D warnings`. |
| Tests | `just test` | Uses `cargo nextest run --all-targets`. |
| Full verify | `just verify-core` | `fmt → clippy → nextest`, the minimum gate before PRs. |
| Dual build | `just build-all` | Builds native + wasm targets to catch cfg drift. |

CI mirrors these commands; never introduce bespoke scripts without updating the `Justfile`.

## 4. Hot Reload & Asset Flow

- Run `cargo watch -x \"run --bin editor\" -s scripts\\sync_assets.ps1` during gameplay iteration to rebuild code + copy assets to the runtime folder.
- Asset sources live under `assets_src/`; runtime-ready copies sit in `assets/`. Only `assets_src/` is editable—the sync script performs conversion/compression.
- Deterministic assets (e.g., procedural seeds) belong in `assets_src/deterministic/` and must record seed metadata for tests.

## 5. Working with LLM Output

1. **Start in a sandbox**: implement experimental systems inside `examples/systems/<feature>` until validated by tests. Reset a sandbox at any time with `git clean -fd examples/systems/<feature>` (never mix multiple prototypes in the same folder).
2. **Capture prompt context**: store the final prompt + response in `.llm_logs/` for reproducibility. Redact secrets before saving.
3. **Run just verify-core** locally before asking for human review, even if the LLM already claimed success.
4. **Diff hygiene**: prefer small staged commits mapped to single features or fixes so regression bisecting stays easy.

## 6. Pull Request Protocol

- PR description template requires: feature summary, validation commands run, checklist entries touched, and links to `.llm_logs`.
- CI gates: `verify-core`, wasm smoke build, regression harness (see `tests/llm_regression/`), and asset validation.
- Failing guardrail CLI runs (see `tools/llm_guardrail_cli`) block merge until the report is attached and addressed.

Following this environment guide keeps the team and LLM assistants aligned, minimizes build drift, and ensures quick, deterministic feedback during game development.

