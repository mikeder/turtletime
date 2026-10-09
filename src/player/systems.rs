use std::time::Duration;

use super::checksum::Checksum;
use super::components::{
    ConnectionNoticeText, ConnectionNoticeUI, Edible, EdibleSpawnTimer, Fireball, FireballAmmo,
    FireballMovement, FireballReady, FireballTimer, Player, PlayerFireballText, PlayerHealth,
    PlayerHealthBar, PlayerHealthBarPart, PlayerHealthText, PlayerPoop, PlayerPoopTimer,
    PlayerSpeed, PlayerSpeedBoost, PlayerSpeedBoostText, RoundComponent, SpectateBtn, SpectateText,
    SpectateUI, SynchronizingText, CHILI_PEPPER_AMMO_COUNT, CHILI_PEPPER_SIZE, FIREBALL_DAMAGE,
    FIREBALL_RADIUS, LETTUCE_HEALTH_GAIN, LETTUCE_SIZE, PLAYER_HEALTH_LOW, PLAYER_HEALTH_MAX,
    PLAYER_HEALTH_MID, PLAYER_SPEED_BOOST, PLAYER_SPEED_BOOST_MAX, PLAYER_SPEED_MAX,
    PLAYER_SPEED_START, POOP_DAMAGE, POOP_ENTITIES_MAX, POOP_SIZE, STRAWBERRY_AMMO_COUNT,
    STRAWBERRY_SIZE,
};
use super::input::{
    GGRSConfig, PlayerControls, INPUT_DOWN, INPUT_EXIT, INPUT_FIRE, INPUT_LEFT, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_UP,
};
use super::resources::{
    AgreedRandom, Connections, HealthBarsAdded, PreviousWinner, Spectating, DISCONNECT_NOTICE_SECS,
};

use crate::audio::{FadedLoopSound, RollbackSound, RollbackSoundBundle};
use crate::graphics::{CharacterSheet, FrameAnimation};
use crate::loading::{AudioAssets, FontAssets, TextureAssets};
use crate::map::tilemap::{EncounterSpawner, PlayerSpawn, TileCollider};
use crate::menu::connect::{player_label, LocalHandle, PlayerCharacters, PlayerNames};
use crate::menu::online::PlayerCount;
use crate::menu::ui;
use crate::menu::win::MatchData;
use crate::player::components::Expired;
use crate::player::resources::PlayersReady;
use crate::{AppState, FIXED_TICK_MS, FPS, HEALTH_BAR_Y_OFFSET};
use crate::{GameState, TILE_SIZE};
use bevy::color::palettes::css::{RED, TOMATO};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy_ggrs::Rollback;
use bevy_ggrs::Session;
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs, RollbackFrameCount};
use ggrs::{GgrsEvent, InputStatus, SessionState};
use rand::RngExt;

