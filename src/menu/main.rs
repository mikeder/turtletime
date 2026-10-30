use super::character::SelectedCharacter;
use super::connect::{ConnectData, LocalHandle, PlayerCharacters};
use super::online::PlayerCount;
use super::plugin::VERSION;
use super::ui::{self, MenuButton};
use crate::graphics::{Character, CharacterSheet};
use crate::loading::FontAssets;
use crate::platform::plugin::{self as platform, Account, AccountStatus, Motd, PlatformRequest};
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

/// The message of the day box, hidden while there is none.
#[derive(Component)]
pub struct MotdBox;

#[derive(Component)]
pub struct MotdText;

/// Says who is signed in, under the buttons.
#[derive(Component)]
pub struct AccountStatusText;

#[derive(Component)]
pub enum MainMenuBtn {
    OnlineMatch,
    LocalMatch,
    Character,
    Options,
    // only shown when the game was built with a platform application id
    Account,
    Leaderboards,
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
    motd: Res<Motd>,
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
            // message of the day, shown once the platform sends one
            parent.spawn((
                Node {
                    display: motd_display(&motd),
                    max_width: Val::Px(640.),
                    padding: UiRect::axes(Val::Px(20.), Val::Px(10.)),
                    border: UiRect::all(Val::Px(3.)),
                    border_radius: BorderRadius::all(Val::Px(8.)),
                    ..Default::default()
                },
                BackgroundColor(ui::SHADE),
                BorderColor::all(ui::MOSS_EDGE),
                MotdBox,
                children![(
                    ui::text(font, motd.text.clone(), ui::HINT_SIZE + 2., ui::CREAM),
                    TextLayout::new_with_justify(Justify::Center),
                    MotdText,
                )],
            ));
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
            // the less used buttons go in pairs, so everything fits on screen
            parent.spawn((
                ui::row(),
                children![
                    (
                        ui::button(font, "Character", MenuButton::Secondary),
                        MainMenuBtn::Character,
                    ),
                    (
                        ui::button(font, "Controls", MenuButton::Secondary),
                        MainMenuBtn::Options,
                    ),
                ],
            ));
            if platform::enabled() {
                parent.spawn((
                    ui::row(),
                    children![
                        (
                            ui::button(font, "Leaderboards", MenuButton::Secondary),
                            MainMenuBtn::Leaderboards,
                        ),
                        (
                            ui::button(font, "Sign in", MenuButton::Secondary),
                            MainMenuBtn::Account,
                        ),
                    ],
                ));
            }
            // there is nothing to quit to in a browser, the player just closes the tab
            #[cfg(not(target_arch = "wasm32"))]
            parent.spawn((
                ui::button(font, "Quit", MenuButton::Secondary),
                MainMenuBtn::Quit,
            ));
            parent.spawn((ui::hint(font, ""), AccountStatusText));
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
                MainMenuBtn::Account => {
                    app_state.set(AppState::MenuAccount);
                }
                MainMenuBtn::Leaderboards => {
                    app_state.set(AppState::MenuLeaderboard);
                }
                MainMenuBtn::Quit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}

/// The MOTD is asked for whenever the main menu opens, the platform plugin makes
/// sure that isn't too often.
pub fn fetch_motd(mut requests: MessageWriter<PlatformRequest>) {
    if platform::enabled() {
        requests.write(PlatformRequest::FetchMotd);
    }
}

fn motd_display(motd: &Motd) -> Display {
    if motd.text.is_empty() {
        Display::None
    } else {
        Display::Flex
    }
}

/// The MOTD usually arrives after the menu is shown.
pub fn update_motd(
    motd: Res<Motd>,
    mut boxes: Query<&mut Node, With<MotdBox>>,
    mut texts: Query<&mut Text, With<MotdText>>,
) {
    if !motd.is_changed() {
        return;
    }
    for mut node in boxes.iter_mut() {
        node.display = motd_display(&motd);
    }
    for mut text in texts.iter_mut() {
        text.0 = motd.text.clone();
    }
}

/// A saved session is restored in the background, so the account line can change
/// while the menu is open.
pub fn update_account_status(
    account: Res<Account>,
    mut status: Query<&mut Text, With<AccountStatusText>>,
    buttons: Query<(&MainMenuBtn, &Children)>,
    mut labels: Query<&mut Text, Without<AccountStatusText>>,
    new_menu: Query<(), Added<AccountStatusText>>,
) {
    if !account.is_changed() && new_menu.is_empty() {
        return;
    }
    let (line, label) = match account.status {
        AccountStatus::SignedIn => (format!("Signed in as {}", account.username), "Account"),
        AccountStatus::Working => ("Signing in...".to_owned(), "Account"),
        AccountStatus::SignedOut if platform::enabled() => {
            ("Playing without an account".to_owned(), "Sign in")
        }
        AccountStatus::SignedOut => (String::new(), "Sign in"),
    };
    for mut text in status.iter_mut() {
        text.0 = line.clone();
    }
    for (btn, children) in buttons.iter() {
        if !matches!(btn, MainMenuBtn::Account) {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = label.to_owned();
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
