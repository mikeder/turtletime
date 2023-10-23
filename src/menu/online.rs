use super::connect::ConnectData;
use super::plugin::{
    BUTTON_TEXT, DISABLED_BUTTON, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, VERSION,
};
use crate::loading::FontAssets;
use crate::AppState;
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
    commands
        .spawn(Camera2dBundle::default())
        .insert(MenuOnlineUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
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
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            // player count buttons
            parent
                .spawn(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Player Count: ".to_owned(),
                                style: TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    color: BUTTON_TEXT,
                                },
                            },
                            TextSection {
                                value: "".to_owned(),
                                style: TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    color: BUTTON_TEXT,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(PlayerCountText);
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::RowReverse,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::NONE),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(65.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(16.)),
                                padding: UiRect::all(Val::Px(16.)),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(NORMAL_BUTTON),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "+",
                                    TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0,
                                        color: BUTTON_TEXT,
                                    },
                                ),
                                ..Default::default()
                            });
                        })
                        .insert(MenuOnlineBtn::PlayerCountUP);

                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(65.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(16.)),
                                padding: UiRect::all(Val::Px(16.)),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(NORMAL_BUTTON),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "-",
                                    TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0,
                                        color: BUTTON_TEXT,
                                    },
                                ),
                                ..Default::default()
                            });
                        })
                        .insert(MenuOnlineBtn::PlayerCountDown);
                });

            // quick match button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(NORMAL_BUTTON),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Quick Match",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::QuickMatch);

            // lobby id text
            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Enter a 4-digit ID!\n".to_owned(),
                                style: TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    color: BUTTON_TEXT,
                                },
                            },
                            TextSection {
                                value: "".to_owned(),
                                style: TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    color: BUTTON_TEXT,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(LobbyCodeText);

            // lobby match button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(NORMAL_BUTTON),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Lobby Match",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::LobbyMatch)
                .insert(ButtonEnabled(false));

            // back button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(NORMAL_BUTTON),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Back to Menu",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::Back);
        })
        .insert(MenuOnlineUI);
}

pub fn update_lobby_id(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    mut lobby_id: ResMut<LobbyID>,
) {
    let lid = &mut lobby_id.0;
    for ev in char_evr.iter() {
        if lid.len() < 4 && ev.char.is_ascii_digit() {
            lid.push(ev.char);
        }
    }
    if keys.just_pressed(KeyCode::Back) {
        let mut chars = lid.chars();
        chars.next_back();
        *lid = chars.as_str().to_owned();
    }
}

pub fn update_lobby_id_display(
    mut query: Query<&mut Text, With<LobbyCodeText>>,
    lobby_id: ResMut<LobbyID>,
) {
    for mut text in query.iter_mut() {
        text.sections[1].value = lobby_id.0.clone();
    }
}

pub fn update_lobby_btn(
    text_query: Query<&Text, With<LobbyCodeText>>,
    mut btn_query: Query<&mut ButtonEnabled, With<MenuOnlineBtn>>,
) {
    let mut lobby_id_complete = false;
    for text in text_query.iter() {
        if text.sections[1].value.len() == 4 {
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
    mut query: Query<&mut Text, With<PlayerCountText>>,
) {
    for mut text in query.iter_mut() {
        text.sections[1].value = player_count.0.clone().to_string();
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuOnlineUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
