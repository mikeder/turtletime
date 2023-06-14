use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

#[derive(Component)]
pub struct ConsoleUI;

#[derive(Component)]
pub struct ConsoleText;

#[derive(Component, Copy, Clone, Default, Debug, Reflect, InspectorOptions)]
#[reflect(Component)]
pub struct ConsoleReady(pub bool);

#[derive(Resource)]

pub struct PeerInfo(pub String);

#[derive(Resource)]

pub struct EdibleCount(pub usize);

#[derive(Resource)]
pub struct ConsoleUpdateTimer(pub Timer);
