//! Tests for `[patch]` table source replacement.

use cargo_test_support::git;
use cargo_test_support::paths;
use cargo_test_support::registry::{self, Package};
use cargo_test_support::{basic_manifest, project};
use std::fs;

#[cargo_test]
fn replace() {
    Package::new("bar", "0.1.0").publish();
    Package::new("baz", "0.1.0")
        .file(
            "src/lib.rs",
            "extern crate bar; pub fn baz() { bar::bar(); }",
        )
        .dep("bar", "0.1.0")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
                baz = "0.1.0"

                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file(
            "src/lib.rs",
            "
            extern crate bar;
            extern crate baz;
            pub fn bar() {
                bar::bar();
                baz::baz();
            }
        ",
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] baz v0.1.0 ([..])
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] baz v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn from_config() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file(
            ".cargo/config.toml",
            r#"
                [patch.crates-io]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn from_config_relative() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file(
            "../.cargo/config.toml",
            r#"
                [patch.crates-io]
                bar = { path = 'foo/bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn from_config_precedence() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.crates-io]
                bar = { path = 'no-such-path' }
            "#,
        )
        .file(
            ".cargo/config.toml",
            r#"
                [patch.crates-io]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn nonexistent() {
    Package::new("baz", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file(
            "src/lib.rs",
            "extern crate bar; pub fn foo() { bar::bar(); }",
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn patch_git() {
    let bar = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "")
        .build();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    authors = []

                    [dependencies]
                    bar = {{ git = '{}' }}

                    [patch.'{0}']
                    bar = {{ path = "bar" }}
                "#,
                bar.url()
            ),
        )
        .file(
            "src/lib.rs",
            "extern crate bar; pub fn foo() { bar::bar(); }",
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `file://[..]`
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn patch_to_git() {
    let bar = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn bar() {}")
        .build();

    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    authors = []

                    [dependencies]
                    bar = "0.1"

                    [patch.crates-io]
                    bar = {{ git = '{}' }}
                "#,
                bar.url()
            ),
        )
        .file(
            "src/lib.rs",
            "extern crate bar; pub fn foo() { bar::bar(); }",
        )
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `file://[..]`
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.0 (file://[..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn unused() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.2.0"))
        .file("bar/src/lib.rs", "not rust code")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[WARNING] Patch `bar v0.2.0 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] Patch `bar v0.2.0 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[FINISHED] [..]
",
        )
        .run();

    // unused patch should be in the lock file
    let lock = p.read_lockfile();
    let toml: toml::Table = toml::from_str(&lock).unwrap();
    assert_eq!(toml["patch"]["unused"].as_array().unwrap().len(), 1);
    assert_eq!(toml["patch"]["unused"][0]["name"].as_str(), Some("bar"));
    assert_eq!(
        toml["patch"]["unused"][0]["version"].as_str(),
        Some("0.2.0")
    );
}

#[cargo_test]
fn unused_with_mismatch_source_being_patched() {
    registry::alt_init();
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.alternative]
                bar = { path = "bar" }

                [patch.crates-io]
                bar = { path = "baz" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.2.0"))
        .file("bar/src/lib.rs", "not rust code")
        .file("baz/Cargo.toml", &basic_manifest("bar", "0.3.0"))
        .file("baz/src/lib.rs", "not rust code")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[WARNING] Patch `bar v0.2.0 ([CWD]/bar)` was not used in the crate graph.
Perhaps you misspelled the source URL being patched.
Possible URLs for `[patch.<URL>]`:
    crates-io
[WARNING] Patch `bar v0.3.0 ([CWD]/baz)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn prefer_patch_version() {
    Package::new("bar", "0.1.2").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[FINISHED] [..]
",
        )
        .run();

    // there should be no patch.unused in the toml file
    let lock = p.read_lockfile();
    let toml: toml::Table = toml::from_str(&lock).unwrap();
    assert!(toml.get("patch").is_none());
}

#[cargo_test]
fn unused_from_config() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file(
            ".cargo/config.toml",
            r#"
                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.2.0"))
        .file("bar/src/lib.rs", "not rust code")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[WARNING] Patch `bar v0.2.0 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] Patch `bar v0.2.0 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[FINISHED] [..]
",
        )
        .run();

    // unused patch should be in the lock file
    let lock = p.read_lockfile();
    let toml: toml::Table = toml::from_str(&lock).unwrap();
    assert_eq!(toml["patch"]["unused"].as_array().unwrap().len(), 1);
    assert_eq!(toml["patch"]["unused"][0]["name"].as_str(), Some("bar"));
    assert_eq!(
        toml["patch"]["unused"][0]["version"].as_str(),
        Some("0.2.0")
    );
}

#[cargo_test]
fn unused_git() {
    Package::new("bar", "0.1.0").publish();

    let foo = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.2.0"))
        .file("src/lib.rs", "")
        .build();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    authors = []

                    [dependencies]
                    bar = "0.1"

                    [patch.crates-io]
                    bar = {{ git = '{}' }}
                "#,
                foo.url()
            ),
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `file://[..]`
[UPDATING] `dummy-registry` index
[WARNING] Patch `bar v0.2.0 ([..])` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] Patch `bar v0.2.0 ([..])` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn add_patch() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.1.0"

            [patch.crates-io]
            bar = { path = 'bar' }
        "#,
    );

    p.cargo("check")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn add_patch_from_config() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    p.change_file(
        ".cargo/config.toml",
        r#"
            [patch.crates-io]
            bar = { path = 'bar' }
        "#,
    );

    p.cargo("check")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn add_ignored_patch() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 [..]
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.1.0"

            [patch.crates-io]
            bar = { path = 'bar' }
        "#,
    );

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] Patch `bar v0.1.1 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] Patch `bar v0.1.1 ([CWD]/bar)` was not used in the crate graph.
Check that [..]
with the [..]
what is [..]
version. [..]
[FINISHED] [..]",
        )
        .run();

    p.cargo("update").run();
    p.cargo("check")
        .with_stderr(
            "\
[CHECKING] bar v0.1.1 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [..]
",
        )
        .run();
}

#[cargo_test]
fn add_patch_with_features() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.1.0"

            [patch.crates-io]
            bar = { path = 'bar', features = ["some_feature"] }
        "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] patch for `bar` uses the features mechanism. \
default-features and features will not take effect because the patch dependency does not support this mechanism
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] patch for `bar` uses the features mechanism. \
default-features and features will not take effect because the patch dependency does not support this mechanism
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn add_patch_with_setting_default_features() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.1.0"

            [patch.crates-io]
            bar = { path = 'bar', default-features = false, features = ["none_default_feature"] }
        "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] patch for `bar` uses the features mechanism. \
default-features and features will not take effect because the patch dependency does not support this mechanism
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.0 ([CWD]/bar)
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("check")
        .with_stderr(
            "\
[WARNING] patch for `bar` uses the features mechanism. \
default-features and features will not take effect because the patch dependency does not support this mechanism
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn no_warn_ws_patch() {
    Package::new("c", "0.1.0").publish();

    // Don't issue an unused patch warning when the patch isn't used when
    // partially building a workspace.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["a", "b", "c"]

                [patch.crates-io]
                c = { path = "c" }
            "#,
        )
        .file("a/Cargo.toml", &basic_manifest("a", "0.1.0"))
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "0.1.0"
                [dependencies]
                c = "0.1.0"
            "#,
        )
        .file("b/src/lib.rs", "")
        .file("c/Cargo.toml", &basic_manifest("c", "0.1.0"))
        .file("c/src/lib.rs", "")
        .build();

    p.cargo("check -p a")
        .with_stderr(
            "\
[UPDATING] [..]
[CHECKING] a [..]
[FINISHED] [..]",
        )
        .run();
}

#[cargo_test]
fn new_minor() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.crates-io]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 [..]
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn transitive_new_minor() {
    Package::new("baz", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = { path = 'bar' }

                [patch.crates-io]
                baz = { path = 'baz' }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                authors = []

                [dependencies]
                baz = '0.1.0'
            "#,
        )
        .file("bar/src/lib.rs", r#""#)
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.1"))
        .file("baz/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] baz v0.1.1 [..]
[CHECKING] bar v0.1.0 [..]
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn new_major() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.2.0"

                [patch.crates-io]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.2.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.2.0 [..]
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    Package::new("bar", "0.2.0").publish();
    p.cargo("update").run();
    p.cargo("check")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();

    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.2.0"
        "#,
    );
    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.2.0 [..]
[CHECKING] bar v0.2.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn transitive_new_major() {
    Package::new("baz", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = { path = 'bar' }

                [patch.crates-io]
                baz = { path = 'baz' }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                authors = []

                [dependencies]
                baz = '0.2.0'
            "#,
        )
        .file("bar/src/lib.rs", r#""#)
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.2.0"))
        .file("baz/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] baz v0.2.0 [..]
[CHECKING] bar v0.1.0 [..]
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn shared_by_transitive() {
    Package::new("baz", "0.1.1").publish();

    let baz = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("baz", "0.1.2"))
        .file("src/lib.rs", "")
        .build();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = " 0.1.0"

                    [dependencies]
                    bar = {{ path = "bar" }}
                    baz = "0.1"

                    [patch.crates-io]
                    baz = {{ git = "{}", version = "0.1" }}
                "#,
                baz.url(),
            ),
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"

                [dependencies]
                baz = "0.1.1"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `file://[..]`
[UPDATING] `dummy-registry` index
[CHECKING] baz v0.1.2 [..]
[CHECKING] bar v0.1.0 [..]
[CHECKING] foo v0.1.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn remove_patch() {
    Package::new("foo", "0.1.0").publish();
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1"

                [patch.crates-io]
                foo = { path = 'foo' }
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .file("foo/Cargo.toml", &basic_manifest("foo", "0.1.0"))
        .file("foo/src/lib.rs", r#""#)
        .build();

    // Generate a lock file where `foo` is unused
    p.cargo("check").run();
    let lock_file1 = p.read_lockfile();

    // Remove `foo` and generate a new lock file form the old one
    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = "0.1"

            [patch.crates-io]
            bar = { path = 'bar' }
        "#,
    );
    p.cargo("check").run();
    let lock_file2 = p.read_lockfile();

    // Remove the lock file and build from scratch
    fs::remove_file(p.root().join("Cargo.lock")).unwrap();
    p.cargo("check").run();
    let lock_file3 = p.read_lockfile();

    assert!(lock_file1.contains("foo"));
    assert_eq!(lock_file2, lock_file3);
    assert_ne!(lock_file1, lock_file2);
}

