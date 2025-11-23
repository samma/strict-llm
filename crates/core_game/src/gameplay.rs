use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::time::{Fixed, Time};
use bevy::window::PrimaryWindow;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

const DEFAULT_SEED: u64 = 42;
const DEFAULT_FIXED_DELTA: f64 = 1.0 / 30.0;
const DEFAULT_BOARD_SIZE: f32 = 1600.0;
const DEFAULT_PLAYER_COUNT: usize = 4;
const DEFAULT_SPAWN_INTERVAL: f32 = 10.0;
const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 8;
const UNIT_SPEED: f32 = 120.0;
const UNIT_SEPARATION_RADIUS: f32 = 40.0;
const MIN_SELECTION_RADIUS: f32 = 40.0;
const SELECTION_GROWTH_RATE: f32 = 80.0;
const FORMATION_SPACING: f32 = 60.0;
const LASER_RANGE: f32 = 260.0;
const LASER_DAMAGE: f32 = 6.0;
const LASER_COOLDOWN: f32 = 0.7;
const LASER_HEAL_RANGE: f32 = 150.0;
const LASER_HEAL_RATE: f32 = 4.0;
const BEAM_LIFETIME: f32 = 0.15;

const PLAYER_COLORS: [Color; MAX_PLAYERS] = [
    Color::srgb(0.93, 0.26, 0.28),
    Color::srgb(0.26, 0.65, 0.93),
    Color::srgb(0.94, 0.76, 0.16),
    Color::srgb(0.63, 0.47, 0.94),
    Color::srgb(0.18, 0.8, 0.57),
    Color::srgb(0.94, 0.44, 0.16),
    Color::srgb(0.22, 0.85, 0.85),
    Color::srgb(0.93, 0.36, 0.6),
];

/// Core gameplay systems live here. We expose deterministic resources so
/// downstream crates can simulate + test gameplay safely.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<SimulationParams>() {
            app.insert_resource(SimulationParams::from_env());
        }
        if !app.world().contains_resource::<BoardSettings>() {
            app.insert_resource(BoardSettings::from_env());
        }
        if !app.world().contains_resource::<ControlSettings>() {
            app.insert_resource(ControlSettings::from_env());
        }
        if app
            .world()
            .get_resource::<ButtonInput<MouseButton>>()
            .is_none()
        {
            app.world_mut()
                .insert_resource(ButtonInput::<MouseButton>::default());
        }

        app.init_resource::<SimulationRng>()
            .init_resource::<SelectionState>()
            .add_systems(Startup, configure_fixed_time)
            .add_systems(
                Startup,
                (setup_board, spawn_initial_units.after(setup_board)),
            )
            .add_systems(
                FixedUpdate,
                (
                    tick_spawn_timers,
                    move_units,
                    update_unit_rally_targets,
                    unit_combat_system.after(move_units),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_selection_input,
                    update_selection_visuals.after(handle_selection_input),
                    issue_move_orders.after(update_selection_visuals),
                    update_beam_effects,
                ),
            );
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

#[derive(Resource, Clone, Debug)]
pub struct BoardSettings {
    pub board_size: f32,
    pub player_count: usize,
    pub spawn_interval: f32,
}

impl BoardSettings {
    pub fn from_env() -> Self {
        let scene_hint = std::env::var("SANDBOX_SCENE").unwrap_or_default();
        let mut player_count = std::env::var("BOARD_PLAYER_COUNT")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(if scene_hint == "rts_board" {
                DEFAULT_PLAYER_COUNT
            } else {
                MIN_PLAYERS
            });
        player_count = player_count.clamp(MIN_PLAYERS, MAX_PLAYERS);
        let board_size = std::env::var("BOARD_SIZE")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(DEFAULT_BOARD_SIZE);
        let spawn_interval = std::env::var("BOARD_SPAWN_INTERVAL")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(DEFAULT_SPAWN_INTERVAL);
        Self {
            board_size,
            player_count,
            spawn_interval,
        }
    }
}

