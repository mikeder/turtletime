//! Runs a local round headless, the same way the "Local" menu button does.
//!
//! A GGRS sync test session rolls back and resimulates every frame, then compares
//! checksums. If any state that is changed in the `GgrsSchedule` isn't registered
//! for rollback in [`RollbackPlugin`], the resimulation diverges and the session
//! stops advancing frames, which is what these tests look for.

use std::time::Duration;

use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_ggrs::{RollbackApp, RollbackFrameCount, Session};
use bevy_matchbox::prelude::PeerId;
use ggrs::{PlayerType, SessionBuilder};
use std::hash::{Hash, Hasher};

use crate::audio::{InternalAudioPlugin, RollbackSound};
use crate::graphics::CharacterSheet;
use crate::loading::{AudioAssets, FontAssets, TextureAssets};
use crate::map::tilemap::TileMapPlugin;
use crate::menu::connect::LocalHandle;
use crate::menu::online::PlayerCount;
use crate::npc::plugin::GoosePlugin;
use crate::player::input::GGRSConfig;
use crate::player::plugin::PlayerPlugin;
use crate::player::resources::AgreedRandom;
use crate::rollback::RollbackPlugin;
use crate::{AppState, GameState, CHECK_DISTANCE, FPS, INPUT_DELAY, MAX_PREDICTION};

const NUM_PLAYERS: usize = 4;
/// frames the players run around, sprint and shoot for
const ACTIVE_FRAMES: i32 = 1800;
/// frames the players stand still for afterwards, longer than any sound effect
const IDLE_FRAMES: i32 = 600;

/// The entity checksum built into bevy_ggrs only notices a desync once the number
/// of rollback entities differs. Checksum positions too, so we notice right away.
fn checksum_transform(transform: &Transform) -> u64 {
    let mut hasher = bevy_ggrs::checksum_hasher();
    transform.translation.x.to_bits().hash(&mut hasher);
    transform.translation.y.to_bits().hash(&mut hasher);
    hasher.finish()
}

fn rollback_frame(app: &App) -> i32 {
    app.world().resource::<RollbackFrameCount>().0
}

fn sound_count(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query::<&RollbackSound>().iter(world).count()
}

fn local_round_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_state::<AppState>()
        .init_state::<GameState>()
        .add_plugins((
            RollbackPlugin,
            TileMapPlugin,
            InternalAudioPlugin,
            PlayerPlugin,
            GoosePlugin,
        ))
        .checksum_component::<Transform>(checksum_transform);

    // sound effects are real assets, their length decides when a sound expires.
    // nothing is rendered, so textures and fonts can stay unloaded.
    let asset_server = app.world().resource::<AssetServer>().clone();
    let audio_assets = AudioAssets {
        fireball_shot: asset_server.load("audio/fireball1.ogg"),
        fireball_hit: asset_server.load("audio/fireball1.ogg"),
        fireball_miss: asset_server.load("audio/fireball1.ogg"),
        walking: asset_server.load("audio/walking.ogg"),
        sprinting: asset_server.load("audio/sprinting.ogg"),
        pickup: asset_server.load("audio/pickup.ogg"),
    };
    let audio_handles = [
        audio_assets.fireball_shot.clone(),
        audio_assets.sprinting.clone(),
        audio_assets.pickup.clone(),
    ];
    app.insert_resource(audio_assets)
        .insert_resource(FontAssets {
            fira_sans: default(),
        })
        .insert_resource(TextureAssets {
            texture_turtle: default(),
            texture_turtle2: default(),
            texture_turtle_cheeks: default(),
            texture_turtle_cheeks2: default(),
            texture_turtle_cheeks_frame: default(),
            texture_turtle_cheeks_frame_ht: default(),
            texture_turtle_cheeks_frame_party_hat: default(),
            texture_poop: default(),
            texture_strawberry: default(),
            texture_chili_pepper: default(),
            texture_fireball: default(),
            texture_lettuce: default(),
            texture_goose: default(),
            texture_dirt: default(),
            texture_grass: default(),
            texture_fenceleft: default(),
            texture_fencebottom: default(),
            texture_fencetop: default(),
            texture_shortgrass: default(),
            texture_shortgrassblue: default(),
            texture_shortgrasspink: default(),
            texture_water: default(),
            texture_shortgrassedge: default(),
            texture_shortgrasstopedge: default(),
            texture_wateredge: default(),
            texture_peanutqueen: default(),
        })
        .insert_resource(CharacterSheet {
            turtle_image: default(),
            turtle_layout: default(),
            turtle_frames: [0, 1, 2, 3],
            goose_image: default(),
            goose_layout: default(),
            goose_frames: [0, 1, 2, 3],
        });

    for _ in 0..500 {
        app.update();
        if audio_handles
            .iter()
            .all(|handle| asset_server.is_loaded(handle))
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        audio_handles
            .iter()
            .all(|handle| asset_server.is_loaded(handle)),
        "sound effects failed to load"
    );

    // same session the main menu creates for a local round, with fixed peer ids
    // so every run of the test plays out the same
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);
    let mut peers = Vec::new();
    for i in 0..NUM_PLAYERS {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Could not add local player");
        peers.push(PeerId(Uuid::from_u128(i as u128 + 1)));
    }
    let sess = sess_build.start_synctest_session().expect("");

    app.insert_resource(Session::SyncTest(sess))
        .insert_resource(LocalHandle(0))
        .insert_resource(AgreedRandom::new(peers))
        .insert_resource(PlayerCount(NUM_PLAYERS));

    let world = app.world_mut();
    world
        .resource_mut::<NextState<AppState>>()
        .set(AppState::RoundLocal);
    world
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);

    app
}

/// Hold down keys depending on the rollback frame: walk in circles, sprint and
/// shoot every now and then, then let go of everything.
fn press_keys(app: &mut App) {
    let frame = rollback_frame(app);
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.reset_all();
    if frame >= ACTIVE_FRAMES {
        return;
    }

    let walk = [KeyCode::KeyW, KeyCode::KeyD, KeyCode::KeyS, KeyCode::KeyA];
    keys.press(walk[(frame / 45) as usize % walk.len()]);
    if frame % 120 < 30 {
        keys.press(KeyCode::ShiftLeft);
    }
    if frame % 20 < 5 {
        keys.press(KeyCode::Space);
    }
}

/// Advance the app until the rollback frame reaches `until`. Frame times are
/// uneven on purpose, a real game also runs zero, one or two rollback frames
/// per rendered frame.
fn run_until(app: &mut App, until: i32) -> usize {
    let frame_times = [16_700, 9_000, 25_000, 16_000, 33_000, 4_000];
    let mut most_sounds = 0;
    let mut stalled = 0;
    let mut update = 0;

    while rollback_frame(app) < until {
        let before = rollback_frame(app);
        press_keys(app);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            frame_times[update % frame_times.len()],
        )));
        app.update();
        update += 1;

        most_sounds = most_sounds.max(sound_count(app));
        stalled = if rollback_frame(app) == before {
            stalled + 1
        } else {
            0
        };
        assert!(
            stalled < 10,
            "rollback stopped advancing at frame {before}, checksums mismatched during resimulation"
        );
    }
    most_sounds
}

#[test]
fn local_round_stays_in_sync() {
    let mut app = local_round_app();

    let most_sounds = run_until(&mut app, ACTIVE_FRAMES);
    assert!(most_sounds > 0, "no sound effects were ever spawned");

    run_until(&mut app, ACTIVE_FRAMES + IDLE_FRAMES);
    assert_eq!(
        sound_count(&mut app),
        0,
        "finished sound effects were not removed"
    );
}