pub fn create_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_handle: Option<Res<LocalHandle>>,
    player_names: Option<Res<PlayerNames>>,
) {
    trace!("create_ui");

    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    let player_name = player_label(player_names.as_deref(), player_handle);
    let font = &font_assets.fira_sans;

    // root node, a shaded panel so the text stays readable on top of the map
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.),
                top: Val::Px(12.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(2.),
                padding: UiRect::axes(Val::Px(16.), Val::Px(10.)),
                border_radius: BorderRadius::all(Val::Px(8.)),
                ..Default::default()
            },
            BackgroundColor(ui::SHADE),
            children![
                ui::text(font, player_name, ui::BUTTON_SIZE, ui::CREAM),
                (
                    ui::text(font, "", ui::BODY_SIZE, ui::CREAM),
                    PlayerHealthText,
                ),
                (
                    ui::text(font, "", ui::BODY_SIZE, ui::CREAM),
                    PlayerFireballText,
                ),
                (
                    ui::text(font, "", ui::BODY_SIZE, ui::CREAM),
                    PlayerSpeedBoostText,
                ),
            ],
        ))
        .insert(RoundComponent)
        .insert(Name::new("PlayerUI"));

    // shown while the session waits for every player, so it is clear why nothing moves yet
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                right: Val::Px(0.),
                top: Val::Percent(30.),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            Visibility::Hidden,
            SynchronizingText,
            children![(
                Node {
                    padding: UiRect::axes(Val::Px(24.), Val::Px(12.)),
                    border_radius: BorderRadius::all(Val::Px(8.)),
                    ..Default::default()
                },
                BackgroundColor(ui::SHADE),
                children![ui::heading(font, "Synchronizing...")],
            )],
        ))
        .insert(RoundComponent)
        .insert(Name::new("SynchronizingUI"));

    // shown once the local player has died and watches the others
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                right: Val::Px(0.),
                bottom: Val::Px(24.),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            Visibility::Hidden,
            SpectateUI,
            children![(
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.),
                    padding: UiRect::axes(Val::Px(16.), Val::Px(10.)),
                    border_radius: BorderRadius::all(Val::Px(8.)),
                    ..Default::default()
                },
                BackgroundColor(ui::SHADE),
                children![
                    (ui::small_button(font, "<"), SpectateBtn::Previous),
                    (ui::body(font, ""), SpectateText),
                    (ui::small_button(font, ">"), SpectateBtn::Next),
                ],
            )],
        ))
        .insert(RoundComponent)
        .insert(Name::new("SpectateUI"));

    // shown while another player has connection trouble or just disconnected
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.),
                top: Val::Px(12.),
                padding: UiRect::axes(Val::Px(16.), Val::Px(10.)),
                border_radius: BorderRadius::all(Val::Px(8.)),
                ..Default::default()
            },
            BackgroundColor(ui::SHADE),
            Visibility::Hidden,
            ConnectionNoticeUI,
            children![(
                ui::text(font, "", ui::BODY_SIZE, ui::CHILI),
                ConnectionNoticeText,
            )],
        ))
        .insert(RoundComponent)
        .insert(Name::new("ConnectionNoticeUI"));
}

/// Keeps track of the connections to the other players. The events are gone
/// once they are read, so this is the only place that reads them.
pub fn read_session_events(
    session: Option<ResMut<Session<GGRSConfig>>>,
    mut connections: ResMut<Connections>,
) {
    let Some(mut session) = session else {
        return;
    };
    let Session::P2P(session) = session.as_mut() else {
        return;
    };

    for event in session.events().collect::<Vec<_>>() {
        info!("GGRS Event: {:?}", event);
        match event {
            GgrsEvent::NetworkInterrupted { addr, .. } => {
                for handle in session.handles_by_address(addr) {
                    if !connections.disconnected.contains_key(&handle) {
                        connections.interrupted.insert(handle);
                    }
                }
            }
            GgrsEvent::NetworkResumed { addr } => {
                for handle in session.handles_by_address(addr) {
                    connections.interrupted.remove(&handle);
                }
            }
            GgrsEvent::Disconnected { addr } => {
                for handle in session.handles_by_address(addr) {
                    connections.interrupted.remove(&handle);
                    connections.disconnected.entry(handle).or_insert_with(|| {
                        Timer::from_seconds(DISCONNECT_NOTICE_SECS, TimerMode::Once)
                    });
                }
            }
            _ => (),
        }
    }
}

