use bevy::prelude::*;

use crate::{loading::TextureAssets, AppState, TILE_SIZE};

pub struct GraphicsPlugin;

/// The looks a turtle can have.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Character {
    #[default]
    Plain,
    Hat,
    /// worn by the winner of the previous round, players can't pick it
    PartyHat,
}

impl Character {
    /// How players tell each other in the lobby which turtle they play as.
    pub fn to_byte(self) -> u8 {
        match self {
            Character::Plain => 0,
            Character::Hat => 1,
            Character::PartyHat => 2,
        }
    }

    pub fn from_byte(byte: u8) -> Character {
        match byte {
            1 => Character::Hat,
            2 => Character::PartyHat,
            _ => Character::Plain,
        }
    }
}

#[derive(Resource)]
pub struct CharacterSheet {
    pub turtle_image: Handle<Image>,
    pub turtle_hat_image: Handle<Image>,
    pub turtle_party_hat_image: Handle<Image>,
    pub turtle_layout: Handle<TextureAtlasLayout>,
    pub goose_image: Handle<Image>,
    pub goose_layout: Handle<TextureAtlasLayout>,
    pub turtle_frames: [usize; 4],
    pub goose_frames: [usize; 4],
}

impl CharacterSheet {
    /// All turtle images share the same layout and frames.
    pub fn turtle_image(&self, character: Character) -> Handle<Image> {
        match character {
            Character::Plain => self.turtle_image.clone(),
            Character::Hat => self.turtle_hat_image.clone(),
            Character::PartyHat => self.turtle_party_hat_image.clone(),
        }
    }
}

#[derive(Component)]
pub struct FrameAnimation {
    pub timer: Timer,
    pub frames: Vec<usize>,
    pub current_frame: usize,
    /// a stopped animation rests on its first frame
    pub playing: bool,
}

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        // before the first menu shows up, the menus show the turtles as well
        app.add_systems(OnExit(AppState::Loading), Self::load_graphics)
            .add_systems(Update, Self::frame_animation);
    }
}

impl GraphicsPlugin {
    fn load_graphics(
        assets: Res<TextureAssets>,
        mut commands: Commands,
        mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        // load turtle atlas sheet
        let turtle_atlas = TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_SIZE as u32),
            4,
            1,
            Some(UVec2 { x: 2, y: 0 }),
            Some(UVec2 { x: 0, y: 0 }),
        );

        // load goose atlas sheet
        let goose_atlas = TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_SIZE as u32),
            4,
            1,
            Some(UVec2 { x: 1, y: 0 }),
            Some(UVec2 { x: 0, y: 0 }),
        );

        // add character sheet with atlas and frame instructions
        commands.insert_resource(CharacterSheet {
            turtle_image: assets.texture_turtle_cheeks_frame.clone(),
            turtle_hat_image: assets.texture_turtle_cheeks_frame_ht.clone(),
            turtle_party_hat_image: assets.texture_turtle_cheeks_frame_party_hat.clone(),
            turtle_layout: texture_atlases.add(turtle_atlas),
            turtle_frames: [0, 1, 2, 3],
            goose_image: assets.texture_goose.clone(),
            goose_layout: texture_atlases.add(goose_atlas),
            goose_frames: [0, 1, 2, 3],
        });
    }

    fn frame_animation(
        mut sprites_query: Query<(&mut Sprite, &mut FrameAnimation)>,
        time: Res<Time>,
    ) {
        for (mut sprite, mut animation) in sprites_query.iter_mut() {
            if !animation.playing {
                if animation.current_frame != 0 {
                    animation.current_frame = 0;
                    animation.timer.reset();
                    if let Some(atlas) = sprite.texture_atlas.as_mut() {
                        atlas.index = animation.frames[0];
                    }
                }
                continue;
            }

            animation.timer.tick(time.delta());
            if animation.timer.just_finished() {
                animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = animation.frames[animation.current_frame];
                }
            }
        }
    }
}
