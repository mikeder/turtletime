use bevy::prelude::*;

use crate::{
    ascii::{spawn_ascii_sprite, AsciiSheet},
    GameState, TILE_SIZE,
};

pub struct TileMapPlugin;

#[derive(Component)]
struct Map;

#[derive(Component)]
pub struct EncounterSpawner;

#[derive(Component)]
pub struct TileCollider;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::spawn_map.in_schedule(OnExit(GameState::Loading)))
            .add_system(Self::show_map.in_schedule(OnEnter(GameState::RoundLocal)))
            .add_system(Self::show_map.in_schedule(OnEnter(GameState::RoundOnline)))
            .add_system(Self::hide_map.in_schedule(OnEnter(GameState::MenuMain)));
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

    fn spawn_map(mut commands: Commands, ascii: Res<AsciiSheet>) {
        let mut tiles = Vec::new();

        let map = "
==========================================
|~~~~~~............................~~~~~~|
|~~~~~~............................~~~~~~|
|~~~~~~............................~~~~~~|
|........................................|
|........................................|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~@~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........~~~~~~~~~~~~~~~~~~~~~~~~........|
|........................................|
|........................................|
|~~~~~~............................~~~~~~|
|~~~~~~............................~~~~~~|
|~~~~~~............................~~~~~~|
=========================================="
            .to_string();

        for (y, line) in map.split("\n").into_iter().enumerate() {
            if y == 0 {
                // skip first line because its only a \n
                continue;
            }
            for (x, char) in line.chars().enumerate() {
                let color = match char {
                    '#' => Color::rgb(0.7, 0.7, 0.7), // walls
                    '|' => Color::rgb(0.7, 0.7, 0.7), // walls
                    '=' => Color::rgb(0.7, 0.7, 0.7), // walls
                    '@' => Color::rgb(0.5, 0.5, 0.2), // npc
                    '~' => Color::rgb(0.2, 0.9, 0.2), // grass
                    _ => Color::rgb(0.9, 0.9, 0.9),
                };
                let tile = spawn_ascii_sprite(
                    &mut commands,
                    &ascii,
                    char as usize,
                    color,
                    Vec3::new(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0),
                    Vec3::splat(1.0),
                );
                if char == '|' || char == '=' {
                    // walls
                    commands.entity(tile).insert(TileCollider);
                }
                if char == '@' {
                    // NPC
                    commands.entity(tile).insert(TileCollider);
                }
                if char == '~' {
                    // Grass
                    commands.entity(tile).insert(EncounterSpawner);
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
