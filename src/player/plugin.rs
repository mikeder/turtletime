use super::checksum::checksum_players;
use crate::player::systems::*;
use crate::GameState;
use bevy::prelude::*;
use bevy_ggrs::GGRSSchedule;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(init_round.before(create_ui))
            .add_system(create_ui.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(create_ui.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(spawn_players.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(cleanup_round.in_schedule(OnExit(GameState::RoundLocal)))
            .add_system(cleanup_round.in_schedule(OnExit(GameState::RoundOnline)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundLocal)))
            .add_system(camera_follow.run_if(in_state(GameState::RoundOnline)))
            // fireball timers only used for despawn of old fireballs
            .add_system(tick_fireball_timers)
            .add_system(check_win_state)
            // these systems will be executed as part of the advance frame update
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
                    spawn_strawberry_over_time,
                    spawn_chili_pepper_over_time,
                    player_ate_chili_pepper_system,
                    player_ate_strawberry_system,
                    despawn_old_fireballs,
                    tick_edible_timer,
                )
                    .chain()
                    .in_schedule(GGRSSchedule),
            );
    }
}
