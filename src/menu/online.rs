use super::connect::ConnectData;
use super::plugin::{
    BUTTON_TEXT, DISABLED_BUTTON, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, VERSION,
};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 8;

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

#[derive(Component)]
pub struct ButtonEnabled(bool);

#[derive(Component)]
pub struct LobbyCodeText;

#[derive(Resource)]
pub struct LobbyID(String);

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // lobby id resource
    commands.insert_resource(LobbyID("".to_owned()));
    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuOnlineUI);

    // root node
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                right: Val::Px(0.),
                top: Val::Px(0.),
                bottom: Val::Px(0.),
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // player count buttons
            parent
                .spawn((
                    Text::new("Player Count: ".to_owned()),
                    TextFont {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(BUTTON_TEXT),
                ))
                .with_children(|p| {
                    p.spawn((
                        TextSpan::new("".to_owned()),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                        PlayerCountText,
                    ));
                });
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::RowReverse,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(65.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(16.)),
                                padding: UiRect::all(Val::Px(16.)),
                                ..Default::default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("+"),
                                TextFont {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    ..default()
                                },
                                TextColor(BUTTON_TEXT),
                            ));
                        })
                        .insert(MenuOnlineBtn::PlayerCountUP);

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(65.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(16.)),
                                padding: UiRect::all(Val::Px(16.)),
                                ..Default::default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("-"),
                                TextFont {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    ..default()
                                },
                                TextColor(BUTTON_TEXT),
                            ));
                        })
                        .insert(MenuOnlineBtn::PlayerCountDown);
                });

            // quick match button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quick Match"),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                })
                .insert(MenuOnlineBtn::QuickMatch);

            // lobby id text
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    Text::new("Enter a 4-digit ID!\n".to_owned()),
                    TextFont {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(BUTTON_TEXT),
                ))
                .with_children(|p| {
                    p.spawn((
                        TextSpan::new("".to_owned()),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                        LobbyCodeText,
                    ));
                });

            // lobby match button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Lobby Match"),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                })
                .insert(MenuOnlineBtn::LobbyMatch)
                .insert(ButtonEnabled(false));

            // back button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back to Menu"),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                })
                .insert(MenuOnlineBtn::Back);
        })
        .insert(MenuOnlineUI);
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
            if lid.len() < 4 && c.is_ascii_digit() {
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

pub fn update_lobby_id_display(
    mut query: Query<&mut TextSpan, With<LobbyCodeText>>,
    lobby_id: ResMut<LobbyID>,
) {
    for mut text in query.iter_mut() {
        text.0 = lobby_id.0.clone();
    }
}

pub fn update_lobby_btn(
    text_query: Query<&TextSpan, With<LobbyCodeText>>,
    mut btn_query: Query<&mut ButtonEnabled, With<MenuOnlineBtn>>,
) {
    let mut lobby_id_complete = false;
    for text in text_query.iter() {
        if text.0.len() == 4 {
            lobby_id_complete = true;
            break;
        }
    }

    for mut enabled in btn_query.iter_mut() {
        enabled.0 = lobby_id_complete;
    }
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&ButtonEnabled>),
        With<MenuOnlineBtn>,
    >,
) {
    for (interaction, mut color, enabled) in interaction_query.iter_mut() {
        let changeable = match enabled {
            Some(e) => e.0,
            None => true,
        };
        if changeable {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                }
            }
        } else {
            *color = DISABLED_BUTTON.into();
        }
    }
}

pub fn btn_listeners(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    lobby_id: Res<LobbyID>,
    mut player_count: ResMut<PlayerCount>,
    mut interaction_query: Query<
        (&Interaction, &MenuOnlineBtn, Option<&ButtonEnabled>),
        Changed<Interaction>,
    >,
) {
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
                MenuOnlineBtn::LobbyMatch => {
                    commands.insert_resource(ConnectData {
                        lobby_id: format!("turtletime_{}_{}", VERSION, lobby_id.0),
                    });
                    state.set(AppState::MenuConnect);
                }
                MenuOnlineBtn::QuickMatch => {
                    commands.insert_resource(ConnectData {
                        lobby_id: format!("turtletime_{}?next={}", VERSION, player_count.0),
                    });
                    state.set(AppState::MenuConnect);
                }
                MenuOnlineBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

pub fn update_player_count_display(
    player_count: Res<PlayerCount>,
    mut query: Query<&mut TextSpan, With<PlayerCountText>>,
) {
    for mut text in query.iter_mut() {
        text.0 = player_count.0.clone().to_string();
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuOnlineUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
