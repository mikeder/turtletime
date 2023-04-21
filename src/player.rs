use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::loading::TextureAssets;
use crate::menu::connect::LocalHandle;
use crate::menu::win::MatchData;
use crate::network::{AgreedRandom, GGRSConfig, INPUT_EXIT, INPUT_FIRE, INPUT_SPRINT};
use crate::network::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};
use crate::tilemap::{EncounterSpawner, PlayerSpawn, TileCollider};
use crate::{GameState, FPS};
use crate::{NUM_PLAYERS, TILE_SIZE};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_ggrs::PlayerInputs;
use bevy_ggrs::Rollback;
use bevy_ggrs::RollbackIdProvider;
use bevy_ggrs::{GGRSSchedule, Session};
use bevy_inspector_egui::prelude::*;
use ggrs::InputStatus;
use rand::Rng;

const FIREBALL_RADIUS: f32 = 8.0;
const FIREBALL_DAMAGE: f32 = 25.0;
const FIREBALL_LIFETIME: f32 = 10.0; // fireballs live for 10s
const STARTING_HEALTH: f32 = 100.;
const STARTING_SPEED: f32 = 150.;
const MAXIMUM_SPEED: f32 = 1500.;
const CHILI_PEPPER_SIZE: f32 = 20.0;
const CHILI_PEPPER_SPAWN_RATE: f32 = 3.5;
const CHILI_PEPPER_AMMO_COUNT: usize = 15;
const STRAWBERRY_SIZE: f32 = 32.0;
const STRAWBERRY_SPAWN_RATE: f32 = 2.5;

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    active: bool,
    fire_ball_ammo: usize,
    sprint_ready: bool,
    sprint_ammo: usize,
    handle: usize,
    health: f32,
    just_moved: bool,
    speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            active: true,
            fire_ball_ammo: 0,
            sprint_ready: false,
            sprint_ammo: 0,
            handle: 0,
            health: STARTING_HEALTH,
            just_moved: false,
            speed: STARTING_SPEED,
        }
    }
}

pub struct PlayerPlugin;

#[derive(Component)]
pub struct RoundComponent;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Fireball {
    move_dir: Vec2,
    shot_by: usize,
    speed: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FireballTimer {
    lifetime: Timer,
}

impl Default for FireballTimer {
    fn default() -> Self {
        FireballTimer {
            lifetime: Timer::from_seconds(FIREBALL_LIFETIME, TimerMode::Once),
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct FireballReady(pub bool);

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct MoveDir(pub Vec2);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EdibleSpawnTimer {
    chili_pepper_timer: Timer,
    strawberry_timer: Timer,
}

impl Default for EdibleSpawnTimer {
    fn default() -> Self {
        EdibleSpawnTimer {
            chili_pepper_timer: Timer::from_seconds(CHILI_PEPPER_SPAWN_RATE, TimerMode::Repeating),
            strawberry_timer: Timer::from_seconds(STRAWBERRY_SPAWN_RATE, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ChiliPepper;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Strawberry;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EdibleSpawnTimer>()
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundLocal)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundOnline)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundLocal)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundOnline)))
            // fireball timers only used for despawn of old fireballs
            .add_system(tick_fireball_timers)
            // these systems will be executed as part of the advance frame update
            .add_systems(
                (
                    move_players,
                    shoot_fireballs,
                    reload_fireballs,
                    move_fireballs,
                    kill_players,
                    tick_edible_timer,
                    spawn_strawberry_over_time,
                    spawn_chili_pepper_over_time,
                    player_ate_chili_pepper_system,
                    player_ate_strawberry_system,
                    check_win_state,
                    exit_to_menu,
                    despawn_old_fireballs,
                )
                    .chain()
                    .in_schedule(GGRSSchedule),
            );
    }
}

fn camera_follow(
    player_handle: Option<Res<LocalHandle>>,
    player_query: Query<(&Transform, &Player), Without<Fireball>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player_transform, player) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    characters: Res<CharacterSheet>,
    mut rip: ResMut<RollbackIdProvider>,
    spawn_query: Query<&mut PlayerSpawn>,
) {
    commands.spawn((Camera2dBundle::default(), RoundComponent));

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    for handle in 0..NUM_PLAYERS {
        let name = format!("Player {}", handle);
        commands.spawn((
            Name::new(name),
            SpriteSheetBundle {
                sprite: sprite.clone(),
                texture_atlas: characters.handle.clone(),
                transform: Transform {
                    translation: Vec3::new(spawns[handle].pos.x, spawns[handle].pos.y, 1.),
                    ..Default::default()
                },
                ..Default::default()
            },
            FrameAnimation {
                timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                frames: characters.turtle_frames.to_vec(),
                current_frame: 0,
            },
            Player {
                handle,
                ..Default::default()
            },
            FireballReady(false),
            RoundComponent,
            MoveDir(Vec2::X),
            rip.next(),
        ));
    }
}

