//! Support for tracking the last time files were used to assist with cleaning
//! up those files if they haven't been used in a while.

use crate::core::gc::GcOpts;
use crate::ops::{CleanContext, CleaningFolderBar};
use crate::util::Filesystem;
use crate::{CargoResult, Config};
use tracing::{debug, trace};
use rusqlite::{params, Connection};
use std::collections::{hash_map, HashMap};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const LAST_USE_FILENAME: &str = ".last-use";

/// TODO
type Timestamp = u64;

/// Tracking for the global shared cache (registry files, etc.).
#[derive(Debug)]
pub struct GlobalLastUse {
    /// Connection to the SQLite database.
    connection: Connection,
    /// Cache of registry keys, used for faster fetching.
    ///
    /// The key is the registry name (which is its directory name) and the
    /// value is the `id` in the `registry_index` table.
    registry_keys: HashMap<String, i64>,
    /// Cache of git keys, used for faster fetching.
    ///
    /// The key is the git db name (which is its directory name) and the value
    /// is the `id` in the `git_db` table.
    git_keys: HashMap<String, i64>,

    registry_index_timestamps: HashMap<RegistryIndex, Timestamp>,
    registry_crate_timestamps: HashMap<RegistryCrate, Timestamp>,
    registry_src_timestamps: HashMap<RegistrySrc, Timestamp>,
    git_db_timestamps: HashMap<GitDb, Timestamp>,
    git_checkout_timestamps: HashMap<GitCheckout, Timestamp>,
    save_err_has_warned: bool,
    auto_gc_checked_this_session: bool,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistryIndex {
    pub encoded_registry_name: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistryCrate {
    pub encoded_registry_name: String,
    pub crate_filename: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistrySrc {
    pub encoded_registry_name: String,
    pub package_dir: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GitDb {
    pub encoded_git_name: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GitCheckout {
    pub encoded_git_name: String,
    pub short_name: String,
}

type Migration = Box<dyn Fn(&Connection) -> CargoResult<()>>;

fn basic_migration(stmt: &'static str) -> Migration {
    Box::new(|connection| {
        connection.execute(stmt, [])?;
        Ok(())
    })
}

fn migrations() -> Vec<Migration> {
    vec![
        // registry_index tracks the overall usage of an index cache, and tracks a
        // numeric ID to refer to that index that is used in other tables.
        basic_migration(
            "CREATE TABLE registry_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL
            )",
        ),
        basic_migration(
            "CREATE TABLE registry_crate (
                registry_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                PRIMARY KEY (registry_id, name)
             )",
        ),
        basic_migration(
            "CREATE TABLE registry_src (
                registry_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                PRIMARY KEY (registry_id, name)
             )",
        ),
        basic_migration(
            "CREATE TABLE git_db (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL
             )",
        ),
        basic_migration(
            "CREATE TABLE git_checkout (
                git_id INTEGER NOT NULL,
                name TEXT UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL,
                PRIMARY KEY (git_id, name)
             )",
        ),
        basic_migration(
            "CREATE TABLE global_data (
                last_auto_gc INTEGER NOT NULL
            )",
        ),
        Box::new(|connection| {
            connection.execute(
                "INSERT INTO global_data (last_auto_gc) VALUES (?1)",
                [now()],
            )?;
            Ok(())
        }),
    ]
}

