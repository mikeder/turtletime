use super::ui::{self, MenuButton};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 8;
pub const NAME_MAX_LEN: usize = 12;

#[derive(Component)]
pub struct MenuOnlineUI;

#[derive(Component)]
pub enum MenuOnlineBtn {
    QuickMatch,
    LobbyMatch,
    Back,
}

#[derive(Resource)]
pub struct PlayerCount(pub usize);

#[derive(Component)]
pub struct PlayerCountText;

#[derive(Component)]
pub enum PlayerCountBtn {
    Up,
    Down,
}

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

/// The box that shows the name of the player.
#[derive(Component)]
pub struct NameField;

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
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
                ui::column(),
                children![
                    ui::body(font, "Your name"),
                    (ui::input_field(font), NameField),
                    ui::hint(
                        font,
                        "Type the name other players see, or play as a numbered player"
                    ),
                ],
            ),
            ui::spacer(10.),
            // quick match button
            (
                ui::button(font, "Quick match", MenuButton::Primary),
                MenuOnlineBtn::QuickMatch,
            ),
            ui::hint(font, "Play with whoever else is looking for a match"),
            // lobby match button
            (
                ui::button(font, "Play with friends", MenuButton::Secondary),
                MenuOnlineBtn::LobbyMatch,
            ),
            ui::hint(font, "Open or join a lobby with a code"),
            ui::spacer(10.),
            // back button
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuOnlineBtn::Back,
            ),
        ],
    ));
}

/// The characters typed since the last frame.
pub fn typed_chars(keyboard_evr: &mut MessageReader<KeyboardInput>) -> String {
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
    typed
}

pub fn update_name(
    mut keyboard_evr: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_name: ResMut<PlayerName>,
) {
    let typed = typed_chars(&mut keyboard_evr);
    let backspace = keys.just_pressed(KeyCode::Backspace);
    if typed.is_empty() && !backspace {
        return;
    }

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

pub fn update_name_display(
    player_name: Res<PlayerName>,
    mut fields: Query<(&mut BorderColor, &Children), With<NameField>>,
    mut texts: Query<&mut Text>,
) {
    for (mut border, children) in fields.iter_mut() {
        // the field lights up once it holds a name
        let edge = if player_name.0.is_empty() {
            ui::CREAM
        } else {
            ui::LETTUCE
        };
        if border.top != edge {
            *border = BorderColor::all(edge);
        }
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                if text.0 != player_name.0 {
                    text.0 = player_name.0.clone();
                }
            }
        }
    }
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuOnlineBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuOnlineBtn::QuickMatch => state.set(AppState::MenuQuick),
                MenuOnlineBtn::LobbyMatch => state.set(AppState::MenuLobby),
                MenuOnlineBtn::Back => state.set(AppState::MenuMain),
            }
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

/// The player count with the buttons to change it, shared by the menus that start a match.
pub fn player_count_picker(font: &Handle<Font>) -> impl Bundle {
    (
        ui::column(),
        children![
            ui::body(font, "Players"),
            (
                ui::row(),
                children![
                    (ui::small_button(font, "-"), PlayerCountBtn::Down),
                    (
                        ui::text(font, "", ui::HEADING_SIZE, ui::LETTUCE),
                        PlayerCountText,
                    ),
                    (ui::small_button(font, "+"), PlayerCountBtn::Up),
                ],
            ),
        ],
    )
}

pub fn player_count_btns(
    mut player_count: ResMut<PlayerCount>,
    mut interaction_query: Query<(&Interaction, &PlayerCountBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                PlayerCountBtn::Up => {
                    if player_count.0 < MAX_PLAYERS {
                        player_count.0 += 1
                    }
                }
                PlayerCountBtn::Down => {
                    if player_count.0 > MIN_PLAYERS {
                        player_count.0 -= 1
                    }
                }
            }
        }
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