fn exit_to_menu(inputs: Res<PlayerInputs<GGRSConfig>>, mut state: ResMut<NextState<GameState>>) {
    for (handle, input) in inputs.iter().enumerate() {
        match input.1 {
            InputStatus::Confirmed => {
                if input.0.input == INPUT_EXIT {
                    info!("Player {} exiting", handle);
                    state.set(GameState::MenuMain)
                }
            }
            _ => {}
        }
    }
}

fn despawn_players(mut commands: Commands, query: Query<Entity, With<RoundComponent>>) {
    commands.remove_resource::<LocalHandle>();
    commands.remove_resource::<Session<GGRSConfig>>();

    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn move_players(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    walls: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    mut players: Query<
        (
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut Player,
            &mut MoveDir,
        ),
        With<Rollback>,
    >,
) {
    // loop over all players and apply their inputs to movement
    // do NOT return early because we need to check all players for input/movement
    for (mut transform, mut sprite, mut player, mut move_dir) in players.iter_mut() {
        // reset just_moved each frame
        player.just_moved = false;
        if !player.active {
            continue; // don't return, we need to check other players for movement
        }

        let input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0.input,
            InputStatus::Predicted => inputs[player.handle].0.input,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };
        if input == 0 {
            continue; // don't return, we need to check other players for movement
        }

        let mut direction = Vec3::ZERO;
        if input & INPUT_UP != 0 {
            direction.y += 1.;
        }
        if input & INPUT_DOWN != 0 {
            direction.y -= 1.;
        }
        if input & INPUT_RIGHT != 0 {
            direction.x += 1.;
        }
        if input & INPUT_LEFT != 0 {
            direction.x -= 1.;
        }
        if input & INPUT_SPRINT != 0 {
            if player.sprint_ready && player.speed <= MAXIMUM_SPEED {
                player.speed += 50.0;
                player.sprint_ammo -= 1;
                if player.sprint_ammo == 0 {
                    player.sprint_ready = false;
                }
            }
        } else {
            if player.speed > STARTING_SPEED {
                player.speed -= 1.;
            }
        }
        if direction == Vec3::ZERO {
            continue; // don't return, we need to check other players for movement
        }
        // make sure we don't move faster diagonally than up/down
        direction = direction.normalize_or_zero();

        // set player MoveDir to the same direction for firing fireballs
        move_dir.0 = Vec2::new(direction.x, direction.y);

        let movement = (direction * player.speed / FPS as f32).extend(0.);

        if movement.x != 0. {
            player.just_moved = true;
            if movement.x > 0. {
                // moving to the right
                sprite.flip_x = false
            }
            if movement.x < 0. {
                // moving to the left
                sprite.flip_x = true
            }
        }
        if movement.y != 0. {
            player.just_moved = true;
        }

        let target = transform.translation + Vec3::new(0.0, movement.y, 0.0);
        if !walls
            .iter()
            .any(|&transform| wall_collision_check(target, transform.translation))
        {
            if movement.y != 0.0 {
                player.just_moved = true;
            }
            transform.translation = target;
        }

        let target = transform.translation + Vec3::new(movement.x, 0.0, 0.0);
        if !walls
            .iter()
            .any(|&transform| wall_collision_check(target, transform.translation))
        {
            if movement.x != 0.0 {
                player.just_moved = true;
                if movement.x > 0.0 {
                    sprite.flip_x = false;
                } else {
                    sprite.flip_x = true;
                }
            }
            transform.translation = target;
        }
    }
}

fn wall_collision_check(target_player_pos: Vec3, wall_translation: Vec3) -> bool {
    let collision = collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9), // give player small amount of leeway
        wall_translation,
        Vec2::splat(TILE_SIZE),
    );
    collision.is_some()
}

fn tick_edible_timer(mut edible_spawn_timer: ResMut<EdibleSpawnTimer>, time: Res<Time>) {
    edible_spawn_timer.chili_pepper_timer.tick(time.delta());
    edible_spawn_timer.strawberry_timer.tick(time.delta());
}

fn spawn_strawberry_over_time(
    mut commands: Commands,
    mut agreed_seed: ResMut<AgreedRandom>,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.strawberry_timer.finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.gen_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands.spawn((
            Name::new("Strawberry"),
            Strawberry {},
            RoundComponent {},
            SpriteBundle {
                transform: Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
                texture: asset_server.texture_strawberry.clone(),
                ..Default::default()
            },
            rip.next(),
        ));
    }
}

// TODO: add sound
// TODO: build sprint
// TODO: cap movement at a certain speed
fn player_ate_strawberry_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player), Without<Fireball>>,
    strawberry_query: Query<(Entity, &Transform), (With<Strawberry>, With<Rollback>)>,
) {
    for (pt, mut p) in player_query.iter_mut() {
        for (s, st) in strawberry_query.iter() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + STRAWBERRY_SIZE / 2.0 {
                p.sprint_ammo += 1;
                p.sprint_ready = true;
                commands.entity(s).despawn_recursive();
            }
        }
    }
}

