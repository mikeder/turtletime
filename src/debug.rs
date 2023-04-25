use bevy::prelude::*;

use bevy_ggrs::{GGRSSchedule, Session};
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
                .add_system(log_ggrs_events.in_set(OnUpdate(GameState::RoundOnline)))
                .add_system(increase_frame_system.in_schedule(GGRSSchedule));
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

#[derive(Resource, Default, Reflect, Hash)]
#[reflect(Resource, Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}
