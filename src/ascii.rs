use crate::TILE_SIZE;
use bevy::prelude::*;

pub struct AsciiPlugin;

#[derive(Resource)]
pub struct AsciiSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

impl Plugin for AsciiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_ascii);
    }
}

#[allow(dead_code)] // hold for later
pub fn spawn_ascii_sprite(
    commands: &mut Commands,
    ascii: &AsciiSheet,
    index: usize,
    color: Color,
    translation: Vec3,
    scale: Vec3,
) -> Entity {
    assert!(index < 256, "Index out of Ascii Range");

    let mut sprite = Sprite::from_atlas_image(
        ascii.image.clone(),
        TextureAtlas {
            layout: ascii.layout.clone(),
            index,
        },
    );
    sprite.color = color;
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    commands
        .spawn((
            sprite,
            Transform {
                translation,
                scale,
                ..Default::default()
            },
        ))
        .id()
}

fn load_ascii(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = assets.load("textures/ascii.png");
    let atlas = TextureAtlasLayout::from_grid(UVec2::splat(9), 16, 16, Some(UVec2::splat(2)), None);

    let layout = texture_atlases.add(atlas);

    commands.insert_resource(AsciiSheet { image, layout });
}
