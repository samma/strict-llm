# Validation Matrix

Every gameplay feature must satisfy the checks below before merge. Link your change to at least one row and update the matrix if you introduce a new subsystem.

| Feature Area | Logic Tests | ECS / Scheduling | Rendering / IO | Telemetry & Metrics | Required Commands |
| --- | --- | --- | --- | --- | --- |
| Core combat loop | `crates/llm_regression/tests/template.rs::regression_template` (seed=42) proves snapshot determinism, while `crates/llm_regression/tests/deterministic_health.rs` drives a headless Bevy world via `MinimalPlugins + ScheduleRunnerPlugin` to validate seeded RNG + `SimulationParams`. Extend with `combat_*.rs` cases seeded via `StdRng::seed_from_u64(9001)` to cover damage resolution, status effects, and cooldown timers. | Verify systems are scheduled in the `PostUpdate` stage with explicit ordering constraints. Include assertions for exclusive world access. | Ensure hit flashes, particle systems, and animation triggers run deterministically; record golden frame hashes under `crates/llm_regression/golden/combat/*.png` alongside snapshot files in `tests/snapshots/`. | Emit `tracing` spans `combat.round` + metrics (`hit_confirmed`, `shield_break`). | `just verify-core`, `cargo nextest run --package llm_regression`, `cargo run -p guardrail_cli -- validate --config tools/llm_guardrail_cli/guardrail.example.toml`. |
| Movement & physics | Unit tests for kinematics + collision resolution (`seed 1337`). Golden position traces stored as JSON. | Schedule physics in a fixed timestep system; assert resources (DeltaTime, PhysicsWorld) are present. | Visual regression via `crates/llm_regression/golden/movement/*.gif`. | Log `movement.step` spans with position/velocity fields. | `just verify-core`, `cargo nextest run --package llm_regression --features movement`. |
| UI / HUD | Logic tests cover state machines (health bars, timers). Snapshot tests stored in `golden/ui/*.ron`. | Ensure UI updates run in `PreUpdate` and don't block rendering. | Pixel-diff using headless renderer; maintain deterministic font atlas seeds. | Track `ui.frame_time` metric. | `just verify-core`, `cargo test -p ui -- --ignored ui_snapshot`. |
| Persistence / save system | Round-trip tests for serialization to/from disk with fixture seeds. | ECS resources must serialize safely; add tests to confirm world restoration order. | Validate thumbnails and icons using hashed PNG outputs. | Emit `savegame.bytes_written` metrics. | `just verify-core`, `cargo test -p persistence`. |
| Networking / rollback | Deterministic simulation with fixed seed lists. Compare timeline digests for divergence. | Schedules must capture authoritative vs client prediction sequences. | Optional, but record rollback visualizations to `crates/llm_regression/golden/net/*.mp4`. | Metrics: `rollback.frames_replayed`, `sync.rtt_ms`. | `just verify-core`, `cargo nextest run --features rollback`. |

## Acceptance Workflow

1. **LLM submission** includes: linked matrix row(s), guardrail CLI report, and deterministic seed list.
2. **Automated gate**: `just verify-core`, wasm build, regression harness, asset validation, guardrail CLI. All must pass.
3. **Dual review**:
   - Human reviewer confirms the matrix row coverage and inspects guardrail report.
   - Guardrail CLI auto-comment (planned) must show `status=pass` or `status=warn` with waivers documented.
4. **Merge** once both reviewers sign off and no blocking risks remain.
5. **Rollback** policy: if guardrail CLI later flags a merged change, revert via `git revert` (no force pushes) and open an incident note referencing the report ID.

Update this matrix whenever new systems land. Every entry should capture logic scope, deterministic data, telemetry expectations, and commands that prove the feature works. The goal is to keep LLM-assisted changes auditable and low-risk.

