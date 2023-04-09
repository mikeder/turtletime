use bevy::{
    prelude::*,
    render::{color, mesh::VertexAttributeValues},
};

use crate::{loading::TextureAssets, GameState, ASPECT_RATIO, MAP_HEIGHT};

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
        app.add_system(Self::load_graphics.in_schedule(OnEnter(GameState::Menu)))
            .add_system(Self::frame_animation);
    }
}

impl GraphicsPlugin {
    fn load_graphics(
        assets: Res<TextureAssets>,
        mut commands: Commands,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let turtle_atlas = TextureAtlas::from_grid(
            assets.texture_turtle.clone(),
            Vec2::splat(32.0),
            4,
            1,
            None,
            None,
        );
        let atlas_handle = texture_atlases.add(turtle_atlas);

        commands.insert_resource(CharacterSheet {
            handle: atlas_handle,
            turtle_frames: [0, 1, 2, 3],
        });

        let map_size = MAP_HEIGHT * ASPECT_RATIO / 2.;
        let map_border = map_size + 2.0;

        // spawn border
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: color::Color::GOLD,
                    ..Default::default()
                },
                transform: Transform::from_scale(Vec3::splat(map_border)),
                ..Default::default()
            })
            .insert(Border);

        // spawn grass
        // https://github.com/bevyengine/bevy/issues/399#issuecomment-1015353924
        let mut mesh = Mesh::from(shape::Quad::default());
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs {
                uv[0] *= map_size;
                uv[1] *= map_size;
            }
        }
        commands.spawn(ColorMesh2dBundle {
            material: materials.add(assets.texture_grass.clone().into()),
            mesh: meshes.add(mesh).into(),
            transform: Transform::from_scale(Vec3::splat(map_size)),
            ..Default::default()
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
