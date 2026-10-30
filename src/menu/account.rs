//! Signing in to a Tiny Game Platform account, or creating one. Accounts are
//! optional, they put the online wins and losses of a player on the leaderboards.

use super::ui::{self, ButtonEnabled, MenuButton};
use crate::loading::FontAssets;
use crate::platform::plugin::{Account, AccountStatus, PlatformRequest, PlayerStats};
use crate::AppState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

/// The platform takes usernames up to 50 characters.
const USERNAME_MAX_LEN: usize = 50;
const PASSWORD_MAX_LEN: usize = 64;

#[derive(Component)]
pub struct AccountUI;

/// Holds what the screen shows, it is built again when the account status changes.
#[derive(Component)]
pub struct AccountContent;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormField {
    Username,
    Password,
}

#[derive(Component)]
pub enum AccountBtn {
    SignIn,
    Register,
    Remember,
    SignOut,
    Leaderboards,
    Back,
}

/// Texts that follow the form or the account.
#[derive(Component)]
pub enum AccountText {
    Remember,
    Error,
    Stats,
}

/// What was typed into the form, only kept while the screen is open.
#[derive(Resource)]
pub struct AccountForm {
    username: String,
    password: String,
    focus: FormField,
    remember: bool,
    /// the status the screen was last built for
    built_for: Option<AccountStatus>,
}

impl Default for AccountForm {
    fn default() -> Self {
        AccountForm {
            username: String::new(),
            password: String::new(),
            focus: FormField::Username,
            remember: true,
            built_for: None,
        }
    }
}

impl AccountForm {
    fn complete(&self) -> bool {
        !self.username.trim().is_empty() && !self.password.is_empty()
    }

    fn field_mut(&mut self, field: FormField) -> (&mut String, usize) {
        match field {
            FormField::Username => (&mut self.username, USERNAME_MAX_LEN),
            FormField::Password => (&mut self.password, PASSWORD_MAX_LEN),
        }
    }

    /// What a field shows: passwords are hidden, the focused field has a cursor.
    fn display(&self, field: FormField) -> String {
        let mut shown = match field {
            FormField::Username => self.username.clone(),
            FormField::Password => "*".repeat(self.password.chars().count()),
        };
        if self.focus == field {
            shown.push('|');
        }
        shown
    }
}

pub fn setup_ui(mut commands: Commands) {
    commands.init_resource::<AccountForm>();
    commands.spawn((Camera2d, Msaa::Off, AccountUI));
    commands.spawn((ui::screen(), AccountUI, AccountContent));
}

