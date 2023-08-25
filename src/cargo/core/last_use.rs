//! Support for tracking the last time files were used to assist with cleaning
//! up those files if they haven't been used in a while.

use crate::core::gc::GcOpts;
use crate::core::Verbosity;
use crate::ops::{CleanContext, CleaningFolderBar};
use crate::util::cache_lock::CacheLockMode;
use crate::util::Filesystem;
use crate::{CargoResult, Config};
use anyhow::Context;
use rusqlite::{params, Connection, ErrorCode, TransactionBehavior};
use std::collections::{hash_map, HashMap};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, trace};

const LAST_USE_FILENAME: &str = ".last-use";

/// TODO
type Timestamp = u64;

/// Tracking for the global shared cache (registry files, etc.).
#[derive(Debug)]
pub struct GlobalLastUse {
    /// Connection to the SQLite database.
    conn: Connection,
    auto_gc_checked_this_session: bool,
}

/// This is a cache of modifications that will be saved to disk all at once
/// via the [`DeferredGlobalLastUse::save`] method.
///
/// This is here to improve performance.
#[derive(Debug)]
pub struct DeferredGlobalLastUse {
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
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistryIndex {
    pub encoded_registry_name: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistryCrate {
    pub encoded_registry_name: String,
    pub crate_filename: String,
    pub size: u64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RegistrySrc {
    pub encoded_registry_name: String,
    pub package_dir: String,
    pub size: Option<u64>,
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
    Box::new(|conn| {
        conn.execute(stmt, [])?;
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
                size INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                PRIMARY KEY (registry_id, name)
             )",
        ),
        basic_migration(
            "CREATE TABLE registry_src (
                registry_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                size INTEGER,
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
        Box::new(|conn| {
            conn.execute(
                "INSERT INTO global_data (last_auto_gc) VALUES (?1)",
                [now()],
            )?;
            Ok(())
        }),
    ]
}

impl GlobalLastUse {
    pub fn new(config: &Config) -> CargoResult<GlobalLastUse> {
        let mut conn = if config.cli_unstable().gc {
            let last_use_path = Self::db_path(config);
            // A package cache lock is required to ensure only one cargo is
            // accessing at the same time. If there is concurrent access, we
            // want to rely on cargo's own "Blocking" system (which can
            // provide user feedback) rather than blocking inside sqlite
            // (which by default has a short timeout).
            let last_use_path = config
                .assert_package_cache_locked(CacheLockMode::DownloadExclusive, &last_use_path);
            Connection::open(last_use_path)?
        } else {
            // To simplify things (so there aren't checks everywhere for being
            // enabled), just process everything in memory.
            Connection::open_in_memory()?
        };
        // EXCLUSIVE ensures that it starts with an exclusive write lock. No
        // other readers will be allowed. This generally shouldn't be needed
        // if there is a package cache lock, but might be helpful in cases
        // where cargo's `FileLock` failed.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
        let user_version =
            tx.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })?;
        let migrations = migrations();
        if user_version < migrations.len() {
            for migration in &migrations[user_version..] {
                migration(&tx)?;
            }
            tx.pragma_update(None, "user_version", &migrations.len())?;
        }
        tx.commit()?;
        Ok(GlobalLastUse {
            conn,
            auto_gc_checked_this_session: false,
        })
    }

    pub fn db_path(config: &Config) -> Filesystem {
        config.home().join(LAST_USE_FILENAME)
    }

    /// Given an encoded registry name, returns its ID.
    fn registry_id_from_name(conn: &Connection, encoded_registry_name: &str) -> CargoResult<i64> {
        let mut stmt = conn.prepare_cached("SELECT id FROM registry_index WHERE name = ?")?;
        let id = stmt.query_row([encoded_registry_name], |row| row.get(0))?;
        Ok(id)
    }

    /// Returns a map of ID to path for the given ids in the given table.
    ///
    /// For example, given `registry_index` IDs, it returns filenames of the
    /// form "index.crates.io-6f17d22bba15001f".
    fn get_id_map(
        conn: &Connection,
        table_name: &str,
        ids: &[i64],
    ) -> CargoResult<HashMap<i64, PathBuf>> {
        let mut stmt =
            conn.prepare_cached(&format!("SELECT name FROM {table_name} WHERE id = ?1"))?;
        ids.iter()
            .map(|id| {
                let name = stmt.query_row(params![id], |row| {
                    Ok(PathBuf::from(row.get::<_, String>(0)?))
                })?;
                Ok((*id, name))
            })
            .collect()
    }

    /// Returns all index cache timestamps.
    pub fn registry_index_all(&self) -> CargoResult<Vec<(RegistryIndex, Timestamp)>> {
        let mut stmt = self
            .conn
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

    /// Returns all registry crate cache timestamps.
    pub fn registry_crate_all(&self) -> CargoResult<Vec<(RegistryCrate, Timestamp)>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT registry_index.name, registry_crate.name, registry_crate.size, registry_crate.timestamp
             FROM registry_index, registry_crate
             WHERE registry_crate.registry_id = registry_index.id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_registry_name = row.get_unwrap(0);
                let crate_filename = row.get_unwrap(1);
                let size = row.get_unwrap(2);
                let timestamp = row.get_unwrap(3);
                let kind = RegistryCrate {
                    encoded_registry_name,
                    crate_filename,
                    size,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Returns all registry source cache timestamps.
    pub fn registry_src_all(&self) -> CargoResult<Vec<(RegistrySrc, Timestamp)>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT registry_index.name, registry_src.name, registry_src.size, registry_src.timestamp
             FROM registry_index, registry_src
             WHERE registry_src.registry_id = registry_index.id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let encoded_registry_name = row.get_unwrap(0);
                let package_dir = row.get_unwrap(1);
                let size = row.get_unwrap(2);
                let timestamp = row.get_unwrap(3);
                let kind = RegistrySrc {
                    encoded_registry_name,
                    package_dir,
                    size,
                };
                Ok((kind, timestamp))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Returns all git db timestamps.
    pub fn git_db_all(&self) -> CargoResult<Vec<(GitDb, Timestamp)>> {
        let mut stmt = self
            .conn
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

    /// Returns all git checkout timestamps.
    pub fn git_checkout_all(&self) -> CargoResult<Vec<(GitCheckout, Timestamp)>> {
        let mut stmt = self.conn.prepare_cached(
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

    /// Returns whether or not an auto GC should be performed, compared to the
    /// last time it was recorded in the database.
    pub fn should_run_auto_gc(&mut self, frequency: Duration) -> CargoResult<bool> {
        trace!("should_run_auto_gc");
        if self.auto_gc_checked_this_session {
            return Ok(false);
        }
        let last_auto_gc: Timestamp =
            self.conn
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

    /// Writes to the database to indicate that an automatic GC has just been
    /// completed.
    pub fn set_last_auto_gc(&self) -> CargoResult<()> {
        self.conn
            .execute("UPDATE global_data SET last_auto_gc = ?1", [now()])?;
        Ok(())
    }

    pub fn clean(&mut self, clean_ctx: &mut CleanContext<'_>, gc_opts: &GcOpts) -> CargoResult<()> {
        let config = clean_ctx.config;
        let now = now();
        trace!("cleaning {gc_opts:?}");
        let tx = self.conn.transaction()?;
        let src_paths = gc_opts
            .max_src_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                Self::get_registry_items_to_clean_age(&tx, max_age, "registry_src")
            })
            .transpose()?
            .unwrap_or_default();
        let crate_paths = gc_opts
            .max_crate_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                Self::get_registry_items_to_clean_age(&tx, max_age, "registry_crate")
            })
            .transpose()?
            .unwrap_or_default();
        let index_paths = gc_opts
            .max_index_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                Self::get_registry_index_to_clean(&tx, max_age)
            })
            .transpose()?
            .unwrap_or_default();
        let git_co_paths = gc_opts
            .max_git_co_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                Self::get_git_co_items_to_clean(&tx, max_age)
            })
            .transpose()?
            .unwrap_or_default();
        let git_db_paths = gc_opts
            .max_git_db_age
            .map(|max_age| {
                let max_age = now - max_age.as_secs();
                Self::get_git_db_items_to_clean(&tx, max_age)
            })
            .transpose()?
            .unwrap_or_default();
        // Size collection must happen after date collection so that dates
        // have precedence, since size constraints are a more blunt
        // instrument.
        let mut size_crate_paths = gc_opts
            .max_crate_size
            .map(|max_size| {
                Self::get_registry_items_to_clean_size(&tx, config, max_size, "registry_crate")
            })
            .transpose()?
            .unwrap_or_default();
        let mut size_src_paths = gc_opts
            .max_src_size
            .map(|max_size| {
                Self::get_registry_items_to_clean_size(&tx, config, max_size, "registry_src")
            })
            .transpose()?
            .unwrap_or_default();
        let (combined_src, combined_crate) = gc_opts
            .max_download_size
            .map(|max_size| Self::get_registry_items_to_clean_size_both(&tx, config, max_size))
            .transpose()?
            .unwrap_or_default();
        size_crate_paths.extend(combined_crate);
        size_src_paths.extend(combined_src);

