use bevy::log::info_span;
use bevy::prelude::*;
use bevy::time::{Fixed, Time};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::ops::RangeInclusive;

const DEFAULT_SEED: u64 = 42;
const DEFAULT_FIXED_DELTA: f64 = 1.0 / 30.0;

/// Core gameplay systems live here. We expose deterministic resources so
/// downstream crates can simulate + test gameplay safely.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<SimulationParams>() {
            app.insert_resource(SimulationParams::from_env());
        }

        app.init_resource::<SimulationRng>()
            .init_resource::<PlayerHealth>()
            .add_systems(Startup, configure_fixed_time)
            .add_systems(FixedUpdate, apply_health_decay)
            .add_systems(Update, tick_health_display);
    }
}

#[derive(Resource, Clone, Debug)]
pub struct SimulationParams {
    pub seed: u64,
    pub fixed_delta: f64,
}

impl SimulationParams {
    pub fn from_env() -> Self {
        let seed = std::env::var("SIMULATION_SEED")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(DEFAULT_SEED);
        let fixed_delta = std::env::var("SIMULATION_FIXED_DT")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(DEFAULT_FIXED_DELTA);
        Self { seed, fixed_delta }
    }

    pub fn from_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            fixed_delta: DEFAULT_FIXED_DELTA,
        }
    }
}

#[derive(Resource, Debug)]
pub struct SimulationRng {
    seed: u64,
    rng: StdRng,
}

impl SimulationRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn gen_range(&mut self, range: RangeInclusive<u32>) -> u32 {
        self.rng.gen_range(range)
    }
}

impl FromWorld for SimulationRng {
    fn from_world(world: &mut World) -> Self {
        let seed = world
            .get_resource::<SimulationParams>()
            .cloned()
            .unwrap_or_default()
            .seed;
        Self::new(seed)
    }
}

#[derive(Resource, Debug)]
pub struct PlayerHealth {
    pub value: Health,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            value: Health::new(100),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Health {
    current: u32,
    max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn max(&self) -> u32 {
        self.max
    }
}

fn apply_health_decay(mut health: ResMut<PlayerHealth>, mut rng: ResMut<SimulationRng>) {
    if health.value.current() == 0 {
        return;
    }

    let _span = info_span!(
        "gameplay.health_decay",
        seed = rng.seed(),
        current = health.value.current()
    )
    .entered();
    let damage = rng.gen_range(1..=4);
    health.value.damage(damage);
}

fn tick_health_display(health: Res<PlayerHealth>) {
    if health.is_changed() {
        info!(
            target: "core_game.health",
            current = health.value.current(),
            max = health.value.max()
        );
    }
}

fn configure_fixed_time(mut fixed_time: ResMut<Time<Fixed>>, params: Res<SimulationParams>) {
    fixed_time.set_timestep_seconds(params.fixed_delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_clamps_to_zero() {
        let mut hp = Health::new(10);
        hp.damage(15);
        assert_eq!(0, hp.current());
    }

    #[test]
    fn rng_is_deterministic() {
        let mut rng_a = SimulationRng::new(7);
        let mut rng_b = SimulationRng::new(7);
        for _ in 0..10 {
            assert_eq!(rng_a.gen_range(1..=10), rng_b.gen_range(1..=10));
        }
    }
}
