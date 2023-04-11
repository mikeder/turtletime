use super::connect::LocalHandles;
use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::loading::{FontAssets, TextureAssets};
use crate::network::GGRSConfig;
use crate::{GameState, CHECK_DISTANCE, FPS, INPUT_DELAY, MAX_PREDICTION, NUM_PLAYERS};
use bevy::{app::AppExit, prelude::*};
use bevy_ggrs::Session;
use ggrs::{PlayerType, SessionBuilder};
use image::flat::NormalForm;

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub enum MainMenuBtn {
    OnlineMatch,
    LocalMatch,
    Quit,
}

pub fn setup_ui(
    mut commands: Commands,
    image_assets: Res<TextureAssets>,
    font_assets: Res<FontAssets>,
) {
    // ui camera
    commands.spawn(Camera2dBundle::default()).insert(MainMenuUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.)),
                flex_direction: FlexDirection::ColumnReverse,
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
            // logo
            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(500.0), Val::Px(139.0)),
                    margin: UiRect::all(Val::Px(16.)),
                    padding: UiRect::all(Val::Px(16.)),
                    ..Default::default()
                },
                image: image_assets.texture_turtle.clone().into(),
                ..Default::default()
            });

            // online match button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
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
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
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

            // quit button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
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
            Interaction::Clicked => {
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
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&Interaction, &MainMenuBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            match btn {
                MainMenuBtn::OnlineMatch => {
                    state.set(GameState::MenuOnline);
                }
                MainMenuBtn::LocalMatch => {
                    create_synctest_session(&mut commands);
                    state.set(GameState::RoundLocal);
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

fn create_synctest_session(commands: &mut Commands) {
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);

    for i in 0..NUM_PLAYERS {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Could not add local player");
    }

    let sess = sess_build.start_synctest_session().expect("");

    commands.insert_resource(Session::SyncTestSession(sess));
    commands.insert_resource(LocalHandles {
        handles: (0..NUM_PLAYERS).collect(),
    });
}
