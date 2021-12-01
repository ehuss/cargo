use anyhow::Context;
use crate::util::errors::CargoResult;
use crate::Config;
use cargo_util::paths;
use serde::{de, ser, Deserialize, Serialize};
use std::collections::HashMap;
use std::time;
use std::fs;

const LAST_USE_FILENAME: &str = ".last-use";
const ON_DISK_VERSION: u32 = 0;

type Timestamp = u64;

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum LastUseKind {
    RegistryIndex,
    RegistryCrate(String),
    RegistrySrc(String),
    GitDb,
    GitCheckout(String),
    Unknown(String),
}

impl ser::Serialize for LastUseKind {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            LastUseKind::RegistryIndex => "registry_index".serialize(s),
            LastUseKind::RegistryCrate(name) => format!("registry_crate-{}", name).serialize(s),
            LastUseKind::RegistrySrc(name) => format!("registry_src-{}", name).serialize(s),
            LastUseKind::GitDb => "git_db".serialize(s),
            LastUseKind::GitCheckout(name) => format!("git_checkout-{}", name).serialize(s),
            LastUseKind::Unknown(value) => value.serialize(s),
        }
    }
}

impl<'de> de::Deserialize<'de> for LastUseKind {
    fn deserialize<D>(d: D) -> Result<LastUseKind, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        let mut parts = s.splitn(2, '-');
        let kind = parts
            .next()
            .ok_or_else(|| de::Error::custom(format!("last use kind invalid format {}", s)))?;
        let mut next = || {
            parts
                .next()
                .ok_or_else(|| de::Error::custom(format!("expected next part in {}", s)))
                .map(|s| s.to_string())
        };
        Ok(match kind {
            "registry_index" => LastUseKind::RegistryIndex,
            "registry_crate" => LastUseKind::RegistryCrate(next()?),
            "registry_src" => LastUseKind::RegistrySrc(next()?),
            "git_db" => LastUseKind::GitDb,
            "git_checkout" => LastUseKind::GitCheckout(next()?),
            _ => LastUseKind::Unknown(s),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct LastUse {
    version: u32,
    last_clean: Timestamp,
    timestamps: HashMap<LastUseKind, CachePaths>,
}

type CachePaths = HashMap<String, Timestamp>;

impl LastUse {
    pub fn load(config: &Config) -> CargoResult<LastUse> {
        let last_use_path = config.home().join(LAST_USE_FILENAME);
        let last_use_path = config.assert_package_cache_locked(&last_use_path);
        let last_use = if last_use_path.exists() {
            let contents = paths::read_bytes(last_use_path)?;
            bincode::deserialize(&contents)?
            // serde_json::from_str(&contents)?
        } else {
            LastUse {
                version: ON_DISK_VERSION,
                last_clean: to_timestamp(&time::SystemTime::now()),
                timestamps: HashMap::new(),
            }
        };
        Ok(last_use)
    }

    pub fn mark_used(&mut self, kind: LastUseKind, location: String) {
        self.mark_used_stamp(kind, location, &time::SystemTime::now());
    }

    pub fn mark_used_stamp(&mut self, kind: LastUseKind, location: String, timestamp: &time::SystemTime) {
        self.timestamps
            .entry(kind)
            .or_default()
            .insert(location, to_timestamp(timestamp));
    }

    pub fn save(&self, config: &Config) -> CargoResult<()> {
        let last_use_path = config.home().join(LAST_USE_FILENAME);
        let last_use_path = config.assert_package_cache_locked(&last_use_path);
        let tmp_path = last_use_path.with_extension(".tmp");
        let content = bincode::serialize(self)?;
        // let content = serde_json::to_string(self)?;
        paths::write(&tmp_path, content)?;
        fs::rename(&tmp_path, &last_use_path)
            .with_context(|| format!("failed to move last-use temp file {}", last_use_path.display()))?;
        Ok(())
    }
}

fn to_timestamp(t: &time::SystemTime) -> Timestamp {
        t.duration_since(time::SystemTime::UNIX_EPOCH + time::Duration::from_secs(1420070400))
        .expect("invalid clock")
        .as_secs()
}
