//! Accounts, stats, leaderboards and the message of the day from the Tiny Game
//! Platform. All of it is optional: players without an account play as before.
//!
//! Menus ask for things by writing a [`PlatformRequest`] and show what the
//! [`Account`], [`Motd`] and [`Leaderboards`] resources hold, which fill in as
//! the platform answers.

use super::api::{self, ApiError, Claims, StatValue, APP_ID};
use super::storage::{self, SavedSession};
use crate::player::resources::OnlineMatchFinished;
use bevy::prelude::*;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// Whether the game was built with an application id, without one there are no accounts.
pub fn enabled() -> bool {
    !APP_ID.is_empty()
}

/// Refresh the access token this long before it runs out.
const REFRESH_LEAD_SECS: u64 = 60;
/// Used when the token doesn't say how long it lasts.
const FALLBACK_LIFETIME_SECS: u64 = 15 * 60;
/// Wait this long before trying a refresh again that failed for network reasons.
const REFRESH_RETRY_SECS: f32 = 30.;
/// Don't ask for the MOTD more often than this.
const MOTD_MIN_INTERVAL_SECS: f64 = 60.;
/// How many entries of a leaderboard are shown.
const LEADERBOARD_PAGE: u32 = 10;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlatformRequest>()
            .init_resource::<Inbox>()
            .init_resource::<Account>()
            .init_resource::<Motd>()
            .init_resource::<Leaderboards>()
            .add_systems(Startup, restore_session.run_if(enabled))
            .add_systems(
                Update,
                (
                    count_matches,
                    handle_requests,
                    handle_answers,
                    refresh_before_expiry,
                )
                    .chain()
                    .run_if(enabled),
            );
    }
}

/// What menus can ask the platform for.
#[derive(Message, Debug, Clone)]
pub enum PlatformRequest {
    Login {
        username: String,
        password: String,
        remember: bool,
    },
    /// creates the account, then signs in with it
    Register {
        username: String,
        password: String,
        remember: bool,
    },
    Logout,
    FetchMotd,
    FetchMyStats,
    FetchLeaderboards,
    /// shows this leaderboard, by key
    FetchLeaderboard(String),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    #[default]
    SignedOut,
    /// signing in, creating an account or restoring the saved session
    Working,
    SignedIn,
}

/// The stats an online match adds to.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PlayerStats {
    pub matches: f64,
    pub wins: f64,
    pub losses: f64,
}

impl PlayerStats {
    fn update(&mut self, stats: &[StatValue]) {
        for stat in stats {
            match stat.key.as_str() {
                api::STAT_MATCHES => self.matches = stat.value,
                api::STAT_WINS => self.wins = stat.value,
                api::STAT_LOSSES => self.losses = stat.value,
                _ => (),
            }
        }
    }
}

/// The player's account, signed out unless they signed in.
#[derive(Resource, Default)]
pub struct Account {
    pub status: AccountStatus,
    pub username: String,
    pub player_id: String,
    /// why the last sign in or account creation didn't work
    pub error: Option<String>,
    /// unknown until fetched
    pub stats: Option<PlayerStats>,
    access_token: String,
    refresh_token: String,
    /// keep the session for the next launch
    remember: bool,
    refreshing: bool,
    refresh_timer: Timer,
    /// match results that couldn't be sent yet, sent once the session is refreshed
    pending_stats: Vec<StatValue>,
}

impl Account {
    pub fn signed_in(&self) -> bool {
        self.status == AccountStatus::SignedIn
    }

    fn token(&self) -> Option<&str> {
        (!self.access_token.is_empty()).then_some(self.access_token.as_str())
    }

