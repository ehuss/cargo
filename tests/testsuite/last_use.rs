//! Tests for last-use tracking and auto-gc.

use super::config::ConfigBuilder;
use cargo::core::last_use::{self, GlobalLastUse};
use cargo_test_support::paths::{self, CargoPathExt};
use cargo_test_support::registry::{Package, RegistryBuilder};
use cargo_test_support::{basic_manifest, git, project};
use std::time::{Duration, SystemTime};

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

fn days_ago(n: u64) -> String {
    let st = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * n);
    // TODO: Export functions for working with timestamps.
    st.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn months_ago(n: u64) -> String {
    days_ago(n * 30)
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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
    let mut last_use = GlobalLastUse::new(&config).unwrap();

    last_use.mark_registry_crate_used(last_use::RegistryCrate {
        encoded_registry_name: "github.com-1ecc6299db9ec823".to_string(),
        crate_filename: "regex-1.8.4.crate".to_string(),
    });
    last_use.mark_registry_src_used(last_use::RegistrySrc {
        encoded_registry_name: "index.crates.io-6f17d22bba15001f".to_string(),
        package_dir: "rand-0.8.5".to_string(),
    });
    last_use.mark_git_checkout_used(last_use::GitCheckout {
        encoded_git_name: "cargo-e7ff1db891893a9e".to_string(),
        short_name: "f0a4ee0".to_string(),
    });
    last_use.save().unwrap();

    let mut indexes = last_use.registry_index_all().unwrap();
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

    let dbs = last_use.git_db_all().unwrap();
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(2))
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
        .env("__CARGO_TEST_LAST_USE_NOW", days_ago(4))
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
        .env("__CARGO_TEST_LAST_USE_NOW", days_ago(2))
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
        .run();
    assert_eq!(get_index_names().len(), 1);
    assert_eq!(get_registry_names("src"), ["bar-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["bar-1.0.0.crate"]);

    p.change_file("Cargo.toml", &basic_manifest("foo", "0.2.0"));

    // Try after the default expiration time.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .run();
    assert_eq!(get_index_names().len(), 1);
    assert_eq!(get_registry_names("src"), ["bar-1.0.0"]);
    assert_eq!(get_registry_names("cache"), ["bar-1.0.0.crate"]);

    // Try again with a setting that allows it to run.
    p.cargo("check -Zgc")
        .env("CARGO_GC_AUTO_FREQUENCY", "1 day")
        .env(
            "CARGO_LOG",
            "cargo::core::last_use=trace,cargo::core::gc=trace",
        )
        .stream()
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(2))
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(6))
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
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(6))
        .run();
    assert_eq!(get_git_db_names().len(), 1);
    let second_short_oid = short_id(&git_repo);
    let mut both = vec![first_short_oid, second_short_oid.clone()];
    both.sort();
    assert_eq!(get_git_checkout_names(&db_names[0]), both);

    // In the future, using the second checkout should delete the first.
    p.cargo("check -Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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
            .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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

    p.cargo("update")
        .arg("-Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
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
    p.cargo("clean --gc")
        .arg("-Zgc")
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
    p.cargo("fetch")
        .arg("-Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .env("__CARGO_TEST_LAST_USE_NOW", months_ago(4))
        .run();

    let expected_files = "\
        [..]/.cargo/registry/src/[..]/bar-1.0.0\n\
        [..]/.cargo/registry/cache/[..]/bar-1.0.0.crate\n\
        [..]/.cargo/registry/index/[..]\n\
    ";
    p.cargo("clean --gc --dry-run")
        .arg("-Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stdout_unordered(expected_files)
        .with_stderr("[SUMMARY] [..] files/directories, [..] total bytes")
        .run();

    // Again, make sure the information is still tracked.
    p.cargo("clean --gc --dry-run")
        .arg("-Zgc")
        .masquerade_as_nightly_cargo(&["gc"])
        .with_stdout_unordered(expected_files)
        .with_stderr("[SUMMARY] [..] files/directories, [..] total bytes")
        .run();
}

#[cargo_test]
fn clean_default_gc() {
    // `clean` without options should also gc
}
