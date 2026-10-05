use crate::audio::RollbackSound;
use crate::npc::components::{EdibleTarget, Goose, HasTarget};
use crate::player::checksum::Checksum;
use crate::player::components::{
    Edible, EdibleSpawnTimer, Expired, Fireball, FireballAmmo, FireballMovement, FireballReady,
    FireballTimer, Player, PlayerHealth, PlayerHealthBar, PlayerPoop, PlayerPoopTimer, PlayerSpeed,
    PlayerSpeedBoost, RoundComponent,
};
use crate::player::input::{input, GGRSConfig, PlayerControls};
use crate::player::resources::AgreedRandom;
use crate::FPS;
use bevy::prelude::*;
use bevy_ggrs::{GgrsPlugin, ReadInputs, RollbackApp, RollbackFrameRate};

pub struct RollbackPlugin;

/// This plugin sets up GGRS and registers everything that has to be saved and
/// restored when the game rolls back. Any component or resource that is changed
/// by a system in the `GgrsSchedule` must be registered here, otherwise peers
/// (and the local sync test) will desync.
impl Plugin for RollbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GgrsPlugin::<GGRSConfig>::default())
            .insert_resource(RollbackFrameRate(FPS))
            .add_systems(ReadInputs, input)
            .rollback_component_with_clone::<Checksum>()
            .rollback_component_with_clone::<Edible>()
            .rollback_component_with_clone::<EdibleTarget>()
            .rollback_component_with_clone::<Expired>()
            .rollback_component_with_clone::<Fireball>()
            .rollback_component_with_clone::<FireballAmmo>()
            .rollback_component_with_clone::<FireballReady>()
            .rollback_component_with_clone::<FireballMovement>()
            .rollback_component_with_clone::<FireballTimer>()
            .rollback_component_with_clone::<Goose>()
            .rollback_component_with_clone::<HasTarget>()
            .rollback_component_with_clone::<Player>()
            .rollback_component_with_clone::<PlayerHealth>()
            .rollback_component_with_clone::<PlayerHealthBar>()
            .rollback_component_with_clone::<PlayerSpeed>()
            .rollback_component_with_clone::<PlayerSpeedBoost>()
            .rollback_component_with_clone::<PlayerControls>()
            .rollback_component_with_clone::<PlayerPoop>()
            .rollback_component_with_clone::<PlayerPoopTimer>()
            .rollback_component_with_clone::<RollbackSound>()
            .rollback_component_with_clone::<RoundComponent>()
            .rollback_component_with_clone::<Transform>()
            .rollback_resource_with_clone::<EdibleSpawnTimer>()
            // edibles spawn at random positions, so the shared rng must roll back with the world
            .rollback_resource_with_clone::<AgreedRandom>();
    }
}
