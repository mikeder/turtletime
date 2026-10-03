// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::NonSendMarker;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use bevy::DefaultPlugins;
use bevy_ggrs::{GgrsPlugin, ReadInputs, RollbackApp, RollbackFrameRate};
use std::io::Cursor;
use turtle_time::npc::components::{EdibleTarget, Goose, HasTarget};
use turtle_time::player::checksum::Checksum;
use turtle_time::player::components::{
    Edible, EdibleSpawnTimer, Expired, Fireball, FireballAmmo, FireballMovement, FireballReady,
    FireballTimer, Player, PlayerHealth, PlayerHealthBar, PlayerPoop, PlayerPoopTimer, PlayerSpeed,
    PlayerSpeedBoost, RoundComponent,
};
use turtle_time::player::input::{input, GGRSConfig, PlayerControls};
use turtle_time::player::resources::AgreedRandom;
use turtle_time::{GamePlugin, ASPECT_RATIO, FPS, MAP_HEIGHT};
use winit::window::Icon;

fn main() {
    let mut app = App::new();

    // TODO: move GGRS plugin setup out of mains
    app.add_plugins(GgrsPlugin::<GGRSConfig>::default())
        .insert_resource(RollbackFrameRate(FPS))
        .add_systems(ReadInputs, input)
        .rollback_component_with_clone::<Checksum>()
        .rollback_component_with_clone::<Edible>()
        .rollback_component_with_clone::<EdibleTarget>()
        .rollback_component_with_clone::<Expired>()
        .rollback_component_with_clone::<Fireball>()
        .rollback_component_with_clone::<FireballAmmo>()
        .rollback_component_with_clone::<FireballReady>()
        .rollback_component_with_clone::<FireballMovement>()
        .rollback_component_with_clone::<FireballTimer>()
        .rollback_component_with_clone::<Goose>()
        .rollback_component_with_clone::<HasTarget>()
        .rollback_component_with_clone::<Player>()
        .rollback_component_with_clone::<PlayerHealth>()
        .rollback_component_with_clone::<PlayerHealthBar>()
        .rollback_component_with_clone::<PlayerSpeed>()
        .rollback_component_with_clone::<PlayerSpeedBoost>()
        .rollback_component_with_clone::<PlayerControls>()
        .rollback_component_with_clone::<PlayerPoop>()
        .rollback_component_with_clone::<PlayerPoopTimer>()
        .rollback_component_with_clone::<RoundComponent>()
        .rollback_component_with_clone::<Transform>()
        .rollback_resource_with_clone::<EdibleSpawnTimer>()
        // edibles spawn at random positions, so the shared rng must roll back with the world
        .rollback_resource_with_clone::<AgreedRandom>();

    app.insert_resource(ClearColor(Color::srgb(0.0, 0.3, 0.0)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Tell bevy skip asset meta file checks
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#bevy".to_owned()), // use for trunk, remove for wasm-server-runner
                        fit_canvas_to_parent: true,
                        title: "Turtle Time".to_string(),
                        resolution: ((MAP_HEIGHT * ASPECT_RATIO) as u32, MAP_HEIGHT as u32).into(),
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter:
                        "warn,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn,turtle_time=warn"
                            .into(),
                    level: bevy::log::Level::WARN,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    // winit windows live in a thread local, so this system must run on the main thread
    _main_thread: NonSendMarker,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(primary_entity) = primary_window.single() else {
        warn!("primary window not found, unable to set icon");
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) else {
        return;
    };
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height).unwrap();

    WINIT_WINDOWS.with_borrow(|windows| {
        // some new issue was introduced with the Bevy 0.12 upgrade,
        // the winit window may not exist yet, so we just log a warning and abort
        // if we can't get the primary window.
        // https://github.com/NiklasEi/bevy_game_template/issues/80
        match windows.get_window(primary_entity) {
            Some(primary) => primary.set_window_icon(Some(icon)),
            None => warn!("window not found, unable to set icon"),
        }
    });
}
