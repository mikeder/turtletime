use bevy::prelude::*;
use bevy_ggrs::GGRSSchedule;

use crate::{
    player::plugin::{EdibleSystemSet, PlayerSystemSet, SpawnSystemSet},
    GameState,
};

use super::systems::{
    geese_target_closest_edible, goose_ate_edible, move_geese_toward_target, spawn_geese,
};

pub struct GoosePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct NpcSystemSet;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for GoosePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_geese.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    geese_target_closest_edible,
                    move_geese_toward_target,
                    goose_ate_edible,
                )
                    .chain()
                    .in_set(NpcSystemSet)
                    .after(EdibleSystemSet)
                    .after(SpawnSystemSet)
                    .after(PlayerSystemSet)
                    .distributive_run_if(in_state(GameState::Playing))
                    .in_schedule(GGRSSchedule),
            );
    }
}
