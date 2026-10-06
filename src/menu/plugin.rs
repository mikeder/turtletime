use crate::menu::{connect, main, online, options, ui, win};
use crate::AppState;
use bevy::prelude::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<online::PlayerName>()
            // hover, press and disabled looks are the same for every menu
            .add_systems(Update, ui::update_buttons)
            // main menu
            .add_systems(OnEnter(AppState::MenuMain), main::setup_ui)
            .add_systems(
                Update,
                main::btn_listeners.run_if(in_state(AppState::MenuMain)),
            )
            .add_systems(OnExit(AppState::MenuMain), main::cleanup_ui)
            //online menu
            .add_systems(OnEnter(AppState::MenuOnline), online::setup_ui)
            .add_systems(
                Update,
                (
                    online::update_focus,
                    online::update_text_input,
                    online::update_name_display,
                    online::update_lobby_code_display,
                    online::update_lobby_btn,
                    online::btn_listeners,
                    online::update_player_count_display,
                )
                    .run_if(in_state(AppState::MenuOnline)),
            )
            .add_systems(OnExit(AppState::MenuOnline), online::cleanup_ui)
            // connect menu
            .add_systems(
                OnEnter(AppState::MenuConnect),
                (connect::create_matchbox_socket, connect::setup_ui),
            )
            .add_systems(
                Update,
                (connect::lobby_system, connect::btn_listeners)
                    .run_if(in_state(AppState::MenuConnect)),
            )
            .add_systems(OnExit(AppState::MenuConnect), connect::cleanup_ui)
            // options menu
            .add_systems(OnEnter(AppState::MenuOptions), options::setup_ui)
            .add_systems(
                Update,
                options::btn_listeners.run_if(in_state(AppState::MenuOptions)),
            )
            .add_systems(OnExit(AppState::MenuOptions), options::cleanup_ui)
            // win menu
            .add_systems(OnEnter(AppState::Win), win::setup_ui)
            .add_systems(Update, win::btn_listeners.run_if(in_state(AppState::Win)))
            .add_systems(OnExit(AppState::Win), win::cleanup_ui);
    }
}
