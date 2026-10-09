use super::ui::{self, MenuButton};
use crate::graphics::{Character, CharacterSheet};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuCharacterUI;

#[derive(Component)]
pub enum MenuCharacterBtn {
    Pick(Character),
    Back,
}

/// The turtle the local player picked to play as.
#[derive(Resource, Default)]
pub struct SelectedCharacter(pub Character);

pub fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    characters: Res<CharacterSheet>,
) {
    let font = &font_assets.fira_sans;

    // ui camera
    commands
        .spawn((Camera2d, Msaa::Off))
        .insert(MenuCharacterUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuCharacterUI,
        children![
            ui::heading(font, "Pick your turtle"),
            (
                ui::row(),
                children![
                    character_option(font, &characters, Character::Plain, "Classic"),
                    character_option(font, &characters, Character::Hat, "Hat"),
                ],
            ),
            ui::hint(
                font,
                "The winner of a round wears a party hat in the next one"
            ),
            ui::spacer(10.),
            // back button
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuCharacterBtn::Back,
            ),
        ],
    ));
}

/// A turtle with the button to pick it.
fn character_option(
    font: &Handle<Font>,
    characters: &CharacterSheet,
    character: Character,
    label: &'static str,
) -> impl Bundle {
    (
        ui::column(),
        children![
            (
                ImageNode::from_atlas_image(
                    characters.turtle_image(character),
                    TextureAtlas {
                        layout: characters.turtle_layout.clone(),
                        index: characters.turtle_frames[0],
                    },
                ),
                Node {
                    width: Val::Px(128.),
                    height: Val::Px(128.),
                    ..Default::default()
                },
            ),
            (
                ui::button(font, label, MenuButton::Secondary),
                MenuCharacterBtn::Pick(character),
            ),
        ],
    )
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut selected: ResMut<SelectedCharacter>,
    mut interaction_query: Query<(&Interaction, &MenuCharacterBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuCharacterBtn::Pick(character) => {
                    selected.0 = *character;
                }
                MenuCharacterBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

/// The button of the picked turtle stands out.
pub fn update_selected(
    selected: Res<SelectedCharacter>,
    mut buttons: Query<(&MenuCharacterBtn, &mut MenuButton)>,
) {
    for (btn, mut kind) in buttons.iter_mut() {
        let MenuCharacterBtn::Pick(character) = btn else {
            continue;
        };
        let wanted = if *character == selected.0 {
            MenuButton::Primary
        } else {
            MenuButton::Secondary
        };
        if *kind != wanted {
            *kind = wanted;
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuCharacterUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
