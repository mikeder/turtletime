use bevy::prelude::*;
use bevy_ggrs::{Rollback, Session};
use bevy_matchbox::{prelude::SingleChannel, MatchboxSocket};

use crate::{
    menu::connect::LocalHandle,
    player::{
        components::EdibleSpawnTimer,
        resources::{HealthBarsAdded, PlayersReady},
    },
};

use super::{
    components::{Expired, RoundComponent},
    input::GGRSConfig,
    resources::AgreedRandom,
};

pub fn setup_round(mut commands: Commands) {
    trace!("setup_round");

    commands.spawn((Camera2dBundle::default(), RoundComponent));
    commands.insert_resource(EdibleSpawnTimer::default());
}

pub fn disconnect_remote_players(
    mut session: ResMut<Session<GGRSConfig>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    trace!("disconnecting remote players...");
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
    debug!("checking socket stats...");
    socket.update_peers();
    for p in socket.connected_peers() {
        debug!("connected: {:?}", p)
    }
    for p in socket.disconnected_peers() {
        debug!("disconnected: {:?}", p)
    }
}

pub fn cleanup_round(
    mut commands: Commands,
    query: Query<Entity, (With<RoundComponent>, Without<Rollback>)>,
) {
    info!("Cleanup Round");

    let mut targets = query.iter().collect::<Vec<_>>();
    targets.sort_by_key(|e| *e);
    debug!(
        "number of non-rollback entities to remove: {:?}",
        targets.len()
    );

    for e in targets {
        debug!("Despawn entity: {:?}", e);
        commands.entity(e).despawn_recursive();
    }
}

pub fn cleanup_session(mut commands: Commands, query: Query<Entity, With<Rollback>>) {
    debug!("Cleanup Session");

    commands.remove_resource::<PlayersReady>();
    commands.remove_resource::<HealthBarsAdded>();

    // cleanup agreed random, players will get new ID's each round
    commands.remove_resource::<AgreedRandom>();

    // cleanup local handle, local player could get a different handle next round
    commands.remove_resource::<LocalHandle>();

    // finally remove old session
    commands.remove_resource::<Session<GGRSConfig>>();

    // remove edible spawn timer, we will spawn a new one each round
    commands.remove_resource::<EdibleSpawnTimer>();

    let mut targets = query.iter().collect::<Vec<_>>();
    targets.sort_by_key(|e| *e);
    debug!("number of rollback entities to remove: {:?}", targets.len());

    for e in targets {
        debug!("Despawn entity: {:?}", e);
        commands.entity(e).despawn_recursive()
    }
}

pub fn remove_expired(mut commands: Commands, expired_query: Query<Entity, With<Expired>>) {
    trace!("remove_expired");

    let mut expired = expired_query.iter().collect::<Vec<_>>();
    expired.sort_by_key(|e| *e);
    debug!("expiring entities: {:?}", expired.len());

    for e in expired {
        commands.entity(e).despawn_recursive();
        debug!("expired: {:?}", e);
    }
}
