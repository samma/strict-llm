use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
}
