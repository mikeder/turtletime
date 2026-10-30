//! Keeps the session of a player who asked to stay signed in, so they don't have
//! to sign in again on the next launch. Only the refresh token is kept: on the
//! desktop in a file only the user can read, on the web in the site's local storage.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedSession {
    pub refresh_token: String,
    pub username: String,
}

// tests sign in and out, that must not touch the session of whoever runs them
pub fn load() -> Option<SavedSession> {
    if cfg!(test) {
        return None;
    }
    let saved: SavedSession = serde_json::from_str(&backend::read()?).ok()?;
    (!saved.refresh_token.is_empty()).then_some(saved)
}

pub fn save(session: &SavedSession) {
    if cfg!(test) {
        return;
    }
    match serde_json::to_string(session) {
        Ok(text) => backend::write(&text),
        Err(e) => bevy::log::warn!("could not save session: {e}"),
    }
}

pub fn clear() {
    if cfg!(test) {
        return;
    }
    backend::remove();
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use bevy::log::warn;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    const FILE: &str = "tgp_session.json";

    /// The folder each OS keeps app settings in.
    fn dir() -> Option<PathBuf> {
        let env = |key| std::env::var_os(key).filter(|v| !v.is_empty());
        let base = if cfg!(target_os = "windows") {
            PathBuf::from(env("APPDATA")?)
        } else if cfg!(target_os = "macos") {
            PathBuf::from(env("HOME")?).join("Library/Application Support")
        } else {
            env("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| Some(PathBuf::from(env("HOME")?).join(".config")))?
        };
        Some(base.join("turtletime"))
    }

    pub fn read() -> Option<String> {
        fs::read_to_string(dir()?.join(FILE)).ok()
    }

    pub fn write(text: &str) {
        let Some(dir) = dir() else {
            warn!("no settings folder, the session is not saved");
            return;
        };
        if let Err(e) = fs::create_dir_all(&dir).and_then(|_| write_private(&dir.join(FILE), text))
        {
            warn!("could not save session: {e}");
        }
    }

    /// The token signs the player in, so only they get to read it.
    fn write_private(path: &PathBuf, text: &str) -> std::io::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(path)?.write_all(text.as_bytes())
    }

    pub fn remove() {
        if let Some(dir) = dir() {
            let _ = fs::remove_file(dir.join(FILE));
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    const KEY: &str = "turtletime.tgp_session";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn read() -> Option<String> {
        storage()?.get_item(KEY).ok()?
    }

    pub fn write(text: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(KEY, text);
        }
    }

    pub fn remove() {
        if let Some(storage) = storage() {
            let _ = storage.remove_item(KEY);
        }
    }
}
