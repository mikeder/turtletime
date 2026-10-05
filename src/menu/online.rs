use super::connect::ConnectData;
use super::plugin::VERSION;
use super::ui::{self, ButtonEnabled, MenuButton};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 8;
const LOBBY_ID_LEN: usize = 4;

#[derive(Component)]
pub struct MenuOnlineUI;

#[derive(Component)]
pub enum MenuOnlineBtn {
    PlayerCountUP,
    PlayerCountDown,
    LobbyMatch,
    QuickMatch,
    Back,
}

#[derive(Resource)]
pub struct PlayerCount(pub usize);

#[derive(Component)]
pub struct PlayerCountText;

/// One of the boxes that show the lobby id, with its position in the id.
#[derive(Component)]
pub struct LobbyCodeSlot(usize);

#[derive(Component)]
pub struct LobbyHintText;

#[derive(Resource)]
pub struct LobbyID(String);

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // lobby id resource
    commands.insert_resource(LobbyID("".to_owned()));
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuOnlineUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuOnlineUI,
        children![
            ui::heading(font, "Online match"),
            // player count buttons
            (
                ui::row(),
                children![
                    ui::body(font, "Players"),
                    (ui::small_button(font, "-"), MenuOnlineBtn::PlayerCountDown,),
                    (
                        ui::text(font, "", ui::HEADING_SIZE, ui::LETTUCE),
                        PlayerCountText,
                    ),
                    (ui::small_button(font, "+"), MenuOnlineBtn::PlayerCountUP),
                ],
            ),
            // quick match button
            (
                ui::button(font, "Quick match", MenuButton::Primary),
                MenuOnlineBtn::QuickMatch,
            ),
            ui::hint(font, "Play with whoever else is looking for a match"),
            ui::spacer(10.),
            // lobby id
            ui::body(font, "Or play with friends"),
            (
                ui::row(),
                children![
                    (ui::input_slot(font), LobbyCodeSlot(0)),
                    (ui::input_slot(font), LobbyCodeSlot(1)),
                    (ui::input_slot(font), LobbyCodeSlot(2)),
                    (ui::input_slot(font), LobbyCodeSlot(3)),
                ],
            ),
            (ui::hint(font, ""), LobbyHintText),
            // lobby match button
            (
                ui::button(font, "Join lobby", MenuButton::Secondary),
                MenuOnlineBtn::LobbyMatch,
                ButtonEnabled(false),
            ),
            // back button
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuOnlineBtn::Back,
            ),
        ],
    ));
}

pub fn update_lobby_id(
    mut keyboard_evr: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut lobby_id: ResMut<LobbyID>,
) {
    let lid = &mut lobby_id.0;
    for ev in keyboard_evr.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        let Key::Character(chars) = &ev.logical_key else {
            continue;
        };
        for c in chars.chars() {
            if lid.len() < LOBBY_ID_LEN && c.is_ascii_digit() {
                lid.push(c);
            }
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        let mut chars = lid.chars();
        chars.next_back();
        *lid = chars.as_str().to_owned();
    }
}

pub fn update_lobby_code_display(
    lobby_id: Res<LobbyID>,
    mut slots: Query<(&LobbyCodeSlot, &mut BorderColor, &Children)>,
    mut texts: Query<&mut Text>,
) {
    if !lobby_id.is_changed() {
        return;
    }

    let typed = lobby_id.0.len();
    for (slot, mut border, children) in slots.iter_mut() {
        // filled slots light up, the slot the next digit goes into stands out
        *border = BorderColor::all(if slot.0 < typed {
            ui::LETTUCE
        } else if slot.0 == typed {
            ui::CREAM
        } else {
            ui::MOSS_EDGE
        });

        let digit = lobby_id.0.chars().nth(slot.0);
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = digit.map(String::from).unwrap_or_default();
            }
        }
    }
}

pub fn update_lobby_btn(
    lobby_id: Res<LobbyID>,
    mut hint_query: Query<&mut Text, With<LobbyHintText>>,
    mut btn_query: Query<&mut ButtonEnabled, With<MenuOnlineBtn>>,
) {
    if !lobby_id.is_changed() {
        return;
    }

    let lobby_id_complete = lobby_id.0.len() == LOBBY_ID_LEN;
    for mut enabled in btn_query.iter_mut() {
        enabled.0 = lobby_id_complete;
    }
    for mut text in hint_query.iter_mut() {
        text.0 = if lobby_id_complete {
            "Press Enter to join. Friends who type the same code end up in your lobby".to_owned()
        } else {
            format!("Type a {LOBBY_ID_LEN}-digit code to open or join a lobby")
        };
    }
}

pub fn btn_listeners(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    lobby_id: Res<LobbyID>,
    mut player_count: ResMut<PlayerCount>,
    mut interaction_query: Query<
        (&Interaction, &MenuOnlineBtn, Option<&ButtonEnabled>),
        Changed<Interaction>,
    >,
) {
    // enter does the same as the lobby match button once the id is complete
    let mut join_lobby = keys.just_pressed(KeyCode::Enter) && lobby_id.0.len() == LOBBY_ID_LEN;

    for (interaction, btn, enabled) in interaction_query.iter_mut() {
        let clickable = match enabled {
            Some(e) => e.0,
            None => true,
        };

        if !clickable {
            continue;
        }

        if let Interaction::Pressed = *interaction {
            match btn {
                MenuOnlineBtn::PlayerCountUP => {
                    if player_count.0 < MAX_PLAYERS {
                        player_count.0 += 1
                    }
                }
                MenuOnlineBtn::PlayerCountDown => {
                    if player_count.0 > MIN_PLAYERS {
                        player_count.0 -= 1
                    }
                }
                MenuOnlineBtn::LobbyMatch => join_lobby = true,
                MenuOnlineBtn::QuickMatch => {
                    commands.insert_resource(ConnectData {
                        lobby_id: format!("turtletime_{}?next={}", VERSION, player_count.0),
                        lobby_code: None,
                    });
                    state.set(AppState::MenuConnect);
                }
                MenuOnlineBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }

    if join_lobby {
        commands.insert_resource(ConnectData {
            lobby_id: format!("turtletime_{}_{}", VERSION, lobby_id.0),
            lobby_code: Some(lobby_id.0.clone()),
        });
        state.set(AppState::MenuConnect);
    }
}

pub fn update_player_count_display(
    player_count: Res<PlayerCount>,
    mut query: Query<&mut Text, With<PlayerCountText>>,
) {
    let count = player_count.0.to_string();
    for mut text in query.iter_mut() {
        if text.0 != count {
            text.0 = count.clone();
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuOnlineUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
