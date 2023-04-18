use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::loading::TextureAssets;
use crate::menu::connect::LocalHandle;
use crate::network::{AgreedRandom, GGRSConfig, INPUT_EXIT};
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

const STARTING_SPEED: f32 = 150.;
const STRAWBERRY_SIZE: f32 = 32.0;
const STRAWBERRY_SPAWN_TIME: f32 = 2.5;

#[derive(Component, Debug, Reflect, Default, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    handle: usize,
    speed: f32,
    active: bool,
    just_moved: bool,
    pub exp: usize,
}

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EdibleSpawnTimer {
    strawberry_timer: Timer,
}

impl Default for EdibleSpawnTimer {
    fn default() -> Self {
        EdibleSpawnTimer {
            strawberry_timer: Timer::from_seconds(STRAWBERRY_SPAWN_TIME, TimerMode::Repeating),
        }
    }
}

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
            .add_system(tick_edible_timer)
            .add_system(spawn_strawberry_over_time.in_set(OnUpdate(GameState::RoundLocal)))
            .add_system(spawn_strawberry_over_time.in_set(OnUpdate(GameState::RoundOnline)))
            // these systems will be executed as part of the advance frame update
            .add_systems(
                (move_players, player_ate_strawberry_system, exit_to_menu)
                    .chain()
                    .in_schedule(GGRSSchedule),
            );
    }
}

fn camera_follow(
    player_handle: Option<Res<LocalHandle>>,
    player_query: Query<(&Transform, &Player)>,
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
    commands
        .spawn(Camera2dBundle::default())
        .insert(PlayerComponent);

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    for handle in 0..NUM_PLAYERS {
        let name = format!("Player {}", handle);
        commands.spawn((
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
            Name::new(name),
            Player {
                handle,
                speed: STARTING_SPEED,
                active: true,
                just_moved: false,
                exp: 1,
            },
            PlayerComponent,
            Rollback::new(rip.next_id()),
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

fn despawn_players(mut commands: Commands, query: Query<Entity, With<PlayerComponent>>) {
    commands.remove_resource::<LocalHandle>();
    commands.remove_resource::<Session<GGRSConfig>>();

    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn move_players(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    walls: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    mut players: Query<(&mut Transform, &mut TextureAtlasSprite, &mut Player), With<Rollback>>,
) {
    // loop over all players and apply their inputs to movement
    // do NOT return early because we need to check all players for input/movement
    for (mut transform, mut sprite, mut player) in players.iter_mut() {
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

        let mut direction = Vec2::ZERO;
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
        if direction == Vec2::ZERO {
            continue; // don't return, we need to check other players for movement
        }

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
    edible_spawn_timer.strawberry_timer.tick(time.delta());
}

fn spawn_strawberry_over_time(
    mut commands: Commands,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    mut agreed_seed: ResMut<AgreedRandom>,
) {
    if timer.strawberry_timer.finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.gen_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        debug!("Spawning strawberry!");
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(pos.translation.x, pos.translation.y, 0.0),
                texture: asset_server.texture_strawberry.clone(),
                ..Default::default()
            },
            Strawberry {},
            PlayerComponent {},
        ));
    }
}

// TODO: add sound
// TODO: build sprint
// TODO: cap movement at a certain speed
fn player_ate_strawberry_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>,
    strawberry_query: Query<(Entity, &Transform), With<Strawberry>>,
) {
    for (pt, mut p) in player_query.iter_mut() {
        for (s, st) in strawberry_query.iter() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + STRAWBERRY_SIZE / 2.0 {
                info!("Player ate strawberry!");
                p.speed += 10.0;
                commands.entity(s).despawn();
            }
        }
    }
}
