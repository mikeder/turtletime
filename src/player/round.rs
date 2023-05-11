use bevy::prelude::*;
use bevy_ggrs::{Rollback, Session};

use crate::{menu::connect::LocalHandle, player::components::EdibleSpawnTimer};

use super::{input::GGRSConfig, resources::AgreedRandom};

pub fn disconnect_remote_players(mut session: ResMut<Session<GGRSConfig>>) {
    match session.as_mut() {
        Session::P2PSession(s) => {
            for player_handle in s.remote_player_handles() {
                match s.disconnect_player(player_handle) {
                    Ok(_) => {
                        debug!("Force disconnect player: {:?}", player_handle)
                    }
                    Err(e) => {
                        error!("Disconnect player error: {:?}", e)
                    }
                }
            }
        }
        _ => (),
    }
}

pub fn cleanup_session(mut commands: Commands, rollback_query: Query<Entity, With<Rollback>>) {
    debug!("Cleanup Session");

    // // cleanup agreed random, players will get new ID's each round
    commands.remove_resource::<AgreedRandom>();

    // cleanup local handle, local player could get a different handle next round
    commands.remove_resource::<LocalHandle>();

    // remove edible spawn timer, we will spawn a new one each round
    commands.remove_resource::<EdibleSpawnTimer>();

    // finally remove old session
    commands.remove_resource::<Session<GGRSConfig>>();

    for e in rollback_query.iter() {
        debug!("Despawn entity: {:?}", e);
        commands.entity(e).despawn_recursive()
    }
}
