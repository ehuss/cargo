//! Tests for last-use tracking and auto-gc.

use super::config::ConfigBuilder;
use cargo::core::last_use::{self, DeferredGlobalLastUse, GlobalLastUse};
use cargo::Config;
use cargo_test_support::paths::{self, CargoPathExt};
use cargo_test_support::registry::{Package, RegistryBuilder};
use cargo_test_support::{basic_manifest, cargo_process, git, project, Project};
use std::fmt::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Helper to get the names of files in a directory as strings.
fn get_names(glob: &str) -> Vec<String> {
    let mut names: Vec<_> = glob::glob(paths::home().join(glob).to_str().unwrap())
        .unwrap()
        .map(|p| p.unwrap().file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

fn get_registry_names(which: &str) -> Vec<String> {
    get_names(&format!(".cargo/registry/{which}/*/*"))
}

fn get_index_names() -> Vec<String> {
    get_names(&format!(".cargo/registry/index/*"))
}

fn get_git_db_names() -> Vec<String> {
    get_names(&format!(".cargo/git/db/*"))
}

fn get_git_checkout_names(db_name: &str) -> Vec<String> {
    get_names(&format!(".cargo/git/checkouts/{db_name}/*"))
}

fn days_ago(n: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(60 * 60 * 24 * n)
}

fn days_ago_unix(n: u64) -> String {
    // TODO: Export functions for working with timestamps.
    days_ago(n)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn months_ago_unix(n: u64) -> String {
    days_ago_unix(n * 30)
}

/// Populates last-use database and the cache files.
fn populate_cache(config: &Config, test_crates: &[(&str, u64, u64, u64)]) -> (PathBuf, PathBuf) {
    let cache_dir = paths::home().join(".cargo/registry/cache/github.com-1ecc6299db9ec823");
    let src_dir = paths::home().join(".cargo/registry/src/github.com-1ecc6299db9ec823");

    GlobalLastUse::db_path(&config).into_path_unlocked().rm_rf();

    let _lock = config.acquire_package_cache_lock().unwrap();
    let mut last_use = DeferredGlobalLastUse::new(&config).unwrap();

    cache_dir.rm_rf();
    cache_dir.mkdir_p();
    src_dir.rm_rf();
    src_dir.mkdir_p();
    let mut create = |name: &str, age, crate_size: u64, src_size: u64| {
        let crate_filename = format!("{name}.crate");
        last_use.mark_registry_crate_used_stamp(
            last_use::RegistryCrate {
                encoded_registry_name: "github.com-1ecc6299db9ec823".to_string(),
                crate_filename: crate_filename.clone(),
                size: crate_size,
            },
            Some(&days_ago(age)),
        );
        last_use.mark_registry_src_used_stamp(
            last_use::RegistrySrc {
                encoded_registry_name: "github.com-1ecc6299db9ec823".to_string(),
                package_dir: name.to_string(),
                size: Some(src_size),
            },
            Some(&days_ago(age)),
        );
        std::fs::write(
            cache_dir.join(crate_filename),
            "x".repeat(crate_size as usize),
        )
        .unwrap();
        let path = src_dir.join(name);
        path.mkdir_p();
        std::fs::write(path.join("data"), "x".repeat(src_size as usize)).unwrap()
    };

    for (name, age, crate_size, src_size) in test_crates {
        create(name, *age, *crate_size, *src_size);
    }
    last_use.save().unwrap();

    (cache_dir, src_dir)
}

#[cargo_test]
fn gated() {
    // Requires -Zgc to both track last-use data and to run auto-gc.
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("check")
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    let config = ConfigBuilder::new().build();
    assert!(!GlobalLastUse::db_path(&config)
        .into_path_unlocked()
        .exists());
    assert_eq!(get_index_names().len(), 1);

    // Again in the future, shouldn't auto-gc.
    p.cargo("check").run();
    assert!(!GlobalLastUse::db_path(&config)
        .into_path_unlocked()
        .exists());
    assert_eq!(get_index_names().len(), 1);
}

#[cargo_test]
fn implies_source() {
    // Checks that when a src, crate, or checkout is marked as used, the
    // corresponding index or git db also gets marked as used.
    let config = ConfigBuilder::new().unstable_flag("gc").build();
    let _lock = config.acquire_package_cache_lock().unwrap();
    let mut last_use = DeferredGlobalLastUse::new(&config).unwrap();

    last_use.mark_registry_crate_used(last_use::RegistryCrate {
        encoded_registry_name: "github.com-1ecc6299db9ec823".to_string(),
        crate_filename: "regex-1.8.4.crate".to_string(),
        size: 123,
    });
    last_use.mark_registry_src_used(last_use::RegistrySrc {
        encoded_registry_name: "index.crates.io-6f17d22bba15001f".to_string(),
        package_dir: "rand-0.8.5".to_string(),
        size: None,
    });
    last_use.mark_git_checkout_used(last_use::GitCheckout {
        encoded_git_name: "cargo-e7ff1db891893a9e".to_string(),
        short_name: "f0a4ee0".to_string(),
    });
    last_use.save().unwrap();

    let mut indexes = last_use.last_use().registry_index_all().unwrap();
    assert_eq!(indexes.len(), 2);
    indexes.sort_by(|a, b| a.0.encoded_registry_name.cmp(&b.0.encoded_registry_name));
    assert_eq!(
        indexes[0].0.encoded_registry_name,
        "github.com-1ecc6299db9ec823"
    );
    assert_eq!(
        indexes[1].0.encoded_registry_name,
        "index.crates.io-6f17d22bba15001f"
    );

    let dbs = last_use.last_use().git_db_all().unwrap();
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].0.encoded_git_name, "cargo-e7ff1db891893a9e");
}

#[cargo_test]
fn auto_gc_defaults() {
    // Checks that the auto-gc deletes old entries, and leaves new ones intact.
    Package::new("old", "1.0.0").publish();
    Package::new("new", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                old = "1.0"
                new = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    // Populate the last-use data.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0", "old-1.0.0"]);
    assert_eq!(
        get_registry_names("cache"),
        ["new-1.0.0.crate", "old-1.0.0.crate"]
    );

    // Run again with just one package. Make sure the old src gets deleted,
    // but .crate does not.
    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            new = "1.0"
        "#,
    );
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(2))
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0"]);
    assert_eq!(
        get_registry_names("cache"),
        ["new-1.0.0.crate", "old-1.0.0.crate"]
    );

    // Run again after the .crate should have aged out.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["new-1.0.0.crate"]);
}

