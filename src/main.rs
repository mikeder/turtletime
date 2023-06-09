// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use bevy_ggrs::GGRSPlugin;
use std::io::Cursor;
use turtle_time::npc::components::{EdibleTarget, Goose, HasTarget};
use turtle_time::player::checksum::Checksum;
use turtle_time::player::components::{
    Edible, EdibleSpawnTimer, Expired, Fireball, FireballAmmo, FireballMovement, FireballReady,
    FireballTimer, Player, PlayerHealth, PlayerPoop, PlayerPoopTimer, PlayerSpeed,
    PlayerSpeedBoost, RoundComponent,
};
use turtle_time::player::input::{input, GGRSConfig, PlayerControls};
use turtle_time::{GamePlugin, ASPECT_RATIO, FPS, MAP_HEIGHT};
use winit::window::Icon;

fn main() {
    let mut app = App::new();

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input)
        .register_rollback_component::<Checksum>()
        .register_rollback_component::<Edible>()
        .register_rollback_component::<EdibleTarget>()
        .register_rollback_component::<Expired>()
        .register_rollback_component::<Fireball>()
        .register_rollback_component::<FireballAmmo>()
        .register_rollback_component::<FireballReady>()
        .register_rollback_component::<FireballMovement>()
        .register_rollback_component::<FireballTimer>()
        .register_rollback_component::<Goose>()
        .register_rollback_component::<HasTarget>()
        .register_rollback_component::<Player>()
        .register_rollback_component::<PlayerHealth>()
        .register_rollback_component::<PlayerSpeed>()
        .register_rollback_component::<PlayerSpeedBoost>()
        .register_rollback_component::<PlayerControls>()
        .register_rollback_component::<PlayerPoop>()
        .register_rollback_component::<PlayerPoopTimer>()
        .register_rollback_component::<RoundComponent>()
        .register_rollback_component::<Transform>()
        .register_rollback_resource::<EdibleSpawnTimer>()
        .build(&mut app);

    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.3, 0.0)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        title: "Turtle Time".to_string(),
                        resolution: (MAP_HEIGHT * ASPECT_RATIO, MAP_HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter:
                        "warn,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn,turtle_time=warn"
                            .into(),
                    level: bevy::log::Level::WARN,
                }),
        )
        .add_plugin(GamePlugin)
        .add_system(set_window_icon.on_startup())
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