fn spawn_chili_pepper_over_time(
    mut commands: Commands,
    mut agreed_seed: ResMut<AgreedRandom>,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.chili_pepper_timer.finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.gen_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands.spawn((
            Name::new("ChiliPepper"),
            ChiliPepper {},
            RoundComponent {},
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(CHILI_PEPPER_SIZE * 1.5)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
                texture: asset_server.texture_chili_pepper.clone(),
                ..Default::default()
            },
            rip.next(),
        ));
    }
}

// TODO: add sound
// TODO: build sprint
// TODO: cap movement at a certain speed
fn player_ate_chili_pepper_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player), Without<Fireball>>,
    pepper_query: Query<(Entity, &Transform), (With<ChiliPepper>, With<Rollback>)>,
) {
    for (pt, mut p) in player_query.iter_mut() {
        for (s, st) in pepper_query.iter() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + CHILI_PEPPER_SIZE / 2.0 {
                p.fire_ball_ammo += CHILI_PEPPER_AMMO_COUNT;
                commands.entity(s).despawn_recursive();
            }
        }
    }
}

// reload_fireball prevents the player from continuously shooting fireballs by holding INPUT_FIRE
fn reload_fireballs(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut query: Query<(&mut FireballReady, &Player)>,
) {
    for (mut can_fire, player) in query.iter_mut() {
        let input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0.input,
            InputStatus::Predicted => inputs[player.handle].0.input,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };
        if !input & INPUT_FIRE != 0 && player.fire_ball_ammo > 0 {
            can_fire.0 = true;
        }
    }
}

fn shoot_fireballs(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
    images: Res<TextureAssets>,
    mut player_query: Query<(&Transform, &mut Player, &MoveDir, &mut FireballReady)>,
) {
    for (transform, mut player, move_dir, mut fireball_ready) in player_query.iter_mut() {
        if !player.active {
            continue; // don't let dead/inactive players continue firing
        }

        let input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0.input,
            InputStatus::Predicted => inputs[player.handle].0.input,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };
        if input == 0 {
            continue; // don't return, we need to check other players for movement
        }

        if input & INPUT_FIRE != 0 {
            if !fireball_ready.0 || player.fire_ball_ammo == 0 {
                // fireball not ready or player out of ammo
                continue;
            }

            // position fireball slightly away from players position
            let player_pos = transform.translation;
            let pos = player_pos
                + (Vec3::new(move_dir.0.x, move_dir.0.y, 0.)) * (TILE_SIZE * 1.5)
                + FIREBALL_RADIUS;

            commands.spawn((
                Name::new("Fireball"),
                Fireball {
                    move_dir: move_dir.0,
                    shot_by: player.handle,
                    speed: player.speed,
                },
                FireballTimer::default(),
                RoundComponent,
                SpriteBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, move_dir.0)),
                    texture: images.texture_fireball.clone(),
                    ..default()
                },
                rip.next(),
            ));

            player.fire_ball_ammo -= 1;
            fireball_ready.0 = false;
        }
    }
}

fn move_fireballs(mut query: Query<(&mut Transform, &Fireball), With<Rollback>>) {
    for (mut transform, fireball) in query.iter_mut() {
        transform.translation += (fireball.move_dir * (fireball.speed * 0.05)).extend(0.);
    }
}

fn tick_fireball_timers(mut query: Query<&mut FireballTimer>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.lifetime.tick(time.delta());
    }
}

fn despawn_old_fireballs(mut commands: Commands, mut query: Query<(Entity, &FireballTimer)>) {
    for (fireball, timer) in query.iter_mut() {
        if timer.lifetime.finished() {
            commands.entity(fireball).despawn_recursive()
        }
    }
}

fn kill_players(
    mut commands: Commands,
    mut player_query: Query<
        (
            &mut Player,
            &mut FrameAnimation,
            &mut TextureAtlasSprite,
            &Transform,
        ),
        (With<Player>, Without<Fireball>),
    >,
    fireball_query: Query<(Entity, &Transform, &Fireball), With<Rollback>>,
) {
    for (mut player, mut animation, mut sprite, player_transform) in player_query.iter_mut() {
        for (entity, fireball_transform, fireball) in fireball_query.iter() {
            let distance = player_transform
                .translation
                .distance(fireball_transform.translation);

            if fireball.shot_by == player.handle {
                continue; // don't allow player to suicide
            };

            if distance < TILE_SIZE + FIREBALL_RADIUS {
                player.health -= FIREBALL_DAMAGE;
                if player.health <= 0. {
                    animation.timer.set_mode(TimerMode::Once);
                    sprite.flip_y = true;
                    player.active = false;
                }
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn check_win_state(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Player, Without<Fireball>>,
) {
    let mut remaning_active = vec![];
    for player in player_query.iter() {
        if player.active {
            remaning_active.push(player);
        }
    }
    if remaning_active.len() == 1 {
        let result = format!("Player {} wins the round!", remaning_active[0].handle);
        commands.insert_resource(MatchData { result });
        next_state.set(GameState::Win)
    }
}