    /// Takes the tokens of a login or refresh, false if they are no good.
    fn take_tokens(&mut self, tokens: api::Tokens) -> bool {
        let Some(claims) = Claims::read(&tokens.access_token) else {
            return false;
        };
        if tokens.refresh_token.is_empty() {
            return false;
        }
        let lifetime = claims.lifetime_secs().unwrap_or(FALLBACK_LIFETIME_SECS);
        let wait = lifetime.saturating_sub(REFRESH_LEAD_SECS).max(5);
        self.refresh_timer = Timer::from_seconds(wait as f32, TimerMode::Once);
        self.access_token = tokens.access_token;
        self.refresh_token = tokens.refresh_token;
        self.username = claims.u;
        self.player_id = claims.sub;
        self.status = AccountStatus::SignedIn;
        self.error = None;
        if self.remember {
            storage::save(&SavedSession {
                refresh_token: self.refresh_token.clone(),
                username: self.username.clone(),
            });
        } else {
            storage::clear();
        }
        true
    }

    /// Forgets the session here, the saved one too.
    fn clear(&mut self) {
        let error = self.error.take();
        *self = Account { error, ..default() };
        storage::clear();
    }
}

/// The message of the day, empty when there is none.
#[derive(Resource, Default)]
pub struct Motd {
    pub text: String,
    /// when it was last asked for, in seconds since the game started
    fetched_at: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Failed(String),
}

/// The leaderboards of the game and the one being looked at.
#[derive(Resource, Default)]
pub struct Leaderboards {
    pub boards: Vec<api::Leaderboard>,
    pub status: LoadStatus,
    /// key of the leaderboard being looked at
    pub selected: Option<String>,
    pub entries: Vec<api::LeaderboardEntry>,
    pub total: u32,
    /// where the signed in player is on the selected leaderboard
    pub me: Option<api::LeaderboardEntry>,
    pub board_status: LoadStatus,
}

/// What a call was for, so its answer ends up in the right place.
#[derive(Debug, Clone)]
enum Call {
    Login,
    Register { username: String, password: String },
    Refresh,
    Logout,
    Motd,
    SubmitStats(Vec<StatValue>),
    MyStats,
    ListLeaderboards,
    Leaderboard(String),
    LeaderboardMe(String),
}

type Answers = Vec<(Call, Result<Value, ApiError>)>;

/// Answers of the platform, collected until the next frame handles them.
#[derive(Resource, Default, Clone)]
struct Inbox(Arc<Mutex<Answers>>);

impl Inbox {
    fn send(&self, call: Call, method: &str, body: Value, token: Option<&str>) {
        let inbox = self.0.clone();
        api::send(method, &body, token, move |answer| {
            if let Ok(mut answers) = inbox.lock() {
                answers.push((call, answer));
            }
        });
    }

    fn take(&self) -> Answers {
        self.0
            .lock()
            .map(|mut answers| std::mem::take(&mut *answers))
            .unwrap_or_default()
    }
}

fn restore_session(mut account: ResMut<Account>, inbox: Res<Inbox>) {
    let Some(saved) = storage::load() else {
        return;
    };
    account.refresh_token = saved.refresh_token;
    account.username = saved.username;
    account.remember = true;
    account.status = AccountStatus::Working;
    start_refresh(&mut account, &inbox);
}

/// Refresh tokens can only be used once, so there is never more than one refresh
/// on the way, a second one would be refused and sign the player out.
fn start_refresh(account: &mut Account, inbox: &Inbox) {
    if account.refreshing || account.refresh_token.is_empty() {
        return;
    }
    account.refreshing = true;
    inbox.send(
        Call::Refresh,
        api::REFRESH,
        json!({ "refresh_token": account.refresh_token }),
        None,
    );
}

fn submit_stats(account: &mut Account, inbox: &Inbox, stats: Vec<StatValue>) {
    let stats = api::merge_stats(stats);
    if stats.is_empty() {
        return;
    }
    // no token right now, it is sent once the session is back
    let Some(token) = account.token().filter(|_| !account.refreshing) else {
        account.pending_stats.extend(stats);
        return;
    };
    let body = json!({
        "stats": stats
            .iter()
            .map(|s| json!({ "key": s.key, "value": s.value }))
            .collect::<Vec<_>>(),
    });
    inbox.send(
        Call::SubmitStats(stats),
        api::SUBMIT_STATS,
        body,
        Some(token),
    );
}