#[cargo_test]
fn auto_gc_config() {
    // Can configure auto gc settings.
    Package::new("old", "1.0.0").publish();
    Package::new("new", "1.0.0").publish();
    let p = project()
        .file(
            ".cargo/config.toml",
            r#"
                [gc.auto]
                frequency = "always"
                max-src-age = "1 day"
                max-crate-age = "3 days"
                max-index-age = "3 days"
                max-git-co-age = "1 day"
                max-git-db-age = "3 days"
            "#,
        )
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                old = "1.0"
                new = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    // Populate the last-use data.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", days_ago_unix(4))
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0", "old-1.0.0"]);
    assert_eq!(
        get_registry_names("cache"),
        ["new-1.0.0.crate", "old-1.0.0.crate"]
    );

    // Run again with just one package. Make sure the old src gets deleted,
    // but .crate does not.
    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            new = "1.0"
        "#,
    );
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", days_ago_unix(2))
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0"]);
    assert_eq!(
        get_registry_names("cache"),
        ["new-1.0.0.crate", "old-1.0.0.crate"]
    );

    // Run again after the .crate should have aged out.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_registry_names("src"), ["new-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["new-1.0.0.crate"]);
}

#[cargo_test]
fn frequency() {
    // gc.auto.frequency settings
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            ".cargo/config.toml",
            r#"
                [gc.auto]
                frequency = "never"
            "#,
        )
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    // Populate data in the past.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    assert_eq!(get_index_names().len(), 1);
    assert_eq!(get_registry_names("src"), ["bar-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["bar-1.0.0.crate"]);

    p.change_file("Cargo.toml", &basic_manifest("foo", "0.2.0"));

    // Try after the default expiration time, with "never" it shouldn't gc.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_index_names().len(), 1);
    assert_eq!(get_registry_names("src"), ["bar-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["bar-1.0.0.crate"]);

    // Try again with a setting that allows it to run.
    p.cargo("check -Zgc")
        .env("CARGO_GC_AUTO_FREQUENCY", "1 day")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_index_names().len(), 0);
    assert_eq!(get_registry_names("src").len(), 0);
    assert_eq!(get_registry_names("cache").len(), 0);
}

#[cargo_test]
fn auto_gc_index() {
    // Deletes the index if it hasn't been used in a while.
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    assert_eq!(get_index_names().len(), 1);

    // Make sure it stays within the time frame.
    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.1.0"
        "#,
    );
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(2))
        .run();
    assert_eq!(get_index_names().len(), 1);

    // After it expires, it should be deleted.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_index_names().len(), 0);
}

