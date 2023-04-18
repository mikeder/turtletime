use bevy::prelude::*;
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use bytemuck::{Pod, Zeroable};
use rand::rngs::StdRng;
use rand_seeder::Seeder;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(increase_frame_system.in_schedule(GGRSSchedule));
    }
}

#[derive(Debug)]
pub struct GGRSConfig;
impl ggrs::Config for GGRSConfig {
    type Input = PlayerInput;
    type State = u8;
    type Address = PeerId;
}

#[derive(Resource)]
pub struct AgreedRandom {
    pub rng: StdRng,
}

pub fn new_agreed_random(peers: Vec<PeerId>) -> AgreedRandom {
    let mut tmp = peers.clone();
    tmp.sort();
    let seed = tmp.iter().fold(String::new(), |mut a, b| {
        a.reserve(b.0.to_string().len() + 1);
        a.push_str(b.0.to_string().as_str());
        a.push_str(" ");
        a.trim_end().to_string()
    });
    let rng: StdRng = Seeder::from(seed).make_rng();

    AgreedRandom { rng }
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

pub fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> PlayerInput {
    let mut input = 0u8;

    if keys.any_pressed([KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::S]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::A]) {
        input |= INPUT_LEFT
    }
    if keys.any_pressed([KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
        input |= INPUT_FIRE;
    }
    if keys.any_pressed([KeyCode::Escape, KeyCode::Delete]) {
        input |= INPUT_EXIT;
    }

    PlayerInput { input }
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
