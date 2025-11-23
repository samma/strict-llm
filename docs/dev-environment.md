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
| Deterministic harness | `cargo test -p llm_regression --test deterministic_health` | Headless Bevy simulation validating `SimulationParams` + RNG determinism. |

CI mirrors these commands; never introduce bespoke scripts without updating the `Justfile`.

## 4. Bevy Targets & Builds

- `core_game` uses feature flags to differentiate targets:
  - `native` (default): standard desktop build.
  - `native_hot_reload`: opt-in Bevy dynamic linking for faster iteration (Windows linker limits may require Visual Studio 2022 tools).
  - `wasm`: enables `bevy/webgl2` for browser demos.
- Run native loops with `cargo run -p game_runner` (set `SANDBOX_SCENE=<feature>` to focus on a prototype). Combine with `RUST_LOG=info` for structured traces.
- Build the browser artifact with `just build-wasm`, which invokes `scripts/build_wasm.ps1` → `wasm-bindgen` and drops output into `web/pkg`.
- Keep `rust-analyzer.cargo.features = ["native", "wasm"]` so edits are validated for both targets.
- Determinism knobs: override `SIMULATION_SEED=<u64>` and `SIMULATION_FIXED_DT=<seconds>` to reproduce or speed up fixed-step simulations. CI sticks to the defaults defined in `core_game::gameplay::SimulationParams`.
- RTS sandbox knobs: `BOARD_PLAYER_COUNT` (2-8), `BOARD_SPAWN_INTERVAL` (seconds), `BOARD_SIZE` (float). Setting `SANDBOX_SCENE=rts_board` applies sandbox defaults automatically.
- Mouse controls (rts_board): hold left mouse to grow a selection circle (units inside are selected on release). Right-click issues move orders (units spread out SC2-style). Units auto-fire laser pistols at the nearest enemies and heal when another friendly unit is nearby. `LOCAL_PLAYER_ID=<idx>` chooses which spawn responds to input.

## 5. Hot Reload & Asset Flow

- Run `cargo watch -x \"run --bin editor\" -s scripts\\sync_assets.ps1` during gameplay iteration to rebuild code + copy assets to the runtime folder.
- Asset sources live under `assets_src/`; runtime-ready copies sit in `assets/`. Only `assets_src/` is editable—the sync script performs conversion/compression.
- Deterministic assets (e.g., procedural seeds) belong in `assets_src/deterministic/` and must record seed metadata for tests.

## 6. Working with LLM Output

1. **Start in a sandbox**: implement experimental systems inside `examples/systems/<feature>` until validated by tests. Reset a sandbox at any time with `git clean -fd examples/systems/<feature>` (never mix multiple prototypes in the same folder).
2. **Capture prompt context**: store the final prompt + response in `.llm_logs/` for reproducibility. Redact secrets before saving.
3. **Run just verify-core** locally before asking for human review, even if the LLM already claimed success.
4. **Diff hygiene**: prefer small staged commits mapped to single features or fixes so regression bisecting stays easy.

## 7. Pull Request Protocol

- PR description template requires: feature summary, validation commands run, checklist entries touched, and links to `.llm_logs`.
- CI gates: `verify-core`, wasm smoke build, regression harness (see `crates/llm_regression/`), and asset validation.
- Failing guardrail CLI runs (see `tools/llm_guardrail_cli`) block merge until the report is attached and addressed.

## 8. CI & Guardrails

- Workflow: `.github/workflows/ci.yml` runs on every push/PR. It executes `just verify-core`, `cargo run -p guardrail_cli -- validate --config tools/llm_guardrail_cli/guardrail.example.toml --id ci`, `cargo test -p llm_regression --test deterministic_health`, `just build-wasm` (artifact uploaded as `wasm-demo`), and `just asset-validate`.
- Guardrail artifacts: the workflow seeds `.llm_logs/latest/` with placeholder prompt/response/diff so the CLI can confirm analyzers succeed. Replace these with real data once CI captures prompts automatically.
- Failure triage: map any `guardrail_cli` failures to the relevant `docs/validation-matrix.md` row and document the remediation plan in the PR.
- Regression harness: because `just test` invokes `cargo nextest run --workspace`, any failing deterministic test automatically surfaces in CI logs.

Following this environment guide keeps the team and LLM assistants aligned, minimizes build drift, and ensures quick, deterministic feedback during game development.

