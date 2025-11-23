# RTS Board Sandbox

Set `SANDBOX_SCENE=rts_board` (and optionally `BOARD_PLAYER_COUNT`, `BOARD_SPAWN_INTERVAL`, `LOCAL_PLAYER_ID`) before running `cargo run -p game_runner` to focus the prototype. The scene spawns 2-8 players around the edge of the board, gives them two starter units, and adds a new unit every 1 second that auto-rallies toward the center of the formation.

-Controls:

- **Select:** Click-drag with the left mouse button to draw a rectangle. Friendly units inside on release are selected and outlined with a glow.
- **Move:** Right-click to send the selected squad to the clicked location. Units spread into a SC2-style formation so they don’t overlap and they steer smoothly to avoid jitter.
- **Combat:** Each unit is a laser-pistol trooper. They automatically shoot the nearest enemy in range, drawing a red beam. Friendly squads form permanent support links whenever they’re within ~150 units of one another; each connection grants +1 HP/s regen and +5% laser damage, and the pulsing green beam stays visible while the buff is active. The supply buff only applies if the chain of beams from the unit can trace all the way back to its spawn marker.
- **Pylons:** Three energy pylons roam the arena following a stylized three-body orbit but stay within the board’s bounds. If any unit in a supply-connected network stands within ~180 units of a pylon, that entire network gains an additional +4% damage per powered unit.

