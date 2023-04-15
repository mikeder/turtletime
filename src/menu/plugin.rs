use crate::menu::{connect, main, online, win};
use crate::GameState;
use bevy::prelude::*;
pub const DISABLED_BUTTON: Color = Color::rgb(0.8, 0.5, 0.5);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // main menu
            .add_system(main::setup_ui.in_schedule(OnEnter(GameState::MenuMain)))
            .add_systems(
                (main::btn_visuals, main::btn_listeners).in_set(OnUpdate(GameState::MenuMain)),
            )
            .add_system(main::cleanup_ui.in_schedule(OnExit(GameState::MenuMain)))
            //online menu
            .add_system(online::setup_ui.in_schedule(OnEnter(GameState::MenuOnline)))
            .add_systems(
                (
                    online::update_lobby_id,
                    online::update_lobby_id_display,
                    online::update_lobby_btn,
                    online::btn_visuals,
                    online::btn_listeners,
                )
                    .in_set(OnUpdate(GameState::MenuOnline)),
            )
            .add_system(online::cleanup_ui.in_schedule(OnExit(GameState::MenuOnline)))
            // connect menu
            .add_systems(
                (connect::create_matchbox_socket, connect::setup_ui)
                    .in_schedule(OnEnter(GameState::MenuConnect)),
            )
            .add_systems(
                (
                    connect::lobby_system,
                    connect::btn_visuals,
                    connect::btn_listeners,
                )
                    .in_set(OnUpdate(GameState::MenuConnect)),
            )
            .add_system(connect::cleanup_ui.in_schedule(OnExit(GameState::MenuConnect)))
            // win menu
            .add_system(win::setup_ui.in_schedule(OnEnter(GameState::Win)))
            .add_systems((win::btn_visuals, win::btn_listeners).in_set(OnUpdate(GameState::Win)))
            .add_system(win::cleanup_ui.in_schedule(OnExit(GameState::Win)));
    }
}
