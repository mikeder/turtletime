use super::components::{
    ConsoleReady, ConsoleText, ConsoleUI, ConsoleUpdateTimer, EdibleCount, PeerInfo,
};
use crate::loading::FontAssets;
use bevy::prelude::*;

const CONSOLE_KEY: KeyCode = KeyCode::Grave;

pub fn open_console(
    keyboard_input: Res<Input<KeyCode>>,
    mut console_vis: Query<(&mut ConsoleReady, &mut Visibility), With<ConsoleUI>>,
) {
    let (mut ready, mut vis) = match console_vis.get_single_mut() {
        Ok(r) => r,
        Err(e) => {
            debug!("{:?}", e);
            return; // console not ready
        }
    };

    if ready.0 && keyboard_input.pressed(CONSOLE_KEY) {
        if *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
        ready.0 = false;
    }
}

pub fn reset_console_ready(
    keyboard_input: Res<Input<KeyCode>>,
    mut ready: Query<&mut ConsoleReady, With<ConsoleUI>>,
) {
    let mut ready: Mut<ConsoleReady> = match ready.get_single_mut() {
        Ok(r) => r,
        Err(e) => {
            debug!("{:?}", e);
            return; // console not ready
        }
    };

    if !keyboard_input.pressed(CONSOLE_KEY) {
        ready.0 = true
    }
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                right: Val::Auto,
                top: Val::Px(5.),
                bottom: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            background_color: BackgroundColor(Color::Rgba {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 0.7,
            }),
            ..Default::default()
        })
        .with_children(|parent| {
            // lobby id display
            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    text: Text {
                        sections: vec![TextSection {
                            value: "".to_owned(),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 15.0,
                                color: Color::GREEN,
                            },
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ConsoleText);
        })
        .insert(Name::new("ConsoleUI"))
        .insert(ConsoleReady::default())
        .insert(ConsoleUI);

    commands.insert_resource(PeerInfo("".to_string()));
    commands.insert_resource(EdibleCount(0));
    commands.insert_resource(ConsoleUpdateTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )))
}

pub fn update_console_text(
    edible_count: ResMut<EdibleCount>,
    peer_info: ResMut<PeerInfo>,
    mut query: Query<&mut Text, With<ConsoleText>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "Edible count: {}\nPeer Info: \n{}",
            edible_count.0, peer_info.0
        );
    }
}