#[cargo_test]
fn auto_gc_git() {
    // Deletes git checkouts and dbs.
    let short_id = |repo: &git2::Repository| -> String {
        let head = repo.revparse_single("HEAD").unwrap();
        let short_id = head.short_id().unwrap();
        short_id.as_str().unwrap().to_owned()
    };

    let (git_project, git_repo) = git::new_repo("bar", |p| {
        p.file("Cargo.toml", &basic_manifest("bar", "1.0.0"))
            .file("src/lib.rs", "")
    });
    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = {{ git = '{}' }}
            "#,
                git_project.url()
            ),
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(6))
        .run();
    let db_names = get_git_db_names();
    assert_eq!(db_names.len(), 1);
    let first_short_oid = short_id(&git_repo);
    assert_eq!(
        get_git_checkout_names(&db_names[0]),
        [first_short_oid.clone()]
    );

    // Use a new git checkout, should keep both.
    git_project.change_file("src/lib.rs", "// modified");
    git::add(&git_repo);
    git::commit(&git_repo);
    p.cargo("update -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(6))
        .run();
    assert_eq!(get_git_db_names().len(), 1);
    let second_short_oid = short_id(&git_repo);
    let mut both = vec![first_short_oid, second_short_oid.clone()];
    both.sort();
    assert_eq!(get_git_checkout_names(&db_names[0]), both);

    // In the future, using the second checkout should delete the first.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    assert_eq!(get_git_db_names().len(), 1);
    assert_eq!(
        get_git_checkout_names(&db_names[0]),
        [second_short_oid.clone()]
    );

    // After three months, the db should get deleted.
    p.change_file("Cargo.toml", &basic_manifest("foo", "0.2.0"));
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_git_db_names().len(), 0);
    assert_eq!(get_git_checkout_names(&db_names[0]).len(), 0);
}

#[cargo_test]
fn auto_gc_various_commands() {
    // Checks that auto gc works with a variety of commands.
    //
    // Auto-gc is only run on a subset of commands. Generally it is run on
    // commands that are already doing a lot of work, or heavily involve the
    // use of the registry.
    Package::new("bar", "1.0.0").publish();
    let cmds = ["check", "fetch"];
    for cmd in cmds {
        eprintln!("checking command {cmd}");
        let p = project()
            .file(
                "Cargo.toml",
                r#"
                    [package]
                    name = "foo"
                    version = "0.1.0"

                    [dependencies]
                    bar = "1.0"
                "#,
            )
            .file("src/lib.rs", "")
            .build();
        // Populate the last-use data.
        p.cargo(cmd)
            .arg("-Zgc")
            .masquerade_as_nightly_cargo(&["gc"])
            .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
            .run();
        let config = ConfigBuilder::new().unstable_flag("gc").build();
        let lock = config.acquire_package_cache_lock().unwrap();
        let last_use = GlobalLastUse::new(&config).unwrap();
        let indexes = last_use.registry_index_all().unwrap();
        assert_eq!(indexes.len(), 1);
        let crates = last_use.registry_crate_all().unwrap();
        assert_eq!(crates.len(), 1);
        let srcs = last_use.registry_src_all().unwrap();
        assert_eq!(srcs.len(), 1);
        drop(lock);

        // After everything is aged out, it should all be deleted.
        p.change_file("Cargo.toml", &basic_manifest("foo", "0.2.0"));
        p.cargo(cmd)
            .arg("-Zgc")
            .masquerade_as_nightly_cargo(&["gc"])
            .run();
        let lock = config.acquire_package_cache_lock().unwrap();
        let indexes = last_use.registry_index_all().unwrap();
        assert_eq!(indexes.len(), 0);
        let crates = last_use.registry_crate_all().unwrap();
        assert_eq!(crates.len(), 0);
        let srcs = last_use.registry_src_all().unwrap();
        assert_eq!(srcs.len(), 0);
        drop(lock);
        paths::home().join(".cargo/registry").rm_rf();
        GlobalLastUse::db_path(&config).into_path_unlocked().rm_rf();
    }
}

