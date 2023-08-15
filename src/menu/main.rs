use super::connect::{ConnectData, LocalHandle};
use super::online::PlayerCount;
use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, VERSION};
use crate::loading::{FontAssets, TextureAssets};
use crate::player::input::GGRSConfig;
use crate::player::resources::AgreedRandom;
use crate::{AppState, GameState, CHECK_DISTANCE, FPS, INPUT_DELAY, MAX_PREDICTION};
use bevy::utils::Uuid;
use bevy::{app::AppExit, prelude::*};
use bevy_ggrs::Session;
use bevy_matchbox::prelude::PeerId;
use ggrs::{PlayerType, SessionBuilder};

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub enum MainMenuBtn {
    OnlineMatch,
    LocalMatch,
    Options,
    Quit,
}

pub fn setup_ui(
    mut commands: Commands,
    image_assets: Res<TextureAssets>,
    font_assets: Res<FontAssets>,
    player_count: Option<Res<PlayerCount>>,
) {
    // default player count
    if player_count.is_none() {
        commands.insert_resource(PlayerCount(4));
    }

    // ui camera
    commands.spawn(Camera2dBundle::default()).insert(MainMenuUI);

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
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Turtle Time!",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 50.0,
                        color: BUTTON_TEXT,
                    },
                ),
                ..Default::default()
            });

            // logo
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(128.0),
                    height: Val::Px(128.0),
                    margin: UiRect::all(Val::Px(16.)),
                    padding: UiRect::all(Val::Px(16.)),
                    ..Default::default()
                },
                image: image_assets.texture_turtle_cheeks2.clone().into(),
                ..Default::default()
            });

            // online match button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
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
                            "Online",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MainMenuBtn::OnlineMatch);

            // local mode button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
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
                            "Local",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MainMenuBtn::LocalMatch);

            // local mode button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
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
                            "Options",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MainMenuBtn::Options);

            // quit button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
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
                            "Quit",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MainMenuBtn::Quit);

            parent.spawn(TextBundle {
                text: Text::from_section(
                    VERSION,
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 28.0,
                        color: BUTTON_TEXT,
                    },
                ),
                ..Default::default()
            });
        })
        .insert(MainMenuUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
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
    }
}

pub fn btn_listeners(
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_count: Res<PlayerCount>,
    mut interaction_query: Query<(&Interaction, &MainMenuBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MainMenuBtn::OnlineMatch => {
                    app_state.set(AppState::MenuOnline);
                }
                MainMenuBtn::LocalMatch => {
                    // remove any lingering online connect data
                    commands.remove_resource::<ConnectData>();

                    create_synctest_session(&mut commands, player_count.0);
                    app_state.set(AppState::RoundLocal);
                    game_state.set(GameState::Playing);
                }
                MainMenuBtn::Options => {
                    app_state.set(AppState::MenuOptions);
                }
                MainMenuBtn::Quit => {
                    exit.send(AppExit);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MainMenuUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn create_synctest_session(commands: &mut Commands, num_players: usize) {
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(num_players)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);

    let mut peers = Vec::new();
    for i in 0..num_players {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Could not add local player");
        peers.push(PeerId(Uuid::new_v4()))
    }

    let sess = sess_build.start_synctest_session().expect("");

    commands.insert_resource(Session::SyncTest(sess));
    commands.insert_resource(LocalHandle(0));
    commands.insert_resource(AgreedRandom::new(peers));
}
