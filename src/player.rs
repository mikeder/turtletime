use crate::actions::Actions;
use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::tilemap::TileCollider;
use crate::GameState;
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_inspector_egui::prelude::*;

const STARTING_SPEED: f32 = 150.;

#[derive(Component, Reflect, Default, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    speed: f32,
    active: bool,
    flip: bool,
    just_moved: bool,
    pub exp: usize,
}

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(GameState::Playing)))
            .add_system(move_player.in_set(OnUpdate(GameState::Playing)))
            .add_system(camera_follow.in_set(OnUpdate(GameState::Playing)));
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

fn spawn_player(mut commands: Commands, characters: Res<CharacterSheet>) {
    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));
    sprite.color = Color::AZURE;

    commands
        .spawn(SpriteSheetBundle {
            sprite: sprite,
            texture_atlas: characters.handle.clone(),
            transform: Transform {
                translation: Vec3::new(TILE_SIZE * 2., TILE_SIZE * -2., 900.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FrameAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frames: characters.turtle_frames.to_vec(),
            current_frame: 0,
        })
        .insert(Player {
            flip: false,
            speed: STARTING_SPEED,
            active: true,
            just_moved: false,
            exp: 1,
        })
        .insert(Name::new("Player1"));
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<(&mut Player, &mut Transform, &mut TextureAtlasSprite)>,
    wall_query: Query<&Transform, (With<TileCollider>, Without<Player>)>,
) {
    let (mut player, mut transform, mut sprite) = player_query.single_mut();
    player.just_moved = false;

    if !player.active {
        return;
    }
    if actions.player_movement.is_none() {
        return;
    }
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * player.speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * player.speed * time.delta_seconds(),
        0.,
    );
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

fn wall_collision_check(target_player_pos: Vec3, wall_translation: Vec3) -> bool {
    let collision = collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9), // give player small amount of leeway
        wall_translation,
        Vec2::splat(TILE_SIZE),
    );
    collision.is_some()
}