/// Builds the form, the "signing in" notice or the profile, whichever fits the account.
pub fn build_content(
    mut commands: Commands,
    account: Res<Account>,
    mut form: ResMut<AccountForm>,
    font_assets: Res<FontAssets>,
    content: Query<Entity, With<AccountContent>>,
    mut requests: MessageWriter<PlatformRequest>,
) {
    if form.built_for == Some(account.status) {
        return;
    }
    let Ok(root) = content.single() else {
        return;
    };
    form.built_for = Some(account.status);
    let font = &font_assets.fira_sans;

    commands.entity(root).despawn_related::<Children>();
    commands.entity(root).with_children(|parent| {
        parent.spawn(ui::heading(font, "Account"));
        match account.status {
            AccountStatus::SignedOut => {
                parent.spawn(ui::hint(
                    font,
                    "Sign in to put your online wins on the leaderboards.\nYou can play without an account too.",
                ))
                .insert(TextLayout::new_with_justify(Justify::Center));
                parent.spawn((
                    ui::column(),
                    children![
                        ui::body(font, "Username"),
                        (ui::input_field(font), Button, FormField::Username),
                    ],
                ));
                parent.spawn((
                    ui::column(),
                    children![
                        ui::body(font, "Password"),
                        (ui::input_field(font), Button, FormField::Password),
                    ],
                ));
                parent.spawn((
                    Button,
                    MenuButton::Secondary,
                    Node {
                        padding: UiRect::axes(Val::Px(12.), Val::Px(6.)),
                        border: UiRect::all(Val::Px(3.)),
                        border_radius: BorderRadius::all(Val::Px(8.)),
                        ..default()
                    },
                    BackgroundColor(ui::MOSS),
                    BorderColor::all(ui::MOSS_EDGE),
                    AccountBtn::Remember,
                    children![(
                        ui::text(font, "", ui::HINT_SIZE + 2., ui::CREAM),
                        AccountText::Remember
                    )],
                ));
                parent.spawn((
                    ui::text(font, "", ui::BODY_SIZE, ui::CHILI),
                    TextLayout::new_with_justify(Justify::Center),
                    AccountText::Error,
                ));
                parent.spawn((
                    ui::row(),
                    children![
                        (
                            ui::button(font, "Sign in", MenuButton::Primary),
                            ButtonEnabled(false),
                            AccountBtn::SignIn,
                        ),
                        (
                            ui::button(font, "Create account", MenuButton::Secondary),
                            ButtonEnabled(false),
                            AccountBtn::Register,
                        ),
                    ],
                ));
                parent.spawn(ui::hint(font, "Tab switches fields, Enter signs in"));
            }
            AccountStatus::Working => {
                parent.spawn(ui::body(font, "Signing in..."));
                parent.spawn((
                    ui::button(font, "Cancel", MenuButton::Secondary),
                    AccountBtn::SignOut,
                ));
            }
            AccountStatus::SignedIn => {
                requests.write(PlatformRequest::FetchMyStats);
                parent.spawn(ui::text(
                    font,
                    format!("Signed in as {}", account.username),
                    ui::BODY_SIZE,
                    ui::LETTUCE,
                ));
                parent.spawn((
                    ui::text(font, "", ui::HEADING_SIZE, ui::CREAM),
                    AccountText::Stats,
                ));
                parent.spawn(ui::hint(font, "Online matches count toward your stats"));
                parent.spawn(ui::spacer(10.));
                parent.spawn((
                    ui::button(font, "Leaderboards", MenuButton::Primary),
                    AccountBtn::Leaderboards,
                ));
                parent.spawn((
                    ui::button(font, "Sign out", MenuButton::Secondary),
                    AccountBtn::SignOut,
                ));
            }
        }
        parent.spawn((
            ui::button(font, "Back to menu", MenuButton::Secondary),
            AccountBtn::Back,
        ));
    });
}

fn stats_line(stats: Option<PlayerStats>) -> String {
    match stats {
        None => "...".to_owned(),
        Some(stats) => format!(
            "Wins {}   Losses {}   Matches {}",
            stats.wins, stats.losses, stats.matches
        ),
    }
}

/// Typing goes into the focused field, Tab switches fields and Enter signs in.
pub fn type_into_form(
    mut keyboard_evr: MessageReader<KeyboardInput>,
    mut form: ResMut<AccountForm>,
    account: Res<Account>,
    mut requests: MessageWriter<PlatformRequest>,
) {
    if account.status != AccountStatus::SignedOut {
        keyboard_evr.clear();
        return;
    }
    for ev in keyboard_evr.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        let focus = form.focus;
        match &ev.logical_key {
            Key::Tab => {
                form.focus = match focus {
                    FormField::Username => FormField::Password,
                    FormField::Password => FormField::Username,
                }
            }
            Key::Enter => {
                if form.complete() {
                    requests.write(login(&form));
                } else if focus == FormField::Username {
                    form.focus = FormField::Password;
                }
            }
            Key::Backspace => {
                form.field_mut(focus).0.pop();
            }
            Key::Character(chars) => push_chars(form.field_mut(focus), chars),
            Key::Space => push_chars(form.field_mut(focus), " "),
            _ => (),
        }
    }
}

fn push_chars((field, max_len): (&mut String, usize), chars: &str) {
    for c in chars.chars() {
        if field.chars().count() < max_len && !c.is_control() {
            field.push(c);
        }
    }
}