pub fn update_connection_notices(
    time: Res<Time>,
    player_names: Option<Res<PlayerNames>>,
    mut connections: ResMut<Connections>,
    mut ui_query: Query<&mut Visibility, With<ConnectionNoticeUI>>,
    mut text_query: Query<&mut Text, With<ConnectionNoticeText>>,
) {
    if connections.interrupted.is_empty() && connections.disconnected.is_empty() {
        return; // nothing to tell, and nothing was shown that has to go
    }

    let names = player_names.as_deref();
    let mut lines = Vec::new();
    for handle in &connections.interrupted {
        lines.push(format!(
            "{} is reconnecting...",
            player_label(names, *handle)
        ));
    }
    for (handle, timer) in connections.disconnected.iter_mut() {
        // who disconnected is kept for the win screen, only the notice goes away
        if !timer.tick(time.delta()).is_finished() {
            lines.push(format!("{} disconnected", player_label(names, *handle)));
        }
    }

    let wanted = if lines.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in ui_query.iter_mut() {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    let lines = lines.join("\n");
    for mut text in text_query.iter_mut() {
        if text.0 != lines {
            text.0 = lines.clone();
        }
    }
}

/// A dead player watches the players that are still alive, the left and right
/// arrow keys or the buttons next to the name switch between them.
#[allow(clippy::too_many_arguments)]
pub fn update_spectating(
    keys: Res<ButtonInput<KeyCode>>,
    local_handle: Option<Res<LocalHandle>>,
    player_names: Option<Res<PlayerNames>>,
    mut spectating: ResMut<Spectating>,
    player_query: Query<&Player>,
    buttons: Query<(&Interaction, &SpectateBtn), Changed<Interaction>>,
    mut ui_query: Query<&mut Visibility, With<SpectateUI>>,
    mut text_query: Query<&mut Text, With<SpectateText>>,
) {
    let Some(local_handle) = local_handle else {
        return; // Session hasn't started yet
    };

    let mut alive = player_query
        .iter()
        .filter(|player| player.active)
        .map(|player| player.handle)
        .collect::<Vec<_>>();
    alive.sort();

    // this is checked every frame, a rollback can bring any player back to life
    let target = if alive.is_empty() || alive.contains(&local_handle.0) {
        None
    } else {
        let mut step = 0;
        if keys.just_pressed(KeyCode::ArrowLeft) {
            step -= 1;
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            step += 1;
        }
        for (interaction, btn) in buttons.iter() {
            if let Interaction::Pressed = *interaction {
                match btn {
                    SpectateBtn::Previous => step -= 1,
                    SpectateBtn::Next => step += 1,
                }
            }
        }

        let watched = spectating
            .0
            .and_then(|h| alive.iter().position(|a| *a == h));
        Some(match watched {
            Some(i) => alive[(i as i32 + step).rem_euclid(alive.len() as i32) as usize],
            // nobody picked yet, or the player that was watched died as well
            None => alive[0],
        })
    };
    if spectating.0 != target {
        spectating.0 = target;
    }

    let wanted = if target.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in ui_query.iter_mut() {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    if let Some(handle) = target {
        let watching = format!("Watching {}", player_label(player_names.as_deref(), handle));
        for mut text in text_query.iter_mut() {
            if text.0 != watching {
                text.0 = watching.clone();
            }
        }
    }
}

pub fn update_synchronizing_text(
    session: Option<Res<Session<GGRSConfig>>>,
    mut query: Query<&mut Visibility, With<SynchronizingText>>,
) {
    // only online sessions have to synchronize with other players before they run
    let synchronizing = match session.as_deref() {
        Some(Session::P2P(s)) => s.current_state() == SessionState::Synchronizing,
        _ => false,
    };
    let wanted = if synchronizing {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in query.iter_mut() {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

pub fn update_player_health_text(
    player_handle: Option<Res<LocalHandle>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<PlayerHealthText>>,
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

        for (mut text, mut text_color) in text_query.iter_mut() {
            let val = format!("Health: {}", health.0);
            let mut color = ui::CREAM;
            if health.0 == PLAYER_HEALTH_MAX {
                color = ui::LETTUCE
            } else if health.0 <= PLAYER_HEALTH_MID && health.0 > PLAYER_HEALTH_LOW {
                color = ui::CHILI
            } else if health.0 <= PLAYER_HEALTH_LOW {
                color = TOMATO.into()
            }
            text_color.0 = color;
            text.0 = val;
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
            text.0 = val;
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
            text.0 = val;
        }
    }
}

pub fn camera_follow(
    player_handle: Option<Res<LocalHandle>>,
    spectating: Res<Spectating>,
    player_query: Query<(&Transform, &Player), Without<Fireball>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };
    // a dead player follows the player they are watching
    let player_handle = spectating.0.unwrap_or(player_handle);

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
    sounds: Res<AudioAssets>,
    characters: Res<CharacterSheet>,
    player_count: Res<PlayerCount>,
    spawn_query: Query<&mut PlayerSpawn>,
    local_handle: Option<Res<LocalHandle>>,
    player_characters: Option<Res<PlayerCharacters>>,
) {
    trace!("spawn_players");

    let local_handle = match local_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    // find all the spawn points on the map
    let spawns: Vec<&PlayerSpawn> = spawn_query.iter().collect();

    for handle in 0..player_count.0 {
        let name = format!("Player {}", handle);
        let character = player_characters
            .as_deref()
            .and_then(|characters| characters.0.get(handle).copied())
            .unwrap_or_default();
        let mut sprite = Sprite::from_atlas_image(
            characters.turtle_image(character),
            TextureAtlas {
                layout: characters.turtle_layout.clone(),
                index: characters.turtle_frames[0],
            },
        );
        sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));
        let player_id = commands
            .spawn((
                Name::new(name),
                sprite,
                Transform {
                    translation: Vec3::new(spawns[handle].pos.x, spawns[handle].pos.y, 1.),
                    ..Default::default()
                },
                FrameAnimation {
                    timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                    frames: characters.turtle_frames.to_vec(),
                    current_frame: 0,
                    playing: false,
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
            ))
            .add_rollback()
            .id();

        if handle == local_handle {
            // add walking sound component to local player only
            commands.entity(player_id).insert(FadedLoopSound {
                audio_instance: None,
                clip: sounds.walking.clone(),
                fade_in: 0.1,
                fade_out: 0.1,
                should_play: false,
            });
        }
    }
    commands.insert_resource(PlayersReady);
}

pub fn set_walking_sound(mut query: Query<(&mut FadedLoopSound, &PlayerControls)>) {
    for (mut sound, controls) in query.iter_mut() {
        if controls.dir == Vec2::ZERO {
            sound.should_play = false
        } else {
            sound.should_play = true
        }
    }
}

/// Turtles only move their legs while they walk. The animation is not part of the
/// rollback state, so this follows the controls instead of being set in a rollback system.
pub fn animate_walking_players(mut query: Query<(&Player, &PlayerControls, &mut FrameAnimation)>) {
    for (player, controls, mut animation) in query.iter_mut() {
        let walking = player.active && controls.dir != Vec2::ZERO;
        if animation.playing != walking {
            animation.playing = walking;
        }
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
            pc.last_dir = pc.dir;
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
            &mut Sprite,
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
    // strict AABB overlap, touching edges do not count as a collision
    let player = Rect::from_center_size(
        target_player_pos.truncate(),
        Vec2::splat(TILE_SIZE * 0.9), // give player small amount of leeway
    );
    let wall = Rect::from_center_size(wall_translation.truncate(), Vec2::splat(TILE_SIZE));
    !player.intersect(wall).is_empty()
}

pub fn player_poops(
    mut commands: Commands,
    frame: Res<RollbackFrameCount>,
    sounds: Res<AudioAssets>,
    textures: Res<TextureAssets>,
    player_query: Query<(&PlayerControls, &Transform, &PlayerSpeedBoost, &Player)>,
    poop_query: Query<&PlayerPoop>,
) {
    let poop_count = poop_query.iter().len();
    for (controls, transform, boost, player) in player_query.iter() {
        if controls.sprinting && boost.0 > 0 && poop_count < POOP_ENTITIES_MAX {
            let player_pos = transform.translation;
            let pos = player_pos
                + (Vec3::new(controls.dir.x, controls.dir.y, 0.)) / (TILE_SIZE * 1.5)
                + POOP_SIZE;

            let poop_instance = commands
                .spawn((
                    Name::new("PlayerPoop"),
                    PlayerPoop {
                        shat_by: player.handle,
                    },
                    PlayerPoopTimer::default(),
                    RoundComponent,
                    Sprite::from_image(textures.texture_poop.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, controls.last_dir)),
                ))
                .add_rollback()
                .id();

            // spawn desired audio clip
            commands
                .spawn(RollbackSoundBundle {
                    sound: RollbackSound {
                        clip: sounds.sprinting.clone(),
                        start_frame: frame.0,
                        sub_key: poop_instance.index_u32(),
                    },
                })
                .add_rollback();
        }
    }
}

// TODO: add sound
pub fn player_stepped_in_poop(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut PlayerHealth, &Player)>,
    poop_query: Query<(Entity, &Transform, &PlayerPoop), (With<Rollback>, Without<Expired>)>,
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
                commands.entity(poop_ent).insert(Expired);
            }
        }
    }
}

