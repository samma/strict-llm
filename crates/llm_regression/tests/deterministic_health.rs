use core_game::gameplay::{SimulationParams, SimulationRng};

#[test]
fn simulation_rng_is_deterministic() {
    let baseline = sample_values(42);
    let repeat = sample_values(42);
    assert_eq!(baseline, repeat, "same seed should match");

    let different = sample_values(7);
    assert_ne!(baseline, different, "different seeds should diverge");
}

fn sample_values(seed: u64) -> Vec<u32> {
    let params = SimulationParams::from_seed(seed);
    let mut rng = SimulationRng::new(params.seed);
    (0..5).map(|_| rng.gen_range(1..=20)).collect()
}