#[cargo_test]
fn updates_last_use_various_commands() {
    // Checks that last-use tracking is updated by various commands.
    //
    // Not *all* commands update the index tracking, even though they
    // technically involve reading the index. There isn't a convenient place
    // to ensure it gets saved while avoiding saving too often in other
    // commands. For the most part, this should be fine, since these commands
    // usually aren't run without running one of the commands that does save
    // the tracking. Some of the commands are:
    //
    // - login, owner, yank, search
    // - report future-incompatibilities
    // - package --no-verify
    // - fetch --locked
    Package::new("bar", "1.0.0").publish();
    let cmds = [
        // name, expected_crates (0=doesn't download)
        ("check", 1),
        ("fetch", 1),
        ("tree", 1),
        ("generate-lockfile", 0),
        ("update", 0),
        ("metadata", 1),
        ("vendor --respect-source-config", 1),
    ];
    for (cmd, expected_crates) in cmds {
        eprintln!("checking command {cmd}");
        let p = project()
            .file(
                "Cargo.toml",
                r#"
                    [package]
                    name = "foo"
                    version = "0.1.0"

                    [dependencies]
                    bar = "1.0"
                "#,
            )
            .file("src/lib.rs", "")
            .build();
        // Populate the last-use data.
        p.cargo(cmd)
            .arg("-Zgc")
            .masquerade_as_nightly_cargo(&["gc"])
            .run();
        let config = ConfigBuilder::new().unstable_flag("gc").build();
        let lock = config.acquire_package_cache_lock().unwrap();
        let last_use = GlobalLastUse::new(&config).unwrap();
        let indexes = last_use.registry_index_all().unwrap();
        assert_eq!(indexes.len(), 1);
        let crates = last_use.registry_crate_all().unwrap();
        assert_eq!(crates.len(), expected_crates);
        let srcs = last_use.registry_src_all().unwrap();
        assert_eq!(srcs.len(), expected_crates);
        drop(lock);
        paths::home().join(".cargo/registry").rm_rf();
        GlobalLastUse::db_path(&config).into_path_unlocked().rm_rf();
    }
}