/// Online matches count toward the stats of signed in players. Local matches
/// don't, every turtle in those is played by the same person.
fn count_matches(
    mut finished: MessageReader<OnlineMatchFinished>,
    mut account: ResMut<Account>,
    inbox: Res<Inbox>,
) {
    for result in finished.read() {
        // a session that is being restored counts too, the result waits for it
        if account.status == AccountStatus::SignedOut {
            continue;
        }
        submit_stats(&mut account, &inbox, api::match_stats(result.won));
    }
}

fn handle_requests(
    mut requests: MessageReader<PlatformRequest>,
    mut account: ResMut<Account>,
    mut motd: ResMut<Motd>,
    mut leaderboards: ResMut<Leaderboards>,
    inbox: Res<Inbox>,
    time: Res<Time<Real>>,
) {
    for request in requests.read() {
        match request {
            PlatformRequest::Login {
                username,
                password,
                remember,
            } => {
                if account.status != AccountStatus::SignedOut {
                    continue;
                }
                account.status = AccountStatus::Working;
                account.error = None;
                account.remember = *remember;
                send_login(&inbox, username, password, *remember);
            }
            PlatformRequest::Register {
                username,
                password,
                remember,
            } => {
                if account.status != AccountStatus::SignedOut {
                    continue;
                }
                account.status = AccountStatus::Working;
                account.error = None;
                account.remember = *remember;
                inbox.send(
                    Call::Register {
                        username: username.clone(),
                        password: password.clone(),
                    },
                    api::REGISTER,
                    json!({ "username": username, "password": password }),
                    None,
                );
            }
            PlatformRequest::Logout => {
                if !account.refresh_token.is_empty() {
                    inbox.send(
                        Call::Logout,
                        api::LOGOUT,
                        json!({ "tokens": { "refresh_token": account.refresh_token } }),
                        None,
                    );
                }
                account.error = None;
                account.clear();
            }
            PlatformRequest::FetchMotd => {
                let now = time.elapsed_secs_f64();
                if motd
                    .fetched_at
                    .is_some_and(|at| now - at < MOTD_MIN_INTERVAL_SECS)
                {
                    continue;
                }
                motd.fetched_at = Some(now);
                // title data can be read without signing in
                inbox.send(
                    Call::Motd,
                    api::TITLE_DATA,
                    json!({ "application_id": APP_ID, "keys": ["motd"] }),
                    None,
                );
            }
            PlatformRequest::FetchMyStats => {
                if let Some(token) = account.token() {
                    inbox.send(
                        Call::MyStats,
                        api::MY_STATS,
                        json!({ "keys": [api::STAT_MATCHES, api::STAT_WINS, api::STAT_LOSSES] }),
                        Some(token),
                    );
                }
            }
            PlatformRequest::FetchLeaderboards => {
                leaderboards.status = LoadStatus::Loading;
                inbox.send(
                    Call::ListLeaderboards,
                    api::LIST_LEADERBOARDS,
                    json!({ "application_id": APP_ID }),
                    None,
                );
            }
            PlatformRequest::FetchLeaderboard(key) => {
                fetch_leaderboard(&mut leaderboards, &account, &inbox, key);
            }
        }
    }
}

fn send_login(inbox: &Inbox, username: &str, password: &str, remember: bool) {
    inbox.send(
        Call::Login,
        api::LOGIN,
        json!({
            "application_id": APP_ID,
            "username": username,
            "password": password,
            "session_only": !remember,
        }),
        None,
    );
}

fn fetch_leaderboard(leaderboards: &mut Leaderboards, account: &Account, inbox: &Inbox, key: &str) {
    leaderboards.selected = Some(key.to_owned());
    leaderboards.entries.clear();
    leaderboards.total = 0;
    leaderboards.me = None;
    leaderboards.board_status = LoadStatus::Loading;
    inbox.send(
        Call::Leaderboard(key.to_owned()),
        api::GET_LEADERBOARD,
        json!({ "application_id": APP_ID, "key": key, "limit": LEADERBOARD_PAGE }),
        None,
    );
    if let Some(token) = account.token() {
        inbox.send(
            Call::LeaderboardMe(key.to_owned()),
            api::LEADERBOARD_AROUND_PLAYER,
            json!({ "key": key, "radius": 1 }),
            Some(token),
        );
    }
}

