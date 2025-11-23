# LLM Guardrails for Rust Game Development

This document defines the explicit contract between humans and large language models (LLMs) when authoring or reviewing code inside this repository. Breaking any rule below blocks merge until corrected.

## 1. Information the Human Must Supply

- **Spec-first input**: Always provide the feature brief (requirements, constraints, failure modes) before asking for code. Include links to the relevant module docs or APIs in `docs/` whenever possible.
- **State of the tree**: Supply `git status` and any staged diffs touching the target area so the LLM never works with stale context.
- **Execution context**: Describe the current target (e.g., Windows native debug build, WASM test harness) and the command used to reproduce.
- **Existing contracts**: Pass the component interface summary (signatures, ECS resources, data layouts). If new APIs are required, call that out explicitly.
- **Acceptance gates**: Clarify which validation checklist entries (see `docs/validation-matrix.md`) must pass for the change.

## 2. Mandatory Output from the LLM

Every response covering code must include:

1. **Diff-oriented changes** referencing concrete files/paths with explanations. Do not restate entire files; highlight the minimal edits.
2. **Validation steps**: exact commands (e.g., `just verify-core` or `cargo nextest run --features wasm`) and expected outcomes.
3. **Risk analysis**: enumerate undefined behavior (UB) risks, concurrency hazards, performance cliffs, and mitigation steps.
4. **Follow-up checklist**: identify missing telemetry, docs, or tests required beyond the immediate change.
5. **Prompt hygiene**: confirm `cargo fmt` + `cargo clippy --all-targets --all-features -D warnings` were considered, even if not executed locally.

## 3. Coding Rules

| Topic | Requirement |
| --- | --- |
| Formatting & linting | Run `cargo fmt --all` and `cargo clippy --all-targets --all-features -D warnings` before proposing final diffs. |
| Testing hierarchy | Unit tests precede engine hooks. New systems require `tests/` coverage plus `examples/systems/` integration snippets. |
| Determinism | All gameplay simulations must use deterministic RNG seeds (see `rand::rngs::StdRng::seed_from_u64`). Record the seed in test logs. |
| Unsafe code | Provide an *unsafe audit checklist* explaining invariants, lifetime guarantees, and why safe abstractions fail. Unsafe blocks without commentary are rejected. |
| Error handling | Prefer `thiserror` for domain errors and instrumented `anyhow::Context` for orchestration code. Bubble detailed diagnostics instead of panicking. |
| Logging & metrics | All systems log via `tracing` with structured fields. Include spans for ECS schedule boundaries and asset loading. |
| Assets & IO | Never assume blocking file IO on the main thread. Use async loaders or job dispatch abstractions. |
| Dependencies | Adding crates requires justification: maintenance state, MSRV compatibility, no native build surprises on Windows/macOS/Linux. |
| Secrets/config | Configuration flows through `config/` manifests or environment variables declared in docs. Hard-coded credentials are forbidden. |

## 4. Bidirectional Contract Summary

- **Human → LLM:** supplies spec, current code context, validation targets, and architectural constraints.
- **LLM → Human:** returns minimal diffs, rationale, validation plan, and risk analysis. Clearly mark uncertainties so reviewers know where to focus.

## 5. Prompt Templates

### 5.1 Subsystem Design

```
Context: <link to module doc / architecture diagram>
Goal: <describe feature + constraints + desired KPIs>
Interfaces touched: <structs/components/resources>
Validation: <required checklist rows or commands>
Deliverable: design sketch + file-level diff plan + open risks.
```

### 5.2 Gameplay Iteration Loop

```
Current behavior: <what the game does now>
Target behavior: <what needs to change>
Telemetry: <available metrics/logs>
Budget: <frame time / memory / bandwidth limits>
Ask: produce minimal diff to achieve change, plus regression tests covering deterministic seeds <seed list>.
```

### 5.3 Engine Integration

```
Engine hook: <ECS schedule stage, render pass, or input system>
Constraints: <threading model, feature flags, platform targets>
Artifacts: <assets, shaders, serialization formats involved>
Ask: patch plan + safety notes (unsafe audit if relevant) + commands to verify native + wasm targets.
```

### 5.4 Regression Triage

```
Failure: <stack trace / log excerpt / failing test>
Recent changes: <links to commits/PRs>
Hypothesis: <suspected root cause>
Ask: reproduce steps, scoped fix diff, updates to regression harness template, and log instrumentation plan.
```

## 6. Review Checklist for LLM Output

1. All code paths reachable by user input have tests (unit or regression) that fail without the change.
2. Deterministic seeds recorded in tests and docs.
3. Unsafe blocks justified or removed.
4. Telemetry hooks added for new behaviors.
5. Validation commands defined and executable without secrets.
6. No TODOs left in code unless explicitly requested by maintainers.

Following this playbook keeps the LLM contributions predictable, auditable, and ready for production-quality Rust game development.

