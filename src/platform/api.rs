//! Calls to the Tiny Game Platform and the messages it answers with.
//!
//! The platform speaks Connect: every call is a JSON `POST` to
//! `<base url><package>.<Service>/<Method>`. Answers use camelCase field names
//! and leave out fields that hold their default value, so every message here
//! defaults the fields it doesn't get.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

/// Where the platform is, `TGP_BASE_URL` at build time points the game elsewhere.
pub const BASE_URL: &str = match option_env!("TGP_BASE_URL") {
    Some(url) => url,
    None => "https://api.tgp.sqweeb.net/",
};

/// The game's application id from the platform dashboard, `TGP_APP_ID` at build
/// time overrides it. Without one the game has no accounts, leaderboards or MOTD.
pub const APP_ID: &str = match option_env!("TGP_APP_ID") {
    Some(id) => id,
    None => "834d9081-25dd-4c40-87e1-a9833835d06c",
};

const TIMEOUT: Duration = Duration::from_secs(15);

pub const LOGIN: &str = "tinygameplatform.auth.v1.AuthService/Login";
pub const REGISTER: &str = "tinygameplatform.auth.v1.AuthService/Register";
pub const REFRESH: &str = "tinygameplatform.auth.v1.AuthService/RefreshToken";
pub const LOGOUT: &str = "tinygameplatform.auth.v1.AuthService/Logout";
pub const TITLE_DATA: &str = "tinygameplatform.title.v1.TitleService/GetTitleData";
pub const SUBMIT_STATS: &str = "tinygameplatform.leaderboard.v1.StatsService/SubmitStats";
pub const MY_STATS: &str = "tinygameplatform.leaderboard.v1.StatsService/GetMyStats";
pub const LIST_LEADERBOARDS: &str =
    "tinygameplatform.leaderboard.v1.LeaderboardService/ListLeaderboards";
pub const GET_LEADERBOARD: &str =
    "tinygameplatform.leaderboard.v1.LeaderboardService/GetLeaderboard";
pub const LEADERBOARD_AROUND_PLAYER: &str =
    "tinygameplatform.leaderboard.v1.LeaderboardService/GetLeaderboardAroundPlayer";

/// Stats an online match adds to, each has to be declared in the dashboard as a
/// SUM stat that players may write.
pub const STAT_MATCHES: &str = "matches_played";
pub const STAT_WINS: &str = "wins";
pub const STAT_LOSSES: &str = "losses";

/// Why a call failed.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// HTTP status, 0 when the platform never answered
    pub status: u16,
    /// Connect error code, like "unauthenticated"
    pub code: String,
    /// what the platform said went wrong, it is written to be shown to players
    pub message: String,
}

impl ApiError {
    fn unreachable(reason: impl Into<String>) -> Self {
        ApiError {
            status: 0,
            code: "unavailable".to_owned(),
            message: reason.into(),
        }
    }

    fn from_response(status: u16, body: &[u8]) -> Self {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct ConnectError {
            code: String,
            message: String,
        }
        let err: ConnectError = serde_json::from_slice(body).unwrap_or_default();
        ApiError {
            status,
            code: err.code,
            message: err.message,
        }
    }

    /// The access token is no good anymore, a refresh may fix that.
    pub fn is_unauthenticated(&self) -> bool {
        self.status == 401 || self.code == "unauthenticated"
    }

    /// Nothing is wrong with the request, asking again later may work.
    pub fn is_transient(&self) -> bool {
        self.status == 0 || self.status >= 500
    }

    /// A message to show the player.
    pub fn player_message(&self) -> String {
        if self.status == 0 {
            "Could not reach the server".to_owned()
        } else if !self.message.is_empty() {
            // the platform starts its messages lower case
            let mut chars = self.message.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        } else {
            format!("Something went wrong (HTTP {})", self.status)
        }
    }
}

