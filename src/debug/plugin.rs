use bevy::prelude::*;

use bevy_ggrs::Session;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    menu::connect::LocalHandle,
    npc::components::EdibleTarget,
    player::input::GGRSConfig,
    player::{
        checksum::Checksum,
        components::{Edible, EdibleSpawnTimer, Player, PlayerHealth},
    },
    AppState, GameState,
};

use super::components::{ConsoleReady, ConsoleUpdateTimer, EdibleCount, PeerInfo};
use super::console::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugins(WorldInspectorPlugin::new())
                .register_type::<Checksum>()
                .register_type::<ConsoleReady>()
                .register_type::<LocalHandle>()
                .register_type::<EdibleTarget>()
                .register_type::<Edible>()
                .register_type::<EdibleSpawnTimer>()
                .register_type::<Player>()
                .register_type::<PlayerHealth>();
        }
    }
}

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Loading), setup_ui)
            .add_systems(Update, log_ggrs_events.run_if(in_state(GameState::Playing)))
            .add_systems(Update, open_console)
            .add_systems(
                Update,
                count_edibles.run_if(resource_exists::<EdibleCount>()),
            )
            .add_systems(
                Update,
                update_console_text.run_if(resource_exists::<PeerInfo>()),
            )
            .add_systems(
                Update,
                reset_console_ready.run_if(resource_exists::<PeerInfo>()),
            )
            .add_systems(
                Update,
                update_peer_info.run_if(resource_exists::<Session<GGRSConfig>>()),
            );
    }
}

pub fn count_edibles(mut edible_count: ResMut<EdibleCount>, edible_query: Query<&Edible>) {
    edible_count.0 = edible_query.iter().collect::<Vec<_>>().len();
}

pub fn update_peer_info(
    session: Res<Session<GGRSConfig>>,
    time: Res<Time>,
    mut timer: ResMut<ConsoleUpdateTimer>,
    mut peer_info: ResMut<PeerInfo>,
) {
    // only update when timer finishes
    if timer.0.tick(time.delta()).just_finished() {
        let mut tmp = String::new();
        match session.as_ref() {
            Session::P2P(s) => {
                for player_handle in s.remote_player_handles() {
                    let stats = match s.network_stats(player_handle) {
                        Ok(res) => {
                            format!("{:?}", res)
                        }
                        Err(e) => {
                            format!("{:?}", e)
                        }
                    };
                    let line = format!("Player {:?}: {:?}", player_handle, stats);
                    tmp.reserve(line.len() + 1);
                    tmp.push_str("\n");
                    tmp.push_str(&line);
                    tmp.push_str("\n");
                }
            }
            Session::SyncTest(s) => {
                let line = format!("Local players {}", s.num_players());
                tmp.reserve(line.len() + 1);
                tmp.push_str("\n");
                tmp.push_str(&line);
                tmp.push_str("\n");
            }
            _ => (),
        }
        peer_info.0 = tmp;
    }
}

pub fn log_ggrs_events(mut session: ResMut<Session<GGRSConfig>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                info!("GGRS Event: {:?}", event);
            }
        }
        _ => (),
    }
}
