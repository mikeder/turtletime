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
    commands.spawn((Camera2d, Msaa::Off)).insert(WinUI);

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
            // match result string
            parent.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                Text::new(match_data.result.clone()),
                TextFont {
                    font: font_assets.fira_sans.clone(),
                    font_size: 96.,
                    ..default()
                },
                TextColor(BUTTON_TEXT),
            ));
            // rematch button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),

                        ..Default::default()
                    },
                    rematch_vis,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Rematch"),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                })
                .insert(MenuWinBtn::Rematch);
            // back to menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.),
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
    mut app_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuWinBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
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
        commands.entity(e).despawn();
    }
}