/// Sends a call to the platform, `on_done` gets the answer on another thread
/// (native) or later in the same one (web).
pub fn send(
    method: &str,
    body: &Value,
    token: Option<&str>,
    on_done: impl 'static + Send + FnOnce(Result<Value, ApiError>),
) {
    let base = BASE_URL.trim_end_matches('/');
    let mut request = ehttp::Request::post(
        format!("{base}/{method}"),
        serde_json::to_vec(body).unwrap_or_default(),
    );
    request.headers = ehttp::Headers::new(&[
        ("Accept", "application/json"),
        ("Content-Type", "application/json"),
    ]);
    if let Some(token) = token {
        request
            .headers
            .insert("Authorization", format!("Bearer {token}"));
    }
    request.timeout = Some(TIMEOUT);
    ehttp::fetch(request, move |result| on_done(read_response(result)));
}

fn read_response(result: ehttp::Result<ehttp::Response>) -> Result<Value, ApiError> {
    let response = result.map_err(ApiError::unreachable)?;
    if !response.ok {
        return Err(ApiError::from_response(response.status, &response.bytes));
    }
    if response.bytes.is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    serde_json::from_slice(&response.bytes)
        .map_err(|e| ApiError::unreachable(format!("unreadable answer: {e}")))
}

/// Reads an answer as `T`, fields it doesn't have keep their default.
pub fn decode<T: DeserializeOwned + Default>(value: Value) -> T {
    serde_json::from_value(value).unwrap_or_default()
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// The answer to Login and RefreshToken.
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct TokensResponse {
    pub tokens: Tokens,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct TitleDataItem {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct TitleDataResponse {
    pub items: Vec<TitleDataItem>,
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct StatValue {
    pub key: String,
    pub value: f64,
}

/// The answer to SubmitStats and GetMyStats, the stored values afterwards.
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct StatsResponse {
    pub stats: Vec<StatValue>,
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Leaderboard {
    pub key: String,
    pub display_name: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct ListLeaderboardsResponse {
    pub leaderboards: Vec<Leaderboard>,
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player_id: String,
    pub player_name: String,
    pub value: f64,
    /// the value formatted the way the leaderboard is set up to show it
    pub display_value: String,
}

impl LeaderboardEntry {
    pub fn shown_value(&self) -> String {
        if self.display_value.is_empty() {
            self.value.to_string()
        } else {
            self.display_value.clone()
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct GetLeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total: u32,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct AroundPlayerResponse {
    /// unset when the player isn't on the leaderboard
    pub player: Option<LeaderboardEntry>,
}

/// What an access token says about the player, read without checking the
/// signature, only the platform can do that.
#[derive(Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Claims {
    /// player id
    pub sub: String,
    /// username
    pub u: String,
    /// issued at, unix seconds
    pub iat: u64,
    /// expires at, unix seconds
    pub exp: u64,
}

impl Claims {
    pub fn read(token: &str) -> Option<Claims> {
        let mut parts = token.split('.');
        let (Some(_), Some(payload), Some(_), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// How long the token is good for, from when it was issued.
    pub fn lifetime_secs(&self) -> Option<u64> {
        (self.iat > 0 && self.exp > self.iat).then(|| self.exp - self.iat)
    }
}

/// Longest MOTD shown, the main menu has room for a couple of lines.
const MOTD_MAX_LEN: usize = 300;

/// The `motd` title data is `{"text": "...", "url": "..."}` or a JSON string,
/// anything else is shown as it is.
pub fn parse_motd(raw: &str) -> String {
    let text = match serde_json::from_str::<Value>(raw) {
        Ok(Value::String(text)) => text,
        Ok(Value::Object(fields)) => fields
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        _ => raw.to_owned(),
    };
    text.chars()
        .filter(|c| *c == '\n' || !c.is_control())
        .take(MOTD_MAX_LEN)
        .collect::<String>()
        .trim()
        .to_owned()
}

/// What an online match adds to the stats of the local player.
pub fn match_stats(won: bool) -> Vec<StatValue> {
    let result = if won { STAT_WINS } else { STAT_LOSSES };
    [STAT_MATCHES, result]
        .into_iter()
        .map(|key| StatValue {
            key: key.to_owned(),
            value: 1.,
        })
        .collect()
}

/// Adds up stats waiting to be sent, so each key is sent once.
pub fn merge_stats(stats: impl IntoIterator<Item = StatValue>) -> Vec<StatValue> {
    let mut merged: Vec<StatValue> = Vec::new();
    for stat in stats {
        match merged.iter_mut().find(|s| s.key == stat.key) {
            Some(existing) => existing.value += stat.value,
            None => merged.push(stat),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn token(claims: Value) -> String {
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
        format!("e30.{payload}.c2ln")
    }

    #[test]
    fn claims_are_read_from_the_token() {
        let t =
            token(json!({"sub": "p1", "u": "Shelly", "iat": 1000, "exp": 1900, "typ": "access"}));
        let claims = Claims::read(&t).unwrap();
        assert_eq!(claims.sub, "p1");
        assert_eq!(claims.u, "Shelly");
        assert_eq!(claims.lifetime_secs(), Some(900));

        assert_eq!(Claims::read("not a token"), None);
        assert_eq!(Claims::read("a.b.c.d"), None);
        assert_eq!(
            Claims::read(&token(json!({}))).unwrap().lifetime_secs(),
            None
        );
    }

    #[test]
    fn motd_can_be_an_object_a_string_or_plain_text() {
        assert_eq!(
            parse_motd(r#"{"text": " Season 2 starts today! ", "url": "https://x"}"#),
            "Season 2 starts today!"
        );
        assert_eq!(parse_motd(r#""Hello turtles""#), "Hello turtles");
        assert_eq!(parse_motd("not json"), "not json");
        assert_eq!(parse_motd(r#"{"url": "https://x"}"#), "");
        assert_eq!(parse_motd("\"two\\nlines\\u0007\""), "two\nlines");
        assert_eq!(parse_motd(&format!("\"{}\"", "a".repeat(500))).len(), 300);
    }

    #[test]
    fn errors_read_the_connect_message() {
        let err = ApiError::from_response(
            401,
            br#"{"code": "unauthenticated", "message": "token expired"}"#,
        );
        assert!(err.is_unauthenticated());
        assert!(!err.is_transient());
        assert_eq!(err.player_message(), "Token expired");

        let err = ApiError::from_response(502, b"<html>bad gateway</html>");
        assert!(err.is_transient());
        assert_eq!(err.player_message(), "Something went wrong (HTTP 502)");

        assert_eq!(
            ApiError::unreachable("timeout").player_message(),
            "Could not reach the server"
        );
    }

    #[test]
    fn leaderboard_answers_leave_out_defaults() {
        // uint64 fields come as strings and zero values are left out
        let page: GetLeaderboardResponse = decode(json!({
            "entries": [
                {"rank": 1, "playerId": "p1", "playerName": "Shelly", "value": 12,
                 "updatedAt": "1761800000", "displayValue": "12"},
                {"rank": 2, "playerId": "p2", "playerName": "Leo"}
            ],
            "total": 2
        }));
        assert_eq!(page.total, 2);
        assert_eq!(page.entries[0].player_name, "Shelly");
        assert_eq!(page.entries[0].shown_value(), "12");
        assert_eq!(page.entries[1].value, 0.);
        assert_eq!(page.entries[1].shown_value(), "0");

        let around: AroundPlayerResponse = decode(json!({"leaderboard": {"key": "wins"}}));
        assert_eq!(around.player, None);
    }

    #[test]
    fn a_match_counts_once_and_as_a_win_or_a_loss() {
        let won = match_stats(true);
        assert_eq!(won.len(), 2);
        assert!(won.iter().any(|s| s.key == STAT_WINS && s.value == 1.));
        assert!(!won.iter().any(|s| s.key == STAT_LOSSES));

        let merged = merge_stats(match_stats(true).into_iter().chain(match_stats(false)));
        let value = |key: &str| merged.iter().find(|s| s.key == key).unwrap().value;
        assert_eq!(merged.len(), 3);
        assert_eq!(value(STAT_MATCHES), 2.);
        assert_eq!(value(STAT_WINS), 1.);
        assert_eq!(value(STAT_LOSSES), 1.);
    }
}