impl Default for BoardSettings {
    fn default() -> Self {
        Self {
            board_size: DEFAULT_BOARD_SIZE,
            player_count: DEFAULT_PLAYER_COUNT,
            spawn_interval: DEFAULT_SPAWN_INTERVAL,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ControlSettings {
    pub local_player: PlayerId,
}

impl ControlSettings {
    pub fn from_env() -> Self {
        let id = std::env::var("LOCAL_PLAYER_ID")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0)
            .clamp(0, MAX_PLAYERS as i32 - 1) as usize;
        Self {
            local_player: PlayerId(id),
        }
    }
}

impl Default for ControlSettings {
    fn default() -> Self {
        Self {
            local_player: PlayerId(0),
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

    pub fn gen_f32(&mut self, range: RangeInclusive<f32>) -> f32 {
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

#[derive(Resource, Default, Debug)]
struct SpawnRegistry {
    entries: Vec<SpawnEntry>,
}

#[derive(Debug, Clone)]
struct SpawnEntry {
    player: PlayerId,
    position: Vec2,
}

#[derive(Resource)]
struct SpawnTimers {
    timers: Vec<Timer>,
}

#[derive(Resource, Default)]
struct SelectionState {
    is_dragging: bool,
    start_world: Vec2,
    radius: f32,
    circle_entity: Option<Entity>,
    selected: Vec<Entity>,
    prev_selected: Vec<Entity>,
    dirty: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

#[derive(Component)]
pub struct Unit {
    pub player: PlayerId,
    pub rally_target: Vec2,
    pub kind: UnitKind,
    pub health: f32,
    pub max_health: f32,
    pub attack_timer: Timer,
}

#[derive(Clone, Copy, Debug)]
pub enum UnitKind {
    Laser,
}

impl UnitKind {
    fn health(&self) -> f32 {
        match self {
            UnitKind::Laser => 45.0,
        }
    }

    fn attack_cooldown(&self) -> f32 {
        match self {
            UnitKind::Laser => LASER_COOLDOWN,
        }
    }
}

#[derive(Component)]
struct SelectionCircle;

#[derive(Component)]
struct SelectionHighlight {
    glow: Entity,
}

#[derive(Component)]
struct BeamEffect {
    timer: Timer,
}

fn setup_board(mut commands: Commands, settings: Res<BoardSettings>) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.09, 0.12, 0.2),
            custom_size: Some(Vec2::splat(settings.board_size)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.5),
    ));

    let mut registry = SpawnRegistry::default();
    let radius = settings.board_size * 0.35;
    for idx in 0..settings.player_count {
        let angle = idx as f32 / settings.player_count as f32 * TAU;
        let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);
        let player = PlayerId(idx);
        registry.entries.push(SpawnEntry { player, position });

        commands.spawn((
            Sprite {
                color: PLAYER_COLORS[idx],
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.1),
        ));
    }

    commands.insert_resource(registry);
}

fn spawn_initial_units(
    mut commands: Commands,
    registry: Res<SpawnRegistry>,
    settings: Res<BoardSettings>,
) {
    let mut timers = SpawnTimers { timers: Vec::new() };
    for entry in registry.entries.iter() {
        let player_color = PLAYER_COLORS[entry.player.0];
        let offset = Vec2::new(18.0, 0.0);
        spawn_unit(
            &mut commands,
            entry.player,
            entry.position + offset,
            entry.position,
            player_color,
        );
        spawn_unit(
            &mut commands,
            entry.player,
            entry.position - offset,
            entry.position,
            player_color,
        );
        timers.timers.push(Timer::from_seconds(
            settings.spawn_interval,
            TimerMode::Repeating,
        ));
    }
    commands.insert_resource(timers);
}

fn spawn_unit(
    commands: &mut Commands,
    player: PlayerId,
    position: Vec2,
    rally_target: Vec2,
    color: Color,
) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(24.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.2),
        Unit {
            player,
            rally_target,
            kind: UnitKind::Laser,
            health: UnitKind::Laser.health(),
            max_health: UnitKind::Laser.health(),
            attack_timer: Timer::from_seconds(
                UnitKind::Laser.attack_cooldown(),
                TimerMode::Repeating,
            ),
        },
    ));
}

