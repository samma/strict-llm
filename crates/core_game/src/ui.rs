use bevy::prelude::*;

use crate::gameplay::SimulationParams;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(8, 10, 24)))
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_debug_hud, spin_marker));
    }
}

#[derive(Component)]
struct DebugHud;

#[derive(Component)]
struct Spinner;

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: Color::srgb(0.35, 0.58, 0.96),
            custom_size: Some(Vec2::splat(140.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.5),
        Spinner,
    ));

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
            "Core Game Sandbox\nseed: {seed}\nfixed Δt: {fixed_dt:.4}s\nframe Δt: {:.2}ms\n\nUse SANDBOX_SCENE=<feature> to load a prototype.",
            time.delta_secs() * 1000.0
        );
        content.clone_into(&mut **text);
    }
}

fn spin_marker(time: Res<Time>, mut sprites: Query<&mut Transform, With<Spinner>>) {
    for mut transform in &mut sprites {
        transform.rotate_z(time.delta_secs() * 0.8);
    }
}