#[cargo_test]
fn non_crates_io() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [patch.some-other-source]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]`

Caused by:
  [patch] entry `some-other-source` should be a URL or registry name

Caused by:
  invalid url `some-other-source`: relative URL without a base
",
        )
        .run();
}

#[cargo_test]
fn replace_with_crates_io() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [patch.crates-io]
                bar = "0.1"
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] [..]
error: failed to resolve patches for `[..]`

Caused by:
  patch for `bar` in `[..]` points to the same source, but patches must point \
  to different sources
",
        )
        .run();
}

#[cargo_test]
fn patch_in_virtual() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo"]

                [patch.crates-io]
                bar = { path = "bar" }
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", r#""#)
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [dependencies]
                bar = "0.1"
            "#,
        )
        .file("foo/src/lib.rs", r#""#)
        .build();

    p.cargo("check").run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn patch_depends_on_another_patch() {
    Package::new("bar", "0.1.0")
        .file("src/lib.rs", "broken code")
        .publish();

    Package::new("baz", "0.1.0")
        .dep("bar", "0.1")
        .file("src/lib.rs", "broken code")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                authors = []
                version = "0.1.0"

                [dependencies]
                bar = "0.1"
                baz = "0.1"

                [patch.crates-io]
                bar = { path = "bar" }
                baz = { path = "baz" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"
                authors = []

                [dependencies]
                bar = "0.1"
            "#,
        )
        .file("baz/src/lib.rs", r#""#)
        .build();

    p.cargo("check").run();

    // Nothing should be rebuilt, no registry should be updated.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn replace_prerelease() {
    Package::new("baz", "1.1.0-pre.1").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar"]

                [patch.crates-io]
                baz = { path = "./baz" }
            "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.5.0"
                authors = []

                [dependencies]
                baz = "1.1.0-pre.1"
            "#,
        )
        .file(
            "bar/src/main.rs",
            "extern crate baz; fn main() { baz::baz() }",
        )
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "1.1.0-pre.1"
                authors = []
                [workspace]
            "#,
        )
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check").run();
}

#[cargo_test]
fn patch_older() {
    Package::new("baz", "1.0.2").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = { path = 'bar' }
                baz = "=1.0.1"

                [patch.crates-io]
                baz = { path = "./baz" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.5.0"
                authors = []

                [dependencies]
                baz = "1.0.0"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "1.0.1"
                authors = []
            "#,
        )
        .file("baz/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] [..]
[CHECKING] baz v1.0.1 [..]
[CHECKING] bar v0.5.0 [..]
[CHECKING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn cycle() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["a", "b"]

                [patch.crates-io]
                a = {path="a"}
                b = {path="b"}
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "1.0.0"

                [dependencies]
                b = "1.0"
            "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "1.0.0"

                [dependencies]
                a = "1.0"
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] [..]
[ERROR] cyclic package dependency: [..]
package `[..]`
    ... which satisfies dependency `[..]` of package `[..]`
    ... which satisfies dependency `[..]` of package `[..]`
",
        )
        .run();
}

#[cargo_test]
fn multipatch() {
    Package::new("a", "1.0.0").publish();
    Package::new("a", "2.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                a1 = { version = "1", package = "a" }
                a2 = { version = "2", package = "a" }

                [patch.crates-io]
                b1 = { path = "a1", package = "a" }
                b2 = { path = "a2", package = "a" }
            "#,
        )
        .file("src/lib.rs", "pub fn foo() { a1::f1(); a2::f2(); }")
        .file(
            "a1/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "1.0.0"
            "#,
        )
        .file("a1/src/lib.rs", "pub fn f1() {}")
        .file(
            "a2/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "2.0.0"
            "#,
        )
        .file("a2/src/lib.rs", "pub fn f2() {}")
        .build();

    p.cargo("check").run();
}

#[cargo_test]
fn patch_same_version() {
    let bar = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "")
        .build();

    cargo_test_support::registry::init();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    [dependencies]
                    bar = "0.1"
                    [patch.crates-io]
                    bar = {{ path = "bar" }}
                    bar2 = {{ git = '{}', package = 'bar' }}
                "#,
                bar.url(),
            ),
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] [..]
error: cannot have two `[patch]` entries which both resolve to `bar v0.1.0`
",
        )
        .run();
}

#[test] fn two_semver_compatible0() { two_semver_compatible(); }
#[test] fn two_semver_compatible1() { two_semver_compatible(); }
#[test] fn two_semver_compatible2() { two_semver_compatible(); }
#[test] fn two_semver_compatible3() { two_semver_compatible(); }
#[test] fn two_semver_compatible4() { two_semver_compatible(); }
#[test] fn two_semver_compatible5() { two_semver_compatible(); }
#[test] fn two_semver_compatible6() { two_semver_compatible(); }
#[test] fn two_semver_compatible7() { two_semver_compatible(); }
#[test] fn two_semver_compatible8() { two_semver_compatible(); }
#[test] fn two_semver_compatible9() { two_semver_compatible(); }
#[test] fn two_semver_compatible10() { two_semver_compatible(); }
#[test] fn two_semver_compatible11() { two_semver_compatible(); }
#[test] fn two_semver_compatible12() { two_semver_compatible(); }
#[test] fn two_semver_compatible13() { two_semver_compatible(); }
#[test] fn two_semver_compatible14() { two_semver_compatible(); }
#[test] fn two_semver_compatible15() { two_semver_compatible(); }
#[test] fn two_semver_compatible16() { two_semver_compatible(); }
#[test] fn two_semver_compatible17() { two_semver_compatible(); }
#[test] fn two_semver_compatible18() { two_semver_compatible(); }
#[test] fn two_semver_compatible19() { two_semver_compatible(); }
#[test] fn two_semver_compatible20() { two_semver_compatible(); }
#[test] fn two_semver_compatible21() { two_semver_compatible(); }
#[test] fn two_semver_compatible22() { two_semver_compatible(); }
#[test] fn two_semver_compatible23() { two_semver_compatible(); }
#[test] fn two_semver_compatible24() { two_semver_compatible(); }
#[test] fn two_semver_compatible25() { two_semver_compatible(); }
#[test] fn two_semver_compatible26() { two_semver_compatible(); }
#[test] fn two_semver_compatible27() { two_semver_compatible(); }
#[test] fn two_semver_compatible28() { two_semver_compatible(); }
#[test] fn two_semver_compatible29() { two_semver_compatible(); }
#[test] fn two_semver_compatible30() { two_semver_compatible(); }
#[test] fn two_semver_compatible31() { two_semver_compatible(); }
#[test] fn two_semver_compatible32() { two_semver_compatible(); }
#[test] fn two_semver_compatible33() { two_semver_compatible(); }
#[test] fn two_semver_compatible34() { two_semver_compatible(); }
#[test] fn two_semver_compatible35() { two_semver_compatible(); }
#[test] fn two_semver_compatible36() { two_semver_compatible(); }
#[test] fn two_semver_compatible37() { two_semver_compatible(); }
#[test] fn two_semver_compatible38() { two_semver_compatible(); }
#[test] fn two_semver_compatible39() { two_semver_compatible(); }
#[test] fn two_semver_compatible40() { two_semver_compatible(); }
#[test] fn two_semver_compatible41() { two_semver_compatible(); }
#[test] fn two_semver_compatible42() { two_semver_compatible(); }
#[test] fn two_semver_compatible43() { two_semver_compatible(); }
#[test] fn two_semver_compatible44() { two_semver_compatible(); }
#[test] fn two_semver_compatible45() { two_semver_compatible(); }
#[test] fn two_semver_compatible46() { two_semver_compatible(); }
#[test] fn two_semver_compatible47() { two_semver_compatible(); }
#[test] fn two_semver_compatible48() { two_semver_compatible(); }
#[test] fn two_semver_compatible49() { two_semver_compatible(); }
#[test] fn two_semver_compatible50() { two_semver_compatible(); }
#[test] fn two_semver_compatible51() { two_semver_compatible(); }
#[test] fn two_semver_compatible52() { two_semver_compatible(); }
#[test] fn two_semver_compatible53() { two_semver_compatible(); }
#[test] fn two_semver_compatible54() { two_semver_compatible(); }
#[test] fn two_semver_compatible55() { two_semver_compatible(); }
#[test] fn two_semver_compatible56() { two_semver_compatible(); }
#[test] fn two_semver_compatible57() { two_semver_compatible(); }
#[test] fn two_semver_compatible58() { two_semver_compatible(); }
#[test] fn two_semver_compatible59() { two_semver_compatible(); }
#[test] fn two_semver_compatible60() { two_semver_compatible(); }
#[test] fn two_semver_compatible61() { two_semver_compatible(); }
#[test] fn two_semver_compatible62() { two_semver_compatible(); }
#[test] fn two_semver_compatible63() { two_semver_compatible(); }
#[test] fn two_semver_compatible64() { two_semver_compatible(); }
#[test] fn two_semver_compatible65() { two_semver_compatible(); }
#[test] fn two_semver_compatible66() { two_semver_compatible(); }
#[test] fn two_semver_compatible67() { two_semver_compatible(); }
#[test] fn two_semver_compatible68() { two_semver_compatible(); }
#[test] fn two_semver_compatible69() { two_semver_compatible(); }
#[test] fn two_semver_compatible70() { two_semver_compatible(); }
#[test] fn two_semver_compatible71() { two_semver_compatible(); }
#[test] fn two_semver_compatible72() { two_semver_compatible(); }
#[test] fn two_semver_compatible73() { two_semver_compatible(); }
#[test] fn two_semver_compatible74() { two_semver_compatible(); }
#[test] fn two_semver_compatible75() { two_semver_compatible(); }
#[test] fn two_semver_compatible76() { two_semver_compatible(); }
#[test] fn two_semver_compatible77() { two_semver_compatible(); }
#[test] fn two_semver_compatible78() { two_semver_compatible(); }
#[test] fn two_semver_compatible79() { two_semver_compatible(); }
#[test] fn two_semver_compatible80() { two_semver_compatible(); }
#[test] fn two_semver_compatible81() { two_semver_compatible(); }
#[test] fn two_semver_compatible82() { two_semver_compatible(); }
#[test] fn two_semver_compatible83() { two_semver_compatible(); }
#[test] fn two_semver_compatible84() { two_semver_compatible(); }
#[test] fn two_semver_compatible85() { two_semver_compatible(); }
#[test] fn two_semver_compatible86() { two_semver_compatible(); }
#[test] fn two_semver_compatible87() { two_semver_compatible(); }
#[test] fn two_semver_compatible88() { two_semver_compatible(); }
#[test] fn two_semver_compatible89() { two_semver_compatible(); }
#[test] fn two_semver_compatible90() { two_semver_compatible(); }
#[test] fn two_semver_compatible91() { two_semver_compatible(); }
#[test] fn two_semver_compatible92() { two_semver_compatible(); }
#[test] fn two_semver_compatible93() { two_semver_compatible(); }
#[test] fn two_semver_compatible94() { two_semver_compatible(); }
#[test] fn two_semver_compatible95() { two_semver_compatible(); }
#[test] fn two_semver_compatible96() { two_semver_compatible(); }
#[test] fn two_semver_compatible97() { two_semver_compatible(); }
#[test] fn two_semver_compatible98() { two_semver_compatible(); }
#[test] fn two_semver_compatible99() { two_semver_compatible(); }
#[test] fn two_semver_compatible100() { two_semver_compatible(); }
#[test] fn two_semver_compatible101() { two_semver_compatible(); }
#[test] fn two_semver_compatible102() { two_semver_compatible(); }
#[test] fn two_semver_compatible103() { two_semver_compatible(); }
#[test] fn two_semver_compatible104() { two_semver_compatible(); }
#[test] fn two_semver_compatible105() { two_semver_compatible(); }
#[test] fn two_semver_compatible106() { two_semver_compatible(); }
#[test] fn two_semver_compatible107() { two_semver_compatible(); }
#[test] fn two_semver_compatible108() { two_semver_compatible(); }
#[test] fn two_semver_compatible109() { two_semver_compatible(); }
#[test] fn two_semver_compatible110() { two_semver_compatible(); }
#[test] fn two_semver_compatible111() { two_semver_compatible(); }
#[test] fn two_semver_compatible112() { two_semver_compatible(); }
#[test] fn two_semver_compatible113() { two_semver_compatible(); }
#[test] fn two_semver_compatible114() { two_semver_compatible(); }
#[test] fn two_semver_compatible115() { two_semver_compatible(); }
#[test] fn two_semver_compatible116() { two_semver_compatible(); }
#[test] fn two_semver_compatible117() { two_semver_compatible(); }
#[test] fn two_semver_compatible118() { two_semver_compatible(); }
#[test] fn two_semver_compatible119() { two_semver_compatible(); }
#[test] fn two_semver_compatible120() { two_semver_compatible(); }
#[test] fn two_semver_compatible121() { two_semver_compatible(); }
#[test] fn two_semver_compatible122() { two_semver_compatible(); }
#[test] fn two_semver_compatible123() { two_semver_compatible(); }
#[test] fn two_semver_compatible124() { two_semver_compatible(); }
#[test] fn two_semver_compatible125() { two_semver_compatible(); }
#[test] fn two_semver_compatible126() { two_semver_compatible(); }
#[test] fn two_semver_compatible127() { two_semver_compatible(); }
#[test] fn two_semver_compatible128() { two_semver_compatible(); }
#[test] fn two_semver_compatible129() { two_semver_compatible(); }
#[test] fn two_semver_compatible130() { two_semver_compatible(); }
#[test] fn two_semver_compatible131() { two_semver_compatible(); }
#[test] fn two_semver_compatible132() { two_semver_compatible(); }
#[test] fn two_semver_compatible133() { two_semver_compatible(); }
#[test] fn two_semver_compatible134() { two_semver_compatible(); }
#[test] fn two_semver_compatible135() { two_semver_compatible(); }
#[test] fn two_semver_compatible136() { two_semver_compatible(); }
#[test] fn two_semver_compatible137() { two_semver_compatible(); }
#[test] fn two_semver_compatible138() { two_semver_compatible(); }
#[test] fn two_semver_compatible139() { two_semver_compatible(); }
#[test] fn two_semver_compatible140() { two_semver_compatible(); }
#[test] fn two_semver_compatible141() { two_semver_compatible(); }
#[test] fn two_semver_compatible142() { two_semver_compatible(); }
#[test] fn two_semver_compatible143() { two_semver_compatible(); }
#[test] fn two_semver_compatible144() { two_semver_compatible(); }
#[test] fn two_semver_compatible145() { two_semver_compatible(); }
#[test] fn two_semver_compatible146() { two_semver_compatible(); }
#[test] fn two_semver_compatible147() { two_semver_compatible(); }
#[test] fn two_semver_compatible148() { two_semver_compatible(); }
#[test] fn two_semver_compatible149() { two_semver_compatible(); }
#[test] fn two_semver_compatible150() { two_semver_compatible(); }
#[test] fn two_semver_compatible151() { two_semver_compatible(); }
#[test] fn two_semver_compatible152() { two_semver_compatible(); }
#[test] fn two_semver_compatible153() { two_semver_compatible(); }
#[test] fn two_semver_compatible154() { two_semver_compatible(); }
#[test] fn two_semver_compatible155() { two_semver_compatible(); }
#[test] fn two_semver_compatible156() { two_semver_compatible(); }
#[test] fn two_semver_compatible157() { two_semver_compatible(); }
#[test] fn two_semver_compatible158() { two_semver_compatible(); }
#[test] fn two_semver_compatible159() { two_semver_compatible(); }
#[test] fn two_semver_compatible160() { two_semver_compatible(); }
#[test] fn two_semver_compatible161() { two_semver_compatible(); }
#[test] fn two_semver_compatible162() { two_semver_compatible(); }
#[test] fn two_semver_compatible163() { two_semver_compatible(); }
#[test] fn two_semver_compatible164() { two_semver_compatible(); }
#[test] fn two_semver_compatible165() { two_semver_compatible(); }
#[test] fn two_semver_compatible166() { two_semver_compatible(); }
#[test] fn two_semver_compatible167() { two_semver_compatible(); }
#[test] fn two_semver_compatible168() { two_semver_compatible(); }
#[test] fn two_semver_compatible169() { two_semver_compatible(); }
#[test] fn two_semver_compatible170() { two_semver_compatible(); }
#[test] fn two_semver_compatible171() { two_semver_compatible(); }
#[test] fn two_semver_compatible172() { two_semver_compatible(); }
#[test] fn two_semver_compatible173() { two_semver_compatible(); }
#[test] fn two_semver_compatible174() { two_semver_compatible(); }
#[test] fn two_semver_compatible175() { two_semver_compatible(); }
#[test] fn two_semver_compatible176() { two_semver_compatible(); }
#[test] fn two_semver_compatible177() { two_semver_compatible(); }
#[test] fn two_semver_compatible178() { two_semver_compatible(); }
#[test] fn two_semver_compatible179() { two_semver_compatible(); }
#[test] fn two_semver_compatible180() { two_semver_compatible(); }
#[test] fn two_semver_compatible181() { two_semver_compatible(); }
#[test] fn two_semver_compatible182() { two_semver_compatible(); }
#[test] fn two_semver_compatible183() { two_semver_compatible(); }
#[test] fn two_semver_compatible184() { two_semver_compatible(); }
#[test] fn two_semver_compatible185() { two_semver_compatible(); }
#[test] fn two_semver_compatible186() { two_semver_compatible(); }
#[test] fn two_semver_compatible187() { two_semver_compatible(); }
#[test] fn two_semver_compatible188() { two_semver_compatible(); }
#[test] fn two_semver_compatible189() { two_semver_compatible(); }
#[test] fn two_semver_compatible190() { two_semver_compatible(); }
#[test] fn two_semver_compatible191() { two_semver_compatible(); }
#[test] fn two_semver_compatible192() { two_semver_compatible(); }
#[test] fn two_semver_compatible193() { two_semver_compatible(); }
#[test] fn two_semver_compatible194() { two_semver_compatible(); }
#[test] fn two_semver_compatible195() { two_semver_compatible(); }
#[test] fn two_semver_compatible196() { two_semver_compatible(); }
#[test] fn two_semver_compatible197() { two_semver_compatible(); }
#[test] fn two_semver_compatible198() { two_semver_compatible(); }
#[test] fn two_semver_compatible199() { two_semver_compatible(); }
#[test] fn two_semver_compatible200() { two_semver_compatible(); }
#[test] fn two_semver_compatible201() { two_semver_compatible(); }
#[test] fn two_semver_compatible202() { two_semver_compatible(); }
#[test] fn two_semver_compatible203() { two_semver_compatible(); }
#[test] fn two_semver_compatible204() { two_semver_compatible(); }
#[test] fn two_semver_compatible205() { two_semver_compatible(); }
#[test] fn two_semver_compatible206() { two_semver_compatible(); }
#[test] fn two_semver_compatible207() { two_semver_compatible(); }
#[test] fn two_semver_compatible208() { two_semver_compatible(); }
#[test] fn two_semver_compatible209() { two_semver_compatible(); }
#[test] fn two_semver_compatible210() { two_semver_compatible(); }
#[test] fn two_semver_compatible211() { two_semver_compatible(); }
#[test] fn two_semver_compatible212() { two_semver_compatible(); }
#[test] fn two_semver_compatible213() { two_semver_compatible(); }
#[test] fn two_semver_compatible214() { two_semver_compatible(); }
#[test] fn two_semver_compatible215() { two_semver_compatible(); }
#[test] fn two_semver_compatible216() { two_semver_compatible(); }
#[test] fn two_semver_compatible217() { two_semver_compatible(); }
#[test] fn two_semver_compatible218() { two_semver_compatible(); }
#[test] fn two_semver_compatible219() { two_semver_compatible(); }
#[test] fn two_semver_compatible220() { two_semver_compatible(); }
#[test] fn two_semver_compatible221() { two_semver_compatible(); }
#[test] fn two_semver_compatible222() { two_semver_compatible(); }
#[test] fn two_semver_compatible223() { two_semver_compatible(); }
#[test] fn two_semver_compatible224() { two_semver_compatible(); }
#[test] fn two_semver_compatible225() { two_semver_compatible(); }
#[test] fn two_semver_compatible226() { two_semver_compatible(); }
#[test] fn two_semver_compatible227() { two_semver_compatible(); }
#[test] fn two_semver_compatible228() { two_semver_compatible(); }
#[test] fn two_semver_compatible229() { two_semver_compatible(); }
#[test] fn two_semver_compatible230() { two_semver_compatible(); }
#[test] fn two_semver_compatible231() { two_semver_compatible(); }
#[test] fn two_semver_compatible232() { two_semver_compatible(); }
#[test] fn two_semver_compatible233() { two_semver_compatible(); }
#[test] fn two_semver_compatible234() { two_semver_compatible(); }
#[test] fn two_semver_compatible235() { two_semver_compatible(); }
#[test] fn two_semver_compatible236() { two_semver_compatible(); }
#[test] fn two_semver_compatible237() { two_semver_compatible(); }
#[test] fn two_semver_compatible238() { two_semver_compatible(); }
#[test] fn two_semver_compatible239() { two_semver_compatible(); }
#[test] fn two_semver_compatible240() { two_semver_compatible(); }
#[test] fn two_semver_compatible241() { two_semver_compatible(); }
#[test] fn two_semver_compatible242() { two_semver_compatible(); }
#[test] fn two_semver_compatible243() { two_semver_compatible(); }
#[test] fn two_semver_compatible244() { two_semver_compatible(); }
#[test] fn two_semver_compatible245() { two_semver_compatible(); }
#[test] fn two_semver_compatible246() { two_semver_compatible(); }
#[test] fn two_semver_compatible247() { two_semver_compatible(); }
#[test] fn two_semver_compatible248() { two_semver_compatible(); }
#[test] fn two_semver_compatible249() { two_semver_compatible(); }
#[test] fn two_semver_compatible250() { two_semver_compatible(); }
#[test] fn two_semver_compatible251() { two_semver_compatible(); }
#[test] fn two_semver_compatible252() { two_semver_compatible(); }
#[test] fn two_semver_compatible253() { two_semver_compatible(); }
#[test] fn two_semver_compatible254() { two_semver_compatible(); }
#[test] fn two_semver_compatible255() { two_semver_compatible(); }
#[test] fn two_semver_compatible256() { two_semver_compatible(); }
#[test] fn two_semver_compatible257() { two_semver_compatible(); }
#[test] fn two_semver_compatible258() { two_semver_compatible(); }
#[test] fn two_semver_compatible259() { two_semver_compatible(); }
#[test] fn two_semver_compatible260() { two_semver_compatible(); }
#[test] fn two_semver_compatible261() { two_semver_compatible(); }
#[test] fn two_semver_compatible262() { two_semver_compatible(); }
#[test] fn two_semver_compatible263() { two_semver_compatible(); }
#[test] fn two_semver_compatible264() { two_semver_compatible(); }
#[test] fn two_semver_compatible265() { two_semver_compatible(); }
#[test] fn two_semver_compatible266() { two_semver_compatible(); }
#[test] fn two_semver_compatible267() { two_semver_compatible(); }
#[test] fn two_semver_compatible268() { two_semver_compatible(); }
#[test] fn two_semver_compatible269() { two_semver_compatible(); }
#[test] fn two_semver_compatible270() { two_semver_compatible(); }
#[test] fn two_semver_compatible271() { two_semver_compatible(); }
#[test] fn two_semver_compatible272() { two_semver_compatible(); }
#[test] fn two_semver_compatible273() { two_semver_compatible(); }
#[test] fn two_semver_compatible274() { two_semver_compatible(); }
#[test] fn two_semver_compatible275() { two_semver_compatible(); }
#[test] fn two_semver_compatible276() { two_semver_compatible(); }
#[test] fn two_semver_compatible277() { two_semver_compatible(); }
#[test] fn two_semver_compatible278() { two_semver_compatible(); }
#[test] fn two_semver_compatible279() { two_semver_compatible(); }
#[test] fn two_semver_compatible280() { two_semver_compatible(); }
#[test] fn two_semver_compatible281() { two_semver_compatible(); }
#[test] fn two_semver_compatible282() { two_semver_compatible(); }
#[test] fn two_semver_compatible283() { two_semver_compatible(); }
#[test] fn two_semver_compatible284() { two_semver_compatible(); }
#[test] fn two_semver_compatible285() { two_semver_compatible(); }
#[test] fn two_semver_compatible286() { two_semver_compatible(); }
#[test] fn two_semver_compatible287() { two_semver_compatible(); }
#[test] fn two_semver_compatible288() { two_semver_compatible(); }
#[test] fn two_semver_compatible289() { two_semver_compatible(); }
#[test] fn two_semver_compatible290() { two_semver_compatible(); }
#[test] fn two_semver_compatible291() { two_semver_compatible(); }
#[test] fn two_semver_compatible292() { two_semver_compatible(); }
#[test] fn two_semver_compatible293() { two_semver_compatible(); }
#[test] fn two_semver_compatible294() { two_semver_compatible(); }
#[test] fn two_semver_compatible295() { two_semver_compatible(); }
#[test] fn two_semver_compatible296() { two_semver_compatible(); }
#[test] fn two_semver_compatible297() { two_semver_compatible(); }
#[test] fn two_semver_compatible298() { two_semver_compatible(); }
#[test] fn two_semver_compatible299() { two_semver_compatible(); }
#[test] fn two_semver_compatible300() { two_semver_compatible(); }
#[test] fn two_semver_compatible301() { two_semver_compatible(); }
#[test] fn two_semver_compatible302() { two_semver_compatible(); }
#[test] fn two_semver_compatible303() { two_semver_compatible(); }
#[test] fn two_semver_compatible304() { two_semver_compatible(); }
#[test] fn two_semver_compatible305() { two_semver_compatible(); }
#[test] fn two_semver_compatible306() { two_semver_compatible(); }
#[test] fn two_semver_compatible307() { two_semver_compatible(); }
#[test] fn two_semver_compatible308() { two_semver_compatible(); }
#[test] fn two_semver_compatible309() { two_semver_compatible(); }
#[test] fn two_semver_compatible310() { two_semver_compatible(); }
#[test] fn two_semver_compatible311() { two_semver_compatible(); }
#[test] fn two_semver_compatible312() { two_semver_compatible(); }
#[test] fn two_semver_compatible313() { two_semver_compatible(); }
#[test] fn two_semver_compatible314() { two_semver_compatible(); }
#[test] fn two_semver_compatible315() { two_semver_compatible(); }
#[test] fn two_semver_compatible316() { two_semver_compatible(); }
#[test] fn two_semver_compatible317() { two_semver_compatible(); }
#[test] fn two_semver_compatible318() { two_semver_compatible(); }
#[test] fn two_semver_compatible319() { two_semver_compatible(); }
#[test] fn two_semver_compatible320() { two_semver_compatible(); }
#[test] fn two_semver_compatible321() { two_semver_compatible(); }
#[test] fn two_semver_compatible322() { two_semver_compatible(); }
#[test] fn two_semver_compatible323() { two_semver_compatible(); }
#[test] fn two_semver_compatible324() { two_semver_compatible(); }
#[test] fn two_semver_compatible325() { two_semver_compatible(); }
#[test] fn two_semver_compatible326() { two_semver_compatible(); }
#[test] fn two_semver_compatible327() { two_semver_compatible(); }
#[test] fn two_semver_compatible328() { two_semver_compatible(); }
#[test] fn two_semver_compatible329() { two_semver_compatible(); }
#[test] fn two_semver_compatible330() { two_semver_compatible(); }
#[test] fn two_semver_compatible331() { two_semver_compatible(); }
#[test] fn two_semver_compatible332() { two_semver_compatible(); }
#[test] fn two_semver_compatible333() { two_semver_compatible(); }
#[test] fn two_semver_compatible334() { two_semver_compatible(); }
#[test] fn two_semver_compatible335() { two_semver_compatible(); }
#[test] fn two_semver_compatible336() { two_semver_compatible(); }
#[test] fn two_semver_compatible337() { two_semver_compatible(); }
#[test] fn two_semver_compatible338() { two_semver_compatible(); }
#[test] fn two_semver_compatible339() { two_semver_compatible(); }
#[test] fn two_semver_compatible340() { two_semver_compatible(); }
#[test] fn two_semver_compatible341() { two_semver_compatible(); }
#[test] fn two_semver_compatible342() { two_semver_compatible(); }
#[test] fn two_semver_compatible343() { two_semver_compatible(); }
#[test] fn two_semver_compatible344() { two_semver_compatible(); }
#[test] fn two_semver_compatible345() { two_semver_compatible(); }
#[test] fn two_semver_compatible346() { two_semver_compatible(); }
#[test] fn two_semver_compatible347() { two_semver_compatible(); }
#[test] fn two_semver_compatible348() { two_semver_compatible(); }
#[test] fn two_semver_compatible349() { two_semver_compatible(); }
#[test] fn two_semver_compatible350() { two_semver_compatible(); }
#[test] fn two_semver_compatible351() { two_semver_compatible(); }
#[test] fn two_semver_compatible352() { two_semver_compatible(); }
#[test] fn two_semver_compatible353() { two_semver_compatible(); }
#[test] fn two_semver_compatible354() { two_semver_compatible(); }
#[test] fn two_semver_compatible355() { two_semver_compatible(); }
#[test] fn two_semver_compatible356() { two_semver_compatible(); }
#[test] fn two_semver_compatible357() { two_semver_compatible(); }
#[test] fn two_semver_compatible358() { two_semver_compatible(); }
#[test] fn two_semver_compatible359() { two_semver_compatible(); }
#[test] fn two_semver_compatible360() { two_semver_compatible(); }
#[test] fn two_semver_compatible361() { two_semver_compatible(); }
#[test] fn two_semver_compatible362() { two_semver_compatible(); }
#[test] fn two_semver_compatible363() { two_semver_compatible(); }
#[test] fn two_semver_compatible364() { two_semver_compatible(); }
#[test] fn two_semver_compatible365() { two_semver_compatible(); }
#[test] fn two_semver_compatible366() { two_semver_compatible(); }
#[test] fn two_semver_compatible367() { two_semver_compatible(); }
#[test] fn two_semver_compatible368() { two_semver_compatible(); }
#[test] fn two_semver_compatible369() { two_semver_compatible(); }
#[test] fn two_semver_compatible370() { two_semver_compatible(); }
#[test] fn two_semver_compatible371() { two_semver_compatible(); }
#[test] fn two_semver_compatible372() { two_semver_compatible(); }
#[test] fn two_semver_compatible373() { two_semver_compatible(); }
#[test] fn two_semver_compatible374() { two_semver_compatible(); }
#[test] fn two_semver_compatible375() { two_semver_compatible(); }
#[test] fn two_semver_compatible376() { two_semver_compatible(); }
#[test] fn two_semver_compatible377() { two_semver_compatible(); }
#[test] fn two_semver_compatible378() { two_semver_compatible(); }
#[test] fn two_semver_compatible379() { two_semver_compatible(); }
#[test] fn two_semver_compatible380() { two_semver_compatible(); }
#[test] fn two_semver_compatible381() { two_semver_compatible(); }
#[test] fn two_semver_compatible382() { two_semver_compatible(); }
#[test] fn two_semver_compatible383() { two_semver_compatible(); }
#[test] fn two_semver_compatible384() { two_semver_compatible(); }
#[test] fn two_semver_compatible385() { two_semver_compatible(); }
#[test] fn two_semver_compatible386() { two_semver_compatible(); }
#[test] fn two_semver_compatible387() { two_semver_compatible(); }
#[test] fn two_semver_compatible388() { two_semver_compatible(); }
#[test] fn two_semver_compatible389() { two_semver_compatible(); }
#[test] fn two_semver_compatible390() { two_semver_compatible(); }
#[test] fn two_semver_compatible391() { two_semver_compatible(); }
#[test] fn two_semver_compatible392() { two_semver_compatible(); }
#[test] fn two_semver_compatible393() { two_semver_compatible(); }
#[test] fn two_semver_compatible394() { two_semver_compatible(); }
#[test] fn two_semver_compatible395() { two_semver_compatible(); }
#[test] fn two_semver_compatible396() { two_semver_compatible(); }
#[test] fn two_semver_compatible397() { two_semver_compatible(); }
#[test] fn two_semver_compatible398() { two_semver_compatible(); }
#[test] fn two_semver_compatible399() { two_semver_compatible(); }
#[test] fn two_semver_compatible400() { two_semver_compatible(); }
#[test] fn two_semver_compatible401() { two_semver_compatible(); }
#[test] fn two_semver_compatible402() { two_semver_compatible(); }
#[test] fn two_semver_compatible403() { two_semver_compatible(); }
#[test] fn two_semver_compatible404() { two_semver_compatible(); }
#[test] fn two_semver_compatible405() { two_semver_compatible(); }
#[test] fn two_semver_compatible406() { two_semver_compatible(); }
#[test] fn two_semver_compatible407() { two_semver_compatible(); }
#[test] fn two_semver_compatible408() { two_semver_compatible(); }
#[test] fn two_semver_compatible409() { two_semver_compatible(); }
#[test] fn two_semver_compatible410() { two_semver_compatible(); }
#[test] fn two_semver_compatible411() { two_semver_compatible(); }
#[test] fn two_semver_compatible412() { two_semver_compatible(); }
#[test] fn two_semver_compatible413() { two_semver_compatible(); }
#[test] fn two_semver_compatible414() { two_semver_compatible(); }
#[test] fn two_semver_compatible415() { two_semver_compatible(); }
#[test] fn two_semver_compatible416() { two_semver_compatible(); }
#[test] fn two_semver_compatible417() { two_semver_compatible(); }
#[test] fn two_semver_compatible418() { two_semver_compatible(); }
#[test] fn two_semver_compatible419() { two_semver_compatible(); }
#[test] fn two_semver_compatible420() { two_semver_compatible(); }
#[test] fn two_semver_compatible421() { two_semver_compatible(); }
#[test] fn two_semver_compatible422() { two_semver_compatible(); }
#[test] fn two_semver_compatible423() { two_semver_compatible(); }
#[test] fn two_semver_compatible424() { two_semver_compatible(); }
#[test] fn two_semver_compatible425() { two_semver_compatible(); }
#[test] fn two_semver_compatible426() { two_semver_compatible(); }
#[test] fn two_semver_compatible427() { two_semver_compatible(); }
#[test] fn two_semver_compatible428() { two_semver_compatible(); }
#[test] fn two_semver_compatible429() { two_semver_compatible(); }
#[test] fn two_semver_compatible430() { two_semver_compatible(); }
#[test] fn two_semver_compatible431() { two_semver_compatible(); }
#[test] fn two_semver_compatible432() { two_semver_compatible(); }
#[test] fn two_semver_compatible433() { two_semver_compatible(); }
#[test] fn two_semver_compatible434() { two_semver_compatible(); }
#[test] fn two_semver_compatible435() { two_semver_compatible(); }
#[test] fn two_semver_compatible436() { two_semver_compatible(); }
#[test] fn two_semver_compatible437() { two_semver_compatible(); }
#[test] fn two_semver_compatible438() { two_semver_compatible(); }
#[test] fn two_semver_compatible439() { two_semver_compatible(); }
#[test] fn two_semver_compatible440() { two_semver_compatible(); }
#[test] fn two_semver_compatible441() { two_semver_compatible(); }
#[test] fn two_semver_compatible442() { two_semver_compatible(); }
#[test] fn two_semver_compatible443() { two_semver_compatible(); }
#[test] fn two_semver_compatible444() { two_semver_compatible(); }
#[test] fn two_semver_compatible445() { two_semver_compatible(); }
#[test] fn two_semver_compatible446() { two_semver_compatible(); }
#[test] fn two_semver_compatible447() { two_semver_compatible(); }
#[test] fn two_semver_compatible448() { two_semver_compatible(); }
#[test] fn two_semver_compatible449() { two_semver_compatible(); }
#[test] fn two_semver_compatible450() { two_semver_compatible(); }
#[test] fn two_semver_compatible451() { two_semver_compatible(); }
#[test] fn two_semver_compatible452() { two_semver_compatible(); }
#[test] fn two_semver_compatible453() { two_semver_compatible(); }
#[test] fn two_semver_compatible454() { two_semver_compatible(); }
#[test] fn two_semver_compatible455() { two_semver_compatible(); }
#[test] fn two_semver_compatible456() { two_semver_compatible(); }
#[test] fn two_semver_compatible457() { two_semver_compatible(); }
#[test] fn two_semver_compatible458() { two_semver_compatible(); }
#[test] fn two_semver_compatible459() { two_semver_compatible(); }
#[test] fn two_semver_compatible460() { two_semver_compatible(); }
#[test] fn two_semver_compatible461() { two_semver_compatible(); }
#[test] fn two_semver_compatible462() { two_semver_compatible(); }
#[test] fn two_semver_compatible463() { two_semver_compatible(); }
#[test] fn two_semver_compatible464() { two_semver_compatible(); }
#[test] fn two_semver_compatible465() { two_semver_compatible(); }
#[test] fn two_semver_compatible466() { two_semver_compatible(); }
#[test] fn two_semver_compatible467() { two_semver_compatible(); }
#[test] fn two_semver_compatible468() { two_semver_compatible(); }
#[test] fn two_semver_compatible469() { two_semver_compatible(); }
#[test] fn two_semver_compatible470() { two_semver_compatible(); }
#[test] fn two_semver_compatible471() { two_semver_compatible(); }
#[test] fn two_semver_compatible472() { two_semver_compatible(); }
#[test] fn two_semver_compatible473() { two_semver_compatible(); }
#[test] fn two_semver_compatible474() { two_semver_compatible(); }
#[test] fn two_semver_compatible475() { two_semver_compatible(); }
#[test] fn two_semver_compatible476() { two_semver_compatible(); }
#[test] fn two_semver_compatible477() { two_semver_compatible(); }
#[test] fn two_semver_compatible478() { two_semver_compatible(); }
#[test] fn two_semver_compatible479() { two_semver_compatible(); }
#[test] fn two_semver_compatible480() { two_semver_compatible(); }
#[test] fn two_semver_compatible481() { two_semver_compatible(); }
#[test] fn two_semver_compatible482() { two_semver_compatible(); }
#[test] fn two_semver_compatible483() { two_semver_compatible(); }
#[test] fn two_semver_compatible484() { two_semver_compatible(); }
#[test] fn two_semver_compatible485() { two_semver_compatible(); }
#[test] fn two_semver_compatible486() { two_semver_compatible(); }
#[test] fn two_semver_compatible487() { two_semver_compatible(); }
#[test] fn two_semver_compatible488() { two_semver_compatible(); }
#[test] fn two_semver_compatible489() { two_semver_compatible(); }
#[test] fn two_semver_compatible490() { two_semver_compatible(); }
#[test] fn two_semver_compatible491() { two_semver_compatible(); }
#[test] fn two_semver_compatible492() { two_semver_compatible(); }
#[test] fn two_semver_compatible493() { two_semver_compatible(); }
#[test] fn two_semver_compatible494() { two_semver_compatible(); }
#[test] fn two_semver_compatible495() { two_semver_compatible(); }
#[test] fn two_semver_compatible496() { two_semver_compatible(); }
#[test] fn two_semver_compatible497() { two_semver_compatible(); }
#[test] fn two_semver_compatible498() { two_semver_compatible(); }
#[test] fn two_semver_compatible499() { two_semver_compatible(); }
#[test] fn two_semver_compatible500() { two_semver_compatible(); }
#[test] fn two_semver_compatible501() { two_semver_compatible(); }
#[test] fn two_semver_compatible502() { two_semver_compatible(); }
#[test] fn two_semver_compatible503() { two_semver_compatible(); }
#[test] fn two_semver_compatible504() { two_semver_compatible(); }
#[test] fn two_semver_compatible505() { two_semver_compatible(); }
#[test] fn two_semver_compatible506() { two_semver_compatible(); }
#[test] fn two_semver_compatible507() { two_semver_compatible(); }
#[test] fn two_semver_compatible508() { two_semver_compatible(); }
#[test] fn two_semver_compatible509() { two_semver_compatible(); }
#[test] fn two_semver_compatible510() { two_semver_compatible(); }
#[test] fn two_semver_compatible511() { two_semver_compatible(); }
#[test] fn two_semver_compatible512() { two_semver_compatible(); }
#[test] fn two_semver_compatible513() { two_semver_compatible(); }
#[test] fn two_semver_compatible514() { two_semver_compatible(); }
#[test] fn two_semver_compatible515() { two_semver_compatible(); }
#[test] fn two_semver_compatible516() { two_semver_compatible(); }
#[test] fn two_semver_compatible517() { two_semver_compatible(); }
#[test] fn two_semver_compatible518() { two_semver_compatible(); }
#[test] fn two_semver_compatible519() { two_semver_compatible(); }
#[test] fn two_semver_compatible520() { two_semver_compatible(); }
#[test] fn two_semver_compatible521() { two_semver_compatible(); }
#[test] fn two_semver_compatible522() { two_semver_compatible(); }
#[test] fn two_semver_compatible523() { two_semver_compatible(); }
#[test] fn two_semver_compatible524() { two_semver_compatible(); }
#[test] fn two_semver_compatible525() { two_semver_compatible(); }
#[test] fn two_semver_compatible526() { two_semver_compatible(); }
#[test] fn two_semver_compatible527() { two_semver_compatible(); }
#[test] fn two_semver_compatible528() { two_semver_compatible(); }
#[test] fn two_semver_compatible529() { two_semver_compatible(); }
#[test] fn two_semver_compatible530() { two_semver_compatible(); }
#[test] fn two_semver_compatible531() { two_semver_compatible(); }
#[test] fn two_semver_compatible532() { two_semver_compatible(); }
#[test] fn two_semver_compatible533() { two_semver_compatible(); }
#[test] fn two_semver_compatible534() { two_semver_compatible(); }
#[test] fn two_semver_compatible535() { two_semver_compatible(); }
#[test] fn two_semver_compatible536() { two_semver_compatible(); }
#[test] fn two_semver_compatible537() { two_semver_compatible(); }
#[test] fn two_semver_compatible538() { two_semver_compatible(); }
#[test] fn two_semver_compatible539() { two_semver_compatible(); }
#[test] fn two_semver_compatible540() { two_semver_compatible(); }
#[test] fn two_semver_compatible541() { two_semver_compatible(); }
#[test] fn two_semver_compatible542() { two_semver_compatible(); }
#[test] fn two_semver_compatible543() { two_semver_compatible(); }
#[test] fn two_semver_compatible544() { two_semver_compatible(); }
#[test] fn two_semver_compatible545() { two_semver_compatible(); }
#[test] fn two_semver_compatible546() { two_semver_compatible(); }
#[test] fn two_semver_compatible547() { two_semver_compatible(); }
#[test] fn two_semver_compatible548() { two_semver_compatible(); }
#[test] fn two_semver_compatible549() { two_semver_compatible(); }
#[test] fn two_semver_compatible550() { two_semver_compatible(); }
#[test] fn two_semver_compatible551() { two_semver_compatible(); }
#[test] fn two_semver_compatible552() { two_semver_compatible(); }
#[test] fn two_semver_compatible553() { two_semver_compatible(); }
#[test] fn two_semver_compatible554() { two_semver_compatible(); }
#[test] fn two_semver_compatible555() { two_semver_compatible(); }
#[test] fn two_semver_compatible556() { two_semver_compatible(); }
#[test] fn two_semver_compatible557() { two_semver_compatible(); }
#[test] fn two_semver_compatible558() { two_semver_compatible(); }
#[test] fn two_semver_compatible559() { two_semver_compatible(); }
#[test] fn two_semver_compatible560() { two_semver_compatible(); }
#[test] fn two_semver_compatible561() { two_semver_compatible(); }
#[test] fn two_semver_compatible562() { two_semver_compatible(); }
#[test] fn two_semver_compatible563() { two_semver_compatible(); }
#[test] fn two_semver_compatible564() { two_semver_compatible(); }
#[test] fn two_semver_compatible565() { two_semver_compatible(); }
#[test] fn two_semver_compatible566() { two_semver_compatible(); }
#[test] fn two_semver_compatible567() { two_semver_compatible(); }
#[test] fn two_semver_compatible568() { two_semver_compatible(); }
#[test] fn two_semver_compatible569() { two_semver_compatible(); }
#[test] fn two_semver_compatible570() { two_semver_compatible(); }
#[test] fn two_semver_compatible571() { two_semver_compatible(); }
#[test] fn two_semver_compatible572() { two_semver_compatible(); }
#[test] fn two_semver_compatible573() { two_semver_compatible(); }
#[test] fn two_semver_compatible574() { two_semver_compatible(); }
#[test] fn two_semver_compatible575() { two_semver_compatible(); }
#[test] fn two_semver_compatible576() { two_semver_compatible(); }
#[test] fn two_semver_compatible577() { two_semver_compatible(); }
#[test] fn two_semver_compatible578() { two_semver_compatible(); }
#[test] fn two_semver_compatible579() { two_semver_compatible(); }
#[test] fn two_semver_compatible580() { two_semver_compatible(); }
#[test] fn two_semver_compatible581() { two_semver_compatible(); }
#[test] fn two_semver_compatible582() { two_semver_compatible(); }
#[test] fn two_semver_compatible583() { two_semver_compatible(); }
#[test] fn two_semver_compatible584() { two_semver_compatible(); }
#[test] fn two_semver_compatible585() { two_semver_compatible(); }
#[test] fn two_semver_compatible586() { two_semver_compatible(); }
#[test] fn two_semver_compatible587() { two_semver_compatible(); }
#[test] fn two_semver_compatible588() { two_semver_compatible(); }
#[test] fn two_semver_compatible589() { two_semver_compatible(); }
#[test] fn two_semver_compatible590() { two_semver_compatible(); }
#[test] fn two_semver_compatible591() { two_semver_compatible(); }
#[test] fn two_semver_compatible592() { two_semver_compatible(); }
#[test] fn two_semver_compatible593() { two_semver_compatible(); }
#[test] fn two_semver_compatible594() { two_semver_compatible(); }
#[test] fn two_semver_compatible595() { two_semver_compatible(); }
#[test] fn two_semver_compatible596() { two_semver_compatible(); }
#[test] fn two_semver_compatible597() { two_semver_compatible(); }
#[test] fn two_semver_compatible598() { two_semver_compatible(); }
#[test] fn two_semver_compatible599() { two_semver_compatible(); }
#[test] fn two_semver_compatible600() { two_semver_compatible(); }
#[test] fn two_semver_compatible601() { two_semver_compatible(); }
#[test] fn two_semver_compatible602() { two_semver_compatible(); }
#[test] fn two_semver_compatible603() { two_semver_compatible(); }
#[test] fn two_semver_compatible604() { two_semver_compatible(); }
#[test] fn two_semver_compatible605() { two_semver_compatible(); }
#[test] fn two_semver_compatible606() { two_semver_compatible(); }
#[test] fn two_semver_compatible607() { two_semver_compatible(); }
#[test] fn two_semver_compatible608() { two_semver_compatible(); }
#[test] fn two_semver_compatible609() { two_semver_compatible(); }
#[test] fn two_semver_compatible610() { two_semver_compatible(); }
#[test] fn two_semver_compatible611() { two_semver_compatible(); }
#[test] fn two_semver_compatible612() { two_semver_compatible(); }
#[test] fn two_semver_compatible613() { two_semver_compatible(); }
#[test] fn two_semver_compatible614() { two_semver_compatible(); }
#[test] fn two_semver_compatible615() { two_semver_compatible(); }
#[test] fn two_semver_compatible616() { two_semver_compatible(); }
#[test] fn two_semver_compatible617() { two_semver_compatible(); }
#[test] fn two_semver_compatible618() { two_semver_compatible(); }
#[test] fn two_semver_compatible619() { two_semver_compatible(); }
#[test] fn two_semver_compatible620() { two_semver_compatible(); }
#[test] fn two_semver_compatible621() { two_semver_compatible(); }
#[test] fn two_semver_compatible622() { two_semver_compatible(); }
#[test] fn two_semver_compatible623() { two_semver_compatible(); }
#[test] fn two_semver_compatible624() { two_semver_compatible(); }
#[test] fn two_semver_compatible625() { two_semver_compatible(); }
#[test] fn two_semver_compatible626() { two_semver_compatible(); }
#[test] fn two_semver_compatible627() { two_semver_compatible(); }
#[test] fn two_semver_compatible628() { two_semver_compatible(); }
#[test] fn two_semver_compatible629() { two_semver_compatible(); }
#[test] fn two_semver_compatible630() { two_semver_compatible(); }
#[test] fn two_semver_compatible631() { two_semver_compatible(); }
#[test] fn two_semver_compatible632() { two_semver_compatible(); }
#[test] fn two_semver_compatible633() { two_semver_compatible(); }
#[test] fn two_semver_compatible634() { two_semver_compatible(); }
#[test] fn two_semver_compatible635() { two_semver_compatible(); }
#[test] fn two_semver_compatible636() { two_semver_compatible(); }
#[test] fn two_semver_compatible637() { two_semver_compatible(); }
#[test] fn two_semver_compatible638() { two_semver_compatible(); }
#[test] fn two_semver_compatible639() { two_semver_compatible(); }
#[test] fn two_semver_compatible640() { two_semver_compatible(); }
#[test] fn two_semver_compatible641() { two_semver_compatible(); }
#[test] fn two_semver_compatible642() { two_semver_compatible(); }
#[test] fn two_semver_compatible643() { two_semver_compatible(); }
#[test] fn two_semver_compatible644() { two_semver_compatible(); }
#[test] fn two_semver_compatible645() { two_semver_compatible(); }
#[test] fn two_semver_compatible646() { two_semver_compatible(); }
#[test] fn two_semver_compatible647() { two_semver_compatible(); }
#[test] fn two_semver_compatible648() { two_semver_compatible(); }
#[test] fn two_semver_compatible649() { two_semver_compatible(); }
#[test] fn two_semver_compatible650() { two_semver_compatible(); }
#[test] fn two_semver_compatible651() { two_semver_compatible(); }
#[test] fn two_semver_compatible652() { two_semver_compatible(); }
#[test] fn two_semver_compatible653() { two_semver_compatible(); }
#[test] fn two_semver_compatible654() { two_semver_compatible(); }
#[test] fn two_semver_compatible655() { two_semver_compatible(); }
#[test] fn two_semver_compatible656() { two_semver_compatible(); }
#[test] fn two_semver_compatible657() { two_semver_compatible(); }
#[test] fn two_semver_compatible658() { two_semver_compatible(); }
#[test] fn two_semver_compatible659() { two_semver_compatible(); }
#[test] fn two_semver_compatible660() { two_semver_compatible(); }
#[test] fn two_semver_compatible661() { two_semver_compatible(); }
#[test] fn two_semver_compatible662() { two_semver_compatible(); }
#[test] fn two_semver_compatible663() { two_semver_compatible(); }
#[test] fn two_semver_compatible664() { two_semver_compatible(); }
#[test] fn two_semver_compatible665() { two_semver_compatible(); }
#[test] fn two_semver_compatible666() { two_semver_compatible(); }
#[test] fn two_semver_compatible667() { two_semver_compatible(); }
#[test] fn two_semver_compatible668() { two_semver_compatible(); }
#[test] fn two_semver_compatible669() { two_semver_compatible(); }
#[test] fn two_semver_compatible670() { two_semver_compatible(); }
#[test] fn two_semver_compatible671() { two_semver_compatible(); }
#[test] fn two_semver_compatible672() { two_semver_compatible(); }
#[test] fn two_semver_compatible673() { two_semver_compatible(); }
#[test] fn two_semver_compatible674() { two_semver_compatible(); }
#[test] fn two_semver_compatible675() { two_semver_compatible(); }
#[test] fn two_semver_compatible676() { two_semver_compatible(); }
#[test] fn two_semver_compatible677() { two_semver_compatible(); }
#[test] fn two_semver_compatible678() { two_semver_compatible(); }
#[test] fn two_semver_compatible679() { two_semver_compatible(); }
#[test] fn two_semver_compatible680() { two_semver_compatible(); }
#[test] fn two_semver_compatible681() { two_semver_compatible(); }
#[test] fn two_semver_compatible682() { two_semver_compatible(); }
#[test] fn two_semver_compatible683() { two_semver_compatible(); }
#[test] fn two_semver_compatible684() { two_semver_compatible(); }
#[test] fn two_semver_compatible685() { two_semver_compatible(); }
#[test] fn two_semver_compatible686() { two_semver_compatible(); }
#[test] fn two_semver_compatible687() { two_semver_compatible(); }
#[test] fn two_semver_compatible688() { two_semver_compatible(); }
#[test] fn two_semver_compatible689() { two_semver_compatible(); }
#[test] fn two_semver_compatible690() { two_semver_compatible(); }
#[test] fn two_semver_compatible691() { two_semver_compatible(); }
#[test] fn two_semver_compatible692() { two_semver_compatible(); }
#[test] fn two_semver_compatible693() { two_semver_compatible(); }
#[test] fn two_semver_compatible694() { two_semver_compatible(); }
#[test] fn two_semver_compatible695() { two_semver_compatible(); }
#[test] fn two_semver_compatible696() { two_semver_compatible(); }
#[test] fn two_semver_compatible697() { two_semver_compatible(); }
#[test] fn two_semver_compatible698() { two_semver_compatible(); }
#[test] fn two_semver_compatible699() { two_semver_compatible(); }
#[test] fn two_semver_compatible700() { two_semver_compatible(); }
#[test] fn two_semver_compatible701() { two_semver_compatible(); }
#[test] fn two_semver_compatible702() { two_semver_compatible(); }
#[test] fn two_semver_compatible703() { two_semver_compatible(); }
#[test] fn two_semver_compatible704() { two_semver_compatible(); }
#[test] fn two_semver_compatible705() { two_semver_compatible(); }
#[test] fn two_semver_compatible706() { two_semver_compatible(); }
#[test] fn two_semver_compatible707() { two_semver_compatible(); }
#[test] fn two_semver_compatible708() { two_semver_compatible(); }
#[test] fn two_semver_compatible709() { two_semver_compatible(); }
#[test] fn two_semver_compatible710() { two_semver_compatible(); }
#[test] fn two_semver_compatible711() { two_semver_compatible(); }
#[test] fn two_semver_compatible712() { two_semver_compatible(); }
#[test] fn two_semver_compatible713() { two_semver_compatible(); }
#[test] fn two_semver_compatible714() { two_semver_compatible(); }
#[test] fn two_semver_compatible715() { two_semver_compatible(); }
#[test] fn two_semver_compatible716() { two_semver_compatible(); }
#[test] fn two_semver_compatible717() { two_semver_compatible(); }
#[test] fn two_semver_compatible718() { two_semver_compatible(); }
#[test] fn two_semver_compatible719() { two_semver_compatible(); }
#[test] fn two_semver_compatible720() { two_semver_compatible(); }
#[test] fn two_semver_compatible721() { two_semver_compatible(); }
#[test] fn two_semver_compatible722() { two_semver_compatible(); }
#[test] fn two_semver_compatible723() { two_semver_compatible(); }
#[test] fn two_semver_compatible724() { two_semver_compatible(); }
#[test] fn two_semver_compatible725() { two_semver_compatible(); }
#[test] fn two_semver_compatible726() { two_semver_compatible(); }
#[test] fn two_semver_compatible727() { two_semver_compatible(); }
#[test] fn two_semver_compatible728() { two_semver_compatible(); }
#[test] fn two_semver_compatible729() { two_semver_compatible(); }
#[test] fn two_semver_compatible730() { two_semver_compatible(); }
#[test] fn two_semver_compatible731() { two_semver_compatible(); }
#[test] fn two_semver_compatible732() { two_semver_compatible(); }
#[test] fn two_semver_compatible733() { two_semver_compatible(); }
#[test] fn two_semver_compatible734() { two_semver_compatible(); }
#[test] fn two_semver_compatible735() { two_semver_compatible(); }
#[test] fn two_semver_compatible736() { two_semver_compatible(); }
#[test] fn two_semver_compatible737() { two_semver_compatible(); }
#[test] fn two_semver_compatible738() { two_semver_compatible(); }
#[test] fn two_semver_compatible739() { two_semver_compatible(); }
#[test] fn two_semver_compatible740() { two_semver_compatible(); }
#[test] fn two_semver_compatible741() { two_semver_compatible(); }
#[test] fn two_semver_compatible742() { two_semver_compatible(); }
#[test] fn two_semver_compatible743() { two_semver_compatible(); }
#[test] fn two_semver_compatible744() { two_semver_compatible(); }
#[test] fn two_semver_compatible745() { two_semver_compatible(); }
#[test] fn two_semver_compatible746() { two_semver_compatible(); }
#[test] fn two_semver_compatible747() { two_semver_compatible(); }
#[test] fn two_semver_compatible748() { two_semver_compatible(); }
#[test] fn two_semver_compatible749() { two_semver_compatible(); }
#[test] fn two_semver_compatible750() { two_semver_compatible(); }
#[test] fn two_semver_compatible751() { two_semver_compatible(); }
#[test] fn two_semver_compatible752() { two_semver_compatible(); }
#[test] fn two_semver_compatible753() { two_semver_compatible(); }
#[test] fn two_semver_compatible754() { two_semver_compatible(); }
#[test] fn two_semver_compatible755() { two_semver_compatible(); }
#[test] fn two_semver_compatible756() { two_semver_compatible(); }
#[test] fn two_semver_compatible757() { two_semver_compatible(); }
#[test] fn two_semver_compatible758() { two_semver_compatible(); }
#[test] fn two_semver_compatible759() { two_semver_compatible(); }
#[test] fn two_semver_compatible760() { two_semver_compatible(); }
#[test] fn two_semver_compatible761() { two_semver_compatible(); }
#[test] fn two_semver_compatible762() { two_semver_compatible(); }
#[test] fn two_semver_compatible763() { two_semver_compatible(); }
#[test] fn two_semver_compatible764() { two_semver_compatible(); }
#[test] fn two_semver_compatible765() { two_semver_compatible(); }
#[test] fn two_semver_compatible766() { two_semver_compatible(); }
#[test] fn two_semver_compatible767() { two_semver_compatible(); }
#[test] fn two_semver_compatible768() { two_semver_compatible(); }
#[test] fn two_semver_compatible769() { two_semver_compatible(); }
#[test] fn two_semver_compatible770() { two_semver_compatible(); }
#[test] fn two_semver_compatible771() { two_semver_compatible(); }
#[test] fn two_semver_compatible772() { two_semver_compatible(); }
#[test] fn two_semver_compatible773() { two_semver_compatible(); }
#[test] fn two_semver_compatible774() { two_semver_compatible(); }
#[test] fn two_semver_compatible775() { two_semver_compatible(); }
#[test] fn two_semver_compatible776() { two_semver_compatible(); }
#[test] fn two_semver_compatible777() { two_semver_compatible(); }
#[test] fn two_semver_compatible778() { two_semver_compatible(); }
#[test] fn two_semver_compatible779() { two_semver_compatible(); }
#[test] fn two_semver_compatible780() { two_semver_compatible(); }
#[test] fn two_semver_compatible781() { two_semver_compatible(); }
#[test] fn two_semver_compatible782() { two_semver_compatible(); }
#[test] fn two_semver_compatible783() { two_semver_compatible(); }
#[test] fn two_semver_compatible784() { two_semver_compatible(); }
#[test] fn two_semver_compatible785() { two_semver_compatible(); }
#[test] fn two_semver_compatible786() { two_semver_compatible(); }
#[test] fn two_semver_compatible787() { two_semver_compatible(); }
#[test] fn two_semver_compatible788() { two_semver_compatible(); }
#[test] fn two_semver_compatible789() { two_semver_compatible(); }
#[test] fn two_semver_compatible790() { two_semver_compatible(); }
#[test] fn two_semver_compatible791() { two_semver_compatible(); }
#[test] fn two_semver_compatible792() { two_semver_compatible(); }
#[test] fn two_semver_compatible793() { two_semver_compatible(); }
#[test] fn two_semver_compatible794() { two_semver_compatible(); }
#[test] fn two_semver_compatible795() { two_semver_compatible(); }
#[test] fn two_semver_compatible796() { two_semver_compatible(); }
#[test] fn two_semver_compatible797() { two_semver_compatible(); }
#[test] fn two_semver_compatible798() { two_semver_compatible(); }
#[test] fn two_semver_compatible799() { two_semver_compatible(); }
#[test] fn two_semver_compatible800() { two_semver_compatible(); }
#[test] fn two_semver_compatible801() { two_semver_compatible(); }
#[test] fn two_semver_compatible802() { two_semver_compatible(); }
#[test] fn two_semver_compatible803() { two_semver_compatible(); }
#[test] fn two_semver_compatible804() { two_semver_compatible(); }
#[test] fn two_semver_compatible805() { two_semver_compatible(); }
#[test] fn two_semver_compatible806() { two_semver_compatible(); }
#[test] fn two_semver_compatible807() { two_semver_compatible(); }
#[test] fn two_semver_compatible808() { two_semver_compatible(); }
#[test] fn two_semver_compatible809() { two_semver_compatible(); }
#[test] fn two_semver_compatible810() { two_semver_compatible(); }
#[test] fn two_semver_compatible811() { two_semver_compatible(); }
#[test] fn two_semver_compatible812() { two_semver_compatible(); }
#[test] fn two_semver_compatible813() { two_semver_compatible(); }
#[test] fn two_semver_compatible814() { two_semver_compatible(); }
#[test] fn two_semver_compatible815() { two_semver_compatible(); }
#[test] fn two_semver_compatible816() { two_semver_compatible(); }
#[test] fn two_semver_compatible817() { two_semver_compatible(); }
#[test] fn two_semver_compatible818() { two_semver_compatible(); }
#[test] fn two_semver_compatible819() { two_semver_compatible(); }
#[test] fn two_semver_compatible820() { two_semver_compatible(); }
#[test] fn two_semver_compatible821() { two_semver_compatible(); }
#[test] fn two_semver_compatible822() { two_semver_compatible(); }
#[test] fn two_semver_compatible823() { two_semver_compatible(); }
#[test] fn two_semver_compatible824() { two_semver_compatible(); }
#[test] fn two_semver_compatible825() { two_semver_compatible(); }
#[test] fn two_semver_compatible826() { two_semver_compatible(); }
#[test] fn two_semver_compatible827() { two_semver_compatible(); }
#[test] fn two_semver_compatible828() { two_semver_compatible(); }
#[test] fn two_semver_compatible829() { two_semver_compatible(); }
#[test] fn two_semver_compatible830() { two_semver_compatible(); }
#[test] fn two_semver_compatible831() { two_semver_compatible(); }
#[test] fn two_semver_compatible832() { two_semver_compatible(); }
#[test] fn two_semver_compatible833() { two_semver_compatible(); }
#[test] fn two_semver_compatible834() { two_semver_compatible(); }
#[test] fn two_semver_compatible835() { two_semver_compatible(); }
#[test] fn two_semver_compatible836() { two_semver_compatible(); }
#[test] fn two_semver_compatible837() { two_semver_compatible(); }
#[test] fn two_semver_compatible838() { two_semver_compatible(); }
#[test] fn two_semver_compatible839() { two_semver_compatible(); }
#[test] fn two_semver_compatible840() { two_semver_compatible(); }
#[test] fn two_semver_compatible841() { two_semver_compatible(); }
#[test] fn two_semver_compatible842() { two_semver_compatible(); }
#[test] fn two_semver_compatible843() { two_semver_compatible(); }
#[test] fn two_semver_compatible844() { two_semver_compatible(); }
#[test] fn two_semver_compatible845() { two_semver_compatible(); }
#[test] fn two_semver_compatible846() { two_semver_compatible(); }
#[test] fn two_semver_compatible847() { two_semver_compatible(); }
#[test] fn two_semver_compatible848() { two_semver_compatible(); }
#[test] fn two_semver_compatible849() { two_semver_compatible(); }
#[test] fn two_semver_compatible850() { two_semver_compatible(); }
#[test] fn two_semver_compatible851() { two_semver_compatible(); }
#[test] fn two_semver_compatible852() { two_semver_compatible(); }
#[test] fn two_semver_compatible853() { two_semver_compatible(); }
#[test] fn two_semver_compatible854() { two_semver_compatible(); }
#[test] fn two_semver_compatible855() { two_semver_compatible(); }
#[test] fn two_semver_compatible856() { two_semver_compatible(); }
#[test] fn two_semver_compatible857() { two_semver_compatible(); }
#[test] fn two_semver_compatible858() { two_semver_compatible(); }
#[test] fn two_semver_compatible859() { two_semver_compatible(); }
#[test] fn two_semver_compatible860() { two_semver_compatible(); }
#[test] fn two_semver_compatible861() { two_semver_compatible(); }
#[test] fn two_semver_compatible862() { two_semver_compatible(); }
#[test] fn two_semver_compatible863() { two_semver_compatible(); }
#[test] fn two_semver_compatible864() { two_semver_compatible(); }
#[test] fn two_semver_compatible865() { two_semver_compatible(); }
#[test] fn two_semver_compatible866() { two_semver_compatible(); }
#[test] fn two_semver_compatible867() { two_semver_compatible(); }
#[test] fn two_semver_compatible868() { two_semver_compatible(); }
#[test] fn two_semver_compatible869() { two_semver_compatible(); }
#[test] fn two_semver_compatible870() { two_semver_compatible(); }
#[test] fn two_semver_compatible871() { two_semver_compatible(); }
#[test] fn two_semver_compatible872() { two_semver_compatible(); }
#[test] fn two_semver_compatible873() { two_semver_compatible(); }
#[test] fn two_semver_compatible874() { two_semver_compatible(); }
#[test] fn two_semver_compatible875() { two_semver_compatible(); }
#[test] fn two_semver_compatible876() { two_semver_compatible(); }
#[test] fn two_semver_compatible877() { two_semver_compatible(); }
#[test] fn two_semver_compatible878() { two_semver_compatible(); }
#[test] fn two_semver_compatible879() { two_semver_compatible(); }
#[test] fn two_semver_compatible880() { two_semver_compatible(); }
#[test] fn two_semver_compatible881() { two_semver_compatible(); }
#[test] fn two_semver_compatible882() { two_semver_compatible(); }
#[test] fn two_semver_compatible883() { two_semver_compatible(); }
#[test] fn two_semver_compatible884() { two_semver_compatible(); }
#[test] fn two_semver_compatible885() { two_semver_compatible(); }
#[test] fn two_semver_compatible886() { two_semver_compatible(); }
#[test] fn two_semver_compatible887() { two_semver_compatible(); }
#[test] fn two_semver_compatible888() { two_semver_compatible(); }
#[test] fn two_semver_compatible889() { two_semver_compatible(); }
#[test] fn two_semver_compatible890() { two_semver_compatible(); }
#[test] fn two_semver_compatible891() { two_semver_compatible(); }
#[test] fn two_semver_compatible892() { two_semver_compatible(); }
#[test] fn two_semver_compatible893() { two_semver_compatible(); }
#[test] fn two_semver_compatible894() { two_semver_compatible(); }
#[test] fn two_semver_compatible895() { two_semver_compatible(); }
#[test] fn two_semver_compatible896() { two_semver_compatible(); }
#[test] fn two_semver_compatible897() { two_semver_compatible(); }
#[test] fn two_semver_compatible898() { two_semver_compatible(); }
#[test] fn two_semver_compatible899() { two_semver_compatible(); }
#[test] fn two_semver_compatible900() { two_semver_compatible(); }
#[test] fn two_semver_compatible901() { two_semver_compatible(); }
#[test] fn two_semver_compatible902() { two_semver_compatible(); }
#[test] fn two_semver_compatible903() { two_semver_compatible(); }
#[test] fn two_semver_compatible904() { two_semver_compatible(); }
#[test] fn two_semver_compatible905() { two_semver_compatible(); }
#[test] fn two_semver_compatible906() { two_semver_compatible(); }
#[test] fn two_semver_compatible907() { two_semver_compatible(); }
#[test] fn two_semver_compatible908() { two_semver_compatible(); }
#[test] fn two_semver_compatible909() { two_semver_compatible(); }
#[test] fn two_semver_compatible910() { two_semver_compatible(); }
#[test] fn two_semver_compatible911() { two_semver_compatible(); }
#[test] fn two_semver_compatible912() { two_semver_compatible(); }
#[test] fn two_semver_compatible913() { two_semver_compatible(); }
#[test] fn two_semver_compatible914() { two_semver_compatible(); }
#[test] fn two_semver_compatible915() { two_semver_compatible(); }
#[test] fn two_semver_compatible916() { two_semver_compatible(); }
#[test] fn two_semver_compatible917() { two_semver_compatible(); }
#[test] fn two_semver_compatible918() { two_semver_compatible(); }
#[test] fn two_semver_compatible919() { two_semver_compatible(); }
#[test] fn two_semver_compatible920() { two_semver_compatible(); }
#[test] fn two_semver_compatible921() { two_semver_compatible(); }
#[test] fn two_semver_compatible922() { two_semver_compatible(); }
#[test] fn two_semver_compatible923() { two_semver_compatible(); }
#[test] fn two_semver_compatible924() { two_semver_compatible(); }
#[test] fn two_semver_compatible925() { two_semver_compatible(); }
#[test] fn two_semver_compatible926() { two_semver_compatible(); }
#[test] fn two_semver_compatible927() { two_semver_compatible(); }
#[test] fn two_semver_compatible928() { two_semver_compatible(); }
#[test] fn two_semver_compatible929() { two_semver_compatible(); }
#[test] fn two_semver_compatible930() { two_semver_compatible(); }
#[test] fn two_semver_compatible931() { two_semver_compatible(); }
#[test] fn two_semver_compatible932() { two_semver_compatible(); }
#[test] fn two_semver_compatible933() { two_semver_compatible(); }
#[test] fn two_semver_compatible934() { two_semver_compatible(); }
#[test] fn two_semver_compatible935() { two_semver_compatible(); }
#[test] fn two_semver_compatible936() { two_semver_compatible(); }
#[test] fn two_semver_compatible937() { two_semver_compatible(); }
#[test] fn two_semver_compatible938() { two_semver_compatible(); }
#[test] fn two_semver_compatible939() { two_semver_compatible(); }
#[test] fn two_semver_compatible940() { two_semver_compatible(); }
#[test] fn two_semver_compatible941() { two_semver_compatible(); }
#[test] fn two_semver_compatible942() { two_semver_compatible(); }
#[test] fn two_semver_compatible943() { two_semver_compatible(); }
#[test] fn two_semver_compatible944() { two_semver_compatible(); }
#[test] fn two_semver_compatible945() { two_semver_compatible(); }
#[test] fn two_semver_compatible946() { two_semver_compatible(); }
#[test] fn two_semver_compatible947() { two_semver_compatible(); }
#[test] fn two_semver_compatible948() { two_semver_compatible(); }
#[test] fn two_semver_compatible949() { two_semver_compatible(); }
#[test] fn two_semver_compatible950() { two_semver_compatible(); }
#[test] fn two_semver_compatible951() { two_semver_compatible(); }
#[test] fn two_semver_compatible952() { two_semver_compatible(); }
#[test] fn two_semver_compatible953() { two_semver_compatible(); }
#[test] fn two_semver_compatible954() { two_semver_compatible(); }
#[test] fn two_semver_compatible955() { two_semver_compatible(); }
#[test] fn two_semver_compatible956() { two_semver_compatible(); }
#[test] fn two_semver_compatible957() { two_semver_compatible(); }
#[test] fn two_semver_compatible958() { two_semver_compatible(); }
#[test] fn two_semver_compatible959() { two_semver_compatible(); }
#[test] fn two_semver_compatible960() { two_semver_compatible(); }
#[test] fn two_semver_compatible961() { two_semver_compatible(); }
#[test] fn two_semver_compatible962() { two_semver_compatible(); }
#[test] fn two_semver_compatible963() { two_semver_compatible(); }
#[test] fn two_semver_compatible964() { two_semver_compatible(); }
#[test] fn two_semver_compatible965() { two_semver_compatible(); }
#[test] fn two_semver_compatible966() { two_semver_compatible(); }
#[test] fn two_semver_compatible967() { two_semver_compatible(); }
#[test] fn two_semver_compatible968() { two_semver_compatible(); }
#[test] fn two_semver_compatible969() { two_semver_compatible(); }
#[test] fn two_semver_compatible970() { two_semver_compatible(); }
#[test] fn two_semver_compatible971() { two_semver_compatible(); }
#[test] fn two_semver_compatible972() { two_semver_compatible(); }
#[test] fn two_semver_compatible973() { two_semver_compatible(); }
#[test] fn two_semver_compatible974() { two_semver_compatible(); }
#[test] fn two_semver_compatible975() { two_semver_compatible(); }
#[test] fn two_semver_compatible976() { two_semver_compatible(); }
#[test] fn two_semver_compatible977() { two_semver_compatible(); }
#[test] fn two_semver_compatible978() { two_semver_compatible(); }
#[test] fn two_semver_compatible979() { two_semver_compatible(); }
#[test] fn two_semver_compatible980() { two_semver_compatible(); }
#[test] fn two_semver_compatible981() { two_semver_compatible(); }
#[test] fn two_semver_compatible982() { two_semver_compatible(); }
#[test] fn two_semver_compatible983() { two_semver_compatible(); }
#[test] fn two_semver_compatible984() { two_semver_compatible(); }
#[test] fn two_semver_compatible985() { two_semver_compatible(); }
#[test] fn two_semver_compatible986() { two_semver_compatible(); }
#[test] fn two_semver_compatible987() { two_semver_compatible(); }
#[test] fn two_semver_compatible988() { two_semver_compatible(); }
#[test] fn two_semver_compatible989() { two_semver_compatible(); }
#[test] fn two_semver_compatible990() { two_semver_compatible(); }
#[test] fn two_semver_compatible991() { two_semver_compatible(); }
#[test] fn two_semver_compatible992() { two_semver_compatible(); }
#[test] fn two_semver_compatible993() { two_semver_compatible(); }
#[test] fn two_semver_compatible994() { two_semver_compatible(); }
#[test] fn two_semver_compatible995() { two_semver_compatible(); }
#[test] fn two_semver_compatible996() { two_semver_compatible(); }
#[test] fn two_semver_compatible997() { two_semver_compatible(); }
#[test] fn two_semver_compatible998() { two_semver_compatible(); }
#[test] fn two_semver_compatible999() { two_semver_compatible(); }

#[cargo_test]
fn two_semver_compatible() {
    let bar = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("src/lib.rs", "")
        .build();

    cargo_test_support::registry::init();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    [dependencies]
                    bar = "0.1"
                    [patch.crates-io]
                    bar = {{ path = "bar" }}
                    bar2 = {{ git = '{}', package = 'bar' }}
                "#,
                bar.url(),
            ),
        )
        .file("src/lib.rs", "pub fn foo() { bar::foo() }")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.2"
            "#,
        )
        .file("bar/src/lib.rs", "pub fn foo() {}")
        .build();

    // assert the build succeeds and doesn't panic anywhere, and then afterwards
    // assert that the build succeeds again without updating anything or
    // building anything else.
    p.cargo("check").run();
    p.cargo("check")
        .with_stderr(
            "\
warning: Patch `bar v0.1.1 [..]` was not used in the crate graph.
Perhaps you misspelled the source URL being patched.
Possible URLs for `[patch.<URL>]`:
    [CWD]/bar
[FINISHED] [..]",
        )
        .run();
}

#[cargo_test]
fn multipatch_select_big() {
    let bar = git::repo(&paths::root().join("override"))
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "")
        .build();

    cargo_test_support::registry::init();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    [dependencies]
                    bar = "*"
                    [patch.crates-io]
                    bar = {{ path = "bar" }}
                    bar2 = {{ git = '{}', package = 'bar' }}
                "#,
                bar.url(),
            ),
        )
        .file("src/lib.rs", "pub fn foo() { bar::foo() }")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.2.0"
            "#,
        )
        .file("bar/src/lib.rs", "pub fn foo() {}")
        .build();

    // assert the build succeeds, which is only possible if 0.2.0 is selected
    // since 0.1.0 is missing the function we need. Afterwards assert that the
    // build succeeds again without updating anything or building anything else.
    p.cargo("check").run();
    p.cargo("check")
        .with_stderr(
            "\
warning: Patch `bar v0.1.0 [..]` was not used in the crate graph.
Perhaps you misspelled the source URL being patched.
Possible URLs for `[patch.<URL>]`:
    [CWD]/bar
[FINISHED] [..]",
        )
        .run();
}

#[cargo_test]
fn canonicalize_a_bunch() {
    let base = git::repo(&paths::root().join("base"))
        .file("Cargo.toml", &basic_manifest("base", "0.1.0"))
        .file("src/lib.rs", "")
        .build();

    let intermediate = git::repo(&paths::root().join("intermediate"))
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "intermediate"
                    version = "0.1.0"

                    [dependencies]
                    # Note the lack of trailing slash
                    base = {{ git = '{}' }}
                "#,
                base.url(),
            ),
        )
        .file("src/lib.rs", "pub fn f() { base::f() }")
        .build();

    let newbase = git::repo(&paths::root().join("newbase"))
        .file("Cargo.toml", &basic_manifest("base", "0.1.0"))
        .file("src/lib.rs", "pub fn f() {}")
        .build();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"

                    [dependencies]
                    # Note the trailing slashes
                    base = {{ git = '{base}/' }}
                    intermediate = {{ git = '{intermediate}/' }}

                    [patch.'{base}'] # Note the lack of trailing slash
                    base = {{ git = '{newbase}' }}
                "#,
                base = base.url(),
                intermediate = intermediate.url(),
                newbase = newbase.url(),
            ),
        )
        .file("src/lib.rs", "pub fn a() { base::f(); intermediate::f() }")
        .build();

    // Once to make sure it actually works
    p.cargo("check").run();

    // Then a few more times for good measure to ensure no weird warnings about
    // `[patch]` are printed.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
}

#[cargo_test]
fn update_unused_new_version() {
    // If there is an unused patch entry, and then you update the patch,
    // make sure `cargo update` will be able to fix the lock file.
    Package::new("bar", "0.1.5").publish();

    // Start with a lock file to 0.1.5, and an "unused" patch because the
    // version is too old.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                bar = "0.1.5"

                [patch.crates-io]
                bar = { path = "../bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // Patch is too old.
    let bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.4"))
        .file("src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr_contains("[WARNING] Patch `bar v0.1.4 [..] was not used in the crate graph.")
        .run();
    // unused patch should be in the lock file
    let lock = p.read_lockfile();
    let toml: toml::Table = toml::from_str(&lock).unwrap();
    assert_eq!(toml["patch"]["unused"].as_array().unwrap().len(), 1);
    assert_eq!(toml["patch"]["unused"][0]["name"].as_str(), Some("bar"));
    assert_eq!(
        toml["patch"]["unused"][0]["version"].as_str(),
        Some("0.1.4")
    );

    // Oh, OK, let's update to the latest version.
    bar.change_file("Cargo.toml", &basic_manifest("bar", "0.1.6"));

    // Create a backup so we can test it with different options.
    fs::copy(p.root().join("Cargo.lock"), p.root().join("Cargo.lock.bak")).unwrap();

    // Try to build again, this should automatically update Cargo.lock.
    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.6 ([..]/bar)
