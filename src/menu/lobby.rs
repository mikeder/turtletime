use super::connect::ConnectData;
use super::online;
use super::plugin::VERSION;
use super::ui::{self, ButtonEnabled, MenuButton};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

const LOBBY_ID_LEN: usize = 4;

#[derive(Component)]
pub struct MenuLobbyUI;

#[derive(Component)]
pub enum MenuLobbyBtn {
    Join,
    Back,
}

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
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuLobbyUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuLobbyUI,
        children![
            ui::heading(font, "Play with friends"),
            online::player_count_picker(font),
            // the lobby only starts once it has as many players as each of them asked for
            ui::hint(
                font,
                "Everyone in the lobby has to pick the same number of players"
            ),
            ui::spacer(10.),
            // lobby id
            (
                ui::column(),
                children![
                    ui::body(font, "Lobby code"),
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
                ],
            ),
            ui::spacer(10.),
            // lobby match button
            (
                ui::button(font, "Join lobby", MenuButton::Primary),
                MenuLobbyBtn::Join,
                ButtonEnabled(false),
            ),
            // back button
            (
                ui::button(font, "Back", MenuButton::Secondary),
                MenuLobbyBtn::Back,
            ),
        ],
    ));
}

pub fn update_lobby_id(
    mut keyboard_evr: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut lobby_id: ResMut<LobbyID>,
) {
    let typed = online::typed_chars(&mut keyboard_evr);
    let backspace = keys.just_pressed(KeyCode::Backspace);
    if typed.is_empty() && !backspace {
        return;
    }

    let lid = &mut lobby_id.0;
    for c in typed.chars() {
        if lid.len() < LOBBY_ID_LEN && c.is_ascii_digit() {
            lid.push(c);
        }
    }
    if backspace {
        lid.pop();
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
    mut btn_query: Query<&mut ButtonEnabled, With<MenuLobbyBtn>>,
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
    mut interaction_query: Query<
        (&Interaction, &MenuLobbyBtn, Option<&ButtonEnabled>),
        Changed<Interaction>,
    >,
) {
    // enter does the same as the join button once the id is complete
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
                MenuLobbyBtn::Join => join_lobby = true,
                MenuLobbyBtn::Back => state.set(AppState::MenuOnline),
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

pub fn cleanup_ui(query: Query<Entity, With<MenuLobbyUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
