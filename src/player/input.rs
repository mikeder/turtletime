use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use bytemuck::{Pod, Zeroable};

#[derive(Debug)]
pub struct GGRSConfig;
impl ggrs::Config for GGRSConfig {
    type Input = PlayerInput;
    type State = u8;
    type Address = PeerId;
}

#[derive(Clone, Default, Reflect, Component)]

pub struct PlayerControls {
    pub dir: Vec2,
    pub last_dir: Vec2,
    pub exiting: bool,
    pub shooting: bool,
    pub sprinting: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct PlayerInput {
    pub input: u8,
}

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_LEFT: u8 = 1 << 2;
pub const INPUT_RIGHT: u8 = 1 << 3;
pub const INPUT_FIRE: u8 = 1 << 4;
pub const INPUT_EXIT: u8 = 1 << 5;
pub const INPUT_SPRINT: u8 = 1 << 6;

pub fn input(mut commands: Commands, keys: Res<Input<KeyCode>>, local_players: Res<LocalPlayers>) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input: u8 = 0;

        if keys.pressed(KeyCode::W) {
            input |= INPUT_UP;
        }
        if keys.pressed(KeyCode::S) {
            input |= INPUT_DOWN;
        }
        if keys.pressed(KeyCode::A) {
            input |= INPUT_LEFT
        }
        if keys.pressed(KeyCode::D) {
            input |= INPUT_RIGHT;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
            input |= INPUT_FIRE;
        }
        if keys.any_pressed([KeyCode::Escape, KeyCode::Delete]) {
            input |= INPUT_EXIT;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            input |= INPUT_SPRINT;
        }

        local_inputs.insert(*handle, PlayerInput { input });
    }

    commands.insert_resource(LocalInputs::<GGRSConfig>(local_inputs));
}