[CHECKING] foo v0.0.1 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();
    // This should not update any registry.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();
    assert!(!p.read_lockfile().contains("unused"));

    // Restore the lock file, and see if `update` will work, too.
    fs::copy(p.root().join("Cargo.lock.bak"), p.root().join("Cargo.lock")).unwrap();

    // Try `update <pkg>`.
    p.cargo("update bar")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[ADDING] bar v0.1.6 ([..]/bar)
[REMOVING] bar v0.1.5
",
        )
        .run();

    // Try with bare `cargo update`.
    fs::copy(p.root().join("Cargo.lock.bak"), p.root().join("Cargo.lock")).unwrap();
    p.cargo("update")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[ADDING] bar v0.1.6 ([..]/bar)
[REMOVING] bar v0.1.5
",
        )
        .run();
}

#[cargo_test]
fn too_many_matches() {
    // The patch locations has multiple versions that match.
    registry::alt_init();
    Package::new("bar", "0.1.0").publish();
    Package::new("bar", "0.1.0").alternative(true).publish();
    Package::new("bar", "0.1.1").alternative(true).publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "0.1"

                [patch.crates-io]
                bar = { version = "0.1", registry = "alternative" }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // Picks 0.1.1, the most recent version.
    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] `alternative` index
[ERROR] failed to resolve patches for `https://github.com/rust-lang/crates.io-index`