        let total = src_paths.len()
            + crate_paths.len()
            + index_paths.len()
            + git_co_paths.len()
            + git_db_paths.len()
            + size_crate_paths.len()
            + size_src_paths.len();
        let progress = CleaningFolderBar::new(config, total);
        clean_ctx.set_progress(Box::new(progress));
        let base_src_path = config.registry_source_path().into_path_unlocked();
        // TODO: rm_rf context
        for path in src_paths.iter().chain(size_src_paths.iter()) {
            clean_ctx.rm_rf(&base_src_path.join(path))?;
        }
        let base_crate_path = config.registry_cache_path().into_path_unlocked();
        for path in crate_paths.iter().chain(size_crate_paths.iter()) {
            clean_ctx.rm_rf(&base_crate_path.join(path))?;
        }
        let base_index_path = config.registry_index_path().into_path_unlocked();
        for path in index_paths {
            clean_ctx.rm_rf(&base_index_path.join(&path))?;
            // Also delete .crate and src directories, since by definition
            // they cannot be used without their index.
            clean_ctx.rm_rf(&base_src_path.join(&path))?;
            clean_ctx.rm_rf(&base_crate_path.join(&path))?;
        }
        let base_git_co_path = config.git_path().into_path_unlocked().join("checkouts");
        for path in git_co_paths {
            clean_ctx.rm_rf(&base_git_co_path.join(path))?;
        }
        let base_git_db_path = config.git_path().into_path_unlocked().join("db");
        for path in git_db_paths {
            clean_ctx.rm_rf(&base_git_db_path.join(path))?;
        }

