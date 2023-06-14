use bevy::prelude::*;

pub const GOOSE_SPEED: i32 = 105;

#[derive(Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct Goose;

#[derive(Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct HasTarget;

#[derive(Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct EdibleTarget;
