use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use rand::rngs::StdRng;
use rand_seeder::Seeder;

#[derive(Resource)]
pub struct AgreedRandom {
    pub rng: StdRng,
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
        let rng: StdRng = Seeder::from(seed).make_rng();

        AgreedRandom { rng }
    }
}

#[derive(Resource)]
pub struct PlayersReady;

#[derive(Debug, Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct HealthBarsAdded;
