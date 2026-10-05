use super::connect::ConnectData;
use super::ui::{self, MenuButton};
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
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(WinUI);

    // root node
    commands
        .spawn((ui::screen(), WinUI))
        .with_children(|parent| {
            // match result string
            parent.spawn(ui::title(font, match_data.result.clone()));
            parent.spawn(ui::spacer(10.));
            // a rematch goes back to the lobby of an online match, local matches have none
            if connect_data.is_some() {
                parent.spawn((
                    ui::button(font, "Rematch", MenuButton::Primary),
                    MenuWinBtn::Rematch,
                ));
            }
            // back to menu button
            parent.spawn((
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuWinBtn::Back,
            ));
        });

    commands.remove_resource::<MatchData>();
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
