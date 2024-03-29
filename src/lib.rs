mod ascii;
mod audio;
pub mod debug;
mod graphics;
mod loading;
mod map;
mod menu;
pub mod npc;
pub mod player;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use ascii::AsciiPlugin;
use bevy::prelude::*;
use bevy::{app::App, diagnostic::FrameTimeDiagnosticsPlugin};
use debug::plugin::{ConsolePlugin, DebugPlugin};
use graphics::GraphicsPlugin;
use map::tilemap::TileMapPlugin;
use menu::plugin::MenuPlugin;
use npc::plugin::GoosePlugin;
use player::plugin::PlayerPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;

pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const MAP_HEIGHT: f32 = 768.0;
pub const TILE_SIZE: f32 = 32.0;
pub const HEALTH_BAR_Y_OFFSET: f32 = TILE_SIZE + 10.;
pub const FPS: usize = 60;
pub const FIXED_TICK_MS: u64 = 1000 / FPS as u64; // use fixed duration tick delta to keep in sync with GGRSSchedule

// const MATCHBOX_ADDR: &str = "ws://localhost:3536";
const MATCHBOX_ADDR: &str = "wss://match.sqweeb.net:443";
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;
const CHECK_DISTANCE: usize = 2;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // Main menu selection
    MenuMain,
    // Online connection menu selection
    MenuConnect,
    // Menu for making online rounds
    MenuOnline,
    // Menu for setting options
    MenuOptions,
    // Game logic for online round is executed
    RoundOnline,
    // Game logic fo local round is executed
    RoundLocal,
    // Win TODO: implement winning
    Win,
}

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // Round is paused or in menu
    #[default]
    Paused,
    // Round is actively being played
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .add_state::<GameState>()
            .add_plugins((
                AsciiPlugin,
                LoadingPlugin,
                GraphicsPlugin,
                TileMapPlugin,
                MenuPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                GoosePlugin,
                ConsolePlugin,
            ));

        #[cfg(debug_assertions)]
        {
            // With FPS
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                DebugPlugin,
                LogDiagnosticsPlugin::default(),
            ));

            // Without FPS
            // app.add_plugin(DebugPlugin)
            //     .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}
