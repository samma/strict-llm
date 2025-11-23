use bevy::app::FixedUpdate;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::time::TimePlugin;
use core_game::gameplay::{BoardSettings, SimulationParams, Unit};
use core_game::CoreGamePlugin;
use std::time::Duration;

#[test]
fn rts_spawns_are_deterministic() {
    let baseline = simulate_player_centroids(42);
    let repeat = simulate_player_centroids(42);
    assert_eq!(baseline, repeat, "same seed should match");

    let different = simulate_player_centroids(7);
    assert_ne!(baseline, different, "different seeds should diverge");
}

fn simulate_player_centroids(seed: u64) -> Vec<(i32, i32)> {
    let mut app = App::new();
    app.insert_resource(SimulationParams::from_seed(seed));
    app.insert_resource(BoardSettings {
        player_count: 3,
        spawn_interval: 0.8,
        board_size: 800.0,
    });
    app.insert_resource(DiagnosticsStore::default());
    app.add_plugins(MinimalPlugins.set(TimePlugin::default()));
    app.add_plugins(CoreGamePlugin);

    app.update();
    for _ in 0..120 {
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(Duration::from_millis(500));
        }
        app.world_mut().run_schedule(FixedUpdate);
    }

    let world = app.world_mut();
    let mut sums = vec![Vec2::ZERO; 3];
    let mut counts = vec![0.0; 3];
    let mut query = world.query::<(&Unit, &Transform)>();
    for (unit, transform) in query.iter(&world) {
        let idx = unit.player.0;
        sums[idx] += transform.translation.truncate();
        counts[idx] += 1.0;
    }
    sums.iter_mut()
        .zip(counts.iter())
        .map(|(sum, count)| {
            if *count > 0.0 {
                *sum /= *count;
            }
            (sum.x.round() as i32, sum.y.round() as i32)
        })
        .collect()
}
