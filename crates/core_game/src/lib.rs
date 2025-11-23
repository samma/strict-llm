//! Core game placeholder logic.

use tracing::info;

/// Basic health tracker to serve as an integration anchor.
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
        info!(target: "core_game.health", current = self.current, max = self.max, "health updated");
    }

    pub fn current(&self) -> u32 {
        self.current
    }
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
}