Caused by:
  patch for `bar` in `https://github.com/rust-lang/crates.io-index` failed to resolve

Caused by:
  patch for `bar` in `registry `alternative`` resolved to more than one candidate
  Found versions: 0.1.0, 0.1.1
  Update the patch definition to select only one package.
  For example, add an `=` version requirement to the patch definition, such as `version = \"=0.1.1\"`.
",
        )
        .run();
}

#[cargo_test]
fn no_matches() {
    // A patch to a location that does not contain the named package.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                 [package]
                 name = "foo"
                 version = "0.1.0"

                 [dependencies]
                 bar = "0.1"

                 [patch.crates-io]
                 bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("abc", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
error: failed to resolve patches for `https://github.com/rust-lang/crates.io-index`

Caused by:
  patch for `bar` in `https://github.com/rust-lang/crates.io-index` failed to resolve

Caused by:
  The patch location `[..]/foo/bar` does not appear to contain any packages matching the name `bar`.
",
        )
        .run();
}

#[cargo_test]
fn mismatched_version() {
    // A patch to a location that has an old version.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                 [package]
                 name = "foo"
                 version = "0.1.0"

                 [dependencies]
                 bar = "0.1.1"

                 [patch.crates-io]
                 bar = { path = "bar", version = "0.1.1" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to resolve patches for `https://github.com/rust-lang/crates.io-index`

Caused by:
  patch for `bar` in `https://github.com/rust-lang/crates.io-index` failed to resolve

Caused by:
  The patch location `[..]/foo/bar` contains a `bar` package with version `0.1.0`, \
  but the patch definition requires `^0.1.1`.
  Check that the version in the patch location is what you expect, \
  and update the patch definition to match.
",
        )
        .run();
}

