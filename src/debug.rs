use bevy::prelude::*;

use bevy_ggrs::Session;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    menu::connect::LocalHandle,
    player::components::{
        EdibleSpawnTimer, Fireball, Player, PlayerHealth, PlayerSpeed, Strawberry,
    },
    player::input::GGRSConfig,
    GameState,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(WorldInspectorPlugin::new())
                .register_type::<LocalHandle>()
                .register_type::<EdibleSpawnTimer>()
                .register_type::<Fireball>()
                .register_type::<Strawberry>()
                .register_type::<Player>()
                .register_type::<PlayerHealth>()
                .register_type::<PlayerSpeed>()
                .add_system(log_ggrs_events.in_set(OnUpdate(GameState::RoundLocal)))
                .add_system(log_ggrs_events.in_set(OnUpdate(GameState::RoundOnline)));
        }
    }
}

pub fn log_ggrs_events(mut session: ResMut<Session<GGRSConfig>>) {
    match session.as_mut() {
        Session::P2PSession(s) => {
            for event in s.events() {
                info!("GGRS Event: {:?}", event);
            }
        }
        _ => (),
    }
}
