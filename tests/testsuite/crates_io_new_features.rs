//! Tests for v2 crate publishing.
//!
//! These tests cannot run concurrently, you must specify one at a time.
//!
//! Required setup:
//! - hosts redirect of github.com to 127.0.0.1
//! - crates.io running locally
//! - https server serving the index under rust-lang/crates.io-index
//!     In `tmp`:
//!         mkdir -p http-serve/rust-lang
//!         cd http-serve/rust-lang
//!         ln -s ../../index-bare crates.io-index
//! - drag self-signed cert into Keychain Access
//!   - Set Trust to Always
//!
//! Need to disable rate limits.
//! insert into publish_rate_overrides values (1, 1000000, null, 0);
//! insert into publish_rate_overrides values (1, 1000000, null, 1);
use cargo_test_support::paths::{home, root, CargoPathExt};
use cargo_test_support::{execs, t, Execs};
use cargo_util::{paths, ProcessBuilder};
use std::fs;
use std::path::{Path, PathBuf};

#[cargo_test]
fn full_test() {
    // Publishes some packages to the registry and verifies they work.
    setup();
    do_publish_all();
    do_test();
}

/// Helper to write a file to the test sandbox.
fn write<P: AsRef<Path>>(path: P, contents: &str) {
    let path = root().join(path.as_ref());
    path.parent().unwrap().mkdir_p();
    paths::write(path, contents).unwrap();
}

fn create_package(name: &str, version: &str, extra: &str) {
    let root_dir = PathBuf::from(format!("{}-{}", name, version));
    write(
        root_dir.join("Cargo.toml"),
        &format!(
            r#"
                [package]
                name = "{}"
                version = "{}"
                license = "MIT"
                description = "{}"

                {}
            "#,
            name, version, name, extra
        ),
    );
    write(root_dir.join("src/lib.rs"), "");
}

/// Sets up the config and creates the cargo packages.
fn setup() {
    let config = home().join(".cargo/config");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    write(
        &config,
        "\
[unstable]
namespaced-features = true
weak-dep-features = true

[net]
git-fetch-with-cli = true

[registry]
token = \"cioM7nw4WIhwfLfolFhZ3O8v7KBSxytPD8w\"
",
    );
    // Just a general dependency to use as a leaf.
    create_package("somedep", "0.1.0", "[features]\nfeat = []");
    // Package that has a dependency on something that has namespaced features.
    create_package(
        "uses-namespaced",
        "0.1.0",
        r#"
            [dependencies.declares-namespaced]
            version = "0.1"
            features = ["feat"]
        "#,
    );
    // Package that has a dependency on something that has weak features.
    create_package(
        "uses-weak",
        "0.1.0",
        r#"
            [dependencies.declares-weak]
            version = "0.1"
            features = ["feat"]

            [features]
            extra = ["declares-weak/feat"]
            extra2 = ["declares-weak/feat", "declares-weak/somedep"]
        "#,
    );
    // Package using namespaced features, first version *without* namespaced.
    create_package(
        "declares-namespaced",
        "0.1.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep"]
        "#,
    );
    // Publish a new minor release that starts using namespaced.
    create_package(
        "declares-namespaced",
        "0.1.1",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["dep:somedep"]
        "#,
    );
    // Package using weak features, first version *without* weak features.
    create_package(
        "declares-weak",
        "0.1.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep/feat"]
        "#,
    );
    // Publish a new minor release that starts using weak features.
    create_package(
        "declares-weak",
        "0.1.1",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep?/feat"]
        "#,
    );

    /////////////////////////////////////////////////////////////////////////

    // Package using namespaced features, first version *without* namespaced,
    // adding namespaced with a major semver bump.
    create_package(
        "major-namespaced",
        "0.1.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep"]
        "#,
    );
    // Package using namespaced features, first version *without* namespaced,
    // adding namespaced with a major semver bump.
    create_package(
        "major-namespaced",
        "0.2.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["dep:somedep"]
        "#,
    );
    // A package using the namespaced crate *before* the major semver bump.
    create_package(
        "uses-major-namespaced",
        "0.1.0",
        r#"
            [dependencies.major-namespaced]
            version = "0.1"
            features = ["feat"]
        "#,
    );

    /////////////////////////////////////////////////////////////////////////

    // Package using weak features, first version *without* weak,
    // adding weak with a major semver bump.
    create_package(
        "major-weak",
        "0.1.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep/feat"]
        "#,
    );
    // Package using weak features, first version *without* weak,
    // adding weak with a major semver bump.
    create_package(
        "major-weak",
        "0.2.0",
        r#"
            [dependencies.somedep]
            version = "0.1"
            optional = true

            [features]
            feat = ["somedep?/feat"]
        "#,
    );
    // A package using the weak crate *before* the major semver bump.
    create_package(
        "uses-major-weak",
        "0.1.0",
        r#"
            [dependencies.major-weak]
            version = "0.1"

            [features]
            extra = ["major-weak/feat"]
        "#,
    );
}