#[cargo_test]
fn patch_walks_backwards() {
    // Starting with a locked patch, change the patch so it points to an older version.
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = "0.1"

            [patch.crates-io]
            bar = {path="bar"}
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();

    // Somehow the user changes the version backwards.
    p.change_file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"));

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.0 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn patch_walks_backwards_restricted() {
    // This is the same as `patch_walks_backwards`, but the patch contains a
    // `version` qualifier. This is unusual, just checking a strange edge case.
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = "0.1"

            [patch.crates-io]
            bar = {path="bar", version="0.1.1"}
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();

    // Somehow the user changes the version backwards.
    p.change_file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"));

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
error: failed to resolve patches for `https://github.com/rust-lang/crates.io-index`

Caused by:
  patch for `bar` in `https://github.com/rust-lang/crates.io-index` failed to resolve

Caused by:
  The patch location `[..]/foo/bar` contains a `bar` package with version `0.1.0`, but the patch definition requires `^0.1.1`.
  Check that the version in the patch location is what you expect, and update the patch definition to match.
",
        )
        .run();
}

#[cargo_test]
fn patched_dep_new_version() {
    // What happens when a patch is locked, and then one of the patched
    // dependencies needs to be updated. In this case, the baz requirement
    // gets updated from 0.1.0 to 0.1.1.
    Package::new("bar", "0.1.0").dep("baz", "0.1.0").publish();
    Package::new("baz", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = "0.1"

            [patch.crates-io]
            bar = {path="bar"}
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.1.0"

            [dependencies]
            baz = "0.1"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    // Lock everything.
    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] baz v0.1.0 [..]
[CHECKING] baz v0.1.0
[CHECKING] bar v0.1.0 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();

    Package::new("baz", "0.1.1").publish();

    // Just the presence of the new version should not have changed anything.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    // Modify the patch so it requires the new version.
    p.change_file(
        "bar/Cargo.toml",
        r#"
            [package]
            name = "bar"
            version = "0.1.0"

            [dependencies]
            baz = "0.1.1"
        "#,
    );

    // Should unlock and update cleanly.
    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] baz v0.1.1 (registry `dummy-registry`)
