use super::connect::ConnectData;
use super::online::{self, PlayerCount};
use super::plugin::VERSION;
use super::ui::{self, MenuButton};
use crate::loading::FontAssets;
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuQuickUI;

#[derive(Component)]
pub enum MenuQuickBtn {
    FindMatch,
    Back,
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuQuickUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuQuickUI,
        children![
            ui::heading(font, "Quick match"),
            online::player_count_picker(font),
            ui::hint(
                font,
                "You are matched with others looking for a match of this size"
            ),
            ui::spacer(10.),
            (
                ui::button(font, "Find match", MenuButton::Primary),
                MenuQuickBtn::FindMatch,
            ),
            (
                ui::button(font, "Back", MenuButton::Secondary),
                MenuQuickBtn::Back,
            ),
        ],
    ));
}

pub fn btn_listeners(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    player_count: Res<PlayerCount>,
    mut interaction_query: Query<(&Interaction, &MenuQuickBtn), Changed<Interaction>>,
) {
    // enter does the same as the find match button
    let mut find_match = keys.just_pressed(KeyCode::Enter);

    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuQuickBtn::FindMatch => find_match = true,
                MenuQuickBtn::Back => state.set(AppState::MenuOnline),
            }
        }
    }

    if find_match {
        commands.insert_resource(ConnectData {
            lobby_id: format!("turtletime_{}?next={}", VERSION, player_count.0),
            lobby_code: None,
        });
        state.set(AppState::MenuConnect);
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuQuickUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
