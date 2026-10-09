use super::character::SelectedCharacter;
use super::connect::{ConnectData, LocalHandle, PlayerCharacters};
use super::online::PlayerCount;
use super::plugin::VERSION;
use super::ui::{self, MenuButton};
use crate::graphics::{Character, CharacterSheet};
use crate::loading::FontAssets;
use crate::player::input::GGRSConfig;
use crate::player::resources::{AgreedRandom, PreviousWinner};
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
    Character,
    Options,
    // not shown on the web build
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Quit,
}

pub fn setup_ui(
    mut commands: Commands,
    characters: Res<CharacterSheet>,
    selected: Res<SelectedCharacter>,
    font_assets: Res<FontAssets>,
    player_count: Option<Res<PlayerCount>>,
    mut previous_winner: ResMut<PreviousWinner>,
) {
    // default player count
    if player_count.is_none() {
        commands.insert_resource(PlayerCount(2));
    }
    // the winner of an online round only keeps the party hat for a rematch
    if *previous_winner == PreviousWinner::Me {
        *previous_winner = PreviousWinner::None;
    }
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MainMenuUI);

    // root node
    commands
        .spawn((ui::screen(), MainMenuUI))
        .with_children(|parent| {
            parent.spawn(ui::title(font, "Turtle Time!"));
            // logo
            parent.spawn((
                ImageNode::from_atlas_image(
                    characters.turtle_image(selected.0),
                    TextureAtlas {
                        layout: characters.turtle_layout.clone(),
                        index: characters.turtle_frames[0],
                    },
                ),
                Node {
                    width: Val::Px(128.0),
                    height: Val::Px(128.0),
                    margin: UiRect::bottom(Val::Px(8.)),
                    ..Default::default()
                },
            ));
            parent.spawn((
                ui::button(font, "Online match", MenuButton::Primary),
                MainMenuBtn::OnlineMatch,
            ));
            parent.spawn((
                ui::button(font, "Local match", MenuButton::Secondary),
                MainMenuBtn::LocalMatch,
            ));
            parent.spawn((
                ui::button(font, "Character", MenuButton::Secondary),
                MainMenuBtn::Character,
            ));
            parent.spawn((
                ui::button(font, "Controls", MenuButton::Secondary),
                MainMenuBtn::Options,
            ));
            // there is nothing to quit to in a browser, the player just closes the tab
            #[cfg(not(target_arch = "wasm32"))]
            parent.spawn((
                ui::button(font, "Quit", MenuButton::Secondary),
                MainMenuBtn::Quit,
            ));
            parent.spawn(ui::hint(font, format!("Version {VERSION}")));
        });
}

pub fn btn_listeners(
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_count: Res<PlayerCount>,
    selected: Res<SelectedCharacter>,
    previous_winner: Res<PreviousWinner>,
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
                    // every turtle of a local round is played by the local player
                    let characters = (0..player_count.0)
                        .map(|handle| {
                            if *previous_winner == PreviousWinner::Handle(handle) {
                                Character::PartyHat
                            } else {
                                selected.0
                            }
                        })
                        .collect();
                    commands.insert_resource(PlayerCharacters(characters));
                    app_state.set(AppState::RoundLocal);
                    game_state.set(GameState::Playing);
                }
                MainMenuBtn::Character => {
                    app_state.set(AppState::MenuCharacter);
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