#[cargo_test]
fn publish_all() {
    // Test for just publishing.
    setup();
    do_publish_all();
}

fn exec_publish(pkg: &str) {
    cargo("nightly", "publish --allow-dirty")
        .cwd(root().join(pkg))
        .run();
    std::thread::sleep(std::time::Duration::new(3, 0));
}

/// Helper to publish all packages.
fn do_publish_all() {
    reset();
    do_publish_before();
    do_publish_after();
}

fn do_publish_before() {
    for pkg in [
        "somedep-0.1.0",
        "declares-namespaced-0.1.0",
        "declares-weak-0.1.0",
        "uses-namespaced-0.1.0",
        "uses-weak-0.1.0",
        "major-namespaced-0.1.0",
        "uses-major-namespaced-0.1.0",
        "major-weak-0.1.0",
        "uses-major-weak-0.1.0",
    ] {
        exec_publish(pkg);
    }
}

fn do_publish_after() {
    for pkg in [
        "declares-namespaced-0.1.1",
        "declares-weak-0.1.1",
        "major-namespaced-0.2.0",
        "major-weak-0.2.0",
    ] {
        exec_publish(pkg);
    }
}

/// Creates an `Execs` for testing a command's output.
fn execs_process(command: &str) -> Execs {
    let mut args = command.split_whitespace();
    let mut p = ProcessBuilder::new(args.next().unwrap());
    for arg in args {
        p.arg(arg);
    }
    p.cwd(root());
    p.env("CARGO_HOME", home().join(".cargo"));
    p.env("GIT_SSL_NO_VERIFY", "1");
    p.env("GIT_CONFIG_NOSYSTEM", "1");
    p.env("__CARGO_TEST_ROOT", root());
    let mut e = execs().with_process_builder(p);
    e.stream();
    e
}

fn cargo(version: &str, args: &str) -> Execs {
    let mut e = execs_process("cargo");
    e.arg(format!("+{}", version));
    for arg in args.split_whitespace() {
        e.arg(arg);
    }
    e
}

/// Resets the index and registry to a clean slate.
fn reset() {
    execs_process("psql -d cargo_registry -c")
        .arg("DELETE FROM crates")
        .run();
    execs_process("psql -d cargo_registry -c")
        .arg("DELETE FROM publish_limit_buckets")
        .run();
    Path::new("/Users/eric/Proj/rust/crates.io/tmp/index-bare").rm_rf();
    execs_process("/Users/eric/Proj/rust/crates.io/script/init-local-index.sh")
        .cwd("/Users/eric/Proj/rust/crates.io")
        .run();
    t!(fs::copy(
        "/Users/eric/Proj/rust/crates.io/tmp/index-bare/hooks/post-update.sample",
        "/Users/eric/Proj/rust/crates.io/tmp/index-bare/hooks/post-update"
    ));
    execs_process("git")
        .arg("update-server-info")
        .cwd("/Users/eric/Proj/rust/crates.io/tmp/index-bare")
        .run();
    home().join(".cargo/registry").rm_rf();
}