fn handle_answers(
    mut account: ResMut<Account>,
    mut motd: ResMut<Motd>,
    mut leaderboards: ResMut<Leaderboards>,
    inbox: Res<Inbox>,
) {
    for (call, answer) in inbox.take() {
        match (call, answer) {
            (Call::Login, Ok(answer)) => {
                let answer: api::TokensResponse = api::decode(answer);
                if !account.take_tokens(answer.tokens) {
                    account.error = Some("The server sent a broken answer".to_owned());
                    account.clear();
                }
            }
            (Call::Login, Err(err)) => {
                account.error = Some(err.player_message());
                account.clear();
            }
            (Call::Register { username, password }, Ok(_)) => {
                let remember = account.remember;
                send_login(&inbox, &username, &password, remember);
            }
            (Call::Register { .. }, Err(err)) => {
                account.error = Some(err.player_message());
                account.clear();
            }
            (Call::Refresh, answer) => {
                account.refreshing = false;
                match answer {
                    Ok(answer) => {
                        let answer: api::TokensResponse = api::decode(answer);
                        if !account.take_tokens(answer.tokens) {
                            account.clear();
                            continue;
                        }
                        let pending = std::mem::take(&mut account.pending_stats);
                        submit_stats(&mut account, &inbox, pending);
                    }
                    Err(err) if err.is_transient() => {
                        // the session may still be good, the server just didn't answer
                        warn!("session refresh failed, trying again: {err:?}");
                        account.refresh_timer =
                            Timer::from_seconds(REFRESH_RETRY_SECS, TimerMode::Once);
                    }
                    Err(err) => {
                        // expired, revoked or banned, the player has to sign in again
                        warn!("session ended: {err:?}");
                        account.error = Some(format!("Signed out: {}", err.player_message()));
                        account.clear();
                    }
                }
            }
            (Call::Logout, _) => (),
            (Call::Motd, Ok(answer)) => {
                let answer: api::TitleDataResponse = api::decode(answer);
                let text = answer
                    .items
                    .iter()
                    .find(|item| item.key == "motd")
                    .map(|item| api::parse_motd(&item.value))
                    .unwrap_or_default();
                if motd.text != text {
                    motd.text = text;
                }
            }
            (Call::Motd, Err(err)) => {
                warn!("could not fetch the message of the day: {err:?}");
                // try again next time the main menu opens
                motd.fetched_at = None;
            }
            (Call::SubmitStats(_), Ok(answer)) => {
                let answer: api::StatsResponse = api::decode(answer);
                account.stats.get_or_insert_default().update(&answer.stats);
            }
            (Call::SubmitStats(stats), Err(err)) => {
                if err.is_unauthenticated() || err.is_transient() {
                    // the token ran out (the game may have been in the background)
                    // or the server is busy, send it with the next session
                    account.pending_stats.extend(stats);
                    if err.is_unauthenticated() {
                        start_refresh(&mut account, &inbox);
                    }
                } else {
                    // likely a stat that isn't declared in the dashboard
                    error!("match result was refused: {err:?}");
                }
            }
            (Call::MyStats, Ok(answer)) => {
                let answer: api::StatsResponse = api::decode(answer);
                let mut stats = PlayerStats::default();
                stats.update(&answer.stats);
                account.stats = Some(stats);
            }
            (Call::MyStats, Err(err)) => {
                warn!("could not fetch stats: {err:?}");
                if err.is_unauthenticated() {
                    start_refresh(&mut account, &inbox);
                }
            }
            (Call::ListLeaderboards, Ok(answer)) => {
                let answer: api::ListLeaderboardsResponse = api::decode(answer);
                leaderboards.boards = answer.leaderboards;
                leaderboards.status = LoadStatus::Loaded;
                // show the one looked at last time, or the first
                let selected = leaderboards
                    .selected
                    .clone()
                    .filter(|key| leaderboards.boards.iter().any(|b| &b.key == key))
                    .or_else(|| leaderboards.boards.first().map(|b| b.key.clone()));
                if let Some(key) = selected {
                    fetch_leaderboard(&mut leaderboards, &account, &inbox, &key);
                }
            }
            (Call::ListLeaderboards, Err(err)) => {
                leaderboards.status = LoadStatus::Failed(err.player_message());
            }
            (Call::Leaderboard(key), answer) => {
                // the player may have picked another one in the meantime
                if leaderboards.selected.as_ref() != Some(&key) {
                    continue;
                }
                match answer {
                    Ok(answer) => {
                        let answer: api::GetLeaderboardResponse = api::decode(answer);
                        leaderboards.entries = answer.entries;
                        leaderboards.total = answer.total;
                        leaderboards.board_status = LoadStatus::Loaded;
                    }
                    Err(err) => {
                        leaderboards.board_status = LoadStatus::Failed(err.player_message());
                    }
                }
            }
            (Call::LeaderboardMe(key), Ok(answer)) => {
                if leaderboards.selected.as_ref() == Some(&key) {
                    let answer: api::AroundPlayerResponse = api::decode(answer);
                    leaderboards.me = answer.player;
                }
            }
            (Call::LeaderboardMe(_), Err(err)) => {
                // not being on the leaderboard isn't an error, so this is rare
                warn!("could not fetch own leaderboard entry: {err:?}");
            }
        }
    }
}

