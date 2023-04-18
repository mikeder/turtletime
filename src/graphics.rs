use bevy::prelude::*;

use crate::{loading::TextureAssets, GameState, TILE_SIZE};

pub struct GraphicsPlugin;

#[derive(Resource)]
pub struct CharacterSheet {
    pub handle: Handle<TextureAtlas>,
    pub turtle_frames: [usize; 4],
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
        app.add_system(Self::load_graphics.in_schedule(OnEnter(GameState::MenuMain)))
            .add_system(Self::frame_animation);
    }
}

impl GraphicsPlugin {
    fn load_graphics(
        assets: Res<TextureAssets>,
        mut commands: Commands,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        let turtle_atlas = TextureAtlas::from_grid(
            assets.texture_turtle_cheeks_frame.clone(),
            Vec2::splat(TILE_SIZE),
            4,
            1,
            Some(Vec2 { x: 2.0, y: 0. }),
            Some(Vec2 { x: 0.0, y: 0. }),
        );
        let atlas_handle = texture_atlases.add(turtle_atlas);

        commands.insert_resource(CharacterSheet {
            handle: atlas_handle,
            turtle_frames: [0, 1, 2, 3],
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
