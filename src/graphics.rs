use bevy::prelude::*;

use crate::{loading::TextureAssets, AppState, TILE_SIZE};

pub struct GraphicsPlugin;

#[derive(Resource)]
pub struct CharacterSheet {
    pub turtle_handle: Handle<TextureAtlas>,
    pub goose_handle: Handle<TextureAtlas>,
    pub turtle_frames: [usize; 4],
    pub goose_frames: [usize; 4],
}

#[derive(Component)]
pub struct FrameAnimation {
    pub timer: Timer,
    pub frames: Vec<usize>,
    pub current_frame: usize,
}
#[derive(Component)]
pub struct Border;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MenuMain), Self::load_graphics)
            .add_systems(Update, Self::frame_animation);
    }
}

impl GraphicsPlugin {
    fn load_graphics(
        assets: Res<TextureAssets>,
        mut commands: Commands,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        // load turtle atlas sheet
        let turtle_atlas = TextureAtlas::from_grid(
            assets.texture_turtle_cheeks_frame_party_hat.clone(),
            Vec2::splat(TILE_SIZE),
            4,
            1,
            Some(Vec2 { x: 2.0, y: 0. }),
            Some(Vec2 { x: 0.0, y: 0. }),
        );

        // load goose atlas sheet
        let goose_atlas = TextureAtlas::from_grid(
            assets.texture_goose.clone(),
            Vec2::splat(TILE_SIZE),
            4,
            1,
            Some(Vec2 { x: 1.0, y: 0. }),
            Some(Vec2 { x: 0.0, y: 0. }),
        );

        // add character sheet with atlas and frame instructions
        commands.insert_resource(CharacterSheet {
            turtle_handle: texture_atlases.add(turtle_atlas),
            turtle_frames: [0, 1, 2, 3],
            goose_handle: texture_atlases.add(goose_atlas),
            goose_frames: [0, 1, 2, 3],
        });
    }

    fn frame_animation(
        mut sprites_query: Query<(&mut TextureAtlasSprite, &mut FrameAnimation)>,
        time: Res<Time>,
    ) {
        for (mut sprite, mut animation) in sprites_query.iter_mut() {
            animation.timer.tick(time.delta());
            if animation.timer.just_finished() {
                animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
                sprite.index = animation.frames[animation.current_frame];
            }
        }
    }
}