fn tick_spawn_timers(
    time: Res<Time>,
    mut rng: ResMut<SimulationRng>,
    registry: Res<SpawnRegistry>,
    mut timers: ResMut<SpawnTimers>,
    mut commands: Commands,
    units: Query<(&Unit, &Transform)>,
) {
    for (idx, timer) in timers.timers.iter_mut().enumerate() {
        if timer.tick(time.delta()).just_finished() {
            if let Some(entry) = registry.entries.get(idx) {
                let jitter = Vec2::new(rng.gen_f32(-20.0..=20.0), rng.gen_f32(-20.0..=20.0));
                let start = entry.position + jitter;
                let rally_target =
                    average_unit_position(entry.player, &units).unwrap_or(entry.position);
                spawn_unit(
                    &mut commands,
                    entry.player,
                    start,
                    rally_target,
                    PLAYER_COLORS[entry.player.0],
                );
            }
        }
    }
}

fn average_unit_position(player: PlayerId, units: &Query<(&Unit, &Transform)>) -> Option<Vec2> {
    let mut sum = Vec2::ZERO;
    let mut count = 0.0;
    for (unit, transform) in units.iter() {
        if unit.player == player {
            sum += transform.translation.truncate();
            count += 1.0;
        }
    }
    if count > 0.0 {
        Some(sum / count)
    } else {
        None
    }
}

fn handle_selection_input(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    mut queries: ParamSet<(
        Query<(Entity, &Transform, &Unit)>,
        Query<(&mut Sprite, &mut Transform), With<SelectionCircle>>,
    )>,
    control: Res<ControlSettings>,
) {
    let cursor_world = cursor_world_position(&windows, &cameras);

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = cursor_world {
            selection.is_dragging = true;
            selection.start_world = pos;
            selection.radius = MIN_SELECTION_RADIUS;

            let entity = commands
                .spawn((
                    Sprite {
                        color: Color::srgba(0.2, 0.8, 1.0, 0.25),
                        custom_size: Some(Vec2::splat(selection.radius * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.5),
                    SelectionCircle,
                ))
                .id();
            selection.circle_entity = Some(entity);
        }
    }

    if selection.is_dragging && buttons.pressed(MouseButton::Left) {
        selection.radius += SELECTION_GROWTH_RATE * time.delta_secs();
        if let Some(entity) = selection.circle_entity {
            if let Ok((mut sprite, mut transform)) = queries.p1().get_mut(entity) {
                sprite.custom_size = Some(Vec2::splat(selection.radius * 2.0));
                transform.translation = selection.start_world.extend(0.5);
            }
        }
    }

    if selection.is_dragging && buttons.just_released(MouseButton::Left) {
        let mut selected = Vec::new();
        let units = queries.p0();
        for (entity, transform, unit) in units.iter() {
            if unit.player == control.local_player {
                let pos = transform.translation.truncate();
                if pos.distance(selection.start_world) <= selection.radius {
                    selected.push(entity);
                }
            }
        }
        selection.prev_selected = std::mem::take(&mut selection.selected);
        selection.selected = selected;
        selection.dirty = true;
        selection.is_dragging = false;
        selection.radius = 0.0;
        if let Some(entity) = selection.circle_entity.take() {
            commands.entity(entity).despawn_recursive();
        }
    }

    {
        let units = queries.p0();
        selection
            .selected
            .retain(|entity| units.get(*entity).is_ok());
    }
}

fn issue_move_orders(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    selection: Res<SelectionState>,
    mut units: Query<&mut Unit>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    let Some(cursor) = cursor_world_position(&windows, &cameras) else {
        return;
    };
    if selection.selected.is_empty() {
        return;
    }

    let offsets = compute_formation_offsets(selection.selected.len());
    for (entity, offset) in selection.selected.iter().zip(offsets.iter()) {
        if let Ok(mut unit) = units.get_mut(*entity) {
            unit.rally_target = cursor + *offset;
        }
    }
}

fn compute_formation_offsets(count: usize) -> Vec<Vec2> {
    let mut offsets = Vec::with_capacity(count);
    if count == 0 {
        return offsets;
    }
    if count >= 1 {
        offsets.push(Vec2::ZERO);
    }
    let mut generated = offsets.len();
    let mut ring = 1;
    while generated < count {
        let slots = (ring * 6) as usize;
        for idx in 0..slots {
            if generated == count {
                break;
            }
            let angle = idx as f32 / slots as f32 * TAU;
            offsets.push(Vec2::from_angle(angle) * (ring as f32 * FORMATION_SPACING));
            generated += 1;
        }
        ring += 1;
    }
    offsets
}

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, camera_transform) = cameras.get_single().ok()?;
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    Some(ray.origin.truncate())
}

