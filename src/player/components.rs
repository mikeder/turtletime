use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::InspectorOptions;

pub const CHILI_PEPPER_SIZE: f32 = 20.0;
pub const CHILI_PEPPER_AMMO_COUNT: i32 = 5;
const CHILI_PEPPER_SPAWN_RATE: f32 = 2.5;

pub const FIREBALL_RADIUS: f32 = 12.0;
pub const FIREBALL_DAMAGE: i32 = 5;
pub const FIREBALL_LIFETIME: f32 = 10.0;

pub const STRAWBERRY_SIZE: f32 = 32.0;
pub const STRAWBERRY_AMMO_COUNT: i32 = 5;
const STRAWBERRY_SPAWN_RATE: f32 = 3.5;

pub const LETTUCE_SIZE: f32 = 32.0;
pub const LETTUCE_HEALTH_GAIN: i32 = 10;
const LETTUCE_SPAWN_RATE: f32 = 5.;

pub const PLAYER_HEALTH_MAX: i32 = 100;
pub const PLAYER_HEALTH_MID: i32 = PLAYER_HEALTH_MAX / 2;
pub const PLAYER_HEALTH_LOW: i32 = PLAYER_HEALTH_MAX / 4;
pub const PLAYER_SPEED_START: i32 = 100;
pub const PLAYER_SPEED_BOOST: i32 = 25;
pub const PLAYER_SPEED_MAX: i32 = 800;

pub const POOP_SIZE: f32 = 16.0;
pub const POOP_DAMAGE: i32 = 2;
pub const POOP_LIFETIME: f32 = 10.0;

#[derive(Component)]
pub struct RoundComponent;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Fireball {
    pub shot_by: usize,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballMovement {
    pub dir: Vec2,
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

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EdibleSpawnTimer {
    pub chili_pepper_timer: Timer,
    pub strawberry_timer: Timer,
    pub lettuce_timer: Timer,
}

impl Default for EdibleSpawnTimer {
    fn default() -> Self {
        EdibleSpawnTimer {
            chili_pepper_timer: Timer::from_seconds(CHILI_PEPPER_SPAWN_RATE, TimerMode::Repeating),
            strawberry_timer: Timer::from_seconds(STRAWBERRY_SPAWN_RATE, TimerMode::Repeating),
            lettuce_timer: Timer::from_seconds(LETTUCE_SPAWN_RATE, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ChiliPepper;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PlayerPoop {
    pub shat_by: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerPoopTimer {
    pub lifetime: Timer,
}

impl Default for PlayerPoopTimer {
    fn default() -> Self {
        PlayerPoopTimer {
            lifetime: Timer::from_seconds(POOP_LIFETIME, TimerMode::Once),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballAmmo(pub i32);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballReady(pub bool);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Strawberry;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Lettuce;

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    pub active: bool,
    pub handle: usize,
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct PlayerSpeed(pub i32);

impl Default for PlayerSpeed {
    fn default() -> Self {
        PlayerSpeed(PLAYER_SPEED_START)
    }
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct PlayerSpeedBoost(pub i32);

impl Default for PlayerSpeedBoost {
    fn default() -> Self {
        PlayerSpeedBoost(0)
    }
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct PlayerHealth(pub i32);

impl Default for PlayerHealth {
    fn default() -> Self {
        PlayerHealth(PLAYER_HEALTH_MAX)
    }
}

#[derive(Component)]
pub struct PlayerHealthText;
#[derive(Component)]
pub struct PlayerFireballText;
#[derive(Component)]
pub struct PlayerSpeedBoostText;

impl Default for Player {
    fn default() -> Self {
        Player {
            active: true,
            handle: 0,
        }
    }
}