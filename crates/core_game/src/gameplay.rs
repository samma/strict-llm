use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::time::{Fixed, Time};
use bevy::utils::{HashMap, HashSet};
use bevy::window::PrimaryWindow;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

const DEFAULT_SEED: u64 = 42;
const DEFAULT_FIXED_DELTA: f64 = 1.0 / 30.0;
const DEFAULT_BOARD_SIZE: f32 = 1600.0;
const DEFAULT_PLAYER_COUNT: usize = 4;
const DEFAULT_SPAWN_INTERVAL: f32 = 1.0;
const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 8;
const UNIT_SPEED: f32 = 120.0;
const UNIT_ACCELERATION: f32 = 8.0;
const UNIT_SEPARATION_RADIUS: f32 = 40.0;
const SEPARATION_FORCE: f32 = 60.0;
const FORMATION_SPACING: f32 = 60.0;
const LASER_RANGE: f32 = 260.0;
const LASER_DAMAGE: f32 = 6.0;
const LASER_COOLDOWN: f32 = 0.7;
const LASER_HEAL_RANGE: f32 = 150.0;
const BEAM_LIFETIME: f32 = 0.15;
const SUPPORT_HEAL_PER_SECOND: f32 = 1.0;
const SUPPORT_DAMAGE_BONUS: f32 = 0.05;
const PYLON_COUNT: usize = 3;
const PYLON_RADIUS: f32 = 180.0;
const PYLON_DAMAGE_BONUS: f32 = 0.04;
const PYLON_GRAVITY: f32 = 18000.0;
const PYLON_MAX_SPEED: f32 = 240.0;

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
                (
                    setup_board,
                    spawn_initial_units.after(setup_board),
                    spawn_pylons.after(setup_board),
                ),
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
                    animate_pylons,
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
    current_world: Vec2,
    rectangle_entity: Option<Entity>,
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
    pub velocity: Vec2,
    pub base_color: Color,
    pub boost_visual: Option<Entity>,
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
struct SelectionRect;

#[derive(Component)]
struct SelectionHighlight {
    glow: Entity,
}

#[derive(Component)]
struct Pylon {
    velocity: Vec2,
    mass: f32,
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

fn spawn_pylons(
    mut commands: Commands,
    settings: Res<BoardSettings>,
    mut rng: ResMut<SimulationRng>,
) {
    for idx in 0..PYLON_COUNT {
        let radius = settings.board_size * (0.15 + rng.gen_f32(0.0..=0.15));
        let angle = rng.gen_f32(0.0..=TAU);
        let position = Vec2::new(angle.cos(), angle.sin()) * radius;
        let speed = rng.gen_f32(20.0..=60.0);
        let velocity = Vec2::new(-angle.sin(), angle.cos()) * speed;
        let color = Color::srgb(0.4, 0.85, 1.0);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(26.0, 38.0)),
                ..default()
            },
            Transform {
                translation: Vec3::new(position.x, position.y, 0.2 + idx as f32 * 0.01),
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                ..default()
            },
            Pylon {
                velocity,
                mass: 1.0 + rng.gen_f32(0.0..=1.0),
            },
        ));
    }
}

