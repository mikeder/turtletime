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
pub const NAME_MAX_LEN: usize = 12;

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

/// The name the local player picked for online matches, empty if they didn't pick one.
#[derive(Resource, Default)]
pub struct PlayerName(pub String);

/// Makes a name safe to show, names of other players come straight off the network.
pub fn clean_name(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_control())
        .take(NAME_MAX_LEN)
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The things the player can type into. As a resource it is the one the keyboard
/// goes to, as a component it marks what to click to get there.
#[derive(Resource, Component, Clone, Copy, PartialEq, Eq)]
pub enum OnlineField {
    Name,
    LobbyCode,
}

/// The box that shows the name of the player.
#[derive(Component)]
pub struct NameField;

/// One of the boxes that show the lobby id, with its position in the id.
#[derive(Component)]
pub struct LobbyCodeSlot(usize);

#[derive(Component)]
pub struct LobbyHintText;

#[derive(Resource)]
pub struct LobbyID(String);

pub fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_name: Res<PlayerName>,
) {
    // lobby id resource
    commands.insert_resource(LobbyID("".to_owned()));
    // a player that already has a name most likely wants to type a lobby code
    commands.insert_resource(if player_name.0.is_empty() {
        OnlineField::Name
    } else {
        OnlineField::LobbyCode
    });
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuOnlineUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuOnlineUI,
        children![
            ui::heading(font, "Online match"),
            // player name
            (
                ui::row(),
                children![
                    ui::body(font, "Your name"),
                    (ui::input_field(font), NameField, Button, OnlineField::Name),
                ],
            ),
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
                    (
                        ui::input_slot(font),
                        LobbyCodeSlot(0),
                        Button,
                        OnlineField::LobbyCode
                    ),
                    (
                        ui::input_slot(font),
                        LobbyCodeSlot(1),
                        Button,
                        OnlineField::LobbyCode
                    ),
                    (
                        ui::input_slot(font),
                        LobbyCodeSlot(2),
                        Button,
                        OnlineField::LobbyCode
                    ),
                    (
                        ui::input_slot(font),
                        LobbyCodeSlot(3),
                        Button,
                        OnlineField::LobbyCode
                    ),
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

/// Clicking a field or pressing tab decides where the keyboard goes.
pub fn update_focus(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<OnlineField>,
    interaction_query: Query<(&Interaction, &OnlineField), Changed<Interaction>>,
) {
    let mut next = *focus;
    if keys.just_pressed(KeyCode::Tab) {
        next = match next {
            OnlineField::Name => OnlineField::LobbyCode,
            OnlineField::LobbyCode => OnlineField::Name,
        };
    }
    // enter finishes the name, it joins the lobby when the code is what was typed
    if keys.just_pressed(KeyCode::Enter) && next == OnlineField::Name {
        next = OnlineField::LobbyCode;
    }
    for (interaction, field) in interaction_query.iter() {
        if let Interaction::Pressed = *interaction {
            next = *field;
        }
    }
    if *focus != next {
        *focus = next;
    }
}

pub fn update_text_input(
    mut keyboard_evr: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<OnlineField>,
    mut lobby_id: ResMut<LobbyID>,
    mut player_name: ResMut<PlayerName>,
) {
    let mut typed = String::new();
    for ev in keyboard_evr.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        match &ev.logical_key {
            Key::Character(chars) => typed.push_str(chars),
            Key::Space => typed.push(' '),
            _ => continue,
        }
    }
    let backspace = keys.just_pressed(KeyCode::Backspace);
    if typed.is_empty() && !backspace {
        return;
    }

    match *focus {
        OnlineField::Name => {
            let name = &mut player_name.0;
            for c in typed.chars() {
                // a name can't start with a space
                let fits = name.chars().count() < NAME_MAX_LEN;
                if fits && !c.is_control() && !(c == ' ' && name.is_empty()) {
                    name.push(c);
                }
            }
            if backspace {
                name.pop();
            }
        }
        OnlineField::LobbyCode => {
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
    }
}

pub fn update_name_display(
    player_name: Res<PlayerName>,
    focus: Res<OnlineField>,
    mut fields: Query<(&mut BorderColor, &Children), With<NameField>>,
    mut texts: Query<&mut Text>,
) {
    if !player_name.is_changed() && !focus.is_changed() {
        return;
    }

    for (mut border, children) in fields.iter_mut() {
        *border = BorderColor::all(if *focus == OnlineField::Name {
            ui::CREAM
        } else if player_name.0.is_empty() {
            ui::MOSS_EDGE
        } else {
            ui::LETTUCE
        });
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = player_name.0.clone();
            }
        }
    }
}

pub fn update_lobby_code_display(
    lobby_id: Res<LobbyID>,
    focus: Res<OnlineField>,
    mut slots: Query<(&LobbyCodeSlot, &mut BorderColor, &Children)>,
    mut texts: Query<&mut Text>,
) {
    if !lobby_id.is_changed() && !focus.is_changed() {
        return;
    }

    let typed = lobby_id.0.len();
    let focused = *focus == OnlineField::LobbyCode;
    for (slot, mut border, children) in slots.iter_mut() {
        // filled slots light up, the slot the next digit goes into stands out
        *border = BorderColor::all(if slot.0 < typed {
            ui::LETTUCE
        } else if slot.0 == typed && focused {
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

pub fn cleanup_ui(
    query: Query<Entity, With<MenuOnlineUI>>,
    mut commands: Commands,
    mut player_name: ResMut<PlayerName>,
) {
    // drop the space a name may still end with
    let name = clean_name(&player_name.0);
    if player_name.0 != name {
        player_name.0 = name;
    }
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
