use crate::menu::{connect, main, online, options, win};
use crate::AppState;
use bevy::prelude::*;

pub const DISABLED_BUTTON: Color = Color::rgb(0.8, 0.5, 0.5);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // main menu
            .add_system(main::setup_ui.in_schedule(OnEnter(AppState::MenuMain)))
            .add_systems(
                (main::btn_visuals, main::btn_listeners).in_set(OnUpdate(AppState::MenuMain)),
            )
            .add_system(main::cleanup_ui.in_schedule(OnExit(AppState::MenuMain)))
            //online menu
            .add_system(online::setup_ui.in_schedule(OnEnter(AppState::MenuOnline)))
            .add_systems(
                (
                    online::update_lobby_id,
                    online::update_lobby_id_display,
                    online::update_lobby_btn,
                    online::btn_visuals,
                    online::btn_listeners,
                    online::update_player_count_display,
                )
                    .in_set(OnUpdate(AppState::MenuOnline)),
            )
            .add_system(online::cleanup_ui.in_schedule(OnExit(AppState::MenuOnline)))
            // connect menu
            .add_systems(
                (connect::create_matchbox_socket, connect::setup_ui)
                    .in_schedule(OnEnter(AppState::MenuConnect)),
            )
            .add_systems(
                (
                    connect::lobby_system,
                    connect::btn_visuals,
                    connect::btn_listeners,
                )
                    .in_set(OnUpdate(AppState::MenuConnect)),
            )
            .add_system(connect::cleanup_ui.in_schedule(OnExit(AppState::MenuConnect)))
            // options menu
            .add_system(options::setup_ui.in_schedule(OnEnter(AppState::MenuOptions)))
            .add_systems(
                (options::btn_visuals, options::btn_listeners)
                    .in_set(OnUpdate(AppState::MenuOptions)),
            )
            .add_system(options::cleanup_ui.in_schedule(OnExit(AppState::MenuOptions)))
            // win menu
            .add_system(win::setup_ui.in_schedule(OnEnter(AppState::Win)))
            .add_systems((win::btn_visuals, win::btn_listeners).in_set(OnUpdate(AppState::Win)))
            .add_system(win::cleanup_ui.in_schedule(OnExit(AppState::Win)));
    }
}
