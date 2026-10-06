use super::online::{clean_name, PlayerCount, PlayerName};
use super::ui::{self, MenuButton};
use crate::loading::FontAssets;
use crate::player::input::GGRSConfig;
use crate::player::resources::AgreedRandom;
use crate::{AppState, GameState, FPS, INPUT_DELAY, MATCHBOX_ADDR, MAX_PREDICTION};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_matchbox::prelude::{PeerId, PeerState, WebRtcSocketBuilder};
use bevy_matchbox::MatchboxSocket;
use ggrs::{PlayerType, SessionBuilder};

#[derive(Component)]
pub struct MenuConnectUI;

#[derive(Component)]
pub enum MenuConnectBtn {
    Back,
}

#[derive(Component)]
pub struct LobbyText;

/// Lists who is in the lobby.
#[derive(Component)]
pub struct LobbyPlayersText;

/// ggrs sends the inputs of the round over this channel
const GAME_CHANNEL: usize = 0;
/// players tell each other their name over this channel while in the lobby
const NAME_CHANNEL: usize = 1;

/// Names the peers in the lobby sent us, a peer without one sends an empty name.
#[derive(Resource, Default)]
pub struct LobbyNames(pub HashMap<PeerId, String>);

/// Names of the players of an online round by handle, empty for players without one.
#[derive(Resource)]
pub struct PlayerNames(pub Vec<String>);

/// What to call a player, players that didn't pick a name are numbered.
pub fn player_label(names: Option<&PlayerNames>, handle: usize) -> String {
    let name = names.and_then(|names| names.0.get(handle));
    name_or_default(name.map_or("", String::as_str), handle)
}

/// The name a player picked or the default for their handle, "Player 1" to "Player 8".
fn name_or_default(name: &str, handle: usize) -> String {
    if name.is_empty() {
        // handles start at zero, people count from one
        format!("Player {}", handle + 1)
    } else {
        name.to_owned()
    }
}

fn open_socket(room_url: impl Into<String>) -> MatchboxSocket {
    // ggrs handles packet loss and ordering on its own, so its channel is unreliable,
    // a name is only sent once and has to arrive
    WebRtcSocketBuilder::new(room_url)
        .add_unreliable_channel()
        .add_reliable_channel()
        .into()
}

#[derive(Resource, Reflect, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct LocalHandle(pub usize);

#[derive(Resource)]
pub struct ConnectData {
    pub lobby_id: String,
    /// the code friends type to end up in the same lobby, quick matches have none
    pub lobby_code: Option<String>,
}

/// How long to wait for the lobby to fill up before giving up.
/// The wait starts over whenever a peer connects or disconnects.
const LOBBY_TIMEOUT_SECS: f32 = 60.;

#[derive(Resource)]
pub struct LobbyTimeout(pub Timer);

impl Default for LobbyTimeout {
    fn default() -> Self {
        LobbyTimeout(Timer::from_seconds(LOBBY_TIMEOUT_SECS, TimerMode::Once))
    }
}

/// How long a full lobby waits before the round starts, so players can see
/// who they are up against.
const LOBBY_COUNTDOWN_SECS: f32 = 3.;

#[derive(Resource)]
pub struct LobbyCountdown(pub Timer);

impl Default for LobbyCountdown {
    fn default() -> Self {
        LobbyCountdown(Timer::from_seconds(LOBBY_COUNTDOWN_SECS, TimerMode::Once))
    }
}

pub fn create_matchbox_socket(mut commands: Commands, connect_data: Res<ConnectData>) {
    let lobby_id = &connect_data.lobby_id;
    let room_url = format!("{MATCHBOX_ADDR}/{lobby_id}");
    info!("connecting to matchbox server: {:?}", room_url);

    // remove old socket that may exist from previous round
    commands.remove_resource::<MatchboxSocket>();
    // insert new socket resource for next session
    commands.insert_resource(open_socket(room_url));
    commands.insert_resource(LobbyNames::default());
    commands.insert_resource(LobbyTimeout::default());
    commands.insert_resource(LobbyCountdown::default());
    // commands.remove_resource::<ConnectData>();
}

