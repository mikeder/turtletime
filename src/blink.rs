use crate::player::components::{Player, PlayerHealth};
use crate::{GameState, TILE_SIZE};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

/// how long a player flashes after taking damage
const BLINK_SECS: f32 = 0.25;
const BLINK_COLOR: LinearRgba = LinearRgba::WHITE;

pub struct BlinkPlugin;

/// This plugin makes players flash white when they take damage.
/// It only draws, nothing in here is rolled back or changes the game state.
impl Plugin for BlinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlinkMaterial>::default())
            .add_systems(
                Update,
                (add_damage_blink, start_damage_blink, update_damage_blink)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Draws the shape of a sprite in a single color, see `assets/shaders/blink.wgsl`.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BlinkMaterial {
    /// the part of the texture to draw: min.xy and max.xy in uv coordinates
    #[uniform(0)]
    pub uv_rect: Vec4,
    /// the color to flash, alpha is how strong the flash currently is
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for BlinkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blink.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
pub struct DamageBlink {
    /// health the last time we looked, to notice when it drops
    last_health: i32,
    timer: Timer,
    /// child entity that draws the flash on top of the player
    overlay: Entity,
}

fn add_damage_blink(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlinkMaterial>>,
    query: Query<(Entity, &Sprite, &PlayerHealth), (With<Player>, Without<DamageBlink>)>,
) {
    for (player, sprite, health) in query.iter() {
        let size = sprite.custom_size.unwrap_or(Vec2::splat(TILE_SIZE));
        let overlay = commands
            .spawn((
                Name::new("DamageBlink"),
                Mesh2d(meshes.add(Rectangle::from_size(size))),
                MeshMaterial2d(materials.add(BlinkMaterial {
                    uv_rect: Vec4::new(0., 0., 1., 1.),
                    color: BLINK_COLOR.with_alpha(0.),
                    texture: sprite.image.clone(),
                })),
                // just in front of the player sprite
                Transform::from_xyz(0., 0., 0.1),
                Visibility::Hidden,
            ))
            .id();

        // start out finished, nobody has been hurt yet
        let mut timer = Timer::from_seconds(BLINK_SECS, TimerMode::Once);
        timer.tick(timer.duration());

        commands
            .entity(player)
            .insert(DamageBlink {
                last_health: health.0,
                timer,
                overlay,
            })
            .add_child(overlay);
    }
}

fn start_damage_blink(mut query: Query<(&PlayerHealth, &mut DamageBlink)>) {
    for (health, mut blink) in query.iter_mut() {
        // compare values instead of using change detection,
        // rollbacks write to the health component all the time
        if health.0 < blink.last_health {
            blink.timer.reset();
        }
        blink.last_health = health.0;
    }
}

fn update_damage_blink(
    time: Res<Time>,
    images: Res<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<BlinkMaterial>>,
    mut players: Query<(&Sprite, &mut DamageBlink)>,
    mut overlays: Query<(&MeshMaterial2d<BlinkMaterial>, &mut Visibility)>,
) {
    for (sprite, mut blink) in players.iter_mut() {
        let Ok((material, mut visibility)) = overlays.get_mut(blink.overlay) else {
            continue;
        };

        if blink.timer.tick(time.delta()).is_finished() {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        let Some(material) = materials.get_mut(&material.0) else {
            continue;
        };
        // follow the animation frame and direction of the player sprite
        if let Some(uv_rect) = sprite_uv_rect(sprite, &images, &layouts) {
            material.uv_rect = uv_rect;
        }
        // start fully white and fade back to the normal colors
        material.color = BLINK_COLOR.with_alpha(blink.timer.fraction_remaining());
        *visibility = Visibility::Visible;
    }
}

/// The part of its texture a sprite currently shows, as min.xy and max.xy in uv coordinates.
fn sprite_uv_rect(
    sprite: &Sprite,
    images: &Assets<Image>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Option<Vec4> {
    let image_size = images.get(&sprite.image)?.size().as_vec2();
    let rect = match &sprite.texture_atlas {
        Some(atlas) => atlas.texture_rect(layouts)?.as_rect(),
        None => Rect::from_corners(Vec2::ZERO, image_size),
    };

    let mut min = rect.min / image_size;
    let mut max = rect.max / image_size;
    if sprite.flip_x {
        std::mem::swap(&mut min.x, &mut max.x);
    }
    if sprite.flip_y {
        std::mem::swap(&mut min.y, &mut max.y);
    }
    Some(Vec4::new(min.x, min.y, max.x, max.y))
}