pub fn tick_poop_timers(mut query: Query<(Entity, &mut PlayerPoopTimer), Without<Expired>>) {
    // collect and sort all poop timers in play so we tick them in a deterministic order
    let mut poop_timers = query.iter_mut().collect::<Vec<_>>();
    poop_timers.sort_by_key(|e| e.0);

    for (_, mut timer) in poop_timers {
        timer.lifetime.tick(Duration::from_millis(FIXED_TICK_MS));
    }
}

pub fn despawn_old_poops(
    mut commands: Commands,
    mut query: Query<(Entity, &PlayerPoopTimer), Without<Expired>>,
) {
    trace!("despawn_old_poops");

    // collect and sort all poops in play so we move them in a deterministic order
    let mut poops = query.iter_mut().collect::<Vec<_>>();
    poops.sort_by_key(|e| e.0);

    for (poop, timer) in poops {
        if timer.lifetime.is_finished() {
            commands.entity(poop).insert(Expired);
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
    mut agreed_seed: ResMut<AgreedRandom>, // TODO: why does this need mut?
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.strawberry_timer.is_finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.random_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands
            .spawn((
                Name::new("Strawberry"),
                Edible::Strawberry,
                RoundComponent,
                Sprite::from_image(asset_server.texture_strawberry.clone()),
                Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
            ))
            .add_rollback();
    }
}

pub fn spawn_strawberry_on_player_spawn_points(
    mut commands: Commands,
    asset_server: Res<TextureAssets>,
    player_spawns: Query<&mut PlayerSpawn>,
    timer: Res<EdibleSpawnTimer>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    edible_query: Query<(Entity, &Edible, &Transform), (With<Edible>, Without<Expired>)>,
) {
    if !timer.strawberry_timer.is_finished() {
        return;
    }

    // find all strawberries on the map and sort them into a vec
    let mut strawberries = edible_query
        .iter()
        .filter(|x| match x.1 {
            Edible::Strawberry => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    strawberries.sort_by_key(|e| e.0);

    // collect and sort all player locations
    let mut players = player_query.iter().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    // select a player spawn point, check if there is already a strawberry or player there
    // if not, spawn a strawberry
    for ps in player_spawns.iter() {
        let mut free_space = true;
        let spawn_pos = ps.pos;
        for s in strawberries.iter() {
            if s.2.translation.distance(spawn_pos) < TILE_SIZE {
                free_space = false;
                break;
            }
        }
        if !free_space {
            continue;
        }
        for p in players.iter() {
            if p.1.translation.distance(spawn_pos) < TILE_SIZE {
                free_space = false;
                break;
            }
        }
        if free_space {
            // spawn a strawberry
            commands
                .spawn((
                    Name::new("Strawberry"),
                    Edible::Strawberry,
                    RoundComponent,
                    Sprite::from_image(asset_server.texture_strawberry.clone()),
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.0),
                ))
                .add_rollback();
        }
    }
}

// TODO: add sound
pub fn player_ate_strawberry_system(
    mut commands: Commands,
    frame: Res<RollbackFrameCount>,
    sounds: Res<AudioAssets>,
    mut player_query: Query<(&Transform, &mut PlayerSpeedBoost), With<Player>>,
    edible_query: Query<(Entity, &Edible, &Transform), Without<Expired>>,
) {
    let mut strawberries = edible_query
        .iter()
        .filter(|x| match x.1 {
            Edible::Strawberry => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    strawberries.sort_by_key(|e| e.0);

    for (pt, mut p) in player_query.iter_mut() {
        for (s, _, st) in &strawberries {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + STRAWBERRY_SIZE / 2.0 {
                p.0 = (p.0 + STRAWBERRY_AMMO_COUNT).clamp(0, PLAYER_SPEED_BOOST_MAX);
                commands.entity(*s).insert(Expired);

                // spawn desired audio clip
                commands
                    .spawn(RollbackSoundBundle {
                        sound: RollbackSound {
                            clip: sounds.pickup.clone(),
                            start_frame: frame.0,
                            sub_key: s.index_u32(),
                        },
                    })
                    .add_rollback();
            }
        }
    }
}

pub fn spawn_chili_pepper_over_time(
    mut commands: Commands,
    mut agreed_seed: ResMut<AgreedRandom>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.chili_pepper_timer.is_finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.random_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands
            .spawn((
                Name::new("ChiliPepper"),
                Edible::ChiliPepper,
                RoundComponent,
                Sprite {
                    image: asset_server.texture_chili_pepper.clone(),
                    custom_size: Some(Vec2::splat(CHILI_PEPPER_SIZE * 1.5)),
                    ..Default::default()
                },
                Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
            ))
            .add_rollback();
    }
}

// TODO: add sound
pub fn player_ate_chili_pepper_system(
    mut commands: Commands,
    frame: Res<RollbackFrameCount>,
    sounds: Res<AudioAssets>,
    mut player_query: Query<(&Transform, &mut FireballAmmo), (With<Player>, Without<Fireball>)>,
    edible_query: Query<(Entity, &Edible, &Transform), Without<Expired>>,
) {
    let mut peppers = edible_query
        .iter()
        .filter(|x| match x.1 {
            Edible::ChiliPepper => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    peppers.sort_by_key(|e| e.0);

    for (pt, mut ammo) in player_query.iter_mut() {
        for (s, _, st) in &peppers {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + CHILI_PEPPER_SIZE / 2.0 {
                ammo.0 += CHILI_PEPPER_AMMO_COUNT;
                commands.entity(*s).insert(Expired);

                // spawn desired audio clip
                commands
                    .spawn(RollbackSoundBundle {
                        sound: RollbackSound {
                            clip: sounds.pickup.clone(),
                            start_frame: frame.0,
                            sub_key: s.index_u32(),
                        },
                    })
                    .add_rollback();
            }
        }
    }
}

pub fn spawn_lettuce_over_time(
    mut commands: Commands,
    mut agreed_seed: ResMut<AgreedRandom>,
    asset_server: Res<TextureAssets>,
    timer: Res<EdibleSpawnTimer>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    if timer.lettuce_timer.is_finished() {
        let spawn_area: Vec<&Transform> = spawner_query.iter().collect();

        let idx = agreed_seed.rng.random_range(0..spawn_area.len());
        let pos = spawn_area[idx];

        commands
            .spawn((
                Name::new("Lettuce"),
                Edible::Lettuce,
                RoundComponent,
                Sprite::from_image(asset_server.texture_lettuce.clone()),
                Transform::from_xyz(pos.translation.x, pos.translation.y, 1.0),
            ))
            .add_rollback();
    }
}

pub fn player_ate_lettuce_system(
    mut commands: Commands,
    frame: Res<RollbackFrameCount>,
    sounds: Res<AudioAssets>,
    mut player_query: Query<(&Transform, &mut PlayerHealth), (With<Player>, Without<Fireball>)>,
    edible_query: Query<(Entity, &Edible, &Transform), Without<Expired>>,
) {
    let mut lettuce = edible_query
        .iter()
        .filter(|x| match x.1 {
            Edible::Lettuce => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    lettuce.sort_by_key(|e| e.0);

    for (pt, mut health) in player_query.iter_mut() {
        for (s, _, st) in &lettuce {
            let distance = pt.translation.distance(st.translation);

            if distance < TILE_SIZE / 2.0 + LETTUCE_SIZE / 2.0 {
                if health.0 < PLAYER_HEALTH_MAX {
                    // clamp health game to max health
                    health.0 = (health.0 + LETTUCE_HEALTH_GAIN).clamp(0, PLAYER_HEALTH_MAX);
                }
                commands.entity(*s).insert(Expired);

                // spawn desired audio clip
                commands
                    .spawn(RollbackSoundBundle {
                        sound: RollbackSound {
                            clip: sounds.pickup.clone(),
                            start_frame: frame.0,
                            sub_key: s.index_u32(),
                        },
                    })
                    .add_rollback();
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
    images: Res<TextureAssets>,
    sounds: Res<AudioAssets>,
    frame: Res<RollbackFrameCount>,

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
                    Sprite::from_image(images.texture_fireball.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, controls.last_dir)),
                ))
                .add_rollback()
                .id();

            ammo.0 -= 1;
            ready.0 = false;

            debug!(
                "Spawned fireball {:?} by {:?} ammo {:?}, ready {:?}",
                fireball_id, player, ammo.0, ready.0
            );

            // spawn desired audio clip
            commands
                .spawn(RollbackSoundBundle {
                    sound: RollbackSound {
                        clip: sounds.fireball_shot.clone(),
                        start_frame: frame.0,
                        sub_key: fireball_id.index_u32(),
                    },
                })
                .add_rollback();
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

pub fn tick_fireball_timers(mut query: Query<(Entity, &mut FireballTimer), Without<Expired>>) {
    // collect and sort all timers in play so we tick them in a deterministic order
    let mut timers = query.iter_mut().collect::<Vec<_>>();
    timers.sort_by_key(|t| t.0);

    for (_, mut timer) in timers {
        timer.lifetime.tick(Duration::from_millis(FIXED_TICK_MS));
    }
}

pub fn despawn_old_fireballs(
    mut commands: Commands,
    mut query: Query<(Entity, &FireballTimer), Without<Expired>>,
) {
    trace!("despawn_old_fireballs");

    // collect and sort all fireballs in play so we despawn them in a deterministic order
    let mut fireballs = query.iter_mut().collect::<Vec<_>>();
    fireballs.sort_by_key(|e| e.0);

    for (fireball, timer) in fireballs {
        if timer.lifetime.is_finished() {
            debug!("Despawning old fireball {:?}", fireball);
            commands.entity(fireball).insert(Expired);
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
                commands.entity(entity).insert(Expired); // despawn fireball
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
            &mut Sprite,
        ),
        (With<Player>, Without<Fireball>),
    >,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    // collect and sort all players in play so we kill players in a deterministic order
    let mut players = player_query.iter_mut().collect::<Vec<_>>();
    players.sort_by_key(|e| e.0);

    for (_, mut player, health, mut animation, mut sprite) in players {
        // a player that disconnected is out of the round, the session agrees with every
        // remaining player on the frame that happened, so this is safe to roll back
        let disconnected = inputs[player.handle].1 == InputStatus::Disconnected;
        if health.0 <= 0 || disconnected {
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
    state: Res<State<AppState>>,
    mut previous_winner: ResMut<PreviousWinner>,
    player_handle: Option<Res<LocalHandle>>,
    player_names: Option<Res<PlayerNames>>,
    connections: Res<Connections>,
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
        let winner = remaning_active[0].handle;
        *previous_winner = if *state.get() != AppState::RoundOnline {
            PreviousWinner::Handle(winner)
        } else if winner == local_handle {
            PreviousWinner::Me
        } else {
            PreviousWinner::None
        };
        commands.insert_resource(MatchData {
            winner: player_label(player_names.as_deref(), winner),
            won: winner == local_handle,
            disconnected: connections
                .disconnected
                .keys()
                .map(|handle| player_label(player_names.as_deref(), *handle))
                .collect(),
        });
        app_state.set(AppState::Win);
        game_state.set(GameState::Paused);
    }
}

pub fn add_player_health_bars(
    mut commands: Commands,
    query: Query<Entity, With<PlayerHealth>>,
    done: Option<Res<HealthBarsAdded>>,
) {
    if done.is_some() {
        return; // hack, already done
    }

    trace!("add_player_health_bars");

    for health_entity in query.iter() {
        trace!("Adding health bar");

        commands.entity(health_entity).with_children(|cb| {
            cb.spawn((
                // black background
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(PLAYER_HEALTH_MAX as f32, TILE_SIZE / 4.)),
                    ..default()
                },
                Transform::from_xyz(0., HEALTH_BAR_Y_OFFSET, 0.),
                PlayerHealthBarPart,
            ));
            cb.spawn((
                // red overlay
                Sprite {
                    color: RED.into(),
                    custom_size: Some(Vec2::new(PLAYER_HEALTH_MAX as f32, TILE_SIZE / 8.)),
                    ..default()
                },
                Transform::from_xyz(0., HEALTH_BAR_Y_OFFSET, 0.2),
                PlayerHealthBarPart,
            ))
            // insert component used to track player health in update system
            .insert(PlayerHealthBar { health_entity });
        });
    }
    trace!("insert HealthBarsAdded");
    commands.insert_resource(HealthBarsAdded)
}

pub fn update_health_bars(
    mut health_bars: Query<(Entity, &PlayerHealthBar, &mut Transform)>,
    health_entities: Query<&PlayerHealth>,
) {
    let mut bars = health_bars.iter_mut().collect::<Vec<_>>();
    bars.sort_by_key(|e| e.0);

    for (e, health_bar, mut transform) in bars {
        trace!("updating health bar for {:?} -> {:?}", e, transform);
        let player_health = health_entities.get(health_bar.health_entity).unwrap();
        let health_percent = player_health.decimal();
        let half = PLAYER_HEALTH_MID as f32;
        let x_offset = half - half * health_percent;

        transform.scale = vec3(health_percent as f32, 1.0, 1.0);
        transform.translation = vec3(-x_offset, HEALTH_BAR_Y_OFFSET, 0.2)
    }
}

/// Dead players have no health left to show. This follows the player instead of
/// being done once on death, a rollback can bring a player back to life.
pub fn hide_dead_health_bars(
    mut parts: Query<(&ChildOf, &mut Visibility), With<PlayerHealthBarPart>>,
    players: Query<&Player>,
) {
    for (child_of, mut visibility) in parts.iter_mut() {
        let Ok(player) = players.get(child_of.parent()) else {
            continue;
        };
        let wanted = if player.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}
