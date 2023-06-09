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
            .add_plugin(AsciiPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(GraphicsPlugin)
            .add_plugin(TileMapPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(GoosePlugin)
            .add_plugin(ConsolePlugin);

        #[cfg(debug_assertions)]
        {
            // With FPS
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(DebugPlugin)
                .add_plugin(LogDiagnosticsPlugin::default());

            // Without FPS
            // app.add_plugin(DebugPlugin)
            //     .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}
