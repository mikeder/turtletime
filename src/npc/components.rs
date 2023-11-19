use bevy::prelude::*;

pub const GOOSE_SPEED: i32 = 105;

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct Goose;

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct HasTarget;

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct EdibleTarget;
