use crate::actions::Actions;
use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::network::{GGRSConfig, INPUT_EXIT};
use crate::network::{INPUT_DOWN, INPUT_FIRE, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};
use crate::tilemap::TileCollider;
use crate::TILE_SIZE;
use crate::{GameState, FPS};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_ggrs::GGRSSchedule;
use bevy_ggrs::PlayerInputs;
use bevy_ggrs::RollbackIdProvider;
use bevy_inspector_egui::prelude::*;

const STARTING_SPEED: f32 = 150.;

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

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (move_players, exit_to_menu)
                .chain()
                .in_schedule(GGRSSchedule), // Chain systems in GGRSSchedule for determinate access.
        )
        .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundLocal)))
        .add_system(despawn_players.in_schedule(OnExit(GameState::RoundLocal)))
        .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundOnline)))
        .add_system(camera_follow.in_set(OnUpdate(GameState::RoundLocal)));
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn spawn_players(
    mut commands: Commands,
    characters: Res<CharacterSheet>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PlayerComponent);

    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    commands
        .spawn((
            rip.next(),
            SpriteSheetBundle {
                sprite: sprite.clone(),
                texture_atlas: characters.handle.clone(),
                transform: Transform {
                    translation: Vec3::new(TILE_SIZE * 2., TILE_SIZE * -2., 900.),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .insert(FrameAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frames: characters.turtle_frames.to_vec(),
            current_frame: 0,
        })
        .insert(Player {
            handle: 0,
            speed: STARTING_SPEED,
            active: true,
            just_moved: false,
            exp: 1,
        })
        .insert(Name::new("Player 1"))
        .insert(PlayerComponent);

    // player 2
    // commands
    //     .spawn((
    //         rip.next(),
    //         SpriteSheetBundle {
    //             sprite: sprite.clone(),
    //             texture_atlas: characters.handle.clone(),
    //             transform: Transform {
    //                 translation: Vec3::new(TILE_SIZE * 2., TILE_SIZE * -2., 900.),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         },
    //     ))
    //     .insert(FrameAnimation {
    //         timer: Timer::from_seconds(0.2, TimerMode::Repeating),
    //         frames: characters.turtle_frames.to_vec(),
    //         current_frame: 0,
    //     })
    //     .insert(Player {
    //         handle: 1,
    //         speed: STARTING_SPEED,
    //         active: true,
    //         just_moved: false,
    //         exp: 1,
    //     })
    //     .insert(Name::new("Player 2"));
}

fn exit_to_menu(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut state: ResMut<NextState<GameState>>,
    player_query: Query<&mut Player>,
) {
    for player in player_query.iter() {
        let (input, _) = inputs[player.handle];

        if input & INPUT_EXIT != 0 {
            state.set(GameState::MenuMain);
        }
    }
}

fn despawn_players(mut commands: Commands, query: Query<Entity, With<PlayerComponent>>) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn move_players(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut player_query: Query<(&mut Player, &mut Transform, &mut TextureAtlasSprite)>,
    wall_query: Query<&Transform, (With<TileCollider>, Without<Player>)>,
) {
    for (mut player, mut transform, mut sprite) in player_query.iter_mut() {
        player.just_moved = false;

        if !player.active {
            return;
        }

        let mut direction = Vec2::ZERO;
        let (input, _) = inputs[player.handle];

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
        if !wall_query
            .iter()
            .any(|&transform| wall_collision_check(target, transform.translation))
        {
            if movement.y != 0.0 {
                player.just_moved = true;
            }
            transform.translation = target;
        }

        let target = transform.translation + Vec3::new(movement.x, 0.0, 0.0);
        if !wall_query
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
