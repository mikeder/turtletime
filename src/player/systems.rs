use std::time::Duration;

use super::checksum::Checksum;
use super::components::{
    ChiliPepper, EdibleSpawnTimer, Fireball, FireballAmmo, FireballMovement, FireballReady,
    FireballTimer, Lettuce, Player, PlayerFireballText, PlayerHealth, PlayerHealthText, PlayerPoop,
    PlayerPoopTimer, PlayerSpeed, PlayerSpeedBoost, PlayerSpeedBoostText, RoundComponent,
    Strawberry, CHILI_PEPPER_AMMO_COUNT, CHILI_PEPPER_SIZE, FIREBALL_DAMAGE, FIREBALL_RADIUS,
    LETTUCE_HEALTH_GAIN, LETTUCE_SIZE, PLAYER_HEALTH_LOW, PLAYER_HEALTH_MAX, PLAYER_HEALTH_MID,
    PLAYER_SPEED_BOOST, PLAYER_SPEED_MAX, PLAYER_SPEED_START, POOP_DAMAGE, POOP_SIZE,
    STRAWBERRY_AMMO_COUNT, STRAWBERRY_SIZE,
};
use super::input::{
    GGRSConfig, PlayerControls, INPUT_DOWN, INPUT_EXIT, INPUT_FIRE, INPUT_LEFT, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_UP,
};
use super::resources::AgreedRandom;

use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::loading::{FontAssets, TextureAssets};
use crate::map::tilemap::{EncounterSpawner, PlayerSpawn, TileCollider};
use crate::menu::connect::LocalHandle;
use crate::menu::online::PlayerCount;
use crate::menu::win::MatchData;
use crate::{AppState, FIXED_TICK_MS, FPS};
use crate::{GameState, TILE_SIZE};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_ggrs::PlayerInputs;
use bevy_ggrs::Rollback;
use bevy_ggrs::RollbackIdProvider;
use ggrs::InputStatus;
use rand::Rng;

pub fn create_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_handle: Option<Res<LocalHandle>>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    let player_name = format!("Player {}", player_handle);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Auto,
                    right: Val::Px(10.),
                    top: Val::Px(10.),
                    bottom: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Start,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    player_name,
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 50.0,
                        color: Color::GOLD,
                    },
                ),
                ..Default::default()
            });
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::GOLD,
                        },
                    ),
                    ..Default::default()
                })
                .insert(PlayerHealthText);
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::GOLD,
                        },
                    ),
                    ..Default::default()
                })
                .insert(PlayerFireballText);
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::GOLD,
                        },
                    ),
                    ..Default::default()
                })
                .insert(PlayerSpeedBoostText);
        })
        .insert(RoundComponent)
        .insert(Name::new("PlayerUI"));
}

pub fn update_player_health_text(
    player_handle: Option<Res<LocalHandle>>,
    mut text_query: Query<&mut Text, With<PlayerHealthText>>,
    player_query: Query<(&Player, &PlayerHealth), Without<Fireball>>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player, health) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        for mut text in text_query.iter_mut() {
            let val = format!("Health: {}", health.0);
            let mut color = Color::GOLD;
            if health.0 == PLAYER_HEALTH_MAX {
                color = Color::GREEN
            } else if health.0 <= PLAYER_HEALTH_MID && health.0 > PLAYER_HEALTH_LOW {
                color = Color::ORANGE_RED
            } else if health.0 <= PLAYER_HEALTH_LOW {
                color = Color::RED
            }
            text.sections[0].style.color = color;
            text.sections[0].value = val;
        }
    }
}

pub fn update_player_fireball_text(
    player_handle: Option<Res<LocalHandle>>,
    mut text_query: Query<&mut Text, With<PlayerFireballText>>,
    player_query: Query<(&Player, &FireballAmmo), Without<Fireball>>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player, ammo) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        for mut text in text_query.iter_mut() {
            let val = format!("Fireballs: {}", ammo.0);
            text.sections[0].value = val;
        }
    }
}

pub fn update_player_speed_boost_text(
    player_handle: Option<Res<LocalHandle>>,
    mut text_query: Query<&mut Text, With<PlayerSpeedBoostText>>,
    player_query: Query<(&Player, &PlayerSpeedBoost), Without<Fireball>>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player, boost) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        for mut text in text_query.iter_mut() {
            let val = format!("Boost: {}", boost.0);
            text.sections[0].value = val;
        }
    }
}

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
            FireballAmmo::default(),
            FireballReady::default(),
            PlayerControls::default(),
            PlayerHealth::default(),
            PlayerSpeed::default(),
            PlayerSpeedBoost::default(),
            Checksum::default(),
            RoundComponent,
            rip.next(),
        ));
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

        if direction != Vec2::ZERO {
            pc.last_dir = pc.dir
        }

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

