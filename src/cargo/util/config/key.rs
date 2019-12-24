use std::fmt;

/// Key for a configuration variable.
///
/// This type represents a configuration variable that we're looking up in
/// Cargo's configuration. This structure simultaneously keeps track of a
/// corresponding environment variable name as well as a TOML config name. The
/// intention here is that this is built up and torn down over time efficiently,
/// avoiding clones and such as possible.
#[derive(Debug, Clone)]
pub struct ConfigKey {
    // The current environment variable this configuration key maps to. This is
    // updated with `push` methods and looks like `CARGO_FOO_BAR` for pushing
    // `foo` and then `bar`.
    env: String,
    // This is used to keep track of how many sub-keys have been pushed on
    // this `ConfigKey`. Each element of this vector is a new sub-key pushed
    // onto this `ConfigKey`. Each element is a pair where the first item is
    // the key part as a string, and the second item is an index into `env`.
    // The `env` index is used on `pop` to truncate `env` to rewind back to
    // the previous `ConfigKey` state before a `push`.
    parts: Vec<(String, usize)>,
}

impl ConfigKey {
    /// Creates a new blank configuration key which is ready to get built up by
    /// using `push`.
    pub fn new() -> ConfigKey {
        ConfigKey {
            env: "CARGO".to_string(),
            parts: Vec::new(),
        }
    }

    /// Creates a `ConfigKey` from the `key` specified.
    ///
    /// The `key` specified is expected to be a period-separated toml
    /// configuration key.
    pub fn from_str(key: &str) -> ConfigKey {
        let mut cfg = ConfigKey::new();
        for part in key.split('.') {
            cfg.push(part);
        }
        cfg
    }

    /// Pushes a new sub-key on this `ConfigKey`. This sub-key should be
    /// equivalent to accessing a sub-table in TOML.
    ///
    /// Note that this considers `name` to be case-insensitive, meaning that the
    /// corrseponding toml key is appended with this `name` as-is and the
    /// corresponding env key is appended with `name` after transforming it to
    /// uppercase characters.
    pub fn push(&mut self, name: &str) {
        let env = name.replace("-", "_").to_uppercase();
        self.parts.push((name.to_string(), self.env.len()));
        self.env.push_str("_");
        self.env.push_str(&env);
    }

    /// Rewinds this `ConfigKey` back to the state it was at before the last
    /// `push` method being called.
    pub fn pop(&mut self) {
        let (_part, env) = self.parts.pop().unwrap();
        self.env.truncate(env);
    }

    /// Returns the corresponding environment variable key for this
    /// configuration value.
    pub fn as_env_key(&self) -> &str {
        &self.env
    }

    /// Returns an iterator of the key parts as strings.
    pub(super) fn parts(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().map(|p| p.0.as_ref())
    }
}

impl fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note: This is not a perfect TOML representation. This really should
        // check if the parts should be quoted.
        let parts: Vec<&str> = self.parts().collect();
        parts.join(".").fmt(f)
    }
}
