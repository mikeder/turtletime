use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::InspectorOptions;

pub const CHILI_PEPPER_SIZE: f32 = 20.0;
pub const CHILI_PEPPER_AMMO_COUNT: usize = 5;
const CHILI_PEPPER_SPAWN_RATE: f32 = 3.5;

pub const FIREBALL_RADIUS: f32 = 12.0;
pub const FIREBALL_DAMAGE: f32 = 5.0;
pub const FIREBALL_LIFETIME: f32 = 10.0;

pub const STRAWBERRY_SIZE: f32 = 32.0;
const STRAWBERRY_SPAWN_RATE: f32 = 2.5;

pub const PLAYER_HEALTH_MAX: f32 = 100.;
pub const PLAYER_SPEED_START: f32 = 100.;
pub const PLAYER_SPEED_MAX: f32 = 1000.;

#[derive(Component)]
pub struct RoundComponent;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Fireball {
    pub move_dir: Vec2,
    pub shot_by: usize,
    pub speed: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FireballTimer {
    pub lifetime: Timer,
}

impl Default for FireballTimer {
    fn default() -> Self {
        FireballTimer {
            lifetime: Timer::from_seconds(FIREBALL_LIFETIME, TimerMode::Once),
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct FireballReady(pub bool);

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct MoveDir(pub Vec2);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EdibleSpawnTimer {
    pub chili_pepper_timer: Timer,
    pub strawberry_timer: Timer,
}

impl Default for EdibleSpawnTimer {
    fn default() -> Self {
        EdibleSpawnTimer {
            chili_pepper_timer: Timer::from_seconds(CHILI_PEPPER_SPAWN_RATE, TimerMode::Repeating),
            strawberry_timer: Timer::from_seconds(STRAWBERRY_SPAWN_RATE, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ChiliPepper;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Strawberry;

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    pub active: bool,
    pub fire_ball_ammo: usize,
    pub sprint_ready: bool,
    pub sprint_ammo: usize,
    pub handle: usize,
    pub health: f32,
    pub just_moved: bool,
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            active: true,
            fire_ball_ammo: 0,
            sprint_ready: false,
            sprint_ammo: 0,
            handle: 0,
            health: PLAYER_HEALTH_MAX,
            just_moved: false,
            speed: PLAYER_SPEED_START,
        }
    }
}