pub fn apply_player_sprint(
    mut players: Query<
        (&mut PlayerSpeed, &mut PlayerSpeedBoost, &PlayerControls),
        (With<Player>, With<Rollback>),
    >,
) {
    for (mut speed, mut boost, controls) in players.iter_mut() {
        if controls.sprinting && boost.0 > 0 && speed.0 <= PLAYER_SPEED_MAX {
            speed.0 += PLAYER_SPEED_BOOST;
            boost.0 -= 1;
        } else {
            if speed.0 > PLAYER_SPEED_START {
                speed.0 -= 1;
            }
        }
    }
}

pub fn move_players(
    walls: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    mut query: Query<
        (
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut Player,
            &PlayerSpeed,
            &PlayerControls,
        ),
        With<Rollback>,
    >,
) {
    // collect and sort all players so we move them in a deterministic order
    let mut players = query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(|p| p.2.handle);

    // loop over all players and apply their inputs to movement
    // do NOT return early because we need to check all players for input/movement
    for (mut transform, mut sprite, player, speed, controls) in players {
        if !player.active {
            continue; // don't return, we need to check other players for movement
        }

        let movement = (controls.dir * speed.0 as f32 / FPS as f32).extend(0.);
        if movement.x != 0. {
            if movement.x > 0. {
                // moving to the right
                sprite.flip_x = false
            }
            if movement.x < 0. {
                // moving to the left
                sprite.flip_x = true
            }
        }

        let target = transform.translation + Vec3::new(0.0, movement.y, 0.0);
        if !walls
            .iter()
            .any(|&transform| wall_collision_check(target, transform.translation))
        {
            transform.translation = target;
        }

        let target = transform.translation + Vec3::new(movement.x, 0.0, 0.0);
        if !walls
            .iter()
            .any(|&transform| wall_collision_check(target, transform.translation))
        {
            if movement.x != 0.0 {
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

pub fn player_poops(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    assets: Res<TextureAssets>,
    player_query: Query<(&PlayerControls, &Transform, &PlayerSpeedBoost, &Player)>,
) {
    for (controls, transform, boost, player) in player_query.iter() {
        if controls.sprinting && boost.0 > 0 {
            // player poops
            // position fireball slightly away from players position
            let player_pos = transform.translation;
            let pos = player_pos
                + (Vec3::new(controls.dir.x, controls.dir.y, 0.)) / (TILE_SIZE * 1.5)
                + POOP_SIZE;

            commands.spawn((
                Name::new("PlayerPoop"),
                PlayerPoop {
                    shat_by: player.handle,
                },
                PlayerPoopTimer::default(),
                RoundComponent {},
                SpriteBundle {
                    sprite: Sprite {
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(pos.x, pos.y, 1.0)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, controls.last_dir)),
                    texture: assets.texture_poop.clone(),
                    ..Default::default()
                },
                rip.next(),
            ));
        }
    }
}

// TODO: add sound
pub fn player_stepped_in_poop(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut PlayerHealth, &Player)>,
    poop_query: Query<(Entity, &Transform, &PlayerPoop), With<Rollback>>,
) {
    for (player_transform, mut health, player) in player_query.iter_mut() {
        for (poop_ent, poop_transform, poop) in poop_query.iter() {
            if poop.shat_by == player.handle {
                continue;
            }
            let distance = player_transform
                .translation
                .distance(poop_transform.translation);

            if distance < TILE_SIZE / 2.0 + POOP_SIZE / 2.0 {
                // stepped in shit, take a little damage
                health.0 -= POOP_DAMAGE;
                commands.entity(poop_ent).despawn_recursive();
            }
        }
    }
}

pub fn tick_poop_timers(mut query: Query<(Entity, &mut PlayerPoopTimer)>) {
    // collect and sort all poop timers in play so we tick them in a deterministic order
    let mut poop_timers = query.iter_mut().collect::<Vec<_>>();
    poop_timers.sort_by_key(|e| e.0);

    for (_, mut timer) in poop_timers {
        timer.lifetime.tick(Duration::from_millis(FIXED_TICK_MS));
    }
}

pub fn despawn_old_poops(mut commands: Commands, mut query: Query<(Entity, &PlayerPoopTimer)>) {
    // collect and sort all poops in play so we move them in a deterministic order
    let mut poops = query.iter_mut().collect::<Vec<_>>();
    poops.sort_by_key(|e| e.0);

    for (poop, timer) in poops {
        if timer.lifetime.finished() {
            commands.entity(poop).despawn_recursive()
        }
    }
}

pub fn tick_edible_timer(mut edible_spawn_timer: ResMut<EdibleSpawnTimer>) {
    edible_spawn_timer
        .chili_pepper_timer
        .tick(Duration::from_millis(FIXED_TICK_MS));
    edible_spawn_timer
        .strawberry_timer
        .tick(Duration::from_millis(FIXED_TICK_MS));
    edible_spawn_timer
        .lettuce_timer
        .tick(Duration::from_millis(FIXED_TICK_MS));
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
pub fn player_ate_strawberry_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut PlayerSpeedBoost), With<Player>>,
    strawberry_query: Query<(Entity, &Transform), (With<Strawberry>, With<Rollback>)>,
) {
    let mut strawberries = strawberry_query.iter().collect::<Vec<_>>();
    strawberries.sort_by_key(|e| e.0);

    for (pt, mut p) in player_query.iter_mut() {
        for (s, st) in strawberries.clone() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + STRAWBERRY_SIZE / 2.0 {
                p.0 += STRAWBERRY_AMMO_COUNT;
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
    mut player_query: Query<(&Transform, &mut FireballAmmo), (With<Player>, Without<Fireball>)>,
    pepper_query: Query<(Entity, &Transform), (With<ChiliPepper>, With<Rollback>)>,
) {
    for (pt, mut ammo) in player_query.iter_mut() {
        for (s, st) in pepper_query.iter() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + CHILI_PEPPER_SIZE / 2.0 {
                ammo.0 += CHILI_PEPPER_AMMO_COUNT;
                commands.entity(s).despawn_recursive();
            }
        }
    }
}

pub fn spawn_lettuce_over_time(
    mut commands: Commands,
    mut agreed_seed: ResMut<AgreedRandom>,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.lettuce_timer.finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.gen_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands.spawn((
            Name::new("Lettuce"),
            Lettuce {},
            RoundComponent {},
            SpriteBundle {
                transform: Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
                texture: asset_server.texture_lettuce.clone(),
                ..Default::default()
            },
            rip.next(),
        ));
    }
}

pub fn player_ate_lettuce_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut PlayerHealth), (With<Player>, Without<Fireball>)>,
    lettuce_query: Query<(Entity, &Transform), (With<Lettuce>, With<Rollback>)>,
) {
    for (pt, mut health) in player_query.iter_mut() {
        for (s, st) in lettuce_query.iter() {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + LETTUCE_SIZE / 2.0 {
                if health.0 < PLAYER_HEALTH_MAX {
                    // clamp health game to max health
                    health.0 = (health.0 + LETTUCE_HEALTH_GAIN).clamp(0, PLAYER_HEALTH_MAX);
                }
                commands.entity(s).despawn_recursive();
            }
        }
    }
}

