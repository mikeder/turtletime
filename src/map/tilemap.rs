use super::map::ASCII_MAP;
use crate::{loading::TextureAssets, AppState, GameState, TILE_SIZE};
use bevy::prelude::*;

pub struct TileMapPlugin;

#[derive(Component)]
struct Map;

#[derive(Component)]
pub struct EncounterSpawner;

#[derive(Component)]
pub struct PlayerSpawn {
    pub pos: Vec3,
}

#[derive(Component)]
pub struct TileCollider;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::spawn_map.in_schedule(OnExit(AppState::Loading)))
            .add_system(Self::show_map.in_schedule(OnEnter(GameState::Playing)))
            .add_system(Self::hide_map.in_schedule(OnEnter(GameState::Paused)));
    }
}

impl TileMapPlugin {
    fn hide_map(
        children_query: Query<&Children, With<Map>>,
        mut child_visibility_query: Query<&mut Visibility, Without<Map>>,
    ) {
        if let Ok(children) = children_query.get_single() {
            for child in children.iter() {
                if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                    *child_vis = Visibility::Hidden;
                }
            }
        }
    }

    fn show_map(
        children_query: Query<&Children, With<Map>>,
        mut child_visibility_query: Query<&mut Visibility, Without<Map>>,
    ) {
        if let Ok(children) = children_query.get_single() {
            for child in children.iter() {
                if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                    *child_vis = Visibility::Visible;
                }
            }
        }
    }

    fn spawn_map(mut commands: Commands, textures: Res<TextureAssets>) {
        let mut tiles = Vec::new();
        let map = ASCII_MAP.to_string();

        let lines = map.split("\n");
        let top: usize = 0;
        for (y, line) in lines.into_iter().enumerate() {
            if y == top {
                // skip first line because its only a \n
                continue;
            }
            for (x, char) in line.chars().enumerate() {
                let texture = match char {
                    // left fence
                    '|' => textures.texture_fenceleft.clone(),
                    // bottom fence
                    '_' => textures.texture_fencebottom.clone(),
                    // top fence
                    '=' => textures.texture_fencetop.clone(),
                    // grass w/ blue flowers
                    '!' => textures.texture_shortgrassblue.clone(),
                    // grass w/ ping flowers
                    '~' => textures.texture_shortgrasspink.clone(),
                    // short grass
                    '.' => textures.texture_shortgrass.clone(),
                    // water
                    '&' => textures.texture_water.clone(),
                    // water edge
                    '%' => textures.texture_wateredge.clone(),
                    // the queen
                    '$' => textures.texture_peanutqueen.clone(),
                    // grass edge bottom
                    '^' | '+' => textures.texture_shortgrasstopedge.clone(),
                    // grass edge top
                    '`' => textures.texture_shortgrassedge.clone(),
                    // dirt path
                    '*' => textures.texture_dirt.clone(),
                    // default to dirt
                    _ => textures.texture_dirt.clone(),
                };
                let translation = Vec3::new(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0);
                let sprite = SpriteBundle {
                    sprite: Sprite {
                        ..Default::default()
                    },
                    transform: Transform::from_translation(translation),
                    texture,
                    ..Default::default()
                };
                let tile = commands.spawn(sprite).id();
                if char == '|'
                    || char == '='
                    || char == '_'
                    || char == '%'
                    || char == '^'
                    || char == '$'
                {
                    // Walls
                    commands.entity(tile).insert(TileCollider);
                }
                if char == '~' {
                    // Grass
                    commands.entity(tile).insert(EncounterSpawner);
                }
                if char == '!' {
                    // Player Spawn
                    commands
                        .entity(tile)
                        .insert(PlayerSpawn { pos: translation });
                }
                tiles.push(tile);
            }
        }

        commands
            .spawn(SpriteSheetBundle {
                visibility: Visibility::Hidden,
                ..Default::default()
            })
            .insert(Map)
            .insert(Name::new("Map"))
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .push_children(&tiles);
    }
}
