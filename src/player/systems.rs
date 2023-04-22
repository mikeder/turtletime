use super::components::{
    ChiliPepper, EdibleSpawnTimer, Fireball, FireballReady, FireballTimer, Player, RoundComponent,
    Strawberry, CHILI_PEPPER_AMMO_COUNT, CHILI_PEPPER_SIZE, FIREBALL_DAMAGE, FIREBALL_RADIUS,
    PLAYER_SPEED_MAX, PLAYER_SPEED_SPRINT, PLAYER_SPEED_START, STRAWBERRY_SIZE,
};
use super::input::{
    GGRSConfig, PlayerControls, INPUT_DOWN, INPUT_EXIT, INPUT_FIRE, INPUT_LEFT, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_UP,
};
use super::resources::AgreedRandom;

use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::loading::TextureAssets;
use crate::map::tilemap::{EncounterSpawner, PlayerSpawn, TileCollider};
use crate::menu::connect::LocalHandle;
use crate::menu::options::PlayerCount;
use crate::menu::win::MatchData;
use crate::TILE_SIZE;
use crate::{GameState, FPS};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_ggrs::PlayerInputs;
use bevy_ggrs::Rollback;
use bevy_ggrs::RollbackIdProvider;
use bevy_ggrs::Session;
use ggrs::InputStatus;
use rand::Rng;

pub fn camera_follow(
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

pub fn spawn_players(
    mut commands: Commands,
    characters: Res<CharacterSheet>,
    player_count: Res<PlayerCount>,
    mut rip: ResMut<RollbackIdProvider>,
    spawn_query: Query<&mut PlayerSpawn>,
) {
    commands.spawn((Camera2dBundle::default(), RoundComponent));

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    let mut sprite = TextureAtlasSprite::new(characters.turtle_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    for handle in 0..player_count.0 {
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
            PlayerControls::default(),
            rip.next(),
        ));
    }
}

pub fn despawn_players(mut commands: Commands, query: Query<Entity, With<RoundComponent>>) {
    commands.remove_resource::<LocalHandle>();
    commands.remove_resource::<Session<GGRSConfig>>();

    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn apply_inputs(
    mut query: Query<(&mut PlayerControls, &Player)>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    for (mut pc, p) in query.iter_mut() {
        let input = match inputs[p.handle].1 {
            InputStatus::Confirmed => inputs[p.handle].0.input,
            InputStatus::Predicted => inputs[p.handle].0.input,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };

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
        pc.dir = direction.normalize_or_zero();

        if input & INPUT_FIRE != 0 {
            pc.shooting = true;
        } else {
            pc.shooting = false;
        }
        if input & INPUT_SPRINT != 0 {
            pc.sprinting = true;
        } else {
            pc.sprinting = false;
        }

        if input & INPUT_EXIT != 0 {
            pc.exiting = true;
        } else {
            pc.exiting = false;
        }
    }
}

pub fn move_players(
    walls: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    mut players: Query<
        (
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut Player,
            &PlayerControls,
        ),
        With<Rollback>,
    >,
) {
    // loop over all players and apply their inputs to movement
    // do NOT return early because we need to check all players for input/movement
    for (mut transform, mut sprite, mut player, controls) in players.iter_mut() {
        // reset just_moved each frame
        player.just_moved = false;
        if !player.active {
            continue; // don't return, we need to check other players for movement
        }

        if controls.sprinting && player.sprint_ready && player.speed <= PLAYER_SPEED_MAX {
            player.speed += PLAYER_SPEED_SPRINT;
            player.sprint_ammo -= 1;
            if player.sprint_ammo == 0 {
                player.sprint_ready = false;
            }
        } else {
            if player.speed > PLAYER_SPEED_START {
                player.speed -= 1.;
            }
        }

        let movement = (controls.dir * player.speed / FPS as f32).extend(0.);
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

pub fn wall_collision_check(target_player_pos: Vec3, wall_translation: Vec3) -> bool {
    let collision = collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9), // give player small amount of leeway
        wall_translation,
        Vec2::splat(TILE_SIZE),
    );
    collision.is_some()
}

pub fn tick_edible_timer(mut edible_spawn_timer: ResMut<EdibleSpawnTimer>, time: Res<Time>) {
    edible_spawn_timer.chili_pepper_timer.tick(time.delta());
    edible_spawn_timer.strawberry_timer.tick(time.delta());
}

pub fn spawn_strawberry_over_time(
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
pub fn player_ate_strawberry_system(
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

pub fn spawn_chili_pepper_over_time(
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
pub fn player_ate_chili_pepper_system(
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
pub fn reload_fireballs(mut query: Query<(&mut FireballReady, &Player, &PlayerControls)>) {
    for (mut can_fire, player, controls) in query.iter_mut() {
        if !controls.shooting && player.fire_ball_ammo > 0 {
            can_fire.0 = true;
        }
    }
}

pub fn shoot_fireballs(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    images: Res<TextureAssets>,
    mut player_query: Query<(&Transform, &mut Player, &PlayerControls, &mut FireballReady)>,
) {
    for (transform, mut player, controls, mut fireball_ready) in player_query.iter_mut() {
        if !player.active {
            continue; // don't let dead/inactive players continue firing
        }

        if controls.shooting {
            if !fireball_ready.0 || player.fire_ball_ammo == 0 {
                // fireball not ready or player out of ammo
                continue;
            }

            // position fireball slightly away from players position
            let player_pos = transform.translation;
            let pos = player_pos
                + (Vec3::new(controls.dir.x, controls.dir.y, 0.)) * (TILE_SIZE * 1.5)
                + FIREBALL_RADIUS;

            commands.spawn((
                Name::new("Fireball"),
                Fireball {
                    move_dir: controls.dir,
                    shot_by: player.handle,
                    speed: player.speed,
                },
                FireballTimer::default(),
                RoundComponent,
                SpriteBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 1.)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, controls.dir)),
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

pub fn move_fireballs(mut query: Query<(&mut Transform, &Fireball), With<Rollback>>) {
    for (mut transform, fireball) in query.iter_mut() {
        transform.translation += (fireball.move_dir * (fireball.speed * 0.05)).extend(0.);
    }
}

pub fn tick_fireball_timers(mut query: Query<&mut FireballTimer>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.lifetime.tick(time.delta());
    }
}

pub fn despawn_old_fireballs(mut commands: Commands, mut query: Query<(Entity, &FireballTimer)>) {
    for (fireball, timer) in query.iter_mut() {
        if timer.lifetime.finished() {
            commands.entity(fireball).despawn_recursive()
        }
    }
}

pub fn kill_players(
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
            if fireball.shot_by == player.handle {
                continue; // don't allow player to suicide
            };

            let distance = player_transform
                .translation
                .distance(fireball_transform.translation);

            if distance < TILE_SIZE + FIREBALL_RADIUS {
                if player.health >= FIREBALL_DAMAGE {
                    player.health -= FIREBALL_DAMAGE
                } else {
                    animation.timer.set_mode(TimerMode::Once);
                    sprite.flip_y = true;
                    player.active = false;
                }
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn check_win_state(
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
