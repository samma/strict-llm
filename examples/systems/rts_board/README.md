# RTS Board Sandbox

Set `SANDBOX_SCENE=rts_board` (and optionally `BOARD_PLAYER_COUNT`, `BOARD_SPAWN_INTERVAL`, `LOCAL_PLAYER_ID`) before running `cargo run -p game_runner` to focus the prototype. The scene spawns 2-8 players around the edge of the board, gives them two starter units, and adds a new unit every 10 seconds that auto-rallies toward the center of the formation.

-Controls:

- **Select:** Click-drag with the left mouse button to draw a rectangle. Friendly units inside on release are selected and outlined with a glow.
- **Move:** Right-click to send the selected squad to the clicked location. Units spread into a SC2-style formation so they don’t overlap and they steer smoothly to avoid jitter.
- **Combat:** Each unit is a laser-pistol trooper. They automatically shoot the nearest enemy in range, drawing a red beam. If two friendlies are close together they emit a pulsing green link and slowly heal.

