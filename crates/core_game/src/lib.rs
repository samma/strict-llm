//! Core Bevy game plugin composed of gameplay, UI, and diagnostics modules.

pub mod diagnostics;
pub mod gameplay;
pub mod ui;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

/// Schedule dedicated to deterministic simulation. Rendering hooks
/// run in the default `Update`/`PostUpdate` stages.
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone)]
pub struct SimulationSchedule;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            gameplay::GameplayPlugin,
            ui::UiPlugin,
            diagnostics::DiagnosticsPlugin,
        ));
    }
}
