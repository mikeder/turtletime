use super::ui::{self, MenuButton};
use crate::loading::{FontAssets, TextureAssets};
use crate::player::components::{
    CHILI_PEPPER_AMMO_COUNT, LETTUCE_HEALTH_GAIN, STRAWBERRY_AMMO_COUNT,
};
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuOptionsUI;

#[derive(Component)]
pub enum MenuOptionsBtn {
    Back,
}

pub fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    image_assets: Res<TextureAssets>,
) {
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuOptionsUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuOptionsUI,
        children![
            ui::heading(font, "Controls"),
            control_row(font, "W A S D", "Move"),
            control_row(font, "Space or Enter", "Shoot a fireball"),
            control_row(font, "Left Shift", "Sprint"),
            ui::spacer(10.),
            ui::heading(font, "Pickups"),
            pickup_row(
                font,
                &image_assets.texture_chili_pepper,
                format!("Chili pepper: {CHILI_PEPPER_AMMO_COUNT} fireballs"),
            ),
            pickup_row(
                font,
                &image_assets.texture_strawberry,
                format!("Strawberry: {STRAWBERRY_AMMO_COUNT} sprint boosts"),
            ),
            pickup_row(
                font,
                &image_assets.texture_lettuce,
                format!("Lettuce: restores {LETTUCE_HEALTH_GAIN} health"),
            ),
            ui::hint(font, "The last turtle standing wins"),
            // back button
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuOptionsBtn::Back,
            ),
        ],
    ));
}

const COLUMN_WIDTH: f32 = 330.;

/// Two columns that meet in the middle of the screen, so rows line up like a table.
fn columns() -> (impl Bundle, impl Bundle) {
    (
        Node {
            width: Val::Px(COLUMN_WIDTH),
            justify_content: JustifyContent::FlexEnd,
            ..Default::default()
        },
        Node {
            width: Val::Px(COLUMN_WIDTH),
            ..Default::default()
        },
    )
}

fn control_row(font: &Handle<Font>, keys: &'static str, action: &'static str) -> impl Bundle {
    let (left, right) = columns();
    (
        ui::row(),
        children![
            (
                left,
                children![ui::text(font, keys, ui::BODY_SIZE, ui::LETTUCE)]
            ),
            (right, children![ui::body(font, action)]),
        ],
    )
}

fn pickup_row(font: &Handle<Font>, image: &Handle<Image>, effect: String) -> impl Bundle {
    let (left, right) = columns();
    (
        ui::row(),
        children![
            (
                left,
                children![(
                    ImageNode::new(image.clone()),
                    Node {
                        width: Val::Px(32.),
                        height: Val::Px(32.),
                        ..Default::default()
                    },
                )]
            ),
            (right, children![ui::body(font, effect)]),
        ],
    )
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

pub fn cleanup_ui(query: Query<Entity, With<MenuOptionsUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
