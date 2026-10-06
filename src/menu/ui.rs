//! Shared look and building blocks for all menus, so every screen uses the
//! same colors, text sizes and buttons.

use bevy::prelude::*;

// colors are picked from the garden the game takes place in
/// main text
pub const CREAM: Color = Color::srgb(1.0, 0.96, 0.84);
/// hints and anything that is switched off
pub const DIM: Color = Color::srgb(0.66, 0.78, 0.62);
/// values the player can change and good news
pub const LETTUCE: Color = Color::srgb(0.66, 0.88, 0.37);
/// warnings and errors
pub const CHILI: Color = Color::srgb(1.0, 0.6, 0.3);
/// see through backdrop for text drawn on top of the game
pub const SHADE: Color = Color::srgba(0.0, 0.08, 0.0, 0.7);

const STRAWBERRY: Color = Color::srgb(0.80, 0.18, 0.27);
const STRAWBERRY_HOVERED: Color = Color::srgb(0.90, 0.29, 0.38);
const STRAWBERRY_PRESSED: Color = Color::srgb(0.62, 0.12, 0.20);
const STRAWBERRY_EDGE: Color = Color::srgb(0.45, 0.08, 0.14);

pub const MOSS: Color = Color::srgb(0.05, 0.17, 0.07);
const MOSS_HOVERED: Color = Color::srgb(0.10, 0.27, 0.12);
const MOSS_PRESSED: Color = Color::srgb(0.03, 0.11, 0.04);
pub const MOSS_EDGE: Color = Color::srgb(0.20, 0.45, 0.22);

const DISABLED: Color = Color::srgb(0.04, 0.24, 0.06);
const DISABLED_EDGE: Color = Color::srgb(0.10, 0.32, 0.12);
const DISABLED_TEXT: Color = Color::srgb(0.38, 0.55, 0.38);

pub const TITLE_SIZE: f32 = 72.;
pub const HEADING_SIZE: f32 = 40.;
pub const BUTTON_SIZE: f32 = 28.;
pub const BODY_SIZE: f32 = 24.;
pub const HINT_SIZE: f32 = 18.;

const BUTTON_WIDTH: f32 = 300.;

/// How much a button should stand out.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuButton {
    /// the one thing the player most likely wants to do on a screen
    Primary,
    /// everything else
    Secondary,
}

/// Buttons without this component are always enabled.
#[derive(Component)]
pub struct ButtonEnabled(pub bool);

/// Root node of a menu screen, lays out its children in a centered column.
pub fn screen() -> impl Bundle {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.),
        right: Val::Px(0.),
        top: Val::Px(0.),
        bottom: Val::Px(0.),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(14.),
        ..Default::default()
    }
}

/// A row of things next to each other, like a label and its buttons.
pub fn row() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(16.),
        ..Default::default()
    }
}

/// A label stacked on top of the thing it describes, closer together than
/// the other things on the screen.
pub fn column() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(6.),
        ..Default::default()
    }
}

/// Empty space to set groups of things apart.
pub fn spacer(height: f32) -> impl Bundle {
    Node {
        height: Val::Px(height),
        ..Default::default()
    }
}

