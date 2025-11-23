use bevy::prelude::*;

use crate::gameplay::SimulationParams;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(8, 10, 24)))
            .add_systems(Startup, setup_ui)
            .add_systems(Update, update_debug_hud);
    }
}

#[derive(Component)]
struct DebugHud;

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("Booting core game…"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.93, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
        DebugHud,
    ));
}

fn update_debug_hud(
    mut text: Query<&mut Text, With<DebugHud>>,
    params: Option<Res<SimulationParams>>,
    time: Res<Time>,
) {
    if let Ok(mut text) = text.get_single_mut() {
        let (seed, fixed_dt) = params
            .map(|p| (p.seed, p.fixed_delta))
            .unwrap_or((0, 1.0 / 60.0));
        let content = format!(
            "Core Game Sandbox\nseed: {seed}\nfixed Δt: {fixed_dt:.4}s\nframe Δt: {:.2}ms\n\nDrag LMB to select units, release to confirm. Right-click to move.\nSet SANDBOX_SCENE=rts_board to load the RTS sandbox.",
            time.delta_secs() * 1000.0
        );
        content.clone_into(&mut **text);
    }
}
