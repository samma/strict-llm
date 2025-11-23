# LLM Regression Harness

This folder holds deterministic tests that prove each gameplay feature still behaves as expected. LLM-generated changes must add or update scenarios here before merge.

## Structure

```
tests/llm_regression/
  README.md                  # this file
  template.rs                # starter test harness
  golden/                    # reference outputs (frames, traces, serialized data)
    .gitkeep
```

## Authoring a Scenario

1. Copy `template.rs` to a descriptive filename (e.g., `combat_round.rs`).
2. Pick a deterministic seed (document it in the test + PR description).
3. Ensure the owning crate declares `rand`, `serde_json`, and `insta` (or equivalent snapshot library) as dev-dependencies.
4. Record golden artifacts (JSON traces, PNG frames, logs) and store them under `golden/<feature>/`.
5. Update `docs/validation-matrix.md` to reference the new scenario if it maps to a new checklist row.

## Running Tests

The eventual workspace will expose these via `cargo nextest run --package llm_regression`. Until then, you can compile the file individually with `rustc` or integrate it into a standalone crate.

## Golden Files

- Keep files small and deterministic. Compress images/animations when reasonable.
- Commit every golden file referenced by a test; failing to do so blocks CI.
- Update goldens only when the new behavior is asserted in docs and reviewed.

## Review Checklist

- Deterministic RNG seed captured as a constant in the test.
- Assertions cover logic, ECS scheduling guarantees, and rendering hashes when applicable.
- Any new telemetry introduced by the feature is asserted via logs or metrics in the test output.

This harness is the backbone of our accuracy guarantees—treat it as a living contract between humans, LLMs, and shipped game behavior.