        if clean_ctx.dry_run {
            tx.rollback()?;
        } else {
            tx.commit()?;
        }
        Ok(())
    }

    fn get_registry_items_to_clean_age(
        conn: &Connection,
        max_age: Timestamp,
        table_name: &str,
    ) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning {table_name} since {max_age:?}");
        let mut stmt = conn.prepare_cached(&format!(
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
        let ids: Vec<_> = rows.iter().map(|r| r.0).collect();
        let id_map = Self::get_id_map(conn, "registry_index", &ids)?;
        let paths = rows
            .iter()
            .map(|(id, name)| {
                let encoded_registry_name = &id_map[&id];
                encoded_registry_name.join(name)
            })
            .collect();
        Ok(paths)
    }

    fn get_registry_items_to_clean_size(
        conn: &Connection,
        config: &Config,
        max_size: u64,
        table_name: &str,
    ) -> CargoResult<Vec<PathBuf>> {
        match table_name {
            "registry_crate" => Self::populate_untracked_crate(conn, config)?,
            "registry_src" => Self::populate_untracked_src(conn, config)?,
            _ => panic!("unexpected table {table_name}"),
        }
        debug!("cleaning {table_name} till under {max_size:?}");
        let total_size: u64 =
            conn.query_row(&format!("SELECT SUM(size) FROM {table_name}"), [], |row| {
                row.get(0)
            })?;
        if total_size <= max_size {
            return Ok(Vec::new());
        }
        // TODO: Explain this sql statement.
        //
        // The ORDER BY includes `name` mainly for test purposes so that
        // entries with the same timestamp have deterministic behavior.
        let mut stmt = conn.prepare(&format!(
            "DELETE FROM {table_name} WHERE rowid IN \
                (SELECT x.rowid FROM \
                    (SELECT rowid, size, sum(size) OVER \
                        (ORDER BY timestamp, name ROWS UNBOUNDED PRECEDING) AS running_amount \
                        FROM {table_name}) x \
                    WHERE coalesce(x.running_amount, 0) - x.size < ?1) \
                RETURNING registry_id, name;"
        ))?;
        let rows = stmt
            .query_map(params![total_size - max_size], |row| {
                let id = row.get_unwrap(0);
                let name: String = row.get_unwrap(1);
                Ok((id, name))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // Convert registry_id to the encoded registry name, and join those.
        let ids: Vec<_> = rows.iter().map(|r| r.0).collect();
        let id_map = Self::get_id_map(conn, "registry_index", &ids)?;
        let paths = rows
            .iter()
            .map(|(id, name)| {
                let encoded_name = &id_map[&id];
                encoded_name.join(name)
            })
            .collect();
        Ok(paths)
    }

    fn get_registry_items_to_clean_size_both(
        conn: &Connection,
        config: &Config,
        max_size: u64,
    ) -> CargoResult<(Vec<PathBuf>, Vec<PathBuf>)> {
        Self::populate_untracked_crate(conn, config)?;
        Self::populate_untracked_src(conn, config)?;
        debug!("cleaning download till under {max_size:?}");

        // TODO: Describe this query. The 1/2 thing, and why it is a single query.
        let mut stmt = conn.prepare_cached(
            "SELECT 1, registry_src.rowid, registry_src.name AS name, registry_index.name,
                    registry_src.size, registry_src.timestamp AS timestamp
             FROM registry_src, registry_index
             WHERE registry_src.registry_id = registry_index.id AND registry_src.size NOT NULL

             UNION

             SELECT 2, registry_crate.rowid, registry_crate.name AS name, registry_index.name,
                    registry_crate.size, registry_crate.timestamp AS timestamp
             FROM registry_crate, registry_index
             WHERE registry_crate.registry_id = registry_index.id

             ORDER BY timestamp, name",
        )?;
        let mut delete_src_stmt =
            conn.prepare_cached("DELETE FROM registry_src WHERE rowid = ?1")?;
        let mut delete_crate_stmt =
            conn.prepare_cached("DELETE FROM registry_crate WHERE rowid = ?1")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get_unwrap(0),
                    row.get_unwrap(1),
                    row.get_unwrap(2),
                    row.get_unwrap(3),
                    row.get_unwrap(4),
                ))
            })?
            .collect::<Result<Vec<(i64, i64, String, String, u64)>, _>>()?;
        let mut total_size: u64 = rows.iter().map(|r| r.4).sum();
        let mut src_result = Vec::new();
        let mut crate_result = Vec::new();
        for (table, rowid, name, index_name, size) in rows {
            if total_size <= max_size {
                break;
            }
            if table == 1 {
                src_result.push(Path::new(&index_name).join(name));
                delete_src_stmt.execute([rowid])?;
            } else {
                crate_result.push(Path::new(&index_name).join(name));
                delete_crate_stmt.execute([rowid])?;
            }
            total_size -= size;
        }
        Ok((src_result, crate_result))
    }

    fn populate_untracked_registry_index_in_path(
        conn: &Connection,
        names: &[String],
    ) -> CargoResult<()> {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO registry_index (name, timestamp)
                VALUES (?1, ?2)
                ON CONFLICT DO NOTHING",
        )?;
        let now = now();
        for name in names {
            stmt.execute(params![name, now])?;
        }
        Ok(())
    }

    /// Returns a list of directory entries in the given path.
    fn names_from(path: &Path) -> CargoResult<Vec<String>> {
        let names = path
            .read_dir()
            .with_context(|| format!("failed to read path `{path:?}`"))?
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect();
        Ok(names)
    }

    /// Updates the database to track any `.crate` files that are currently
    /// not tracked (such as when they are downloaded by an older version of
    /// cargo).
    fn populate_untracked_crate(conn: &Connection, config: &Config) -> CargoResult<()> {
        debug!("populating untracked crate files");
        let base_path = config.registry_cache_path().into_path_unlocked();
        let index_names = Self::names_from(&base_path)?;
        Self::populate_untracked_registry_index_in_path(conn, &index_names)?;

        let mut insert_stmt = conn.prepare_cached(
            "INSERT INTO registry_crate (registry_id, name, size, timestamp)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT DO NOTHING",
        )?;
        let now = now();
        for index_name in index_names {
            let id = Self::registry_id_from_name(conn, &index_name)?;
            let index_path = base_path.join(index_name);
            for crate_name in Self::names_from(&index_path)? {
                if crate_name.ends_with(".crate") {
                    // TODO: context;
                    let size = index_path.join(&crate_name).metadata()?.len();
                    insert_stmt.execute(params![id, crate_name, size, now])?;
                }
            }
        }
        Ok(())
    }

    /// Updates the database to track any `src` directories that are currently
    /// not tracked (such as when they are downloaded by an older version of
    /// cargo).
    fn populate_untracked_src(conn: &Connection, config: &Config) -> CargoResult<()> {
        debug!("populating untracked src files");
        let base_path = config.registry_source_path().into_path_unlocked();
        let index_names = Self::names_from(&base_path)?;
        Self::populate_untracked_registry_index_in_path(conn, &index_names)?;

        // TODO: Is this select necessary?
        let mut select_stmt = conn.prepare_cached(
            "SELECT 1 FROM registry_src
             WHERE registry_id=?1 AND name=?2",
        )?;
        let mut insert_stmt = conn.prepare_cached(
            "INSERT INTO registry_src (registry_id, name, size, timestamp)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT DO NOTHING",
        )?;
        let now = now();
        for index_name in index_names {
            let id = Self::registry_id_from_name(conn, &index_name)?;
            let index_path = base_path.join(index_name);
            for src_name in Self::names_from(&index_path)? {
                if select_stmt.exists(params![id, src_name])? {
                    continue;
                }
                let src_path = index_path.join(&src_name);
                let meta = src_path.metadata()?; // TODO context
                if !meta.is_dir() {
                    continue;
                }
                let size = cargo_util::paths::du(&src_path)?;
                insert_stmt.execute(params![id, src_name, size, now])?;
            }
        }

        // Update NULL size entries.
        let mut null_stmt = conn.prepare_cached(
            "SELECT registry_src.rowid, registry_src.name, registry_index.name
             FROM registry_src, registry_index
             WHERE registry_src.size IS NULL AND registry_src.registry_id = registry_index.id",
        )?;
        let mut update_stmt =
            conn.prepare_cached("UPDATE registry_src SET size=?1 WHERE rowid=?2")?;
        let rows = null_stmt.query_map([], |row| {
            Ok((row.get_unwrap(0), row.get_unwrap(1), row.get_unwrap(2)))
        })?;
        for row in rows {
            let (rowid, src_name, index_name): (i64, String, String) = row?;
            let path = base_path.join(index_name).join(src_name);
            if !path.exists() {
                // TODO: Should this delete the entry?
                tracing::info!("`{path:?}` is missing");
                continue;
            }
            let size = cargo_util::paths::du(&path)?;
            update_stmt.execute(params![size, rowid])?;
        }

        Ok(())
    }

    fn get_registry_index_to_clean(
        conn: &Connection,
        max_age: Timestamp,
    ) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning index since {max_age:?}");
        let mut stmt = conn.prepare_cached(&format!(
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

    fn get_git_co_items_to_clean(
        conn: &Connection,
        max_age: Timestamp,
    ) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning git co since {max_age:?}");
        let mut stmt = conn.prepare_cached(&format!(
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
        let ids: Vec<_> = rows.iter().map(|r| r.0).collect();
        let id_map = Self::get_id_map(conn, "git_db", &ids)?;
        let paths = rows
            .iter()
            .map(|(id, name)| {
                let encoded_git_name = &id_map[&id];
                encoded_git_name.join(name)
            })
            .collect();
        Ok(paths)
    }

    fn get_git_db_items_to_clean(
        conn: &Connection,
        max_age: Timestamp,
    ) -> CargoResult<Vec<PathBuf>> {
        debug!("cleaning git db since {max_age:?}");
        let mut stmt = conn.prepare_cached(&format!(
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

impl DeferredGlobalLastUse {
    pub fn new() -> DeferredGlobalLastUse {
        DeferredGlobalLastUse {
            registry_keys: HashMap::new(),
            git_keys: HashMap::new(),
            registry_index_timestamps: HashMap::new(),
            registry_crate_timestamps: HashMap::new(),
            registry_src_timestamps: HashMap::new(),
            git_db_timestamps: HashMap::new(),
            git_checkout_timestamps: HashMap::new(),
            save_err_has_warned: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.registry_index_timestamps.is_empty()
            && self.registry_crate_timestamps.is_empty()
            && self.registry_src_timestamps.is_empty()
            && self.git_db_timestamps.is_empty()
            && self.git_checkout_timestamps.is_empty()
    }

    fn clear(&mut self) {
        self.registry_index_timestamps.clear();
        self.registry_crate_timestamps.clear();
        self.registry_src_timestamps.clear();
        self.git_db_timestamps.clear();
        self.git_checkout_timestamps.clear();
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

    /// Saves all of the deferred information to the database.
    ///
    /// This will also clear the state of self.
    pub fn save(&mut self, last_use: &mut GlobalLastUse) -> CargoResult<()> {
        trace!("saving last-use data");
        if self.is_empty() {
            return Ok(());
        }
        let tx = last_use.conn.transaction()?;
        // These must run before the ones that refer to their IDs.
        self.insert_registry_index_from_cache(&tx)?;
        self.insert_git_db_from_cache(&tx)?;
        self.insert_registry_crate_from_cache(&tx)?;
        self.insert_registry_src_from_cache(&tx)?;
        self.insert_git_checkout_from_cache(&tx)?;
        tx.commit()?;
        trace!("last-use save complete");
        Ok(())
    }

    /// Variant of [`DeferredGlobalLastUse::save`] that does not return an
    /// error.
    ///
    /// This will log or display a warning to the user.
    pub fn save_no_error(&mut self, config: &Config) {
        if let Err(e) = self.save_with_config(config) {
            // Because there is an assertion in auto-gc that checks if this is
            // empty, be sure to clear it so that assertion doesn't fail.
            self.clear();
            if !self.save_err_has_warned {
                if is_silent_error(&e) && config.shell().verbosity() != Verbosity::Verbose {
                    tracing::warn!("failed to save last-use data: {e:?}");
                } else {
                    crate::display_warning_with_error(
                        "failed to save last-use data\n\
                        This may prevent cargo from accurately tracking what is being \
                        used in its global cache. This information is used for \
                        automatically removing unused data in the cache.",
                        &e,
                        &mut config.shell(),
                    );
                    self.save_err_has_warned = true;
                }
            }
        }
    }

    fn save_with_config(&mut self, config: &Config) -> CargoResult<()> {
        let mut last_use = config.global_last_use()?;
        self.save(&mut last_use)
    }

    fn insert_registry_index_from_cache(&mut self, conn: &Connection) -> CargoResult<()> {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO registry_index (name, timestamp)
                VALUES (?1, ?2)
                ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp
                RETURNING id",
        )?;
        for (index, timestamp) in self.registry_index_timestamps.drain() {
            trace!("insert registry index {index:?} {timestamp}");
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

    fn insert_git_db_from_cache(&mut self, conn: &Connection) -> CargoResult<()> {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO git_db (name, timestamp)
                VALUES (?1, ?2)
                ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp
                RETURNING id",
        )?;
        for (git_db, timestamp) in self.git_db_timestamps.drain() {
            trace!("insert git db used {git_db:?} {timestamp}");
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

    fn insert_registry_crate_from_cache(&mut self, conn: &Connection) -> CargoResult<()> {
        let mut registry_crate_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.registry_crate_timestamps,
            &mut registry_crate_timestamps,
        );
        for (registry_crate, timestamp) in registry_crate_timestamps {
            trace!("insert registry crate {registry_crate:?} {timestamp}");
            let registry_id = self.registry_id(conn, &registry_crate.encoded_registry_name)?;
            let mut stmt = conn.prepare_cached(
                "INSERT INTO registry_crate (registry_id, name, size, timestamp)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            stmt.execute(params![
                registry_id,
                registry_crate.crate_filename,
                registry_crate.size,
                timestamp
            ])?;
        }
        Ok(())
    }

    fn insert_registry_src_from_cache(&mut self, conn: &Connection) -> CargoResult<()> {
        let mut registry_src_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.registry_src_timestamps,
            &mut registry_src_timestamps,
        );
        for (registry_src, timestamp) in registry_src_timestamps {
            trace!("insert registry src {registry_src:?} {timestamp}");
            let registry_id = self.registry_id(conn, &registry_src.encoded_registry_name)?;
            let mut stmt = conn.prepare_cached(
                "INSERT INTO registry_src (registry_id, name, size, timestamp)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            debug!(
                "inserting registry_src {:?} {:?}",
                registry_src.package_dir, timestamp
            );
            stmt.execute(params![
                registry_id,
                registry_src.package_dir,
                registry_src.size,
                timestamp
            ])?;
        }

        Ok(())
    }

    fn insert_git_checkout_from_cache(&mut self, conn: &Connection) -> CargoResult<()> {
        let mut git_checkout_timestamps = HashMap::new();
        std::mem::swap(
            &mut self.git_checkout_timestamps,
            &mut git_checkout_timestamps,
        );
        for (git_checkout, timestamp) in git_checkout_timestamps {
            trace!("insert git checkout used {git_checkout:?} {timestamp}");
            let git_id = self.git_id(conn, &git_checkout.encoded_git_name)?;
            let mut stmt = conn.prepare_cached(
                "INSERT INTO git_checkout (git_id, name, timestamp)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT DO UPDATE SET timestamp=excluded.timestamp",
            )?;
            stmt.execute(params![git_id, git_checkout.short_name, timestamp])?;
        }

        Ok(())
    }

    fn registry_id(&mut self, conn: &Connection, encoded_registry_name: &str) -> CargoResult<i64> {
        match self.registry_keys.get(encoded_registry_name) {
            Some(i) => Ok(*i),
            None => {
                let id = GlobalLastUse::registry_id_from_name(conn, encoded_registry_name)?;
                self.registry_keys
                    .insert(encoded_registry_name.to_string(), id);
                Ok(id)
            }
        }
    }

    fn git_id(&mut self, conn: &Connection, encoded_git_name: &str) -> CargoResult<i64> {
        match self.git_keys.get(encoded_git_name) {
            Some(i) => Ok(*i),
            None => {
                let id = conn.query_row(
                    "SELECT git_id FROM git_db WHERE name = ?",
                    [encoded_git_name],
                    |row| row.get(0),
                )?;
                self.git_keys.insert(encoded_git_name.to_string(), id);
                Ok(id)
            }
        }
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

/// Returns whether or not the given error should cause a warning to be
/// displayed to the user.
///
/// In some situations, like a read-only global cache, we don't want to spam
/// the user with a warning. I think once cargo has controllable lints, I
/// think we should consider changing this to always warn, but give the user
/// an option to silence the warning.
pub fn is_silent_error(e: &anyhow::Error) -> bool {
    if let Some(e) = e.downcast_ref::<rusqlite::Error>() {
        if matches!(
            e.sqlite_error_code(),
            Some(ErrorCode::CannotOpen | ErrorCode::ReadOnly)
        ) {
            return true;
        }
    }
    false
}
