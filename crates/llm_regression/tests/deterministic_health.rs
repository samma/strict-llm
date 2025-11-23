use bevy::app::{FixedUpdate, ScheduleRunnerPlugin};
use bevy::prelude::*;
use core_game::gameplay::{PlayerHealth, SimulationParams};
use core_game::CoreGamePlugin;
use std::time::Duration;

#[test]
fn health_decay_is_deterministic_with_seed() {
    let first = simulate_health(42, 10);
    let second = simulate_health(42, 10);
    assert_eq!(first, second, "same seed must yield identical results");

    let other = simulate_health(7, 10);
    assert_ne!(first, other, "changing the seed should alter the outcome");
}

fn simulate_health(seed: u64, frames: usize) -> u32 {
    let mut app = App::new();
    app.insert_resource(SimulationParams::from_seed(seed));
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    );
    app.add_plugins(CoreGamePlugin);

    for _ in 0..frames {
        app.world_mut().run_schedule(FixedUpdate);
    }

    app.world().resource::<PlayerHealth>().value.current()
}
