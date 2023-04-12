use super::plugin::{BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::loading::FontAssets;
use crate::network::GGRSConfig;
use crate::{GameState, FPS, INPUT_DELAY, MATCHBOX_ADDR, MAX_PREDICTION, NUM_PLAYERS};
use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_matchbox::prelude::SingleChannel;
use bevy_matchbox::MatchboxSocket;
use ggrs::{PlayerHandle, PlayerType, SessionBuilder};

#[derive(Component)]
pub struct MenuConnectUI;

#[derive(Component)]
pub enum MenuConnectBtn {
    Back,
}

#[derive(Resource)]
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
    commands.insert_resource(socket);
    commands.remove_resource::<ConnectData>();
}

pub fn update_matchbox_socket(
    commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    socket.update_peers();
    if socket.players().len() >= NUM_PLAYERS {
        create_ggrs_session(commands, socket);
        state.set(GameState::RoundOnline);
    }
}

// TODO: maybe not needed
// pub fn cleanup(mut commands: Commands) {
//     commands.remove_resource::<Option<WebRtcSocket>>();
// }

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
            parent.spawn(TextBundle {
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
            });

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

fn create_ggrs_session(
    mut commands: Commands,
    mut mb_socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    // create a new ggrs session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY);

    // add players
    for (i, player_type) in mb_socket.players().into_iter().enumerate() {
        if player_type == PlayerType::Local {
            commands.insert_resource(LocalHandle(i)); // track local player for camera follow, etc.
        }
        sess_build = sess_build
            .add_player(player_type.clone(), i)
            .expect("Invalid player added.");
    }

    // start the GGRS session
    let channel = mb_socket.take_channel(0).unwrap();

    let sess = sess_build
        .start_p2p_session(channel)
        .expect("Session could not be created.");

    commands.insert_resource(Session::P2PSession(sess));
}