// reload_fireball prevents the player from continuously shooting fireballs by holding INPUT_FIRE
pub fn reload_fireballs(
    mut query: Query<(Entity, &mut FireballReady, &FireballAmmo, &PlayerControls)>,
) {
    let mut players = query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    for (_, mut ready, ammo, controls) in players {
        if !controls.shooting && ammo.0 > 0 && ready.0 == false {
            ready.0 = true;
        }
    }
}

pub fn shoot_fireballs(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    images: Res<TextureAssets>,
    mut query: Query<(
        Entity,
        &Transform,
        &mut FireballAmmo,
        &mut FireballReady,
        &PlayerControls,
        &PlayerSpeed,
        &Player,
    )>,
) {
    // collect and sort all players in play so we move them in a deterministic order
    let mut players = query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(
        |t: &(
            Entity,
            &Transform,
            Mut<FireballAmmo>,
            Mut<FireballReady>,
            &PlayerControls,
            &PlayerSpeed,
            &Player,
        )| t.0,
    );

    for (_, transform, mut ammo, mut ready, controls, speed, player) in players {
        if !player.active {
            continue; // prevent dead players from shooting
        }

        if controls.shooting {
            if !ready.0 || ammo.0 == 0 {
                // fireball not ready or player out of ammo
                continue;
            }

            // position fireball slightly away from players position
            let player_pos = transform.translation;
            let pos = player_pos
                + (Vec3::new(controls.dir.x, controls.dir.y, 0.)) * (TILE_SIZE * 1.5)
                + FIREBALL_RADIUS;

            debug!(
                "Spawning fireball by {:?} ammo {:?}, ready {:?}",
                player, ammo.0, ready.0
            );

            let fireball_id = commands
                .spawn((
                    Name::new("Fireball"),
                    Fireball {
                        shot_by: player.handle,
                    },
                    FireballMovement {
                        speed: speed.0 as f32,
                        dir: controls.last_dir,
                    },
                    FireballTimer::default(),
                    RoundComponent,
                    SpriteBundle {
                        transform: Transform::from_xyz(pos.x, pos.y, 1.)
                            .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, controls.last_dir)),
                        texture: images.texture_fireball.clone(),
                        ..default()
                    },
                    rip.next(),
                ))
                .id();

            ammo.0 -= 1;
            ready.0 = false;

            debug!(
                "Spawned fireball {:?} by {:?} ammo {:?}, ready {:?}",
                fireball_id, player, ammo.0, ready.0
            )
        }
    }
}