fn animate_pylons(
    time: Res<Time>,
    settings: Res<BoardSettings>,
    mut pylons: Query<(Entity, &mut Transform, &mut Pylon)>,
) {
    let dt = time.delta_secs();
    if pylons.is_empty() {
        return;
    }
    let snapshots: Vec<(Entity, Vec2, Vec2, f32)> = pylons
        .iter()
        .map(|(entity, transform, pylon)| {
            (
                entity,
                transform.translation.truncate(),
                pylon.velocity,
                pylon.mass,
            )
        })
        .collect();

    let mut accelerations: HashMap<Entity, Vec2> = HashMap::default();
    for (entity_a, pos_a, _, _) in &snapshots {
        let mut acc = Vec2::ZERO;
        for (entity_b, pos_b, _, mass_b) in &snapshots {
            if entity_a == entity_b {
                continue;
            }
            let offset = *pos_b - *pos_a;
            let dist_sq = offset.length_squared().max(4000.0);
            acc += offset.normalize() * (PYLON_GRAVITY * *mass_b / dist_sq);
        }
        accelerations.insert(*entity_a, acc);
    }

    let boundary = settings.board_size * 0.45;
    for (entity, mut transform, mut pylon) in pylons.iter_mut() {
        if let Some(acc) = accelerations.get(&entity) {
            pylon.velocity += *acc * dt;
        }
        pylon.velocity = pylon.velocity.clamp_length_max(PYLON_MAX_SPEED);
        transform.translation.x += pylon.velocity.x * dt;
        transform.translation.y += pylon.velocity.y * dt;
        if transform.translation.x.abs() > boundary {
            transform.translation.x = transform.translation.x.clamp(-boundary, boundary);
            pylon.velocity.x = -pylon.velocity.x;
        }
        if transform.translation.y.abs() > boundary {
            transform.translation.y = transform.translation.y.clamp(-boundary, boundary);
            pylon.velocity.y = -pylon.velocity.y;
        }
        transform.translation.z = 0.2;
    }
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
            velocity: Vec2::ZERO,
            base_color: color,
            boost_visual: None,
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
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    mut queries: ParamSet<(
        Query<(Entity, &Transform, &Unit)>,
        Query<(&mut Sprite, &mut Transform), With<SelectionRect>>,
    )>,
    control: Res<ControlSettings>,
) {
    let cursor_world = cursor_world_position(&windows, &cameras);

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = cursor_world {
            selection.is_dragging = true;
            selection.start_world = pos;
            selection.current_world = pos;

            let entity = commands
                .spawn((
                    Sprite {
                        color: Color::srgba(0.2, 0.8, 1.0, 0.2),
                        custom_size: Some(Vec2::splat(2.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.5),
                    SelectionRect,
                ))
                .id();
            selection.rectangle_entity = Some(entity);
        }
    }

    if selection.is_dragging && buttons.pressed(MouseButton::Left) {
        if let Some(pos) = cursor_world {
            selection.current_world = pos;
        }
        if let Some(entity) = selection.rectangle_entity {
            if let Ok((mut sprite, mut transform)) = queries.p1().get_mut(entity) {
                let min = Vec2::new(
                    selection.start_world.x.min(selection.current_world.x),
                    selection.start_world.y.min(selection.current_world.y),
                );
                let max = Vec2::new(
                    selection.start_world.x.max(selection.current_world.x),
                    selection.start_world.y.max(selection.current_world.y),
                );
                let size = max - min;
                sprite.custom_size = Some(Vec2::new(size.x.abs().max(2.0), size.y.abs().max(2.0)));
                transform.translation =
                    Vec3::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, 0.5);
            }
        }
    }

    if selection.is_dragging && buttons.just_released(MouseButton::Left) {
        let units = queries.p0();
        let min = Vec2::new(
            selection.start_world.x.min(selection.current_world.x),
            selection.start_world.y.min(selection.current_world.y),
        );
        let max = Vec2::new(
            selection.start_world.x.max(selection.current_world.x),
            selection.start_world.y.max(selection.current_world.y),
        );
        let padded_min = min - Vec2::splat(8.0);
        let padded_max = max + Vec2::splat(8.0);
        let mut newly_selected = Vec::new();
        for (entity, transform, unit) in units.iter() {
            if unit.player == control.local_player {
                let pos = transform.translation.truncate();
                if pos.x >= padded_min.x
                    && pos.x <= padded_max.x
                    && pos.y >= padded_min.y
                    && pos.y <= padded_max.y
                {
                    newly_selected.push(entity);
                }
            }
        }

        let drag_delta = selection.current_world - selection.start_world;
        let is_click = drag_delta.length_squared() < 16.0;
        selection.prev_selected = std::mem::take(&mut selection.selected);
        if is_click && newly_selected.is_empty() {
            selection.selected.clear();
        } else {
            let mut set: HashSet<Entity> = selection.prev_selected.iter().copied().collect();
            for entity in newly_selected {
                set.insert(entity);
            }
            selection.selected = set.into_iter().collect();
        }
        selection.dirty = true;
        selection.is_dragging = false;
        selection.current_world = selection.start_world;
        if let Some(entity) = selection.rectangle_entity.take() {
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

fn update_boost_visual(entity: Entity, unit: &mut Unit, active: bool, commands: &mut Commands) {
    if active {
        if unit.boost_visual.is_some() {
            return;
        }
        let glow = commands
            .spawn((
                Sprite {
                    color: Color::srgba(0.98, 0.95, 0.45, 0.2),
                    custom_size: Some(Vec2::splat(40.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.05),
            ))
            .id();
        commands.entity(entity).add_child(glow);
        unit.boost_visual = Some(glow);
    } else if let Some(glow) = unit.boost_visual.take() {
        if let Some(cmds) = commands.get_entity(glow) {
            cmds.despawn_recursive();
        }
    }
}

fn move_units(time: Res<Time>, mut units: Query<(&mut Transform, &mut Unit)>) {
    let dt = time.delta_secs();
    let accel = 1.0 - (-UNIT_ACCELERATION * dt).exp();
    for (mut transform, mut unit) in units.iter_mut() {
        let pos = transform.translation.truncate();
        let delta = unit.rally_target - pos;
        let desired = if delta.length_squared() > 1.0 {
            delta.normalize() * UNIT_SPEED
        } else {
            Vec2::ZERO
        };
        unit.velocity = unit.velocity.lerp(desired, accel);
        transform.translation.x += unit.velocity.x * dt;
        transform.translation.y += unit.velocity.y * dt;
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
            let push_dir = push.normalize_or_zero();
            unit.rally_target += push_dir * 5.0;
            unit.velocity += push_dir * SEPARATION_FORCE;
            unit.velocity = unit.velocity.clamp_length_max(UNIT_SPEED * 1.5);
        }
    }
}

fn unit_combat_system(
    time: Res<Time>,
    spawn_registry: Res<SpawnRegistry>,
    pylons: Query<&Transform, (With<Pylon>, Without<Unit>)>,
    mut commands: Commands,
    mut unit_queries: ParamSet<(
        Query<(Entity, &Transform, &Unit)>,
        Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    )>,
) {
    let snapshot: Vec<_> = {
        let query = unit_queries.p0();
        query
            .iter()
            .map(|(entity, transform, unit)| {
                (entity, unit.player, transform.translation.truncate())
            })
            .collect()
    };

    let mut entity_info: HashMap<Entity, (PlayerId, Vec2)> = HashMap::default();
    for (entity, player, pos) in &snapshot {
        entity_info.insert(*entity, (*player, *pos));
    }

    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::default();
    let mut connections: HashMap<Entity, usize> = HashMap::default();
    let mut support_links: Vec<(Entity, Entity)> = Vec::new();
    for i in 0..snapshot.len() {
        for j in (i + 1)..snapshot.len() {
            let (entity_a, player_a, pos_a) = snapshot[i];
            let (entity_b, player_b, pos_b) = snapshot[j];
            if player_a != player_b {
                continue;
            }
            if pos_a.distance(pos_b) <= LASER_HEAL_RANGE {
                connections
                    .entry(entity_a)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                connections
                    .entry(entity_b)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                adjacency.entry(entity_a).or_default().push(entity_b);
                adjacency.entry(entity_b).or_default().push(entity_a);
                support_links.push((entity_a, entity_b));
            }
        }
    }

    let mut connected_entities: HashSet<Entity> = HashSet::default();
    let mut supply_components: Vec<Vec<Entity>> = Vec::new();
    for entry in spawn_registry.entries.iter() {
        let mut queue = VecDeque::new();
        let mut component = Vec::new();
        for (entity, _player, pos) in snapshot
            .iter()
            .filter(|(_, player, _)| *player == entry.player)
        {
            if pos.distance(entry.position) <= LASER_HEAL_RANGE {
                if connected_entities.insert(*entity) {
                    queue.push_back(*entity);
                    component.push(*entity);
                }
            }
        }
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if entity_info
                        .get(&neighbor)
                        .map(|(player, _)| *player == entry.player)
                        .unwrap_or(false)
                    {
                        if connected_entities.insert(neighbor) {
                            queue.push_back(neighbor);
                            component.push(neighbor);
                        }
                    }
                }
            }
        }
        if !component.is_empty() {
            supply_components.push(component);
        }
    }

    let pylon_positions: Vec<Vec2> = pylons
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();

    let mut component_bonus: HashMap<Entity, f32> = HashMap::default();
    let mut component_pylon_active: HashSet<Entity> = HashSet::default();
    for component in supply_components {
        let mut bonus = 0.0;
        let mut component_has_pylon = false;
        for entity in &component {
            if let Some((_, pos)) = entity_info.get(entity) {
                if pylon_positions
                    .iter()
                    .any(|pylon_pos| pos.distance(*pylon_pos) <= PYLON_RADIUS)
                {
                    bonus += PYLON_DAMAGE_BONUS;
                    component_has_pylon = true;
                }
            }
        }
        for entity in component.iter().copied() {
            component_bonus.insert(entity, bonus);
            if component_has_pylon {
                component_pylon_active.insert(entity);
            }
        }
    }

    let delta = time.delta();
    let delta_secs = delta.as_secs_f32();
    let mut damage_events: Vec<(Entity, f32)> = Vec::new();
    let mut deaths: Vec<Entity> = Vec::new();
    let mut beams: Vec<(Vec2, Vec2, Color, f32)> = Vec::new();

    let mut unit_write = unit_queries.p1();
    for (entity, mut transform, mut sprite, mut unit) in unit_write.iter_mut() {
        unit.attack_timer.tick(delta);
        let connection_count = connections.get(&entity).copied().unwrap_or(0);
        let boost_active = connected_entities.contains(&entity);
        let pylon_bonus = component_bonus.get(&entity).copied().unwrap_or(0.0);
        update_boost_visual(entity, &mut unit, boost_active, &mut commands);
        let scale = if boost_active { 1.12 } else { 1.0 };
        transform.scale = Vec3::new(scale, scale, 1.0);
        sprite.color = unit.base_color;

        if boost_active && connection_count > 0 && unit.health < unit.max_health {
            let heal_amount = connection_count as f32 * SUPPORT_HEAL_PER_SECOND * delta_secs;
            unit.health = (unit.health + heal_amount).min(unit.max_health);
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
                let mut damage_multiplier = 1.0;
                if boost_active {
                    damage_multiplier += connection_count as f32 * SUPPORT_DAMAGE_BONUS;
                    damage_multiplier += pylon_bonus;
                }
                damage_events.push((target_entity, LASER_DAMAGE * damage_multiplier));
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

    for (entity, amount) in damage_events {
        if let Ok((_, _, _, mut unit)) = unit_write.get_mut(entity) {
            unit.health -= amount;
            if unit.health <= 0.0 {
                deaths.push(entity);
            }
        }
    }

    let mut pylon_energy_links: Vec<(Vec2, Vec2)> = Vec::new();
    for pylon_pos in &pylon_positions {
        for (entity, (_, unit_pos)) in entity_info.iter() {
            if !component_pylon_active.contains(entity) {
                continue;
            }
            if unit_pos.distance(*pylon_pos) <= PYLON_RADIUS {
                pylon_energy_links.push((*pylon_pos, *unit_pos));
            }
        }
    }

    for (entity_a, entity_b) in support_links {
        let Some((_, pos_a)) = entity_info.get(&entity_a) else {
            continue;
        };
        let Some((_, pos_b)) = entity_info.get(&entity_b) else {
            continue;
        };
        let pylon_active = component_pylon_active.contains(&entity_a)
            || component_pylon_active.contains(&entity_b);
        let color = support_link_color(pylon_active);
        spawn_support_link(&mut commands, *pos_a, *pos_b, color);
    }

    for entity in deaths {
        commands.entity(entity).despawn_recursive();
    }

    for (start, end, color, thickness) in beams {
        spawn_beam(&mut commands, start, end, color, thickness);
    }

    for (pylon_pos, unit_pos) in pylon_energy_links {
        spawn_support_link(
            &mut commands,
            pylon_pos,
            unit_pos,
            Color::srgb(0.2, 0.7, 1.0),
        );
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

fn spawn_support_link(commands: &mut Commands, start: Vec2, end: Vec2, color: Color) {
    spawn_beam(commands, start, end, color.with_alpha(0.85), 2.6);
    spawn_beam(commands, start, end, color.with_alpha(0.3), 4.4);
}

fn support_link_color(pylon_active: bool) -> Color {
    if pylon_active {
        Color::srgb(0.22, 0.8, 0.95)
    } else {
        Color::srgb(0.24, 0.98, 0.55)
    }
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
