use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuOptionsUI;

#[derive(Component)]
pub enum MenuOptionsBtn {
    Back,
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuOptionsUI);

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
            parent
                .spawn((
                    Text::new("Controls:\n".to_owned()),
                    TextFont {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(BUTTON_TEXT),
                ))
                .with_children(|p| {
                    p.spawn((
                        TextSpan::new("Movement: [W A S D]\n".to_owned()),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                    p.spawn((
                        TextSpan::new("Fireball: [SPACE or RETURN]\n".to_owned()),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                    p.spawn((
                        TextSpan::new("Sprint: [LEFT SHIFT]\n".to_owned()),
                        TextFont {
                            font: font_assets.fira_sans.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                    ));
                });

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
                .insert(MenuOptionsBtn::Back);
        })
        .insert(MenuOptionsUI);
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuOptionsBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuOptionsBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuOptionsBtn>),
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

pub fn cleanup_ui(query: Query<Entity, With<MenuOptionsUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
