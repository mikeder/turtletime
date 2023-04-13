use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::menu::connect::LocalHandle;
use crate::network::{GGRSConfig, INPUT_EXIT};
use crate::network::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};
use crate::tilemap::{EncounterSpawner, PlayerSpawn, TileCollider};
use crate::TILE_SIZE;
use crate::{GameState, FPS};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_ggrs::PlayerInputs;
use bevy_ggrs::Rollback;
use bevy_ggrs::RollbackIdProvider;
use bevy_ggrs::{GGRSSchedule, Session};
use bevy_inspector_egui::prelude::*;

const STARTING_SPEED: f32 = 150.;
const NUM_PLAYERS: usize = 4;

#[derive(Component, Reflect, Default, InspectorOptions)]
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

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EncounterTracker {
    timer: Timer,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_players.in_schedule(GGRSSchedule))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundLocal)))
            .add_system(despawn_players.in_schedule(OnExit(GameState::RoundOnline)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundLocal)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundOnline)))
            .add_system(exit_to_menu);
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

    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    for handle in 0..NUM_PLAYERS {
        let name = format!("Player {}", handle);
        commands
            .spawn((
                SpriteSheetBundle {
                    sprite: sprite.clone(),
                    texture_atlas: characters.handle.clone(),
                    transform: Transform {
                        translation: Vec3::new(spawns[handle].pos.x, spawns[handle].pos.y, 900.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Rollback::new(rip.next_id()),
            ))
            .insert(FrameAnimation {
                timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                frames: characters.turtle_frames.to_vec(),
                current_frame: 0,
            })
            .insert(Player {
                handle,
                speed: STARTING_SPEED,
                active: true,
                just_moved: false,
                exp: 1,
            })
            .insert(Name::new(name))
            .insert(PlayerComponent)
            .insert(EncounterTracker {
                timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            });
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
    mut players: Query<(&mut Transform, &mut TextureAtlasSprite, &mut Player), With<Rollback>>,
) {
    for (mut transform, mut sprite, mut player) in players.iter_mut() {
        debug!("inputs: {:?}", **inputs);

        // reset just_moved each frame
        player.just_moved = false;
        if !player.active {
            return;
        }

        let (input, _) = inputs[player.handle];
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
            return;
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

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct Checksum {
    value: u16,
}

pub fn checksum_players(
    mut query: Query<(&Transform, &mut Checksum), (With<Player>, With<Rollback>)>,
) {
    for (t, mut checksum) in query.iter_mut() {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&t.translation.x.to_le_bytes());
        bytes.extend_from_slice(&t.translation.y.to_le_bytes());
        bytes.extend_from_slice(&t.translation.z.to_le_bytes());

        // naive checksum implementation
        checksum.value = fletcher16(&bytes);
    }
}

/// Computes the fletcher16 checksum, copied from wikipedia: <https://en.wikipedia.org/wiki/Fletcher%27s_checksum>
fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;

    for byte in data {
        sum1 = (sum1 + *byte as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    (sum2 << 8) | sum1
}