pub fn text(font: &Handle<Font>, value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

pub fn title(font: &Handle<Font>, value: impl Into<String>) -> impl Bundle {
    text(font, value, TITLE_SIZE, CREAM)
}

pub fn heading(font: &Handle<Font>, value: impl Into<String>) -> impl Bundle {
    text(font, value, HEADING_SIZE, CREAM)
}

pub fn body(font: &Handle<Font>, value: impl Into<String>) -> impl Bundle {
    text(font, value, BODY_SIZE, CREAM)
}

pub fn hint(font: &Handle<Font>, value: impl Into<String>) -> impl Bundle {
    text(font, value, HINT_SIZE, DIM)
}

/// A button that is as wide as the others on the screen and grows with its label.
pub fn button(font: &Handle<Font>, label: impl Into<String>, kind: MenuButton) -> impl Bundle {
    sized_button(font, label, kind, BUTTON_WIDTH)
}

/// A button just big enough for a single character, like + and -.
pub fn small_button(font: &Handle<Font>, label: impl Into<String>) -> impl Bundle {
    sized_button(font, label, MenuButton::Secondary, 56.)
}

fn sized_button(
    font: &Handle<Font>,
    label: impl Into<String>,
    kind: MenuButton,
    min_width: f32,
) -> impl Bundle {
    let (background, edge) = button_colors(kind, true, Interaction::None);
    (
        Button,
        kind,
        Node {
            min_width: Val::Px(min_width),
            // no fixed height, the label decides how much room the button needs
            padding: UiRect::axes(Val::Px(20.), Val::Px(10.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.)),
            border_radius: BorderRadius::all(Val::Px(8.)),
            ..Default::default()
        },
        BackgroundColor(background),
        BorderColor::all(edge),
        children![text(font, label, BUTTON_SIZE, CREAM)],
    )
}

fn button_colors(kind: MenuButton, enabled: bool, interaction: Interaction) -> (Color, Color) {
    if !enabled {
        return (DISABLED, DISABLED_EDGE);
    }
    match (kind, interaction) {
        (MenuButton::Primary, Interaction::Pressed) => (STRAWBERRY_PRESSED, STRAWBERRY_EDGE),
        (MenuButton::Primary, Interaction::Hovered) => (STRAWBERRY_HOVERED, CREAM),
        (MenuButton::Primary, Interaction::None) => (STRAWBERRY, STRAWBERRY_EDGE),
        (MenuButton::Secondary, Interaction::Pressed) => (MOSS_PRESSED, MOSS_EDGE),
        (MenuButton::Secondary, Interaction::Hovered) => (MOSS_HOVERED, CREAM),
        (MenuButton::Secondary, Interaction::None) => (MOSS, MOSS_EDGE),
    }
}

/// Shows whether a button is hovered, pressed or switched off.
pub fn update_buttons(
    mut buttons: Query<(
        &Interaction,
        &MenuButton,
        Option<&ButtonEnabled>,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut labels: Query<&mut TextColor>,
) {
    for (interaction, kind, enabled, mut background, mut border, children) in buttons.iter_mut() {
        let enabled = enabled.is_none_or(|e| e.0);
        let (color, edge) = button_colors(*kind, enabled, *interaction);
        // only write on change, so the ui isn't laid out again every frame
        if background.0 != color {
            background.0 = color;
        }
        if border.top != edge {
            *border = BorderColor::all(edge);
        }

        let label_color = if enabled { CREAM } else { DISABLED_TEXT };
        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                if text_color.0 != label_color {
                    text_color.0 = label_color;
                }
            }
        }
    }
}

/// A box that shows a line of typed text and grows with it.
pub fn input_field(font: &Handle<Font>) -> impl Bundle {
    (
        Node {
            min_width: Val::Px(BUTTON_WIDTH),
            height: Val::Px(52.),
            padding: UiRect::horizontal(Val::Px(16.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.)),
            border_radius: BorderRadius::all(Val::Px(8.)),
            ..Default::default()
        },
        BackgroundColor(MOSS),
        BorderColor::all(MOSS_EDGE),
        children![text(font, "", BUTTON_SIZE, LETTUCE)],
    )
}

/// A box that shows a single typed character.
pub fn input_slot(font: &Handle<Font>) -> impl Bundle {
    (
        Node {
            width: Val::Px(52.),
            height: Val::Px(60.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.)),
            border_radius: BorderRadius::all(Val::Px(8.)),
            ..Default::default()
        },
        BackgroundColor(MOSS),
        BorderColor::all(MOSS_EDGE),
        children![text(font, "", HEADING_SIZE, LETTUCE)],
    )
}