#[allow(clippy::too_many_arguments)]
pub fn lobby_system(
    mut commands: Commands,
    socket: Option<ResMut<MatchboxSocket>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut timeout: ResMut<LobbyTimeout>,
    mut countdown: ResMut<LobbyCountdown>,
    time: Res<Time>,
    player_count: Res<PlayerCount>,
    player_name: Res<PlayerName>,
    mut lobby_names: ResMut<LobbyNames>,
    mut query: Query<(&mut Text, &mut TextColor), With<LobbyText>>,
    mut players_query: Query<&mut Text, (With<LobbyPlayersText>, Without<LobbyText>)>,
) {
    // the socket is closed when the lobby fails, the only way out is back to the menu
    let Some(mut socket) = socket else {
        return;
    };

    // regularly call update_peers to update the list of connected peers
    let peer_changes = match socket.try_update_peers() {
        Ok(peer_changes) => peer_changes,
        Err(e) => {
            warn!("matchbox socket closed: {:?}", e);
            close_lobby(
                &mut commands,
                &mut query,
                "Lost connection to the matchmaking server.\nGo back and try again.",
            );
            return;
        }
    };
    for (peer, new_state) in &peer_changes {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => {
                info!("peer {peer:?} connected");
                // tell the new peer who we are
                if let Ok(channel) = socket.get_channel_mut(NAME_CHANNEL) {
                    if let Err(e) = channel.try_send(player_name.0.as_bytes().into(), *peer) {
                        warn!("could not send name to {peer:?}: {:?}", e);
                    }
                }
            }
            PeerState::Disconnected => {
                info!("peer {peer:?} disconnected");
                lobby_names.0.remove(peer);
            }
        }
    }
    if let Ok(channel) = socket.get_channel_mut(NAME_CHANNEL) {
        for (peer, packet) in channel.receive() {
            let name = clean_name(&String::from_utf8_lossy(&packet));
            lobby_names.0.insert(peer, name);
        }
    }
    if !peer_changes.is_empty() {
        // the lobby is still filling up, give it more time
        timeout.0.reset();
    }

    // players are listed by handle, which is also the number in their default name,
    // the numbers can still shift while the lobby fills up
    let players = socket.players();
    let mut names_missing = false;
    let lines = players
        .iter()
        .enumerate()
        .map(|(i, player_type)| match player_type {
            PlayerType::Local => format!("{} (you)", name_or_default(&player_name.0, i)),
            PlayerType::Remote(id) | PlayerType::Spectator(id) => match lobby_names.0.get(id) {
                Some(name) => name_or_default(name, i),
                None => {
                    // the round needs the name of every player, so it can't start
                    // before they are in
                    names_missing = true;
                    "Connecting...".to_owned()
                }
            },
        })
        .collect::<Vec<_>>()
        .join("\n");
    if let Ok(mut text) = players_query.single_mut() {
        if text.0 != lines {
            text.0 = lines;
        }
    }

    let remaining = player_count.0 - players.len();
    if remaining > 0 || names_missing {
        // a countdown that was running is off, someone left
        countdown.0.reset();
        let waiting = if remaining > 0 {
            let players = if remaining == 1 { "player" } else { "players" };
            format!("Waiting for {remaining} more {players}")
        } else {
            "Waiting for player names".to_owned()
        };
        set_lobby_text(&mut query, waiting);
        // a peer we can't reach never shows up as connected and matchbox doesn't
        // report the failure, so all we can do is stop waiting at some point
        if timeout.0.tick(time.delta()).is_finished() {
            warn!("lobby timed out waiting for {remaining} more player(s)");
            close_lobby(
                &mut commands,
                &mut query,
                "Could not connect to all players.\nGo back and try again.",
            );
        }
        return;
    }

    // everyone is here, give them a moment to see who they are up against
    if !countdown.0.tick(time.delta()).is_finished() {
        let seconds = countdown.0.remaining_secs().ceil() as u32;
        set_lobby_text(&mut query, format!("Starting in {seconds}"));
        return;
    }

    // the player list is final now
    let mut peers = Vec::new();
    for p in players.clone() {
        match p {
            PlayerType::Remote(id) => peers.push(id),
            PlayerType::Spectator(id) => peers.push(id),
            PlayerType::Local => (),
        }
    }
    // if we made it here we should have a local peer ID
    match socket.id() {
        Some(id) => peers.push(id),
        None => (), // TODO: something more reliable
    };

    // Create GGRS P2P Session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(player_count.0)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: 10 })
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY);

    let mut names = Vec::new();
    for (i, player_type) in players.into_iter().enumerate() {
        match &player_type {
            PlayerType::Local => {
                info!("Adding local player {}", i);
                commands.insert_resource(LocalHandle(i));
                names.push(player_name.0.clone());
            }
            PlayerType::Remote(id) | PlayerType::Spectator(id) => {
                info!("Adding remote player {}", i);
                names.push(lobby_names.0.get(id).cloned().unwrap_or_default());
            }
        }
        sess_build = sess_build
            .add_player(player_type.clone(), i)
            .expect("Invalid player added.");
    }

    // Start P2P session
    let channel = socket.take_channel(GAME_CHANNEL).unwrap();
    let sess = sess_build
        .start_p2p_session(channel)
        .expect("Session could not be created.");

    commands.insert_resource(Session::P2P(sess));
    commands.insert_resource(AgreedRandom::new(peers));
    commands.insert_resource(PlayerNames(names));
    app_state.set(AppState::RoundOnline);
    game_state.set(GameState::Playing);
}