fn update_selection_visuals(
    mut commands: Commands,
    mut selection: ResMut<SelectionState>,
    units: Query<&Unit>,
    highlights: Query<&SelectionHighlight>,
) {
    if !selection.dirty {
        return;
    }

    for entity in selection.prev_selected.drain(..) {
        if let Ok(highlight) = highlights.get(entity) {
            commands.entity(highlight.glow).despawn_recursive();
            commands.entity(entity).remove::<SelectionHighlight>();
        }
    }

    for &entity in &selection.selected {
        if highlights.get(entity).is_ok() {
            continue;
        }
        if units.get(entity).is_err() {
            continue;
        }
        let glow = commands
            .spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.95, 0.2, 0.35),
                    custom_size: Some(Vec2::new(42.0, 56.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
            ))
            .id();
        commands.entity(entity).add_child(glow);
        commands.entity(entity).insert(SelectionHighlight { glow });
    }

    selection.dirty = false;
}

fn move_units(time: Res<Time>, mut units: Query<(&mut Transform, &Unit)>) {
    for (mut transform, unit) in units.iter_mut() {
        let pos = transform.translation.truncate();
        let delta = unit.rally_target - pos;
        if delta.length() > 1.0 {
            let dir = delta.normalize();
            transform.translation.x += dir.x * UNIT_SPEED * time.delta_secs();
            transform.translation.y += dir.y * UNIT_SPEED * time.delta_secs();
        }
    }
}

fn update_unit_rally_targets(mut units: Query<(Entity, &mut Unit, &Transform)>) {
    let mut positions = Vec::new();
    for (entity, unit, transform) in units.iter() {
        positions.push((entity, unit.player, transform.translation.truncate()));
    }

    for (entity, mut unit, transform) in units.iter_mut() {
        let mut push = Vec2::ZERO;
        for (other_entity, other_player, other_pos) in positions.iter() {
            if entity == *other_entity || unit.player != *other_player {
                continue;
            }
            let offset = transform.translation.truncate() - *other_pos;
            let distance = offset.length();
            if distance > 0.1 && distance < UNIT_SEPARATION_RADIUS {
                push += offset.normalize() * (UNIT_SEPARATION_RADIUS - distance)
                    / UNIT_SEPARATION_RADIUS;
            }
        }
        if push.length_squared() > 0.0 {
            unit.rally_target = unit.rally_target + push * 0.5;
        }
    }
}

