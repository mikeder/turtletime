use crate::loading::TextureAssets;
use crate::menu::connect::LocalHandle;
use crate::network::GGRSConfig;
use crate::network::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};
use crate::tilemap::{PlayerSpawn, TileCollider};
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

const STARTING_SPEED: f32 = 150.;

#[derive(Component, Debug, Reflect, Default, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    handle: usize,
    speed: f32,
    active: bool,
    just_moved: bool,
    pub exp: usize,
}

#[derive(Resource, Default, Reflect, Hash)]
#[reflect(Resource, Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EncounterTracker {
    timer: Timer,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameCount>()
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundLocal)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundOnline)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundLocal)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundOnline)))
            .add_system(exit_to_menu)
            // these systems will be executed as part of the advance frame update
            .add_systems((move_players, increase_frame_system).in_schedule(GGRSSchedule));
    }
}

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
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
    textures: Res<TextureAssets>,
    mut rip: ResMut<RollbackIdProvider>,
    spawn_query: Query<&mut PlayerSpawn>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PlayerComponent);

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    for handle in 0..NUM_PLAYERS {
        let name = format!("Player {}", handle);
        commands.spawn((
            EncounterTracker {
                timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(TILE_SIZE * 2.)),
                    ..Default::default()
                },
                texture: textures.texture_turtle2.clone(),
                transform: Transform {
                    translation: Vec3::new(spawns[handle].pos.x, spawns[handle].pos.y, 900.),
                    ..Default::default()
                },
                ..Default::default()
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

fn exit_to_menu(keys: Res<Input<KeyCode>>, mut state: ResMut<NextState<GameState>>) {
    if keys.any_pressed([KeyCode::Escape, KeyCode::Delete]) {
        state.set(GameState::MenuMain);
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
    mut players: Query<(&mut Transform, &mut Sprite, &mut Player), With<Rollback>>,
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
