use super::checksum::checksum_players;
use super::components::EdibleSpawnTimer;
use crate::player::systems::*;
use crate::GameState;
use bevy::prelude::*;
use bevy_ggrs::GGRSSchedule;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EdibleSpawnTimer>()
            .add_system(
                create_ui
                    .in_schedule(OnEnter(GameState::RoundLocal))
                    .in_set(SpawnSystemSet),
            )
            .add_system(
                create_ui
                    .in_schedule(OnEnter(GameState::RoundOnline))
                    .in_set(SpawnSystemSet),
            )
            .add_system(
                spawn_players
                    .in_schedule(OnEnter(GameState::RoundLocal))
                    .in_set(SpawnSystemSet),
            )
            .add_system(
                spawn_players
                    .in_schedule(OnEnter(GameState::RoundOnline))
                    .in_set(SpawnSystemSet),
            )
            .add_system(cleanup_round.in_schedule(OnExit(GameState::RoundLocal)))
            .add_system(cleanup_round.in_schedule(OnExit(GameState::RoundOnline)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundLocal)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundOnline)))
            // fireball timers only used for despawn of old fireballs
            .add_system(tick_fireball_timers)
            .add_system(tick_poop_timers)
            .add_system(check_win_state)
            .add_system(update_player_health_text)
            .add_system(update_player_fireball_text)
            .add_system(update_player_speed_boost_text)
            // these systems will be executed as part of the advance frame update
            // player rollback systems
            .add_systems(
                (
                    apply_inputs,
                    apply_player_sprint,
                    move_players,
                    checksum_players,
                    shoot_fireballs,
                    reload_fireballs,
                    move_fireballs,
                    damage_players,
                    kill_players,
                    player_poops,
                    player_stepped_in_poop,
                )
                    .chain()
                    .in_set(PlayerSystemSet)
                    .in_schedule(GGRSSchedule),
            )
            // edible rollback systems
            .add_systems(
                (
                    spawn_strawberry_over_time.run_if(resource_exists::<EdibleSpawnTimer>()),
                    spawn_chili_pepper_over_time.run_if(resource_exists::<EdibleSpawnTimer>()),
                    spawn_lettuce_over_time.run_if(resource_exists::<EdibleSpawnTimer>()),
                    tick_edible_timer.run_if(resource_exists::<EdibleSpawnTimer>()),
                    player_ate_chili_pepper_system,
                    player_ate_strawberry_system,
                    player_ate_lettuce_system,
                    despawn_old_fireballs,
                    despawn_old_poops,
                )
                    .chain()
                    .in_set(EdibleSystemSet)
                    .after(SpawnSystemSet)
                    .after(PlayerSystemSet)
                    .in_schedule(GGRSSchedule),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct EdibleSystemSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct PlayerSystemSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SpawnSystemSet;
