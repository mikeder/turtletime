use bevy::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{menu::connect::LocalHandle, player::FrameCount, player::Player};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(WorldInspectorPlugin::new())
                .register_type::<LocalHandle>()
                .register_type::<FrameCount>()
                .register_type::<Player>();
        }
    }
}