pub fn move_fireballs(
    mut query: Query<(Entity, &mut Transform, &FireballMovement), (With<Fireball>, With<Rollback>)>,
) {
    // collect and sort all fireballs in play so we move them in a deterministic order
    let mut fireballs = query.iter_mut().collect::<Vec<_>>();
    fireballs.sort_by_key(|t| t.0);

    for (_, mut transform, movement) in fireballs {
        transform.translation += (movement.dir * (movement.speed * 0.05)).extend(0.);
    }
}

pub fn tick_fireball_timers(mut query: Query<(Entity, &mut FireballTimer)>) {
    // collect and sort all timers in play so we tick them in a deterministic order
    let mut timers = query.iter_mut().collect::<Vec<_>>();
    timers.sort_by_key(|t| t.0);

    for (_, mut timer) in timers {
        timer.lifetime.tick(Duration::from_millis(FIXED_TICK_MS));
    }
}

pub fn despawn_old_fireballs(mut commands: Commands, mut query: Query<(Entity, &FireballTimer)>) {
    // collect and sort all fireballs in play so we despawn them in a deterministic order
    let mut fireballs = query.iter_mut().collect::<Vec<_>>();
    fireballs.sort_by_key(|e| e.0);

    for (fireball, timer) in fireballs {
        if timer.lifetime.finished() {
            debug!("Despawning old fireball {:?}", fireball);
            commands.entity(fireball).despawn_recursive()
        }
    }
}

pub fn fireball_damage_players(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut PlayerHealth, &Transform, &Player),
        (With<Rollback>, Without<Fireball>),
    >,
    fireball_query: Query<(Entity, &Transform, &Fireball), With<Rollback>>,
) {
    // collect and sort all players and fireballs in play so we damage players in a deterministic order
    let mut players = player_query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    let mut fireballs = fireball_query.iter().collect::<Vec<_>>();
    fireballs.sort_by_key(|e| e.0);

    for (_, mut health, transform, player) in players {
        for (entity, fireball_transform, fireball) in fireballs.clone() {
            if !player.active {
                continue; // don't continue to damage dead players
            }
            if fireball.shot_by == player.handle {
                continue; // don't allow player to suicide
            };

            let distance = transform
                .translation
                .distance(fireball_transform.translation);

            if distance < TILE_SIZE + FIREBALL_RADIUS {
                health.0 -= FIREBALL_DAMAGE;
                commands.entity(entity).despawn_recursive(); // despawn fireball
                debug!(
                    "Fireball {:?} hit player, new health {:?}",
                    entity, health.0
                )
            }
        }
    }
}

pub fn kill_players(
    mut player_query: Query<
        (
            Entity,
            &mut Player,
            &PlayerHealth,
            &mut FrameAnimation,
            &mut TextureAtlasSprite,
        ),
        (With<Player>, Without<Fireball>),
    >,
) {
    // collect and sort all players in play so we kill players in a deterministic order
    let mut players = player_query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    for (_, mut player, health, mut animation, mut sprite) in players {
        if health.0 <= 0 {
            animation.timer.set_mode(TimerMode::Once);
            sprite.flip_y = true;
            player.active = false;
        }
    }
}

pub fn check_win_state(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_handle: Option<Res<LocalHandle>>,
    player_query: Query<(Entity, &Player), Without<Fireball>>,
) {
    let local_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    let mut players = player_query.iter().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    let mut remaning_active = vec![];
    for (_, player) in players {
        if player.active {
            remaning_active.push(player);
        }
    }
    if remaning_active.len() == 1 {
        if remaning_active[0].handle == local_handle {
            commands.insert_resource(MatchData {
                result: format!("You Win!"),
            })
        } else {
            commands.insert_resource(MatchData {
                result: format!("You Lost!"),
            })
        }
        app_state.set(AppState::Win);
        game_state.set(GameState::Paused);
    }
}