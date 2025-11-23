# Systems Sandbox

Use this directory to spike gameplay or engine ideas without touching the production crates.

Workflow:

1. Create a subfolder per experiment (e.g., `examples/systems/loot_drop_balancer`).
2. Add a minimal `Cargo.toml` + `main.rs` or Bevy app that isolates the new logic.
3. Capture prompts and responses for the spike inside `.llm_logs/<feature>.md`.
4. Once validated, port the relevant code into the core engine crates and delete the spike with `git clean -fd examples/systems/<feature>`.
5. Run `cargo run -p game_runner` to load the sandbox registry; set `SANDBOX_SCENE=<feature>` to focus on a single experiment.

Available sandboxes:

- `rts_board`: square RTS-style board with 2-8 player spawns, periodic reinforcements, and auto-rally logic. Configure via `BOARD_PLAYER_COUNT` / `BOARD_SPAWN_INTERVAL`. Hold the left mouse button to grow a selection circle, release to select nearby units, then right-click to issue move orders (units spread out automatically). Units fire laser pistols at the closest enemies and heal when another friendly unit is nearby.

Nothing inside this directory ships to players; treat it as disposable scaffolding for the LLM collaboration loop.