fn refresh_before_expiry(mut account: ResMut<Account>, inbox: Res<Inbox>, time: Res<Time<Real>>) {
    if account.refresh_token.is_empty() || account.refreshing {
        return;
    }
    // only borrow mutably when the timer runs out, so the account isn't marked as
    // changed every frame
    let remaining = account.refresh_timer.remaining();
    if remaining > time.delta() {
        account
            .bypass_change_detection()
            .refresh_timer
            .tick(time.delta());
        return;
    }
    start_refresh(&mut account, &inbox);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    fn tokens(lifetime: u64) -> api::Tokens {
        let claims = json!({"sub": "p1", "u": "Shelly", "iat": 1000, "exp": 1000 + lifetime});
        api::Tokens {
            access_token: format!("e30.{}.c2ln", URL_SAFE_NO_PAD.encode(claims.to_string())),
            refresh_token: "refresh".to_owned(),
        }
    }

    #[test]
    fn tokens_sign_the_player_in_and_schedule_a_refresh() {
        let mut account = Account {
            error: Some("old".to_owned()),
            ..default()
        };
        assert!(account.take_tokens(tokens(900)));
        assert!(account.signed_in());
        assert_eq!(account.username, "Shelly");
        assert_eq!(account.player_id, "p1");
        assert_eq!(account.error, None);
        assert_eq!(account.refresh_timer.duration().as_secs(), 840);

        // a very short token is still refreshed a bit later, not in a tight loop
        assert!(account.take_tokens(tokens(30)));
        assert_eq!(account.refresh_timer.duration().as_secs(), 5);

        let broken = api::Tokens {
            access_token: "nope".to_owned(),
            refresh_token: "refresh".to_owned(),
        };
        assert!(!account.take_tokens(broken));
    }

    #[test]
    fn results_wait_while_there_is_no_token() {
        let mut account = Account {
            status: AccountStatus::Working,
            refresh_token: "refresh".to_owned(),
            refreshing: true,
            ..default()
        };
        submit_stats(&mut account, &Inbox::default(), api::match_stats(true));
        submit_stats(&mut account, &Inbox::default(), api::match_stats(false));
        let pending = api::merge_stats(account.pending_stats.clone());
        assert_eq!(pending.len(), 3);
        assert_eq!(
            pending
                .iter()
                .find(|s| s.key == api::STAT_MATCHES)
                .unwrap()
                .value,
            2.
        );
    }

    #[test]
    fn stats_only_take_the_keys_they_know() {
        let mut stats = PlayerStats::default();
        stats.update(&[
            StatValue {
                key: api::STAT_WINS.to_owned(),
                value: 3.,
            },
            StatValue {
                key: "fireballs".to_owned(),
                value: 99.,
            },
        ]);
        assert_eq!(
            stats,
            PlayerStats {
                wins: 3.,
                ..default()
            }
        );
    }
}
