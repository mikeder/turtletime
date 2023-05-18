use super::checksum::checksum_players;
use super::components::EdibleSpawnTimer;
use super::round::{cleanup_round, cleanup_session, disconnect_remote_players, setup_round};
use crate::player::systems::*;
use crate::{AppState, GameState};
use bevy::prelude::*;
use bevy_ggrs::GGRSSchedule;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EdibleSpawnTimer>()
            // round setup
            .add_systems(
                (setup_round, create_ui, spawn_players)
                    .in_set(SpawnSystemSet)
                    .in_schedule(OnEnter(GameState::Playing)),
            )
            .add_system(camera_follow.run_if(in_state(GameState::Playing)))
            // round cleanup
            .add_system(disconnect_remote_players.in_schedule(OnExit(AppState::RoundOnline)))
            .add_system(cleanup_round.in_schedule(OnEnter(AppState::Win)))
            .add_system(cleanup_session.in_schedule(OnExit(AppState::Win)))
            // stateless timers and UI text updates
            .add_systems(
                (
                    check_win_state,
                    update_player_health_text,
                    update_player_fireball_text,
                    update_player_speed_boost_text,
                )
                    .distributive_run_if(in_state(GameState::Playing)),
            )
            // these systems will be executed as part of the advance frame update
            // player rollback systems
            .add_systems(
                (
                    apply_inputs,
                    apply_player_sprint,
                    move_players,
                    checksum_players,
                    reload_fireballs,
                    shoot_fireballs,
                    move_fireballs,
                    fireball_damage_players,
                    kill_players,
                    player_poops,
                    player_stepped_in_poop,
                )
                    .chain()
                    .in_set(PlayerSystemSet)
                    .distributive_run_if(in_state(GameState::Playing))
                    .in_schedule(GGRSSchedule),
            )
            // edible rollback systems
            .add_systems(
                (
                    spawn_strawberry_over_time,
                    spawn_chili_pepper_over_time,
                    spawn_lettuce_over_time,
                    tick_edible_timer,
                    player_ate_chili_pepper_system,
                    player_ate_strawberry_system,
                    player_ate_lettuce_system,
                    despawn_old_fireballs,
                    despawn_old_poops,
                    tick_fireball_timers,
                    tick_poop_timers,
                )
                    .chain()
                    .in_set(EdibleSystemSet)
                    .after(SpawnSystemSet)
                    .after(PlayerSystemSet)
                    .distributive_run_if(in_state(GameState::Playing))
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
