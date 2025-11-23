mod sandbox;

use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::{Schedule, Schedules};
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
use core_game::CoreGamePlugin;
use sandbox::SandboxPlugin;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();
    configure_default_plugins(&mut app);
    register_simulation_schedule(&mut app);
    app.add_plugins((CoreGamePlugin, SandboxPlugin::default()));
    app.run();
}

fn configure_default_plugins(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas".into()),
            fit_canvas_to_parent: true,
            present_mode: PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "Core Game Sandbox".into(),
            present_mode: PresentMode::Fifo,
            resolution: WindowResolution::new(1280.0, 720.0),
            resizable: true,
            ..default()
        }),
        ..default()
    };

    let mut plugins = DefaultPlugins.set(window_plugin);

    #[cfg(target_arch = "wasm32")]
    {
        plugins = plugins.set(AssetPlugin {
            file_path: "assets".into(),
            ..default()
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        plugins = plugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        });
    }

    app.add_plugins(plugins);
}

fn register_simulation_schedule(app: &mut App) {
    let mut schedules = app.world_mut().resource_mut::<Schedules>();
    if !schedules.contains(core_game::SimulationSchedule) {
        schedules.insert(Schedule::new(core_game::SimulationSchedule));
    }
}