#[cargo_test]
fn test() {
    // Only tests that the published packages work. Need to run the
    // `publish_all` test before running this.
    setup();
    do_test();
}

/// Helper for running all the tests after publishing.
fn do_test() {
    // check_uses_namespaced();
    // check_major_namespaced();
    // check_uses_weak();
    // check_major_weak();
    // namespaced_hidden();
    // check_declares_weak();
    // ignore_151();
    locked_150();
}

fn check_uses_namespaced() {
    cargo("nightly", "tree -f")
        .arg("{p} features={f}")
        .cwd("uses-namespaced-0.1.0")
        .with_stdout(
            "\
uses-namespaced v0.1.0 ([ROOT]/uses-namespaced-0.1.0) features=
└── declares-namespaced v0.1.1 features=feat
    └── somedep v0.1.0 features=
",
        )
        .run();
}

fn check_major_namespaced() {
    cargo("nightly", "tree -f")
        .arg("{p} features={f}")
        .cwd("uses-major-namespaced-0.1.0")
        .with_stdout(
            "\
uses-major-namespaced v0.1.0 ([ROOT]/uses-major-namespaced-0.1.0) features=
└── major-namespaced v0.1.0 features=feat,somedep
    └── somedep v0.1.0 features=
",
        )
        .run();
}

fn check_uses_weak() {
    cargo("nightly", "tree -f")
        .arg("{p} features={f}")
        .cwd("uses-weak-0.1.0")
        .with_stdout(
            "\
uses-weak v0.1.0 ([ROOT]/uses-weak-0.1.0) features=
└── declares-weak v0.1.1 features=feat
",
        )
        .run();

    cargo("nightly", "tree --features extra -f")
        .arg("{p} features={f}")
        .cwd("uses-weak-0.1.0")
        .with_stdout(
            "\
uses-weak v0.1.0 ([ROOT]/uses-weak-0.1.0) features=extra
└── declares-weak v0.1.1 features=feat
",
        )
        .run();

    cargo("nightly", "tree --features extra2 -f")
        .arg("{p} features={f}")
        .cwd("uses-weak-0.1.0")
        .with_stdout(
            "\
uses-weak v0.1.0 ([ROOT]/uses-weak-0.1.0) features=extra2
└── declares-weak v0.1.1 features=feat,somedep
    └── somedep v0.1.0 features=feat
",
        )
        .run();
}

fn check_major_weak() {
    cargo("nightly", "tree -f")
        .arg("{p} features={f}")
        .cwd("uses-major-weak-0.1.0")
        .with_stdout(
            "\
uses-major-weak v0.1.0 ([ROOT]/uses-major-weak-0.1.0) features=
└── major-weak v0.1.0 features=
",
        )
        .run();

    cargo("nightly", "tree --features extra -f")
        .arg("{p} features={f}")
        .cwd("uses-major-weak-0.1.0")
        .with_stdout(
            "\
uses-major-weak v0.1.0 ([ROOT]/uses-major-weak-0.1.0) features=extra
└── major-weak v0.1.0 features=feat,somedep
    └── somedep v0.1.0 features=feat
",
        )
        .run();
}

fn namespaced_hidden() {
    cargo("nightly", "tree --features somedep")
        .cwd("declares-namespaced-0.1.1")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] crates.io index
error: Package `declares-namespaced v0.1.1 ([ROOT]/declares-namespaced-0.1.1)` \
does not have feature `somedep`. It has an optional dependency with that name, \
but that dependency uses the \"dep:\" syntax in the features table, so it does \
not have an implicit feature with that name.
",
        )
        .run();
}

