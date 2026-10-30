//! The leaderboards of the game, anyone can look at them, signed in or not.

use super::ui::{self, MenuButton};
use crate::loading::FontAssets;
use crate::platform::plugin::{Account, Leaderboards, LoadStatus, PlatformRequest};
use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct LeaderboardUI;

/// Holds a button for every leaderboard.
#[derive(Component)]
pub struct BoardTabs;

/// Switches to the leaderboard with this key.
#[derive(Component)]
pub struct BoardTab(String);

/// The texts that show the leaderboard.
#[derive(Component)]
pub enum BoardText {
    Status,
    /// ranks and names
    Names,
    Values,
    /// where the signed in player is
    Me,
}

#[derive(Component)]
pub struct BackBtn;

pub fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut requests: MessageWriter<PlatformRequest>,
) {
    let font = &font_assets.fira_sans;
    requests.write(PlatformRequest::FetchLeaderboards);

    commands.spawn((Camera2d, Msaa::Off, LeaderboardUI));
    commands.spawn((
        ui::screen(),
        LeaderboardUI,
        children![
            ui::heading(font, "Leaderboards"),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(12.),
                    row_gap: Val::Px(12.),
                    max_width: Val::Px(960.),
                    ..default()
                },
                BoardTabs,
            ),
            (ui::body(font, ""), BoardText::Status),
            // ranks and names on the left, values lined up on the right
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(32.),
                    min_width: Val::Px(420.),
                    padding: UiRect::axes(Val::Px(24.), Val::Px(16.)),
                    border: UiRect::all(Val::Px(3.)),
                    border_radius: BorderRadius::all(Val::Px(8.)),
                    ..default()
                },
                BackgroundColor(ui::SHADE),
                BorderColor::all(ui::MOSS_EDGE),
                children![
                    (
                        ui::body(font, ""),
                        TextLayout::new_with_justify(Justify::Left),
                        BoardText::Names,
                    ),
                    (
                        ui::text(font, "", ui::BODY_SIZE, ui::LETTUCE),
                        TextLayout::new_with_justify(Justify::Right),
                        BoardText::Values,
                    ),
                ],
            ),
            (ui::hint(font, ""), BoardText::Me),
            ui::spacer(10.),
            (
                ui::button(font, "Back to menu", MenuButton::Secondary),
                BackBtn,
            ),
        ],
    ));
}

/// Adds a button for every leaderboard once they are known, and marks the one shown.
pub fn update_tabs(
    mut commands: Commands,
    leaderboards: Res<Leaderboards>,
    font_assets: Res<FontAssets>,
    container: Query<(Entity, Option<&Children>), With<BoardTabs>>,
    mut tabs: Query<(&BoardTab, &mut MenuButton)>,
) {
    if !leaderboards.is_changed() {
        return;
    }
    let Ok((container, children)) = container.single() else {
        return;
    };
    let built = children.map_or(0, |c| c.len());
    // a single leaderboard needs no buttons to pick it
    if built == 0 && leaderboards.boards.len() > 1 {
        let font = &font_assets.fira_sans;
        commands.entity(container).with_children(|parent| {
            for board in &leaderboards.boards {
                let name = if board.display_name.is_empty() {
                    &board.key
                } else {
                    &board.display_name
                };
                parent.spawn((
                    ui::button(font, name.clone(), tab_kind(&leaderboards, &board.key)),
                    BoardTab(board.key.clone()),
                ));
            }
        });
        return;
    }
    for (tab, mut kind) in tabs.iter_mut() {
        let wanted = tab_kind(&leaderboards, &tab.0);
        if *kind != wanted {
            *kind = wanted;
        }
    }
}

/// The leaderboard being shown stands out.
fn tab_kind(leaderboards: &Leaderboards, key: &str) -> MenuButton {
    if leaderboards.selected.as_deref() == Some(key) {
        MenuButton::Primary
    } else {
        MenuButton::Secondary
    }
}