#[cargo_test]
fn both_git_and_http_index_cleans() {
    // Checks that either the git or http index cache gets cleaned.
    let _crates_io = RegistryBuilder::new().build();
    let _alternative = RegistryBuilder::new().alternative().http_index().build();
    Package::new("from_git", "1.0.0").publish();
    Package::new("from_http", "1.0.0")
        .alternative(true)
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                from_git = "1.0"
                from_http = { version = "1.0", registry = "alternative" }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("update -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    let config = ConfigBuilder::new().unstable_flag("gc").build();
    let lock = config.acquire_package_cache_lock().unwrap();
    let last_use = GlobalLastUse::new(&config).unwrap();
    let indexes = last_use.registry_index_all().unwrap();
    assert_eq!(indexes.len(), 2);
    assert_eq!(get_index_names().len(), 2);
    drop(lock);

    // Running in the future without these indexes should delete them.
    p.change_file("Cargo.toml", &basic_manifest("foo", "0.2.0"));
    p.cargo("clean --gc -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    let lock = config.acquire_package_cache_lock().unwrap();
    let indexes = last_use.registry_index_all().unwrap();
    assert_eq!(indexes.len(), 0);
    assert_eq!(get_index_names().len(), 0);
    drop(lock);
}

#[cargo_test]
fn clean_gc_dry_run() {
    // Basic `clean --gc --dry-run` test.
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    // Populate the last-use data.
    p.cargo("fetch -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();

    let expected_files = "\
        [..]/.cargo/registry/src/[..]/bar-1.0.0\n\
        [..]/.cargo/registry/cache/[..]/bar-1.0.0.crate\n\
        [..]/.cargo/registry/index/[..]\n\
    ";
    p.cargo("clean --gc --dry-run -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stdout_unordered(expected_files)
        .with_stderr("[SUMMARY] [..] files/directories, [..] total bytes")
        .run();

    // Again, make sure the information is still tracked.
    p.cargo("clean --gc --dry-run -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stdout_unordered(expected_files)
        .with_stderr("[SUMMARY] [..] files/directories, [..] total bytes")
        .run();
}

#[cargo_test]
fn clean_default_gc() {
    // `clean` without options should also gc
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    // Populate the last-use data.
    p.cargo("fetch -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago_unix(4))
        .run();
    p.cargo("clean -v -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stderr_unordered(
            "\
[REMOVING] [ROOT]/home/.cargo/registry/src/[..]/bar-1.0.0
[REMOVING] [ROOT]/home/.cargo/registry/cache/[..]/bar-1.0.0.crate
[REMOVING] [ROOT]/home/.cargo/registry/index/[..]
[REMOVED] [..] files/directories, [..] total bytes
",
        )
        .run();
}

#[cargo_test]
fn tracks_sizes() {
    // Checks that sizes are properly tracked in the db.
    Package::new("dep1", "1.0.0")
        .file("src/lib.rs", "")
        .publish();
    Package::new("dep2", "1.0.0")
        .file("src/lib.rs", "")
        .file("data", &"abcdefghijklmnopqrstuvwxyz".repeat(1000))
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                dep1 = "1.0"
                dep2 = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("fetch -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();

    // Check that the crate sizes are the same as on disk.
    let config = ConfigBuilder::new().unstable_flag("gc").build();
    let _lock = config.acquire_package_cache_lock().unwrap();
    let last_use = GlobalLastUse::new(&config).unwrap();
    let mut crates = last_use.registry_crate_all().unwrap();
    crates.sort_by(|a, b| a.0.crate_filename.cmp(&b.0.crate_filename));
    let db_sizes: Vec<_> = crates.iter().map(|c| c.0.size).collect();

    let mut actual: Vec<_> = p
        .glob(paths::home().join(".cargo/registry/cache/*/*"))
        .map(|p| p.unwrap())
        .collect();
    actual.sort();
    let actual_sizes: Vec<_> = actual
        .iter()
        .map(|path| std::fs::metadata(path).unwrap().len())
        .collect();
    assert_eq!(db_sizes, actual_sizes);

    // Also check the src sizes are computed.
    let mut srcs = last_use.registry_src_all().unwrap();
    srcs.sort_by(|a, b| a.0.package_dir.cmp(&b.0.package_dir));
    let db_sizes: Vec<_> = srcs.iter().map(|c| c.0.size.unwrap()).collect();
    let mut actual: Vec<_> = p
        .glob(paths::home().join(".cargo/registry/src/*/*"))
        .map(|p| p.unwrap())
        .collect();
    actual.sort();
    // .cargo-ok is not tracked in the size.
    actual.iter().for_each(|p| p.join(".cargo-ok").rm_rf());
    let actual_sizes: Vec<_> = actual
        .iter()
        .map(|path| cargo_util::paths::du(path).unwrap())
        .collect();
    assert_eq!(db_sizes, actual_sizes);
    assert!(db_sizes[1] > 26000);
}

#[cargo_test]
fn max_size() {
    // Checks --max-crate-size and --max-src-size with various cleaning thresholds.
    let config = ConfigBuilder::new().unstable_flag("gc").build();

    let test_crates = [
        // name, age, crate_size, src_size
        ("a-1.0.0", 5, 1, 1),
        ("b-1.0.0", 6, 2, 2),
        ("c-1.0.0", 3, 3, 3),
        ("d-1.0.0", 2, 4, 4),
        ("e-1.0.0", 2, 5, 5),
        ("f-1.0.0", 9, 6, 6),
        ("g-1.0.0", 1, 1, 1),
    ];

    // Determine the order things get deleted so they can be verified.
    let mut names_by_timestamp: Vec<_> = test_crates
        .iter()
        .map(|(name, age, _, _)| (days_ago_unix(*age), name))
        .collect();
    names_by_timestamp.sort();
    let names_by_timestamp: Vec<_> = names_by_timestamp
        .into_iter()
        .map(|(_, name)| name)
        .collect();

    // This exercises the different boundary conditions.
    for (clean_size, files, bytes) in [
        (22, 0, 0),
        (21, 1, 6),
        (16, 1, 6),
        (15, 2, 8),
        (14, 2, 8),
        (13, 3, 9),
        (12, 4, 12),
        (10, 4, 12),
        (9, 5, 16),
        (6, 5, 16),
        (5, 6, 21),
        (1, 6, 21),
        (0, 7, 22),
    ] {
        let (removed, kept) = names_by_timestamp.split_at(files);
        // --max-crate-size
        let (cache_dir, src_dir) = populate_cache(&config, &test_crates);
        let mut stderr = String::new();
        for name in removed {
            writeln!(stderr, "[REMOVING] [..]{name}.crate").unwrap();
        }
        write!(
            stderr,
            "[REMOVED] {files} files/directories, {bytes} total bytes"
        )
        .unwrap();
        cargo_process(&format!("clean -Zgc -v --max-crate-size={clean_size}"))
            .masquerade_as_nightly_cargo(&["gc"])
            .with_stderr_unordered(&stderr)
            .run();
        for name in kept {
            assert!(cache_dir.join(format!("{name}.crate")).exists());
        }
        for name in removed {
            assert!(!cache_dir.join(format!("{name}.crate")).exists());
        }

        // --max-src-size
        populate_cache(&config, &test_crates);
        let mut stderr = String::new();
        for name in removed {
            writeln!(stderr, "[REMOVING] [..]{name}").unwrap();
        }
        let total = files * 2; // dir + file
        write!(
            stderr,
            "[REMOVED] {total} files/directories, {bytes} total bytes"
        )
        .unwrap();
        cargo_process(&format!("clean -Zgc -v --max-src-size={clean_size}"))
            .masquerade_as_nightly_cargo(&["gc"])
            .with_stderr_unordered(&stderr)
            .run();
        for name in kept {
            assert!(src_dir.join(name).exists());
        }
        for name in removed {
            assert!(!src_dir.join(name).exists());
        }
    }
}

#[cargo_test]
fn max_size_untracked_crate() {
    // When a .crate file exists from an older version of cargo that did not
    // track sizes, `clean --max-crate-size` should populate the db with the
    // sizes.
    let config = ConfigBuilder::new().unstable_flag("gc").build();
    let cache = paths::home().join(".cargo/registry/cache/github.com-1ecc6299db9ec823");
    cache.mkdir_p();
    // Create the `.crate files.
    let test_crates = [
        // name, size
        ("a-1.0.0.crate", 1234),
        ("b-1.0.0.crate", 42),
        ("c-1.0.0.crate", 0),
    ];
    for (name, size) in test_crates {
        std::fs::write(cache.join(name), "x".repeat(size as usize)).unwrap()
    }
    // This should scan the directory and populate the db with the size information.
    cargo_process("clean -Zgc -v --max-crate-size=100000")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stderr("[REMOVED] 0 files/directories, 0 total bytes")
        .run();
    // Check that it stored the size data.
    let _lock = config.acquire_package_cache_lock().unwrap();
    let last_use = GlobalLastUse::new(&config).unwrap();
    let crates = last_use.registry_crate_all().unwrap();
    let mut actual: Vec<_> = crates
        .iter()
        .map(|(rc, _time)| (rc.crate_filename.as_str(), rc.size))
        .collect();
    actual.sort();
    assert_eq!(test_crates, actual.as_slice());
}

/// Helper to prepare the max-size test.
fn max_size_untracked_prepare() -> (Config, Project) {
    // First, publish and download a dependency.
    Package::new("bar", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("fetch").run();
    // Pretend it was an older version that did not track last-use.
    let config = ConfigBuilder::new().unstable_flag("gc").build();
    GlobalLastUse::db_path(&config).into_path_unlocked().rm_rf();
    (config, p)
}

/// Helper to verify the max-size test.
fn max_size_untracked_verify(config: &Config) {
    let actual: Vec<_> = glob::glob(
        paths::home()
            .join(".cargo/registry/src/*/*")
            .to_str()
            .unwrap(),
    )
    .unwrap()
    .map(|p| p.unwrap())
    .collect();
    assert_eq!(actual.len(), 1);
    let actual_size = cargo_util::paths::du(&actual[0]).unwrap();
    let lock = config.acquire_package_cache_lock().unwrap();
    let last_use = GlobalLastUse::new(&config).unwrap();
    let srcs = last_use.registry_src_all().unwrap();
    assert_eq!(srcs.len(), 1);
    assert_eq!(srcs[0].0.size, Some(actual_size));
    drop(lock);
}

#[cargo_test]
fn max_size_untracked_src_from_use() {
    // When a src directory exists from an older version of cargo that did not
    // track sizes, doing a build should populate the db with an entry with an
    // unknown size. `clean --max-src-size` should then fix the size.
    let (config, p) = max_size_untracked_prepare();

    // Run a command that will update the db with an unknown src size.
    p.cargo("tree -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    // Check that it is None.
    let lock = config.acquire_package_cache_lock().unwrap();
    let last_use = GlobalLastUse::new(&config).unwrap();
    let srcs = last_use.registry_src_all().unwrap();
    assert_eq!(srcs.len(), 1);
    assert_eq!(srcs[0].0.size, None);
    drop(lock);

    // Fix the size.
    p.cargo("clean -v --max-src-size=10000 -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stderr("[REMOVED] 0 files/directories, 0 total bytes")
        .run();
    max_size_untracked_verify(&config);
}

#[cargo_test]
fn max_size_untracked_src_from_clean() {
    // When a src directory exists from an older version of cargo that did not
    // track sizes, `clean --max-src-size` should populate the db with the
    // sizes.
    let (config, p) = max_size_untracked_prepare();

    // Clean should scan the src and update the db.
    p.cargo("clean -v --max-src-size=10000 -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stderr("[REMOVED] 0 files/directories, 0 total bytes")
        .run();
    max_size_untracked_verify(&config);
}

#[cargo_test]
fn max_download_size() {
    // --max-download-size
    let config = ConfigBuilder::new().unstable_flag("gc").build();

    let test_crates = [
        // name, age, crate_size, src_size
        ("d-1.0.0", 4, 4, 5),
        ("c-1.0.0", 3, 3, 3),
        ("a-1.0.0", 1, 2, 5),
        ("b-1.0.0", 1, 1, 7),
    ];

    for (max_size, num_deleted, files_deleted, bytes) in [
        (30, 0, 0, 0),
        (29, 1, 2, 5),
        (24, 2, 3, 9),
        (20, 3, 5, 12),
        (1, 7, 11, 29),
        (0, 8, 12, 30),
    ] {
        populate_cache(&config, &test_crates);
        // Determine the order things will be deleted.
        let delete_order: Vec<String> = test_crates
            .iter()
            .flat_map(|(name, _, _, _)| [name.to_string(), format!("{name}.crate")])
            .collect();
        let (removed, _kept) = delete_order.split_at(num_deleted);
        let mut stderr = String::new();
        for name in removed {
            writeln!(stderr, "[REMOVING] [..]{name}").unwrap();
        }
        write!(
            stderr,
            "[REMOVED] {files_deleted} files/directories, {bytes} total bytes",
        )
        .unwrap();
        cargo_process(&format!("clean -Zgc -v --max-download-size={max_size}"))
            .masquerade_as_nightly_cargo(&["gc"])
            .with_stderr_unordered(&stderr)
            .run();
    }
}