fn check_declares_weak() {
    cargo("nightly", "tree -f")
        .arg("{p} features={f}")
        .cwd("declares-weak-0.1.1")
        .with_stdout("declares-weak v0.1.1 ([ROOT]/declares-weak-0.1.1) features=")
        .run();
    cargo("nightly", "tree --features feat -f")
        .arg("{p} features={f}")
        .cwd("declares-weak-0.1.1")
        .with_stdout("declares-weak v0.1.1 ([ROOT]/declares-weak-0.1.1) features=feat")
        .run();
    cargo("nightly", "tree --features somedep -f")
        .arg("{p} features={f}")
        .cwd("declares-weak-0.1.1")
        .with_stdout(
            "\
declares-weak v0.1.1 ([ROOT]/declares-weak-0.1.1) features=somedep
└── somedep v0.1.0 features=
",
        )
        .run();
    cargo("nightly", "tree --features somedep,feat -f")
        .arg("{p} features={f}")
        .cwd("declares-weak-0.1.1")
        .with_stdout(
            "\
declares-weak v0.1.1 ([ROOT]/declares-weak-0.1.1) features=feat,somedep
└── somedep v0.1.0 features=feat
",
        )
        .run();
}

fn ignore_151() {
    // Ignore new packages in 1.51
    for version in ["1.51.0", "1.52.0", "1.53.0"] {
        root().join("uses-weak-0.1.0/Cargo.lock").rm_rf();
        cargo(version, "tree -f")
            .arg("{p} features={f}")
            .cwd("uses-weak-0.1.0")
            .with_stdout(
                "\
uses-weak v0.1.0 ([ROOT]/uses-weak-0.1.0) features=
└── declares-weak v0.1.0 features=feat,somedep
    └── somedep v0.1.0 features=feat
",
            )
            .run();

        root().join("uses-namespaced-0.1.0/Cargo.lock").rm_rf();
        cargo(version, "tree -f")
            .arg("{p} features={f}")
            .cwd("uses-namespaced-0.1.0")
            .with_stdout(
                "\
uses-namespaced v0.1.0 ([ROOT]/uses-namespaced-0.1.0) features=
└── declares-namespaced v0.1.0 features=feat,somedep
    └── somedep v0.1.0 features=
",
            )
            .run();

        root().join("uses-major-weak-0.1.0/Cargo.lock").rm_rf();
        cargo(version, "tree -f")
            .arg("{p} features={f}")
            .cwd("uses-major-weak-0.1.0")
            .with_stdout(
                "\
uses-major-weak v0.1.0 ([ROOT]/uses-major-weak-0.1.0) features=
└── major-weak v0.1.0 features=
",
            )
            .run();

        root()
            .join("uses-major-namespaced-0.1.0/Cargo.lock")
            .rm_rf();
        cargo(version, "tree -f")
            .arg("{p} features={f}")
            .cwd("uses-major-namespaced-0.1.0")
            .with_stdout(
                "\
uses-major-namespaced v0.1.0 ([ROOT]/uses-major-namespaced-0.1.0) features=
└── major-namespaced v0.1.0 features=feat,somedep
    └── somedep v0.1.0 features=
",
            )
            .run();
    }
}

fn locked_150() {
    // 1.50 and older should work with Cargo.lock without the new features.
    reset();
    do_publish_before();
    // First prep the Cargo.lock.
    root().join("uses-weak-0.1.0/Cargo.lock").rm_rf();
    // Create a lock file that all versions can use.
    // Need a version old enough to make a Cargo.lock that is compatible (includes the [root] table).
    cargo("1.21.0", "generate-lockfile")
        .cwd("uses-weak-0.1.0")
        .run();
    do_publish_after();

    for version in 16..=50 {
        let vers = format!("1.{}.0", version);
        let status = cargo(&vers, "--version").build_command().status().unwrap();
        if !status.success() {
            eprintln!("Skipping {}, not installed?", vers);
            continue;
        }
        home().join(".cargo/registry").rm_rf();
        root().join("uses-weak-0.1.0/target").rm_rf();
        cargo(&vers, "build -v").cwd("uses-weak-0.1.0").run();
    }
}
