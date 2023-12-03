use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use percentage::Percentage;

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
pub const PLAYER_SPEED_BOOST_MAX: i32 = 25;
pub const PLAYER_SPEED_MAX: i32 = 800;

pub const POOP_SIZE: f32 = 16.0;
pub const POOP_DAMAGE: i32 = 2;
pub const POOP_LIFETIME: f32 = 10.0;

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub enum Edible {
    #[default]
    PLACEHOLDER,
    ChiliPepper,
    Strawberry,
    Lettuce,
}

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct RoundComponent;

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct Fireball {
    pub shot_by: usize,
}

#[derive(Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballMovement {
    pub dir: Vec2,
    pub speed: f32,
}

#[derive(Clone, Component, Reflect)]
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

#[derive(Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct EdibleSpawnTimer {
    // TODO: impl hash for sync test?
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

#[derive(Clone, Component, Default, Reflect, Hash)]
#[reflect(Component, Hash)]
pub struct PlayerPoop {
    pub shat_by: usize,
}

#[derive(Clone, Component, Reflect)]
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

#[derive(Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballAmmo(pub i32);

#[derive(Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct FireballReady(pub bool);

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions, Hash)]
#[reflect(Component, InspectorOptions, Hash)]
pub struct Player {
    pub active: bool,
    pub handle: usize,
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions, Hash)]
#[reflect(Component, InspectorOptions, Hash)]
pub struct PlayerSpeed(pub i32);

impl Default for PlayerSpeed {
    fn default() -> Self {
        PlayerSpeed(PLAYER_SPEED_START)
    }
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions, Hash)]
#[reflect(Component, InspectorOptions, Hash)]
pub struct PlayerSpeedBoost(pub i32);

impl Default for PlayerSpeedBoost {
    fn default() -> Self {
        PlayerSpeedBoost(0)
    }
}

#[derive(Component, Copy, Clone, Debug, Reflect, InspectorOptions, Hash)]
#[reflect(Component, InspectorOptions, Hash)]
pub struct PlayerHealth(pub i32);

impl Default for PlayerHealth {
    fn default() -> Self {
        PlayerHealth(PLAYER_HEALTH_MAX)
    }
}

impl PlayerHealth {
    pub fn decimal(self) -> f32 {
        self.percentage() as f32 / 100.
    }

    pub fn percentage(self) -> i32 {
        Percentage::from(PLAYER_HEALTH_MAX)
            .apply_to(self.0)
            .clamp(0, 100)
    }
}

#[derive(Clone, Component, Debug, Hash, Reflect)]
#[reflect(Hash)]

pub struct PlayerHealthBar {
    pub health_entity: Entity,
}

impl Default for PlayerHealthBar {
    fn default() -> Self {
        PlayerHealthBar {
            health_entity: Entity::PLACEHOLDER,
        }
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

#[derive(Clone, Component, Default, Hash, Reflect)]
#[reflect(Component, Hash)]
pub struct Expired;
