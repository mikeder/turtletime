use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MenuMain),
        )
        .add_collection_to_loading_state::<_, FontAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading);
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/turtle.png")]
    pub texture_turtle: Handle<Image>,
    #[asset(path = "textures/turtle2.png")]
    pub texture_turtle2: Handle<Image>,
    #[asset(path = "textures/turtlecheeks.png")]
    pub texture_turtle_cheeks: Handle<Image>,
    #[asset(path = "textures/turtlecheeks2.png")]
    pub texture_turtle_cheeks2: Handle<Image>,
    #[asset(path = "textures/turtlecheeksframe.png")]
    pub texture_turtle_cheeks_frame: Handle<Image>,
    #[asset(path = "textures/turtlecheeksframehat.png")]
    pub texture_turtle_cheeks_frame_ht: Handle<Image>,
    #[asset(path = "textures/turtlecheeksframepartyhat.png")]
    pub texture_turtle_cheeks_frame_party_hat: Handle<Image>,
    #[asset(path = "textures/poop.png")]
    pub texture_poop: Handle<Image>,
    #[asset(path = "textures/strawberry.png")]
    pub texture_strawberry: Handle<Image>,
    #[asset(path = "textures/chili_pepper.png")]
    pub texture_chili_pepper: Handle<Image>,
    #[asset(path = "textures/fireball.png")]
    pub texture_fireball: Handle<Image>,
    #[asset(path = "textures/lettuce.png")]
    pub texture_lettuce: Handle<Image>,
    // map textures
    #[asset(path = "textures/dirt.png")]
    pub texture_dirt: Handle<Image>,
    #[asset(path = "textures/grass.png")]
    pub texture_grass: Handle<Image>,
    #[asset(path = "textures/leftfence.png")]
    pub texture_fenceleft: Handle<Image>,
    #[asset(path = "textures/fencebottom.png")]
    pub texture_fencebottom: Handle<Image>,
    #[asset(path = "textures/fencetop.png")]
    pub texture_fencetop: Handle<Image>,
    #[asset(path = "textures/shortgrass.png")]
    pub texture_shortgrass: Handle<Image>,
    #[asset(path = "textures/shortgrassblue.png")]
    pub texture_shortgrassblue: Handle<Image>,
    #[asset(path = "textures/shortgrasspink.png")]
    pub texture_shortgrasspink: Handle<Image>,
    #[asset(path = "textures/water.png")]
    pub texture_water: Handle<Image>,
    #[asset(path = "textures/shortgrassedge.png")]
    pub texture_shortgrassedge: Handle<Image>,
    #[asset(path = "textures/shortgrasstopedge.png")]
    pub texture_shortgrasstopedge: Handle<Image>,
    #[asset(path = "textures/wateredge.png")]
    pub texture_wateredge: Handle<Image>,
    #[asset(path = "textures/peanut.png")]
    pub texture_peanutqueen: Handle<Image>,
}
