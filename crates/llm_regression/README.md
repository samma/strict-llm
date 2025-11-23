# LLM Regression Harness

This crate houses deterministic tests that guard gameplay behavior. Any LLM-generated change must add or update scenarios in this crate before merge.

## Structure

```
crates/llm_regression/
  README.md
  Cargo.toml
  src/lib.rs
  tests/
    template.rs
  golden/
    .gitkeep
```

## Authoring a Scenario

1. Copy `tests/template.rs` to a descriptive filename (e.g., `tests/combat_round.rs`).
2. Pick a deterministic seed and record it in the test + PR.
3. Ensure required dev-dependencies (`rand`, `serde_json`, `insta`) cover the assertions.
4. Store golden artifacts (JSON traces, PNG frames, logs) under `golden/<feature>/`.
5. Update `docs/validation-matrix.md` when the scenario maps to a new checklist row.

## Running Tests

Use `cargo nextest run --package llm_regression` (automatically triggered by `just test` once the workspace builds). Run locally before submitting guardrail reports.

### Updating Snapshots

1. Run `cargo test -p llm_regression --test <name>` to reproduce the failure.
2. Accept new golden outputs with `cargo insta review` (or `cargo insta accept` for non-interactive CI updates).
3. Commit the updated files under `tests/snapshots/` alongside any JSON/PNG artifacts in `golden/`.

## Golden Files

- Keep files minimal + deterministic. Compress images/animations.
- Commit every golden file referenced by a test; failing to update goldens blocks CI.
- Update golden outputs only when the behavioral change is intentional and documented.
- Reference implementation: `tests/snapshots/template__combat_round.snap` pairs with `tests/template.rs` to prove deterministic combat rolls.

## Review Checklist

- Deterministic RNG seed captured as a constant inside the test.
- Assertions cover logic, ECS ordering, and rendering hashes (when applicable).
- New telemetry is verified via logs or metrics emitted during the test.

Treat this harness as a living contract between humans, LLMs, and shipped behavior.