[CHECKING] baz v0.1.1
[CHECKING] bar v0.1.0 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn patch_update_doesnt_update_other_sources() {
    // Very extreme edge case, make sure a patch update doesn't update other
    // sources.
    registry::alt_init();
    Package::new("bar", "0.1.0").publish();
    Package::new("bar", "0.1.0").alternative(true).publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = "0.1"
            bar_alt = { version = "0.1", registry = "alternative", package = "bar"  }

            [patch.crates-io]
            bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr_unordered(
            "\
[UPDATING] `dummy-registry` index
[UPDATING] `alternative` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 (registry `alternative`)
[CHECKING] bar v0.1.0 (registry `alternative`)
[CHECKING] bar v0.1.0 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();

    // Publish new versions in both sources.
    Package::new("bar", "0.1.1").publish();
    Package::new("bar", "0.1.1").alternative(true).publish();

    // Since it is locked, nothing should change.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    // Require new version on crates.io.
    p.change_file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"));

    // This should not update bar_alt.
    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[CHECKING] bar v0.1.1 ([..]/foo/bar)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn can_update_with_alt_reg() {
    // A patch to an alt reg can update.
    registry::alt_init();
    Package::new("bar", "0.1.0").publish();
    Package::new("bar", "0.1.0").alternative(true).publish();
    Package::new("bar", "0.1.1").alternative(true).publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "0.1"

                [patch.crates-io]
                bar = { version = "=0.1.1", registry = "alternative" }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `alternative` index
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.1 (registry `alternative`)
[CHECKING] bar v0.1.1 (registry `alternative`)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();

    Package::new("bar", "0.1.2").alternative(true).publish();

    // Should remain locked.
    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    // This does nothing, due to `=` requirement.
    p.cargo("update bar")
        .with_stderr(
            "\
[UPDATING] `alternative` index
[UPDATING] `dummy-registry` index
",
        )
        .run();

    // Bump to 0.1.2.
    p.change_file(
        "Cargo.toml",
        r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = "0.1"

            [patch.crates-io]
            bar = { version = "=0.1.2", registry = "alternative" }
        "#,
    );

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] `alternative` index
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.2 (registry `alternative`)
[CHECKING] bar v0.1.2 (registry `alternative`)
[CHECKING] foo v0.1.0 ([..]/foo)
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn gitoxide_clones_shallow_old_git_patch() {
    perform_old_git_patch(true)
}

fn perform_old_git_patch(shallow: bool) {
    // Example where an old lockfile with an explicit branch="master" in Cargo.toml.
    Package::new("bar", "1.0.0").publish();
    let (bar, bar_repo) = git::new_repo("bar", |p| {
        p.file("Cargo.toml", &basic_manifest("bar", "1.0.0"))
            .file("src/lib.rs", "")
    });

    let bar_oid = bar_repo.head().unwrap().target().unwrap();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.1.0"

                    [dependencies]
                    bar = "1.0"

                    [patch.crates-io]
                    bar = {{ git = "{}", branch = "master" }}
                "#,
                bar.url()
            ),
        )
        .file(
            "Cargo.lock",
            &format!(
                r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
[[package]]
name = "bar"
version = "1.0.0"
source = "git+{}#{}"

[[package]]
name = "foo"
version = "0.1.0"
dependencies = [
 "bar",
]
            "#,
                bar.url(),
                bar_oid
            ),
        )
        .file("src/lib.rs", "")
        .build();

    bar.change_file("Cargo.toml", &basic_manifest("bar", "2.0.0"));
    git::add(&bar_repo);
    git::commit(&bar_repo);

    // This *should* keep the old lock.
    let mut cargo = p.cargo("tree");
    if shallow {
        cargo
            .arg("-Zgitoxide=fetch,shallow-deps")
            .masquerade_as_nightly_cargo(&["unstable features must be available for -Z gitoxide"]);
    }
    cargo
        // .env("CARGO_LOG", "trace")
        .with_stderr(
            "\
[UPDATING] [..]
",
        )
        // .with_status(1)
        .with_stdout(format!(
            "\
foo v0.1.0 [..]
└── bar v1.0.0 (file:///[..]branch=master#{})
",
            &bar_oid.to_string()[..8]
        ))
        .run();
}

#[cargo_test]
fn old_git_patch() {
    perform_old_git_patch(false)
}

// From https://github.com/rust-lang/cargo/issues/7463
#[cargo_test]
fn patch_eq_conflict_panic() {
    Package::new("bar", "0.1.0").publish();
    Package::new("bar", "0.1.1").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "=0.1.0"

                [dev-dependencies]
                bar = "=0.1.1"

                [patch.crates-io]
                bar = {path="bar"}
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("generate-lockfile")
        .with_status(101)
        .with_stderr(
            r#"[UPDATING] `dummy-registry` index
[ERROR] failed to select a version for `bar`.
    ... required by package `foo v0.1.0 ([..])`
versions that meet the requirements `=0.1.1` are: 0.1.1

all possible versions conflict with previously selected packages.

  previously selected package `bar v0.1.0`
    ... which satisfies dependency `bar = "=0.1.0"` of package `foo v0.1.0 ([..])`

failed to select a version for `bar` which could resolve this conflict
"#,
        )
        .run();
}

// From https://github.com/rust-lang/cargo/issues/11336
#[cargo_test]
fn mismatched_version2() {
    Package::new("qux", "0.1.0-beta.1").publish();
    Package::new("qux", "0.1.0-beta.2").publish();
    Package::new("bar", "0.1.0")
        .dep("qux", "=0.1.0-beta.1")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                 [package]
                 name = "foo"
                 version = "0.1.0"

                 [dependencies]
                 bar = "0.1.0"
                 qux = "0.1.0-beta.2"

                 [patch.crates-io]
                 qux = { path = "qux" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "qux/Cargo.toml",
            r#"
                [package]
                name = "qux"
                version = "0.1.0-beta.1"
            "#,
        )
        .file("qux/src/lib.rs", "")
        .build();

    p.cargo("generate-lockfile")
        .with_status(101)
        .with_stderr(
            r#"[UPDATING] `dummy-registry` index
[ERROR] failed to select a version for `qux`.
    ... required by package `bar v0.1.0`
    ... which satisfies dependency `bar = "^0.1.0"` of package `foo v0.1.0 ([..])`
versions that meet the requirements `=0.1.0-beta.1` are: 0.1.0-beta.1

all possible versions conflict with previously selected packages.

  previously selected package `qux v0.1.0-beta.2`
    ... which satisfies dependency `qux = "^0.1.0-beta.2"` of package `foo v0.1.0 ([..])`

failed to select a version for `qux` which could resolve this conflict"#,
        )
        .run();
}

#[cargo_test]
fn mismatched_version_with_prerelease() {
    Package::new("prerelease-deps", "0.0.1").publish();
    // A patch to a location that has an prerelease version
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                 [package]
                 name = "foo"
                 version = "0.1.0"

                 [dependencies]
                 prerelease-deps = "0.1.0"

                 [patch.crates-io]
                 prerelease-deps = { path = "./prerelease-deps" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "prerelease-deps/Cargo.toml",
            &basic_manifest("prerelease-deps", "0.1.1-pre1"),
        )
        .file("prerelease-deps/src/lib.rs", "")
        .build();

    p.cargo("generate-lockfile")
        .with_status(101)
        .with_stderr(
            r#"[UPDATING] `dummy-registry` index
[ERROR] failed to select a version for the requirement `prerelease-deps = "^0.1.0"`
candidate versions found which didn't match: 0.1.1-pre1, 0.0.1
location searched: `dummy-registry` index (which is replacing registry `crates-io`)
required by package `foo v0.1.0 [..]`
if you are looking for the prerelease package it needs to be specified explicitly
    prerelease-deps = { version = "0.1.1-pre1" }
perhaps a crate was updated and forgotten to be re-vendored?"#,
        )
        .run();
}

#[cargo_test]
fn from_config_empty() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file(
            ".cargo/config.toml",
            r#"
                [patch.'']
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] [patch] entry `` should be a URL or registry name

Caused by:
  invalid url ``: relative URL without a base
",
        )
        .run();
}

#[cargo_test]
fn from_manifest_empty() {
    Package::new("bar", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                bar = "0.1.0"

                [patch.'']
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", r#""#)
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  [patch] entry `` should be a URL or registry name

Caused by:
  invalid url ``: relative URL without a base
",
        )
        .run();
}
