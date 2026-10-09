use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Resource, Clone)]
pub struct AgreedRandom {
    pub rng: Pcg64,
}

impl AgreedRandom {
    pub fn new(peers: Vec<PeerId>) -> AgreedRandom {
        let mut tmp = peers.clone();
        tmp.sort();
        let seed = tmp.iter().fold(String::new(), |mut a, b| {
            a.reserve(b.0.to_string().len() + 1);
            a.push_str(b.0.to_string().as_str());
            a.push_str(" ");
            a.trim_end().to_string()
        });
        let rng: Pcg64 = Seeder::from(seed).into_rng();

        AgreedRandom { rng }
    }
}

#[derive(Resource)]
pub struct PlayersReady;

/// How long the notice about a player that disconnected stays on screen.
pub const DISCONNECT_NOTICE_SECS: f32 = 6.;

/// What the session told us about the connections to the other players, by handle.
/// This differs between players, so it is not part of the rollback state.
#[derive(Resource, Default)]
pub struct Connections {
    /// players we stopped hearing from, they are dropped if that goes on
    pub interrupted: BTreeSet<usize>,
    /// players that are gone for good, with how long that is still news
    pub disconnected: BTreeMap<usize, Timer>,
}

/// Handle of the player the camera follows once the local player has died.
/// Only the local player picks and sees this, so it is not part of the rollback state.
#[derive(Resource, Default)]
pub struct Spectating(pub Option<usize>);

/// Who won the previous round, they wear the party hat in the next one.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum PreviousWinner {
    #[default]
    None,
    /// handle of the winner of a local round, those are the same every round
    Handle(usize),
    /// the local player won an online round. Handles change between online rounds,
    /// so every player tells the lobby on their own whether they won
    Me,
}

#[derive(Debug, Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct HealthBarsAdded;
