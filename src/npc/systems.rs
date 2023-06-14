use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ggrs::RollbackIdProvider;
use rand::Rng;

use crate::{
    debug,
    graphics::{CharacterSheet, FrameAnimation},
    map::tilemap::EncounterSpawner,
    npc::components::{EdibleTarget, Goose, HasTarget},
    player::{
        components::{Edible, Expired, RoundComponent},
        resources::AgreedRandom,
    },
    FPS, TILE_SIZE,
};

use super::components::GOOSE_SPEED;

pub fn spawn_geese(
    mut commands: Commands,
    characters: Res<CharacterSheet>,
    mut rip: ResMut<RollbackIdProvider>,
    mut agreed_seed: ResMut<AgreedRandom>,
    spawner_query: Query<&Transform, With<EncounterSpawner>>,
) {
    trace!("spawn_geese");

    let spawn_area: Vec<&Transform> = spawner_query.iter().collect();
    let idx = agreed_seed.rng.gen_range(0..spawn_area.len());
    let pos = spawn_area[idx].translation;

    let mut sprite = TextureAtlasSprite::new(characters.goose_frames[0]);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 2.));

    commands.spawn((
        Name::new("Goose"),
        SpriteSheetBundle {
            sprite,
            texture_atlas: characters.goose_handle.clone(),
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 1.),
                ..Default::default()
            },
            ..Default::default()
        },
        FrameAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frames: characters.goose_frames.to_vec(),
            current_frame: 0,
        },
        Goose,
        RoundComponent,
        rip.next(),
    ));
}

pub fn geese_target_closest_edible(
    mut commands: Commands,
    edible_query: Query<(Entity, &Transform), (With<Edible>, Without<Expired>)>,
    goose_query: Query<(Entity, &Transform), (With<Goose>, Without<HasTarget>)>,
) {
    trace!("geese_target_closest_edible");

    for (goose_entity, transform) in goose_query.iter() {
        let mut closest_entity = Entity::PLACEHOLDER;
        let mut closest_transform = Vec3::splat(0.);
        let goose_pos = transform.translation;

        // find closest edible entity
        for (edible_entity, edible_transform) in edible_query.iter() {
            // replace placeholder values on first iteration to avoid failing to
            // find a target in certain locations
            if closest_entity == Entity::PLACEHOLDER && edible_entity != Entity::PLACEHOLDER {
                closest_entity = edible_entity;
                closest_transform = edible_transform.translation;
            }

            let distance_to_next = goose_pos.distance(edible_transform.translation);
            let distance_to_previous = goose_pos.distance(closest_transform);

            if distance_to_next < distance_to_previous {
                closest_entity = edible_entity;
                closest_transform = edible_transform.translation;
            }
        }

        if closest_entity == Entity::PLACEHOLDER {
            // do not target placeholder entities
            // they may be outside of the map
            debug!("skip target placeholder entity");
            continue;
        }

        debug!(
            "goose targeting closest entity {:?} at distance {:?}",
            closest_entity,
            goose_pos.distance(closest_transform)
        );
        commands.entity(closest_entity).insert(EdibleTarget);
        commands.entity(goose_entity).insert(HasTarget);
    }
}

pub fn move_geese_toward_target(
    mut commands: Commands,
    target_query: Query<(Entity, &Transform), (With<EdibleTarget>, Without<Expired>)>,
    mut goose_query: Query<
        (Entity, &mut Transform, &mut TextureAtlasSprite),
        (With<Goose>, With<HasTarget>, Without<EdibleTarget>),
    >,
) {
    trace!("move_geese_toward_target");

    // collect and sort all targets in play so we move towards them in a deterministic order
    let mut targets = target_query.iter().collect::<Vec<_>>();
    targets.sort_by_key(|e| e.0);
    debug!("number of targets: {:?}", targets.len());

    for (goose_entity, mut goose_pos, mut sprite) in goose_query.iter_mut() {
        debug!("goose loop");
        if targets.len() == 0 {
            commands.entity(goose_entity).remove::<HasTarget>();
            return;
        }
        let target = &targets[0]; // move towards first target

        let dir = (goose_pos.translation.xy() - target.1.translation.xy()).normalize();
        let movement = (dir * GOOSE_SPEED as f32 / FPS as f32).extend(0.);
        if movement.x != 0.0 {
            if movement.x > 0.0 {
                sprite.flip_x = true;
            } else {
                sprite.flip_x = false;
            }
        }
        let target = goose_pos.translation - Vec3::new(movement.x, movement.y, 0.0);
        goose_pos.translation = target; // TODO: prevent goose from clipping through walls
    }
}

pub fn goose_ate_edible(
    mut commands: Commands,
    goose_query: Query<(Entity, &Transform), (With<Goose>, With<HasTarget>)>,
    target_query: Query<(Entity, &Transform), (With<EdibleTarget>, Without<Expired>)>,
) {
    trace!("goose_ate_edible");

    // collect and sort all fireballs in play so we despawn them in a deterministic order
    let mut targets = target_query.iter().collect::<Vec<_>>();
    targets.sort_by_key(|e| e.0);

    for (goose_entity, goose_pos) in goose_query.iter() {
        for target in &targets {
            let distance = goose_pos.translation.distance(target.1.translation);
            debug!(
                "goose distance from target: {:?} - {:?}",
                target.0, distance
            );

            if distance < TILE_SIZE / 2. {
                debug!("goose reached target, despawn and find new one");
                commands.entity(goose_entity).remove::<HasTarget>();
                commands.entity(target.0).insert(Expired);
            }
        }
    }
}