/// What the leaderboard table shows, one line per entry.
fn table(leaderboards: &Leaderboards, player_id: &str) -> (String, String, String) {
    let status = match (&leaderboards.status, &leaderboards.board_status) {
        (LoadStatus::Failed(e), _) | (_, LoadStatus::Failed(e)) => e.clone(),
        (LoadStatus::Loaded, _) if leaderboards.boards.is_empty() => {
            "There are no leaderboards yet".to_owned()
        }
        (LoadStatus::Loading, _) | (_, LoadStatus::Loading) => "Loading...".to_owned(),
        (_, LoadStatus::Loaded) if leaderboards.entries.is_empty() => {
            "No one is on this leaderboard yet, win an online match!".to_owned()
        }
        _ => {
            // name the board when there are no tabs to tell which one this is
            let only = (leaderboards.boards.len() == 1).then(|| &leaderboards.boards[0]);
            only.map(|b| b.display_name.clone()).unwrap_or_default()
        }
    };
    let mut names = Vec::new();
    let mut values = Vec::new();
    for entry in &leaderboards.entries {
        let you = if !player_id.is_empty() && entry.player_id == player_id {
            " (you)"
        } else {
            ""
        };
        names.push(format!("{}. {}{you}", entry.rank, entry.player_name));
        values.push(entry.shown_value());
    }
    (status, names.join("\n"), values.join("\n"))
}

pub fn update_table(
    leaderboards: Res<Leaderboards>,
    account: Res<Account>,
    mut texts: Query<(&BoardText, &mut Text)>,
) {
    if !leaderboards.is_changed() && !account.is_changed() {
        return;
    }
    let (status, names, values) = table(&leaderboards, &account.player_id);
    let me = if !account.signed_in() {
        "Sign in to get on the leaderboards".to_owned()
    } else {
        match &leaderboards.me {
            Some(me) => format!("You are #{} with {}", me.rank, me.shown_value()),
            None if leaderboards.board_status == LoadStatus::Loaded => {
                "You are not on this leaderboard yet".to_owned()
            }
            None => String::new(),
        }
    };
    for (kind, mut text) in texts.iter_mut() {
        let value = match kind {
            BoardText::Status => &status,
            BoardText::Names => &names,
            BoardText::Values => &values,
            BoardText::Me => &me,
        };
        if &text.0 != value {
            text.0 = value.clone();
        }
    }
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut requests: MessageWriter<PlatformRequest>,
    leaderboards: Res<Leaderboards>,
    tabs: Query<(&Interaction, &BoardTab), Changed<Interaction>>,
    back: Query<&Interaction, (With<BackBtn>, Changed<Interaction>)>,
) {
    for (interaction, tab) in tabs.iter() {
        if *interaction == Interaction::Pressed && leaderboards.selected.as_ref() != Some(&tab.0) {
            requests.write(PlatformRequest::FetchLeaderboard(tab.0.clone()));
        }
    }
    if back.iter().any(|i| *i == Interaction::Pressed) {
        state.set(AppState::MenuMain);
    }
}

pub fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<LeaderboardUI>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::api::{Leaderboard, LeaderboardEntry};

    fn entry(rank: u32, id: &str, name: &str, value: &str) -> LeaderboardEntry {
        LeaderboardEntry {
            rank,
            player_id: id.to_owned(),
            player_name: name.to_owned(),
            display_value: value.to_owned(),
            ..default()
        }
    }

    #[test]
    fn the_table_marks_the_player() {
        let leaderboards = Leaderboards {
            boards: vec![Leaderboard {
                key: "wins".to_owned(),
                display_name: "Most wins".to_owned(),
            }],
            status: LoadStatus::Loaded,
            board_status: LoadStatus::Loaded,
            entries: vec![entry(1, "p1", "Shelly", "12"), entry(2, "p2", "Leo", "3")],
            ..default()
        };
        let (status, names, values) = table(&leaderboards, "p2");
        assert_eq!(status, "Most wins");
        assert_eq!(names, "1. Shelly\n2. Leo (you)");
        assert_eq!(values, "12\n3");
    }

    #[test]
    fn the_table_says_why_it_is_empty() {
        let mut leaderboards = Leaderboards {
            status: LoadStatus::Loading,
            ..default()
        };
        assert_eq!(table(&leaderboards, "").0, "Loading...");
        leaderboards.status = LoadStatus::Loaded;
        assert_eq!(table(&leaderboards, "").0, "There are no leaderboards yet");
        leaderboards.status = LoadStatus::Failed("Could not reach the server".to_owned());
        assert_eq!(table(&leaderboards, "").0, "Could not reach the server");
    }
}