fn unit_combat_system(
    time: Res<Time>,
    mut commands: Commands,
    mut units: Query<(Entity, &Transform, &mut Unit)>,
) {
    let snapshot: Vec<_> = units
        .iter()
        .map(|(entity, transform, unit)| (entity, unit.player, transform.translation.truncate()))
        .collect();

    let mut damage_events: Vec<(Entity, f32)> = Vec::new();
    let mut heal_events: Vec<(Entity, f32)> = Vec::new();
    let mut deaths: Vec<Entity> = Vec::new();
    let mut beams: Vec<(Vec2, Vec2, Color, f32)> = Vec::new();

    for (entity, transform, mut unit) in units.iter_mut() {
        unit.attack_timer.tick(time.delta());

        // Healing pulse
        if unit.health < unit.max_health {
            if let Some(ally_pos) = snapshot
                .iter()
                .filter(|(other_entity, player, _)| {
                    *player == unit.player && *other_entity != entity
                })
                .map(|(_, _, pos)| *pos)
                .find(|pos| pos.distance(transform.translation.truncate()) <= LASER_HEAL_RANGE)
            {
                let heal_amount = LASER_HEAL_RATE * time.delta_secs();
                heal_events.push((entity, heal_amount));
                beams.push((
                    transform.translation.truncate(),
                    ally_pos,
                    Color::srgb(0.2, 1.0, 0.4),
                    6.0,
                ));
            }
        }

        // Attack
        if let Some((target_entity, target_pos)) = snapshot
            .iter()
            .filter(|(_, player, _)| *player != unit.player)
            .min_by(|(_, _, a), (_, _, b)| {
                a.distance_squared(transform.translation.truncate())
                    .partial_cmp(&b.distance_squared(transform.translation.truncate()))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(entity, _, pos)| (*entity, *pos))
        {
            let distance = target_pos.distance(transform.translation.truncate());
            if distance <= LASER_RANGE && unit.attack_timer.finished() {
                damage_events.push((target_entity, LASER_DAMAGE));
                beams.push((
                    transform.translation.truncate(),
                    target_pos,
                    Color::srgb(1.0, 0.2, 0.2),
                    4.0,
                ));
                let cooldown = unit.kind.attack_cooldown();
                unit.attack_timer
                    .set_duration(std::time::Duration::from_secs_f32(cooldown));
                unit.attack_timer.reset();
            }
        }
    }

    for (entity, amount) in heal_events {
        if let Ok((_, _, mut unit)) = units.get_mut(entity) {
            unit.health = (unit.health + amount).min(unit.max_health);
        }
    }

    for (entity, amount) in damage_events {
        if let Ok((_, _, mut unit)) = units.get_mut(entity) {
            unit.health -= amount;
            if unit.health <= 0.0 {
                deaths.push(entity);
            }
        }
    }

    for entity in deaths {
        commands.entity(entity).despawn_recursive();
    }

    for (start, end, color, thickness) in beams {
        spawn_beam(&mut commands, start, end, color, thickness);
    }
}

fn spawn_beam(commands: &mut Commands, start: Vec2, end: Vec2, color: Color, thickness: f32) {
    let diff = end - start;
    let length = diff.length().max(1.0);
    let angle = diff.y.atan2(diff.x);
    let translation = Vec3::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0, 0.6);
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, thickness)),
            ..default()
        },
        Transform {
            translation,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        BeamEffect {
            timer: Timer::from_seconds(BEAM_LIFETIME, TimerMode::Once),
        },
    ));
}

fn update_beam_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut beams: Query<(Entity, &mut BeamEffect)>,
) {
    for (entity, mut effect) in beams.iter_mut() {
        if effect.timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn configure_fixed_time(mut fixed_time: ResMut<Time<Fixed>>, params: Res<SimulationParams>) {
    fixed_time.set_timestep_seconds(params.fixed_delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_registry_matches_player_count() {
        let mut app = App::new();
        app.insert_resource(BoardSettings {
            player_count: 3,
            ..Default::default()
        });
        app.add_systems(Startup, setup_board);
        app.update();
        let registry = app.world().resource::<SpawnRegistry>();
        assert_eq!(registry.entries.len(), 3);
    }
}
