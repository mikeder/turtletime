mod actions;
mod ascii;
mod audio;
mod debug;
mod graphics;
mod loading;
mod menu;
pub mod network;
mod player;
mod tilemap;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::debug::DebugPlugin;
use crate::loading::LoadingPlugin;
use crate::player::PlayerPlugin;
use menu::plugin::MenuPlugin;

use ascii::AsciiPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use graphics::GraphicsPlugin;
use network::NetworkPlugin;
use tilemap::TileMapPlugin;

pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const MAP_HEIGHT: f32 = 768.0;
pub const TILE_SIZE: f32 = 32.0;
pub const FPS: usize = 60;

const NUM_PLAYERS: usize = 4;
const ROLLBACK_SYSTEMS: &str = "rollback_systems";
const CHECKSUM_UPDATE: &str = "checksum_update";
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;
const CHECK_DISTANCE: usize = 2;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // Main menu selection
    MenuMain,
    // Online connection menu selection
    MenuConnect,
    // Menu for making online rounds
    MenuOnline,
    // Menu for making local rounds
    MenuLocal,
    // Game logic for online round is executed
    RoundOnline,
    // Game logic fo local round is executed
    RoundLocal,
    // Win TODO: implement winning
    Win,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugin(AsciiPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(GraphicsPlugin)
            .add_plugin(TileMapPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(NetworkPlugin)
            .add_plugin(PlayerPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(DebugPlugin)
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}