fn set_lobby_text(query: &mut Query<(&mut Text, &mut TextColor), With<LobbyText>>, value: String) {
    if let Ok((mut text, _)) = query.single_mut() {
        if text.0 != value {
            text.0 = value;
        }
    }
}

/// Give up on the lobby: leave the room so other players stop waiting on us
/// and tell the player what happened.
fn close_lobby(
    commands: &mut Commands,
    query: &mut Query<(&mut Text, &mut TextColor), With<LobbyText>>,
    reason: &str,
) {
    commands.remove_resource::<MatchboxSocket>();
    if let Ok((mut text, mut color)) = query.single_mut() {
        text.0 = reason.to_owned();
        color.0 = ui::CHILI;
    }
}

pub fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    connect_data: Res<ConnectData>,
) {
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuConnectUI);

    // root node
    commands
        .spawn((ui::screen(), MenuConnectUI))
        .with_children(|parent| {
            parent.spawn(ui::heading(font, "Online match"));
            // lobby status display
            parent.spawn((
                ui::body(font, "Looking for players"),
                TextLayout::new_with_justify(Justify::Center),
                LobbyText,
            ));
            // who is in the lobby
            parent.spawn((
                ui::text(font, "", ui::BODY_SIZE, ui::LETTUCE),
                TextLayout::new_with_justify(Justify::Center),
                LobbyPlayersText,
            ));
            // lobby code display, so the player can pass it on while waiting
            if let Some(lobby_code) = &connect_data.lobby_code {
                parent.spawn((
                    ui::column(),
                    children![
                        ui::body(font, "Lobby code"),
                        ui::text(font, lobby_code.clone(), ui::HEADING_SIZE, ui::LETTUCE),
                        ui::hint(font, "Friends join by typing this code"),
                    ],
                ));
            }
            parent.spawn(ui::spacer(10.));
            // back button
            parent.spawn((
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuConnectBtn::Back,
            ));
        });
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuConnectBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuConnectBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuConnectUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn lobby_gives_up_when_it_does_not_fill() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<AppState>()
            .init_state::<GameState>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .insert_resource(PlayerCount(2))
            .insert_resource(LobbyTimeout::default())
            .insert_resource(LobbyCountdown::default())
            .insert_resource(PlayerName::default())
            .insert_resource(LobbyNames::default())
            // nothing is listening here, no peer will ever connect
            .insert_resource(open_socket("ws://127.0.0.1:9/test"))
            .add_systems(Update, lobby_system);
        let text = app.world_mut().spawn((Text::default(), LobbyText)).id();
        let players = app
            .world_mut()
            .spawn((Text::default(), LobbyPlayersText))
            .id();

        // the first update has no time delta yet, so this stops just short of the timeout
        for _ in 0..LOBBY_TIMEOUT_SECS as usize * 10 {
            app.update();
        }
        assert!(app.world().contains_resource::<MatchboxSocket>());
        assert_eq!(
            app.world().get::<Text>(text).unwrap().0,
            "Waiting for 1 more player"
        );
        assert_eq!(
            app.world().get::<Text>(players).unwrap().0,
            "Player 1 (you)"
        );

        app.update();
        app.update();
        assert!(!app.world().contains_resource::<MatchboxSocket>());
        assert_eq!(
            app.world().get::<Text>(text).unwrap().0,
            "Could not connect to all players.\nGo back and try again."
        );
        assert_eq!(
            *app.world().resource::<State<AppState>>().get(),
            AppState::Loading
        );
    }

    #[test]
    fn players_without_a_name_are_numbered() {
        let names = PlayerNames(vec!["Shelly".to_owned(), String::new()]);
        assert_eq!(player_label(Some(&names), 0), "Shelly");
        assert_eq!(player_label(Some(&names), 1), "Player 2");
        assert_eq!(player_label(None, 0), "Player 1");
    }

    #[test]
    fn names_from_the_network_are_cleaned() {
        assert_eq!(clean_name("  Shelly\n"), "Shelly");
        assert_eq!(clean_name("a very long turtle name"), "a very long");
    }
}
