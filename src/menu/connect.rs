use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::loading::FontAssets;
use crate::network::{self, GGRSConfig};
use crate::{GameState, FPS, INPUT_DELAY, MATCHBOX_ADDR, MAX_PREDICTION, NUM_PLAYERS};
use bevy::prelude::*;
use bevy::utils::Uuid;
use bevy_ggrs::Session;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_matchbox::prelude::{PeerId, PeerState, SingleChannel};
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

pub fn create_matchbox_socket(mut commands: Commands, connect_data: Res<ConnectData>) {
    let lobby_id = &connect_data.lobby_id;
    let room_url = format!("{MATCHBOX_ADDR}/{lobby_id}");
    info!("connecting to matchbox server: {:?}", room_url);
    let socket = MatchboxSocket::new_reliable(room_url);
    commands.insert_resource(socket); // TODO: Remove on exit menu?
    commands.remove_resource::<ConnectData>();
}

pub fn lobby_system(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut state: ResMut<NextState<GameState>>,
    mut query: Query<&mut Text, With<LobbyText>>,
) {
    // regularly call update_peers to update the list of connected peers
    for (peer, new_state) in socket.update_peers() {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }

    let connected_peers = socket.connected_peers().count();
    let remaining = NUM_PLAYERS - (connected_peers + 1);
    query.single_mut().sections[0].value = format!("Waiting for {remaining} more player(s)",);
    if remaining > 0 {
        return;
    }

    // set final player list
    let players = socket.players();

    let mut player_ids = Vec::new();
    for p in players.clone() {
        match p {
            PlayerType::Remote(id) => player_ids.push(id),
            PlayerType::Spectator(id) => player_ids.push(id),
            PlayerType::Local => (),
        }
    }
    // if we made it here we should have a local peer ID
    let local_id = match socket.id() {
        Some(id) => id,
        None => PeerId(Uuid::new_v4()), // TODO: something more reliable
    };
    player_ids.push(local_id);

    // Create GGRS P2P Session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
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

    // Create agreed random resource
    let agreed_random = network::new_agreed_random(player_ids);

    // Start P2P session
    let channel = socket.take_channel(0).unwrap();
    let sess = sess_build
        .start_p2p_session(channel)
        .expect("Session could not be created.");

    commands.insert_resource(Session::P2PSession(sess));
    commands.insert_resource(agreed_random);
    state.set(GameState::RoundOnline);
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // ui camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(MenuConnectUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.)),
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            // lobby id display
            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    text: Text::from_section(
                        "Searching a match...",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 32.,
                            color: BUTTON_TEXT,
                        },
                    ),
                    ..Default::default()
                })
                .insert(LobbyText);

            // back button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(NORMAL_BUTTON),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Back to Menu",
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuConnectBtn::Back);
        })
        .insert(MenuConnectUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuConnectBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
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

pub fn btn_listeners(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&Interaction, &MenuConnectBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            match btn {
                MenuConnectBtn::Back => {
                    state.set(GameState::MenuMain);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuConnectUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
