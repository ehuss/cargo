// use anyhow::Context;
use crate::util::errors::CargoResult;
use crate::Config;
// use cargo_util::paths;
use std::collections::HashMap;
use std::time;
// use std::fs;
use rusqlite::{params, Connection};

const LAST_USE_FILENAME: &str = ".last-use";
// const ON_DISK_VERSION: u32 = 0;

type Timestamp = u64;

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum LastUseKind {
    RegistryIndex,
    RegistryCrate(String),
    RegistrySrc(String),
    GitDb,
    GitCheckout(String),
    // Unknown(String),
}

pub struct LastUse {
    connection: Connection,
    registry_sources: HashMap<String, i64>,
    // version: u32,
    // last_clean: Timestamp,
    // timestamps: HashMap<LastUseKind, CachePaths>,
}

impl LastUse {
    pub fn load(config: &Config) -> CargoResult<LastUse> {
        let last_use_path = config.home().join(LAST_USE_FILENAME);
        let last_use_path = config.assert_package_cache_locked(&last_use_path);
        let connection = Connection::open(last_use_path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        // TODO: last_clean (systemtime::now)
        connection.execute(
            "CREATE TABLE IF NOT EXISTS registry_index (
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL
              )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS registry_crate (
                source INT NOT NULL,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL
              )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS registry_src (
                source INT NOT NULL,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL
              )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS git_db (
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL
              )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS git_checkout (
                source TEXT NOT NULL,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL
              )",
            [],
        )?;
        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS registry_crate_idx ON registry_index (name)",
            [],
        )?;
        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS registry_crate_idx ON registry_crate (source, name)",
            [],
        )?;
        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS registry_src_idx ON registry_src (source, name)",
            [],
        )?;
        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS git_db_idx ON git_db (name)",
            [],
        )?;
        connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS git_checkout_idx ON git_checkout (source, name)",
            [],
        )?;
        connection.execute("BEGIN TRANSACTION", [])?;
        Ok(LastUse {
            connection,
            registry_sources: HashMap::new(),
        })
    }

    pub fn mark_used(&mut self, kind: LastUseKind, location: String) {
        self.mark_used_stamp(kind, location, &time::SystemTime::now());
    }

    pub fn mark_used_stamp(
        &mut self,
        kind: LastUseKind,
        location: String,
        timestamp: &time::SystemTime,
    ) {
        match kind {
            LastUseKind::RegistryIndex => {
                let mut stmt = self
                    .connection
                    .prepare_cached("REPLACE INTO registry_index (name, timestamp) values (?1, ?2)")
                    .unwrap();
                stmt.execute(params![location, to_timestamp(timestamp)])
                    .unwrap();
            }
            LastUseKind::RegistryCrate(source) => {
                let source_id = self.source_id(&source);
                // TODO: unwrap
                let mut stmt = self
                    .connection
                    .prepare_cached(
                        "REPLACE INTO registry_crate (source, name, timestamp) values (?1, ?2, ?3)",
                    )
                    .unwrap();
                stmt.execute(params![source_id, location, to_timestamp(timestamp)])
                    .unwrap();
            }
            LastUseKind::RegistrySrc(source) => {
                // TODO: unwrap
                let source_id = self.source_id(&source);
                let mut stmt = self
                    .connection
                    .prepare_cached(
                        "REPLACE INTO registry_src (source, name, timestamp) values (?1, ?2, ?3)",
                    )
                    .unwrap();
                stmt.execute(params![source_id, location, to_timestamp(timestamp)])
                    .unwrap();
            }
            LastUseKind::GitDb => {
                // TODO: unwrap
                let mut stmt = self
                    .connection
                    .prepare_cached("REPLACE INTO git_db (name, timestamp) values (?1, ?2)")
                    .unwrap();
                stmt.execute(params![location, to_timestamp(timestamp)])
                    .unwrap();
            }
            LastUseKind::GitCheckout(source) => {
                // TODO: unwrap
                let mut stmt = self
                    .connection
                    .prepare_cached(
                        "REPLACE INTO git_checkout (source, name, timestamp) values (?1, ?2, ?3)",
                    )
                    .unwrap();
                stmt.execute(params![source, location, to_timestamp(timestamp)])
                    .unwrap();
            }
        }
    }

    fn source_id(&mut self, source: &str) -> i64 {
        match self.registry_sources.get(source) {
            Some(i) => *i,
            None => {
                let id = self
                    .connection
                    .query_row(
                        "SELECT rowid FROM registry_index WHERE name = ?",
                        [source],
                        |row| row.get(0),
                    )
                    .unwrap();
                self.registry_sources.insert(source.to_string(), id);
                id
            }
        }
    }

    pub fn save(&self, _config: &Config) -> CargoResult<()> {
        self.connection.execute("COMMIT", [])?;
        self.connection.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }
}

fn to_timestamp(t: &time::SystemTime) -> Timestamp {
    t.duration_since(time::SystemTime::UNIX_EPOCH + time::Duration::from_secs(1420070400))
        .expect("invalid clock")
        .as_secs()
}
