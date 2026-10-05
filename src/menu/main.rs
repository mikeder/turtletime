use super::connect::{ConnectData, LocalHandle};
use super::online::PlayerCount;
use super::plugin::VERSION;
use super::ui::{self, MenuButton};
use crate::loading::{FontAssets, TextureAssets};
use crate::player::input::GGRSConfig;
use crate::player::resources::AgreedRandom;
use crate::{AppState, GameState, CHECK_DISTANCE, FPS, INPUT_DELAY, MAX_PREDICTION};
use bevy::asset::uuid::Uuid;
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
        commands.insert_resource(PlayerCount(2));
    }
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MainMenuUI);

    // root node
    commands.spawn((
        ui::screen(),
        MainMenuUI,
        children![
            ui::title(font, "Turtle Time!"),
            // logo
            (
                ImageNode::new(image_assets.texture_turtle_cheeks2.clone()),
                Node {
                    width: Val::Px(128.0),
                    height: Val::Px(128.0),
                    margin: UiRect::bottom(Val::Px(8.)),
                    ..Default::default()
                },
            ),
            (
                ui::button(font, "Online match", MenuButton::Primary),
                MainMenuBtn::OnlineMatch,
            ),
            (
                ui::button(font, "Local match", MenuButton::Secondary),
                MainMenuBtn::LocalMatch,
            ),
            (
                ui::button(font, "Controls", MenuButton::Secondary),
                MainMenuBtn::Options,
            ),
            (
                ui::button(font, "Quit", MenuButton::Secondary),
                MainMenuBtn::Quit,
            ),
            ui::hint(font, format!("Version {VERSION}")),
        ],
    ));
}

pub fn btn_listeners(
    mut exit: MessageWriter<AppExit>,
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
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MainMenuUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
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
