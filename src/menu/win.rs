use super::connect::ConnectData;
use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct WinUI;

#[derive(Component)]
pub enum MenuWinBtn {
    Back,
    Rematch,
}

#[derive(Resource)]
pub struct MatchData {
    pub result: String,
}

pub fn setup_ui(
    mut commands: Commands,
    match_data: Res<MatchData>,
    font_assets: Res<FontAssets>,
    connect_data: Option<Res<ConnectData>>,
) {
    let mut rematch_vis = Visibility::Hidden;
    if connect_data.is_some() {
        rematch_vis = Visibility::Visible;
    }

    // ui camera
    commands.spawn(Camera2dBundle::default()).insert(WinUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.)),
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // match result string
            parent.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text::from_section(
                    match_data.result.clone(),
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 96.,
                        color: BUTTON_TEXT,
                    },
                ),
                ..Default::default()
            });
            // rematch button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),

                        ..Default::default()
                    },
                    visibility: rematch_vis,
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Rematch",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuWinBtn::Rematch);
            // back to menu button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
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
                .insert(MenuWinBtn::Back);
        })
        .insert(WinUI);

    commands.remove_resource::<MatchData>();
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuWinBtn>),
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
    mut app_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuWinBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            match btn {
                MenuWinBtn::Back => {
                    app_state.set(AppState::MenuMain);
                }
                MenuWinBtn::Rematch => {
                    app_state.set(AppState::MenuConnect);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<WinUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
