use super::online::PlayerCount;
use super::ui::{self, MenuButton};
use crate::loading::FontAssets;
use crate::player::input::GGRSConfig;
use crate::player::resources::AgreedRandom;
use crate::{AppState, GameState, FPS, INPUT_DELAY, MATCHBOX_ADDR, MAX_PREDICTION};
use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_matchbox::prelude::PeerState;
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

#[derive(Resource, Reflect, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct LocalHandle(pub usize);

#[derive(Resource)]
pub struct ConnectData {
    pub lobby_id: String,
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

pub fn create_matchbox_socket(mut commands: Commands, connect_data: Res<ConnectData>) {
    let lobby_id = &connect_data.lobby_id;
    let room_url = format!("{MATCHBOX_ADDR}/{lobby_id}");
    info!("connecting to matchbox server: {:?}", room_url);

    // remove old socket that may exist from previous round
    commands.remove_resource::<MatchboxSocket>();
    // insert new socket resource for next session
    // ggrs handles packet loss and ordering on its own, so the channel is unreliable
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
    commands.insert_resource(LobbyTimeout::default());
    // commands.remove_resource::<ConnectData>();
}

#[allow(clippy::too_many_arguments)]
pub fn lobby_system(
    mut commands: Commands,
    socket: Option<ResMut<MatchboxSocket>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut timeout: ResMut<LobbyTimeout>,
    time: Res<Time>,
    player_count: Res<PlayerCount>,
    mut query: Query<(&mut Text, &mut TextColor), With<LobbyText>>,
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
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }
    if !peer_changes.is_empty() {
        // the lobby is still filling up, give it more time
        timeout.0.reset();
    }

    let connected_peers = socket.connected_peers().count();
    let remaining = player_count.0 - (connected_peers + 1);
    if let Ok((mut text, _)) = query.single_mut() {
        let players = if remaining == 1 { "player" } else { "players" };
        let waiting = format!("Waiting for {remaining} more {players}");
        if text.0 != waiting {
            text.0 = waiting;
        }
    }
    if remaining > 0 {
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

    // set final player list
    let players = socket.players();

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

    for (i, player_type) in players.into_iter().enumerate() {
        if player_type == PlayerType::Local {
            info!("Adding local player {}", i);
            commands.insert_resource(LocalHandle(i));
        } else {
            info!("Adding remote player {}", i)
        }
        sess_build = sess_build
            .add_player(player_type.clone(), i)
            .expect("Invalid player added.");
    }

    // Start P2P session
    let channel = socket.take_channel(0).unwrap();
    let sess = sess_build
        .start_p2p_session(channel)
        .expect("Session could not be created.");

    commands.insert_resource(Session::P2P(sess));
    commands.insert_resource(AgreedRandom::new(peers));
    app_state.set(AppState::RoundOnline);
    game_state.set(GameState::Playing);
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

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    let font = &font_assets.fira_sans;

    // ui camera
    commands.spawn((Camera2d, Msaa::Off)).insert(MenuConnectUI);

    // root node
    commands.spawn((
        ui::screen(),
        MenuConnectUI,
        children![
            ui::heading(font, "Online match"),
            // lobby status display
            (
                ui::body(font, "Looking for players"),
                TextLayout::new_with_justify(Justify::Center),
                LobbyText,
            ),
            ui::spacer(10.),
            // back button
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                MenuConnectBtn::Back,
            ),
        ],
    ));
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
            // nothing is listening here, no peer will ever connect
            .insert_resource(MatchboxSocket::new_unreliable("ws://127.0.0.1:9/test"))
            .add_systems(Update, lobby_system);
        let text = app.world_mut().spawn((Text::default(), LobbyText)).id();

        // the first update has no time delta yet, so this stops just short of the timeout
        for _ in 0..LOBBY_TIMEOUT_SECS as usize * 10 {
            app.update();
        }
        assert!(app.world().contains_resource::<MatchboxSocket>());
        assert_eq!(
            app.world().get::<Text>(text).unwrap().0,
            "Waiting for 1 more player"
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
}