fn login(form: &AccountForm) -> PlatformRequest {
    PlatformRequest::Login {
        username: form.username.trim().to_owned(),
        password: form.password.clone(),
        remember: form.remember,
    }
}

pub fn update_form_display(
    form: Res<AccountForm>,
    account: Res<Account>,
    mut fields: Query<(&FormField, &mut BorderColor, &Children)>,
    mut field_texts: Query<&mut Text, Without<AccountText>>,
    mut texts: Query<(&AccountText, &mut Text)>,
    mut buttons: Query<(&AccountBtn, &mut ButtonEnabled)>,
) {
    for (field, mut border, children) in fields.iter_mut() {
        let edge = if form.focus == *field {
            ui::CREAM
        } else {
            ui::MOSS_EDGE
        };
        if border.top != edge {
            *border = BorderColor::all(edge);
        }
        let shown = form.display(*field);
        for child in children.iter() {
            if let Ok(mut text) = field_texts.get_mut(child) {
                if text.0 != shown {
                    text.0 = shown.clone();
                }
            }
        }
    }
    for (kind, mut text) in texts.iter_mut() {
        let value = match kind {
            AccountText::Remember => {
                let state = if form.remember { "On" } else { "Off" };
                format!("Keep me signed in: {state}")
            }
            AccountText::Error => account.error.clone().unwrap_or_default(),
            AccountText::Stats => stats_line(account.stats),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (btn, mut enabled) in buttons.iter_mut() {
        if matches!(btn, AccountBtn::SignIn | AccountBtn::Register) && enabled.0 != form.complete()
        {
            enabled.0 = form.complete();
        }
    }
}

pub fn btn_listeners(
    mut state: ResMut<NextState<AppState>>,
    mut form: ResMut<AccountForm>,
    mut requests: MessageWriter<PlatformRequest>,
    fields: Query<(&Interaction, &FormField), Changed<Interaction>>,
    buttons: Query<(&Interaction, &AccountBtn, Option<&ButtonEnabled>), Changed<Interaction>>,
) {
    for (interaction, field) in fields.iter() {
        if *interaction == Interaction::Pressed {
            form.focus = *field;
        }
    }
    for (interaction, btn, enabled) in buttons.iter() {
        if *interaction != Interaction::Pressed || enabled.is_some_and(|e| !e.0) {
            continue;
        }
        match btn {
            AccountBtn::SignIn => {
                requests.write(login(&form));
            }
            AccountBtn::Register => {
                requests.write(PlatformRequest::Register {
                    username: form.username.trim().to_owned(),
                    password: form.password.clone(),
                    remember: form.remember,
                });
            }
            AccountBtn::Remember => form.remember = !form.remember,
            AccountBtn::SignOut => {
                requests.write(PlatformRequest::Logout);
            }
            AccountBtn::Leaderboards => state.set(AppState::MenuLeaderboard),
            AccountBtn::Back => state.set(AppState::MenuMain),
        }
    }
}

pub fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<AccountUI>>) {
    // the password doesn't stay in memory after the screen is closed
    commands.remove_resource::<AccountForm>();
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_are_hidden_and_the_focus_has_a_cursor() {
        let form = AccountForm {
            username: "Shelly".to_owned(),
            password: "pässword".to_owned(),
            focus: FormField::Password,
            ..default()
        };
        assert_eq!(form.display(FormField::Username), "Shelly");
        assert_eq!(form.display(FormField::Password), "********|");
    }

    #[test]
    fn fields_stop_at_their_limit() {
        let mut form = AccountForm::default();
        push_chars(form.field_mut(FormField::Username), &"a".repeat(60));
        assert_eq!(form.username.len(), USERNAME_MAX_LEN);
        push_chars(form.field_mut(FormField::Password), "a\tb");
        assert_eq!(form.password, "ab");
    }

    #[test]
    fn a_form_needs_a_name_and_a_password() {
        let mut form = AccountForm {
            username: "  ".to_owned(),
            password: "pw".to_owned(),
            ..default()
        };
        assert!(!form.complete());
        form.username = "Shelly".to_owned();
        assert!(form.complete());
    }
}
