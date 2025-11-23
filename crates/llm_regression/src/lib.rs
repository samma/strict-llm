//! Helpers for deterministic regression tests.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

pub const DEFAULT_SEED: u64 = 42;

pub fn sample_combat_roll(seed: u64) -> serde_json::Value {
    let mut rng = StdRng::seed_from_u64(seed);
    let roll = rng.gen_range(1..=20);
    json!({ "roll": roll, "seed": seed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_is_deterministic() {
        let a = sample_combat_roll(DEFAULT_SEED);
        let b = sample_combat_roll(DEFAULT_SEED);
        assert_eq!(a, b);
    }
}