impl GlobalLastUse {
    pub fn new(config: &Config) -> CargoResult<GlobalLastUse> {
        let connection = if config.cli_unstable().gc {
            let last_use_path = Self::db_path(config);
            let last_use_path = config.assert_package_cache_locked(&last_use_path);
            Connection::open(last_use_path)?
        } else {
            // To simplify things (so there aren't checks everywhere for being
            // enabled), just process everything in memory.
            Connection::open_in_memory()?
        };

        connection.execute("BEGIN TRANSACTION", [])?;
        let user_version =
            connection.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })?;
        let migrations = migrations();
        if user_version < migrations.len() {
            for migration in &migrations[user_version..] {
                migration(&connection)?;
            }
            connection.pragma_update(None, "user_version", &migrations.len())?;
        }
        connection.execute("COMMIT", [])?;

        Ok(GlobalLastUse {
            connection,
            registry_keys: HashMap::new(),
            git_keys: HashMap::new(),
            registry_index_timestamps: HashMap::new(),
            registry_crate_timestamps: HashMap::new(),
            registry_src_timestamps: HashMap::new(),
            git_db_timestamps: HashMap::new(),
            git_checkout_timestamps: HashMap::new(),
            save_err_has_warned: false,
            auto_gc_checked_this_session: false,
        })
    }

    pub fn db_path(config: &Config) -> Filesystem {
        config.home().join(LAST_USE_FILENAME)
    }

    pub fn mark_registry_index_used(&mut self, registry_index: RegistryIndex) {
        self.mark_registry_index_used_stamp(registry_index, None);
    }

    pub fn mark_registry_crate_used(&mut self, registry_crate: RegistryCrate) {
        self.mark_registry_crate_used_stamp(registry_crate, None);
    }

    pub fn mark_registry_src_used(&mut self, registry_src: RegistrySrc) {
        self.mark_registry_src_used_stamp(registry_src, None);
    }

    pub fn mark_git_checkout_used(&mut self, git_checkout: GitCheckout) {
        self.mark_git_checkout_used_stamp(git_checkout, None);
    }

    pub fn mark_registry_index_used_stamp(
        &mut self,
        registry_index: RegistryIndex,
        timestamp: Option<&SystemTime>,
    ) {
        let timestamp = timestamp.map_or_else(|| now(), |t| to_timestamp(t));
        self.registry_index_timestamps
            .insert(registry_index, timestamp);
    }

    pub fn mark_registry_crate_used_stamp(
        &mut self,
        registry_crate: RegistryCrate,
        timestamp: Option<&SystemTime>,
    ) {
        let timestamp = timestamp.map_or_else(|| now(), |t| to_timestamp(t));
        let index = RegistryIndex {
            encoded_registry_name: registry_crate.encoded_registry_name.clone(),
        };
        self.registry_index_timestamps.insert(index, timestamp);
        self.registry_crate_timestamps
            .insert(registry_crate, timestamp);
    }

    pub fn mark_registry_src_used_stamp(
        &mut self,
        registry_src: RegistrySrc,
        timestamp: Option<&SystemTime>,
    ) {
        let timestamp = timestamp.map_or_else(|| now(), |t| to_timestamp(t));
        let index = RegistryIndex {
            encoded_registry_name: registry_src.encoded_registry_name.clone(),
        };
        self.registry_index_timestamps.insert(index, timestamp);
        self.registry_src_timestamps.insert(registry_src, timestamp);
    }

    pub fn mark_git_checkout_used_stamp(
        &mut self,
        git_checkout: GitCheckout,
        timestamp: Option<&SystemTime>,
    ) {
        let timestamp = timestamp.map_or_else(|| now(), |t| to_timestamp(t));
        let db = GitDb {
            encoded_git_name: git_checkout.encoded_git_name.clone(),
        };
        self.git_db_timestamps.insert(db, timestamp);
        self.git_checkout_timestamps.insert(git_checkout, timestamp);
    }

    fn registry_id(&mut self, encoded_registry_name: &str) -> CargoResult<i64> {
        match self.registry_keys.get(encoded_registry_name) {
            Some(i) => Ok(*i),
            None => {
                let id = self.connection.query_row(
                    "SELECT registry_id FROM registry_index WHERE name = ?",
                    [encoded_registry_name],
                    |row| row.get(0),
                )?;
                self.registry_keys
                    .insert(encoded_registry_name.to_string(), id);
                Ok(id)
            }
        }
    }

    fn git_id(&mut self, encoded_git_name: &str) -> CargoResult<i64> {
        match self.git_keys.get(encoded_git_name) {
            Some(i) => Ok(*i),
            None => {
                let id = self.connection.query_row(
                    "SELECT git_id FROM git_db WHERE name = ?",
                    [encoded_git_name],
                    |row| row.get(0),
                )?;
                self.git_keys.insert(encoded_git_name.to_string(), id);
                Ok(id)
            }
        }
    }

    fn insert_registry_index(&mut self) -> CargoResult<()> {
        for (index, timestamp) in self.registry_index_timestamps.drain() {
            trace!("insert registry index {index:?} {timestamp}");
            let mut stmt = self.connection.prepare_cached(
                "INSERT INTO registry_index (name, timestamp)
                 VALUES (?1, ?2)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp
                 RETURNING id",
            )?;
            let id = stmt.query_row(params![index.encoded_registry_name, timestamp], |row| {
                row.get(0)
            })?;
            // TODO clone: InternedString, or use get instead?
            match self
                .registry_keys
                .entry(index.encoded_registry_name.clone())
            {
                hash_map::Entry::Occupied(o) => {
                    assert_eq!(*o.get(), id);
                }
                hash_map::Entry::Vacant(v) => {
                    v.insert(id);
                }
            }
        }
        Ok(())
    }

    fn insert_git_db(&mut self) -> CargoResult<()> {
        for (git_db, timestamp) in self.git_db_timestamps.drain() {
            trace!("insert git db used {git_db:?} {timestamp}");
            let mut stmt = self.connection.prepare_cached(
                "INSERT INTO git_db (name, timestamp)
                    VALUES (?1, ?2)
                    ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp
                    RETURNING id",
            )?;
            let id = stmt.query_row(params![git_db.encoded_git_name, timestamp], |row| {
                row.get(0)
            })?;
            // TODO: clone
            match self.git_keys.entry(git_db.encoded_git_name.clone()) {
                hash_map::Entry::Occupied(o) => assert_eq!(*o.get(), id),
                hash_map::Entry::Vacant(v) => {
                    v.insert(id);
                }
            }
        }
        Ok(())
    }

    fn insert_registry_crate(&mut self) -> CargoResult<()> {
        let mut registry_crate_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.registry_crate_timestamps,
            &mut registry_crate_timestamps,
        );
        for (registry_crate, timestamp) in registry_crate_timestamps {
            trace!("insert registry crate {registry_crate:?} {timestamp}");
            let registry_id = self.registry_id(&registry_crate.encoded_registry_name)?;
            let mut stmt = self.connection.prepare_cached(
                "INSERT INTO registry_crate (registry_id, name, timestamp)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            stmt.execute(params![
                registry_id,
                registry_crate.crate_filename,
                timestamp
            ])?;
        }
        Ok(())
    }

    fn insert_registry_src(&mut self) -> CargoResult<()> {
        let mut registry_src_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.registry_src_timestamps,
            &mut registry_src_timestamps,
        );
        for (registry_src, timestamp) in registry_src_timestamps {
            trace!("insert registry src {registry_src:?} {timestamp}");
            let registry_id = self.registry_id(&registry_src.encoded_registry_name)?;
            debug!(
                "inserting registry_src {:?} {:?}",
                registry_src.package_dir, timestamp
            );
            let mut stmt = self.connection.prepare_cached(
                "INSERT INTO registry_src (registry_id, name, timestamp)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            stmt.execute(params![registry_id, registry_src.package_dir, timestamp])?;
        }

        Ok(())
    }

    fn insert_git_checkout(&mut self) -> CargoResult<()> {
        let mut git_checkout_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.git_checkout_timestamps,
            &mut git_checkout_timestamps,
        );
        for (git_checkout, timestamp) in git_checkout_timestamps {
            trace!("insert git checkout used {git_checkout:?} {timestamp}");
            let git_id = self.git_id(&git_checkout.encoded_git_name)?;
            let mut stmt = self.connection.prepare_cached(
                "INSERT INTO git_checkout (git_id, name, timestamp)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            stmt.execute(params![git_id, git_checkout.short_name, timestamp])?;
        }

        Ok(())
    }

    pub fn save_no_error(&mut self, config: &Config) {
        if let Err(e) = self.save() {
            // TODO: Consider if this should be a hard error?
            if !self.save_err_has_warned {
                crate::display_warning_with_error(
                    "failed to save last-use data",
                    &e,
                    &mut config.shell(),
                );
                self.save_err_has_warned = true;
            }
        }
    }

    pub fn save(&mut self) -> CargoResult<()> {
        trace!("saving last-use data");
        if self.registry_index_timestamps.is_empty()
            && self.git_db_timestamps.is_empty()
            && self.registry_crate_timestamps.is_empty()
            && self.registry_src_timestamps.is_empty()
            && self.git_checkout_timestamps.is_empty()
        {
            return Ok(());
        }
        self.connection.execute("BEGIN TRANSACTION", [])?;
        // These must run before the ones that refer to their IDs.
        self.insert_registry_index()?;
        self.insert_git_db()?;
        self.insert_registry_crate()?;
        self.insert_registry_src()?;
        self.insert_git_checkout()?;

        self.connection.execute("COMMIT", [])?;
        trace!("last-use save complete");
        Ok(())
    }

    pub fn registry_index_all(&self) -> CargoResult<Vec<(RegistryIndex, Timestamp)>> {
        let mut stmt = self
            .connection
            .prepare_cached("SELECT name, timestamp FROM registry_index")?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_registry_name = row.get_unwrap(0);
                let timestamp = row.get_unwrap(1);
                let kind = RegistryIndex {
                    encoded_registry_name,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn registry_crate_all(&self) -> CargoResult<Vec<(RegistryCrate, Timestamp)>> {
        let mut stmt = self.connection.prepare_cached(
            "SELECT registry_index.name, registry_crate.name, registry_crate.timestamp
             FROM registry_index, registry_crate
             WHERE registry_crate.registry_id = registry_index.id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_registry_name = row.get_unwrap(0);
                let crate_filename = row.get_unwrap(1);
                let timestamp = row.get_unwrap(2);
                let kind = RegistryCrate {
                    encoded_registry_name,
                    crate_filename,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn registry_src_all(&self) -> CargoResult<Vec<(RegistrySrc, Timestamp)>> {
        let mut stmt = self.connection.prepare_cached(
            "SELECT registry_index.name, registry_src.name, registry_src.timestamp
             FROM registry_index, registry_src
             WHERE registry_src.registry_id = registry_index.id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_registry_name = row.get_unwrap(0);
                let package_dir = row.get_unwrap(1);
                let timestamp = row.get_unwrap(2);
                let kind = RegistrySrc {
                    encoded_registry_name,
                    package_dir,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn git_db_all(&self) -> CargoResult<Vec<(GitDb, Timestamp)>> {
        let mut stmt = self
            .connection
            .prepare_cached("SELECT name, timestamp FROM git_db")?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_git_name = row.get_unwrap(0);
                let timestamp = row.get_unwrap(1);
                let kind = GitDb { encoded_git_name };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn git_checkout_all(&self) -> CargoResult<Vec<(GitCheckout, Timestamp)>> {
        let mut stmt = self.connection.prepare_cached(
            "SELECT git_db.name, git_checkout.name, git_checkout.timestamp
             FROM git_db, git_checkout
             WHERE git_checkout.registry_id = git_db.id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_git_name = row.get_unwrap(0);
                let short_name = row.get_unwrap(1);
                let timestamp = row.get_unwrap(2);
                let kind = GitCheckout {
                    encoded_git_name,
                    short_name,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn should_run_auto_gc(&mut self, frequency: Duration) -> CargoResult<bool> {
        trace!("should_run_auto_gc");
        if self.auto_gc_checked_this_session {
            return Ok(false);
        }
        let last_auto_gc: Timestamp =
            self.connection
                .query_row("SELECT last_auto_gc FROM global_data", [], |row| row.get(0))?;
        let should_run = last_auto_gc + frequency.as_secs() < now();
        trace!(
            "last auto gc was {}, {}",
            last_auto_gc,
            if should_run { "running" } else { "skipping" }
        );
        self.auto_gc_checked_this_session = true;
        Ok(should_run)
    }

    pub fn set_last_auto_gc(&self) -> CargoResult<()> {
        self.connection
            .execute("UPDATE global_data SET last_auto_gc = ?1", [now()])?;
        Ok(())
    }

    pub fn clean(&self, clean_ctx: &mut CleanContext<'_>, gc_opts: &GcOpts) -> CargoResult<()> {
        let now = now();
        trace!("cleaning {gc_opts:?}");
        self.connection.execute("BEGIN TRANSACTION", [])?;
        let src_paths = gc_opts
            .max_src_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                self.get_registry_items_to_clean(max_age, "registry_src")
            })
            .transpose()?
            .unwrap_or_default();
        let crate_paths = gc_opts
            .max_crate_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                self.get_registry_items_to_clean(max_age, "registry_crate")
            })
            .transpose()?
            .unwrap_or_default();
        let index_paths = gc_opts
            .max_index_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                self.get_registry_index_to_clean(max_age)
            })
            .transpose()?
            .unwrap_or_default();
        let git_co_paths = gc_opts
            .max_git_co_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                self.get_git_co_items_to_clean(max_age)
            })
            .transpose()?
            .unwrap_or_default();
        let git_db_paths = gc_opts
            .max_git_db_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                self.get_git_db_items_to_clean(max_age)
            })
            .transpose()?
            .unwrap_or_default();

        let config = clean_ctx.config;
        let total = src_paths.len()
            + crate_paths.len()
            + index_paths.len()
            + git_co_paths.len()
            + git_db_paths.len();
        let progress = CleaningFolderBar::new(config, total);
        clean_ctx.set_progress(Box::new(progress));
        let base_path = config.registry_source_path().into_path_unlocked();
        // TODO: rm_rf context
        for path in src_paths {
            clean_ctx.rm_rf(&base_path.join(path))?;
        }
        let base_path = config.registry_cache_path().into_path_unlocked();
        for path in crate_paths {
            clean_ctx.rm_rf(&base_path.join(path))?;
        }
        let base_path = config.registry_index_path().into_path_unlocked();
        for path in index_paths {
            clean_ctx.rm_rf(&base_path.join(path))?;
        }
        let base_path = config.git_path().into_path_unlocked().join("checkouts");
        for path in git_co_paths {
            clean_ctx.rm_rf(&base_path.join(path))?;
        }
        let base_path = config.git_path().into_path_unlocked().join("db");
        for path in git_db_paths {
            clean_ctx.rm_rf(&base_path.join(path))?;
        }

        if clean_ctx.dry_run {
            self.connection.execute("ROLLBACK", [])?;
        } else {
            self.connection.execute("COMMIT", [])?;
        }
        Ok(())
    }

    fn get_registry_items_to_clean(
        &self,
        max_age: Timestamp,
        table_name: &str,
    ) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning {table_name} since {max_age:?}");
        let mut stmt = self.connection.prepare_cached(&format!(
            "DELETE FROM {table_name} WHERE timestamp < ?1
                RETURNING registry_id, name"
        ))?;
        let rows = stmt
            .query_map(params![max_age], |row| {
                let registry_id = row.get_unwrap(0);
                let name: String = row.get_unwrap(1);
                Ok((registry_id, name))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        debug!("rows={rows:?}");
        let ids = rows.iter().map(|row| row.0);
        let mut registry_name_stmt = self
            .connection
            .prepare_cached("SELECT name FROM registry_index WHERE id = ?1")?;
        let id_map = ids
            .map(|id| {
                let name = registry_name_stmt.query_row(params![id], |row| {
                    Ok(PathBuf::from(row.get::<_, String>(0)?))
                })?;
                Ok((id, name))
            })
            .collect::<CargoResult<HashMap<i64, PathBuf>>>()?;
        debug!("id_map={id_map:#?}");
        let paths = rows
            .iter()
            .map(|(id, name)| {
                let encoded_registry_name = &id_map[&id];
                encoded_registry_name.join(name)
            })
            .collect();
        Ok(paths)
    }

    fn get_registry_index_to_clean(&self, max_age: Timestamp) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning index since {max_age:?}");
        let mut stmt = self.connection.prepare_cached(&format!(
            "DELETE FROM registry_index WHERE timestamp < ?1
                RETURNING name"
        ))?;
        let paths = stmt
            .query_map(params![max_age], |row| {
                Ok(PathBuf::from(row.get_unwrap::<_, String>(0)))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(paths)
    }

    fn get_git_co_items_to_clean(&self, max_age: Timestamp) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning git co since {max_age:?}");
        let mut stmt = self.connection.prepare_cached(&format!(
            "DELETE FROM git_checkout WHERE timestamp < ?1
                RETURNING git_id, name"
        ))?;
        let rows = stmt
            .query_map(params![max_age], |row| {
                let git_id = row.get_unwrap(0);
                let name: String = row.get_unwrap(1);
                Ok((git_id, name))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        debug!("rows={rows:?}");
        let ids = rows.iter().map(|row| row.0);
        let mut git_name_stmt = self
            .connection
            .prepare_cached("SELECT name FROM git_db WHERE id = ?1")?;
        let id_map = ids
            .map(|id| {
                let name = git_name_stmt.query_row(params![id], |row| {
                    Ok(PathBuf::from(row.get::<_, String>(0)?))
                })?;
                Ok((id, name))
            })
            .collect::<CargoResult<HashMap<i64, PathBuf>>>()?;
        debug!("id_map={id_map:#?}");
        let paths = rows
            .iter()
            .map(|(id, name)| {
                let encoded_git_name = &id_map[&id];
                encoded_git_name.join(name)
            })
            .collect();
        Ok(paths)
    }

    fn get_git_db_items_to_clean(&self, max_age: Timestamp) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning git db since {max_age:?}");
        let mut stmt = self.connection.prepare_cached(&format!(
            "DELETE FROM git_db WHERE timestamp < ?1
                RETURNING name"
        ))?;
        let paths = stmt
            .query_map(params![max_age], |row| {
                Ok(PathBuf::from(row.get_unwrap::<_, String>(0)))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(paths)
    }
}

fn to_timestamp(t: &SystemTime) -> Timestamp {
    // This offsets from January 1, 2015 12:00:00 AM to generate smaller integers.
    t.duration_since(SystemTime::UNIX_EPOCH)
        .expect("invalid clock")
        .as_secs()
}

// fn from_timestamp(t: Timestamp) -> SystemTime {
//     SystemTime::UNIX_EPOCH + Duration::from_secs(t)
// }

fn now() -> Timestamp {
    match std::env::var("__CARGO_TEST_LAST_USE_NOW") {
        Ok(now) => now.parse().unwrap(),
        Err(_) => to_timestamp(&SystemTime::now()),
    }
}
