//! Tests for the new feature resolver.

use cargo_test_support::cross_compile::{self, alternate};
use cargo_test_support::install::cargo_home;
use cargo_test_support::paths::CargoPathExt;
use cargo_test_support::publish::validate_crate_contents;
use cargo_test_support::registry::{Dependency, Package};
use cargo_test_support::{basic_manifest, cargo_process, project, rustc_host, Project};
use std::fs::File;

/// Switches Cargo.toml to use `resolver = "2"`.
pub fn switch_to_resolver_2(p: &Project) {
    let mut manifest = p.read_file("Cargo.toml");
    if manifest.contains("resolver =") {
        panic!("did not expect manifest to already contain a resolver setting");
    }
    if let Some(index) = manifest.find("[workspace]\n") {
        manifest.insert_str(index + 12, "resolver = \"2\"\n");
    } else if let Some(index) = manifest.find("[package]\n") {
        manifest.insert_str(index + 10, "resolver = \"2\"\n");
    } else {
        panic!("expected [package] or [workspace] in manifest");
    }
    p.change_file("Cargo.toml", &manifest);
}

#[cargo_test]
fn inactivate_targets() {
    // Basic test of `itarget`. A shared dependency where an inactive [target]
    // changes the features.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            compile_error!("f1 should not activate");
            "#,
        )
        .publish();

    Package::new("bar", "1.0.0")
        .add_dep(
            Dependency::new("common", "1.0")
                .target("cfg(whatever)")
                .enable_features(&["f1"]),
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            common = "1.0"
            bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr_contains("[..]f1 should not activate[..]")
        .run();

    switch_to_resolver_2(&p);
    p.cargo("check").run();
}

#[cargo_test]
fn inactive_target_optional() {
    // Activating optional [target] dependencies for inactivate target.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .feature("f2", &[])
        .feature("f3", &[])
        .feature("f4", &[])
        .file(
            "src/lib.rs",
            r#"
            pub fn f() {
                if cfg!(feature="f1") { println!("f1"); }
                if cfg!(feature="f2") { println!("f2"); }
                if cfg!(feature="f3") { println!("f3"); }
                if cfg!(feature="f4") { println!("f4"); }
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [dependencies]
            common = "1.0"

            [target.'cfg(whatever)'.dependencies]
            dep1 = {path='dep1', optional=true}
            dep2 = {path='dep2', optional=true, features=["f3"]}
            common = {version="1.0", optional=true, features=["f4"]}

            [features]
            foo1 = ["dep1/f2"]
            foo2 = ["dep2"]
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                if cfg!(feature="foo1") { println!("foo1"); }
                if cfg!(feature="foo2") { println!("foo2"); }
                if cfg!(feature="dep1") { println!("dep1"); }
                if cfg!(feature="dep2") { println!("dep2"); }
                if cfg!(feature="common") { println!("common"); }
                common::f();
            }
            "#,
        )
        .file(
            "dep1/Cargo.toml",
            r#"
            [package]
            name = "dep1"
            version = "0.1.0"

            [dependencies]
            common = {version="1.0", features=["f1"]}

            [features]
            f2 = ["common/f2"]
            "#,
        )
        .file(
            "dep1/src/lib.rs",
            r#"compile_error!("dep1 should not build");"#,
        )
        .file(
            "dep2/Cargo.toml",
            r#"
            [package]
            name = "dep2"
            version = "0.1.0"

            [dependencies]
            common = "1.0"

            [features]
            f3 = ["common/f3"]
            "#,
        )
        .file(
            "dep2/src/lib.rs",
            r#"compile_error!("dep2 should not build");"#,
        )
        .build();

    p.cargo("run --all-features")
        .with_stdout("foo1\nfoo2\ndep1\ndep2\ncommon\nf1\nf2\nf3\nf4\n")
        .run();
    p.cargo("run --features dep1")
        .with_stdout("dep1\nf1\n")
        .run();
    p.cargo("run --features foo1")
        .with_stdout("foo1\ndep1\nf1\nf2\n")
        .run();
    p.cargo("run --features dep2")
        .with_stdout("dep2\nf3\n")
        .run();
    p.cargo("run --features common")
        .with_stdout("common\nf4\n")
        .run();

    switch_to_resolver_2(&p);
    p.cargo("run --all-features")
        .with_stdout("foo1\nfoo2\ndep1\ndep2\ncommon")
        .run();
    p.cargo("run --features dep1").with_stdout("dep1\n").run();
    p.cargo("run --features foo1").with_stdout("foo1\n").run();
    p.cargo("run --features dep2").with_stdout("dep2\n").run();
    p.cargo("run --features common").with_stdout("common").run();
}

#[cargo_test]
fn itarget_proc_macro() {
    // itarget inside a proc-macro while cross-compiling
    if cross_compile::disabled() {
        return;
    }
    Package::new("hostdep", "1.0.0").publish();
    Package::new("pm", "1.0.0")
        .proc_macro(true)
        .target_dep("hostdep", "1.0", rustc_host())
        .file("src/lib.rs", "extern crate hostdep;")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            pm = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // Old behavior
    p.cargo("check").run();
    p.cargo("check --target").arg(alternate()).run();

    // New behavior
    switch_to_resolver_2(&p);
    p.cargo("check").run();
    p.cargo("check --target").arg(alternate()).run();
    // For good measure, just make sure things don't break.
    p.cargo("check --target").arg(alternate()).run();
}

#[cargo_test]
fn decouple_host_deps() {
    // Basic test for `host_dep` decouple.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            pub fn foo() {}
            #[cfg(not(feature = "f1"))]
            pub fn bar() {}
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [build-dependencies]
            common = {version="1.0", features=["f1"]}

            [dependencies]
            common = "1.0"
            "#,
        )
        .file(
            "build.rs",
            r#"
            use common::foo;
            fn main() {}
            "#,
        )
        .file("src/lib.rs", "use common::bar;")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr_contains("[..]unresolved import `common::bar`[..]")
        .run();

    switch_to_resolver_2(&p);
    p.cargo("check").run();
}

#[cargo_test]
fn decouple_host_deps_nested() {
    // `host_dep` decouple of transitive dependencies.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            pub fn foo() {}
            #[cfg(not(feature = "f1"))]
            pub fn bar() {}
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [build-dependencies]
            bdep = {path="bdep"}

            [dependencies]
            common = "1.0"
            "#,
        )
        .file(
            "build.rs",
            r#"
            use bdep::foo;
            fn main() {}
            "#,
        )
        .file("src/lib.rs", "use common::bar;")
        .file(
            "bdep/Cargo.toml",
            r#"
            [package]
            name = "bdep"
            version = "0.1.0"
            edition = "2018"

            [dependencies]
            common = {version="1.0", features=["f1"]}
            "#,
        )
        .file("bdep/src/lib.rs", "pub use common::foo;")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr_contains("[..]unresolved import `common::bar`[..]")
        .run();

    switch_to_resolver_2(&p);
    p.cargo("check").run();
}

#[cargo_test]
fn decouple_dev_deps() {
    // Basic test for `dev_dep` decouple.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .feature("f2", &[])
        .file(
            "src/lib.rs",
            r#"
            // const ensures it uses the correct dependency at *build time*
            // compared to *link time*.
            #[cfg(all(feature="f1", not(feature="f2")))]
            pub const X: u32 = 1;

            #[cfg(all(feature="f1", feature="f2"))]
            pub const X: u32 = 3;

            pub fn foo() -> u32 {
                let mut res = 0;
                if cfg!(feature = "f1") {
                    res |= 1;
                }
                if cfg!(feature = "f2") {
                    res |= 2;
                }
                res
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [dependencies]
            common = {version="1.0", features=["f1"]}

            [dev-dependencies]
            common = {version="1.0", features=["f2"]}
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                let expected: u32 = std::env::args().skip(1).next().unwrap().parse().unwrap();
                assert_eq!(foo::foo(), expected);
                assert_eq!(foo::build_time(), expected);
                assert_eq!(common::foo(), expected);
                assert_eq!(common::X, expected);
            }

            #[test]
            fn test_bin() {
                assert_eq!(foo::foo(), 3);
                assert_eq!(common::foo(), 3);
                assert_eq!(common::X, 3);
                assert_eq!(foo::build_time(), 3);
            }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn foo() -> u32 {
                common::foo()
            }

            pub fn build_time() -> u32 {
                common::X
            }

            #[test]
            fn test_lib() {
                assert_eq!(foo(), 3);
                assert_eq!(common::foo(), 3);
                assert_eq!(common::X, 3);
            }
            "#,
        )
        .file(
            "tests/t1.rs",
            r#"
            #[test]
            fn test_t1() {
                assert_eq!(foo::foo(), 3);
                assert_eq!(common::foo(), 3);
                assert_eq!(common::X, 3);
                assert_eq!(foo::build_time(), 3);
            }

            #[test]
            fn test_main() {
                // Features are unified for main when run with `cargo test`,
                // even with the new resolver.
                let s = std::process::Command::new("target/debug/foo")
                    .arg("3")
                    .status().unwrap();
                assert!(s.success());
            }
            "#,
        )
        .build();

    // Old behavior
    p.cargo("run 3").run();
    p.cargo("test").run();

    // New behavior
    switch_to_resolver_2(&p);
    p.cargo("run 1").run();
    p.cargo("test").run();
}

#[cargo_test]
fn build_script_runtime_features() {
    // Check that the CARGO_FEATURE_* environment variable is set correctly.
    //
    // This has a common dependency between build/normal/dev-deps, and it
    // queries which features it was built with in different circumstances.
    Package::new("common", "1.0.0")
        .feature("normal", &[])
        .feature("dev", &[])
        .feature("build", &[])
        .file(
            "build.rs",
            r#"
            fn is_set(name: &str) -> bool {
                std::env::var(name) == Ok("1".to_string())
            }

            fn main() {
                let mut res = 0;
                if is_set("CARGO_FEATURE_NORMAL") {
                    res |= 1;
                }
                if is_set("CARGO_FEATURE_DEV") {
                    res |= 2;
                }
                if is_set("CARGO_FEATURE_BUILD") {
                    res |= 4;
                }
                println!("cargo:rustc-cfg=RunCustomBuild=\"{}\"", res);

                let mut res = 0;
                if cfg!(feature = "normal") {
                    res |= 1;
                }
                if cfg!(feature = "dev") {
                    res |= 2;
                }
                if cfg!(feature = "build") {
                    res |= 4;
                }
                println!("cargo:rustc-cfg=CustomBuild=\"{}\"", res);
            }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn foo() -> u32 {
                let mut res = 0;
                if cfg!(feature = "normal") {
                    res |= 1;
                }
                if cfg!(feature = "dev") {
                    res |= 2;
                }
                if cfg!(feature = "build") {
                    res |= 4;
                }
                res
            }

            pub fn build_time() -> u32 {
                #[cfg(RunCustomBuild="1")] return 1;
                #[cfg(RunCustomBuild="3")] return 3;
                #[cfg(RunCustomBuild="4")] return 4;
                #[cfg(RunCustomBuild="5")] return 5;
                #[cfg(RunCustomBuild="7")] return 7;
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [build-dependencies]
            common = {version="1.0", features=["build"]}

            [dependencies]
            common = {version="1.0", features=["normal"]}

            [dev-dependencies]
            common = {version="1.0", features=["dev"]}
            "#,
        )
        .file(
            "build.rs",
            r#"
            fn main() {
                assert_eq!(common::foo(), common::build_time());
                println!("cargo:rustc-cfg=from_build=\"{}\"", common::foo());
            }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn foo() -> u32 {
                common::foo()
            }

            pub fn build_time() -> u32 {
                common::build_time()
            }

            #[test]
            fn test_lib() {
                assert_eq!(common::foo(), common::build_time());
                assert_eq!(common::foo(),
                    std::env::var("CARGO_FEATURE_EXPECT").unwrap().parse().unwrap());
            }
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                assert_eq!(common::foo(), common::build_time());
                assert_eq!(common::foo(),
                    std::env::var("CARGO_FEATURE_EXPECT").unwrap().parse().unwrap());
            }

            #[test]
            fn test_bin() {
                assert_eq!(common::foo(), common::build_time());
                assert_eq!(common::foo(),
                    std::env::var("CARGO_FEATURE_EXPECT").unwrap().parse().unwrap());
            }
            "#,
        )
        .file(
            "tests/t1.rs",
            r#"
            #[test]
            fn test_t1() {
                assert_eq!(common::foo(), common::build_time());
                assert_eq!(common::foo(),
                    std::env::var("CARGO_FEATURE_EXPECT").unwrap().parse().unwrap());
            }

            #[test]
            fn test_main() {
                // Features are unified for main when run with `cargo test`,
                // even with the new resolver.
                let s = std::process::Command::new("target/debug/foo")
                    .status().unwrap();
                assert!(s.success());
            }
            "#,
        )
        .build();

    // Old way, unifies all 3.
    p.cargo("run").env("CARGO_FEATURE_EXPECT", "7").run();
    p.cargo("test").env("CARGO_FEATURE_EXPECT", "7").run();

    // New behavior.
    switch_to_resolver_2(&p);

    // normal + build unify
    p.cargo("run").env("CARGO_FEATURE_EXPECT", "1").run();

    // dev_deps are still unified with `cargo test`
    p.cargo("test").env("CARGO_FEATURE_EXPECT", "3").run();
}

#[cargo_test]
fn cyclical_dev_dep() {
    // Check how a cyclical dev-dependency will work.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [features]
            dev = []

            [dev-dependencies]
            foo = { path = '.', features = ["dev"] }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn assert_dev(enabled: bool) {
                assert_eq!(enabled, cfg!(feature="dev"));
            }

            #[test]
            fn test_in_lib() {
                assert_dev(true);
            }
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                let expected: bool = std::env::args().skip(1).next().unwrap().parse().unwrap();
                foo::assert_dev(expected);
            }
            "#,
        )
        .file(
            "tests/t1.rs",
            r#"
            #[test]
            fn integration_links() {
                foo::assert_dev(true);
                // The lib linked with main.rs will also be unified.
                let s = std::process::Command::new("target/debug/foo")
                    .arg("true")
                    .status().unwrap();
                assert!(s.success());
            }
            "#,
        )
        .build();

    // Old way unifies features.
    p.cargo("run true").run();
    // dev feature should always be enabled in tests.
    p.cargo("test").run();

    // New behavior.
    switch_to_resolver_2(&p);
    // Should decouple main.
    p.cargo("run false").run();

    // And this should be no different.
    p.cargo("test").run();
}

#[cargo_test]
fn all_feature_opts() {
    // All feature options at once.
    Package::new("common", "1.0.0")
        .feature("normal", &[])
        .feature("build", &[])
        .feature("dev", &[])
        .feature("itarget", &[])
        .file(
            "src/lib.rs",
            r#"
            pub fn feats() -> u32 {
                let mut res = 0;
                if cfg!(feature="normal") { res |= 1; }
                if cfg!(feature="build") { res |= 2; }
                if cfg!(feature="dev") { res |= 4; }
                if cfg!(feature="itarget") { res |= 8; }
                res
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [dependencies]
            common = {version = "1.0", features=["normal"]}

            [dev-dependencies]
            common = {version = "1.0", features=["dev"]}

            [build-dependencies]
            common = {version = "1.0", features=["build"]}

            [target.'cfg(whatever)'.dependencies]
            common = {version = "1.0", features=["itarget"]}
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                expect();
            }

            fn expect() {
                let expected: u32 = std::env::var("EXPECTED_FEATS").unwrap().parse().unwrap();
                assert_eq!(expected, common::feats());
            }

            #[test]
            fn from_test() {
                expect();
            }
            "#,
        )
        .build();

    p.cargo("run").env("EXPECTED_FEATS", "15").run();
    p.cargo("test").env("EXPECTED_FEATS", "15").run();

    // New behavior.
    switch_to_resolver_2(&p);
    // Only normal feature.
    p.cargo("run").env("EXPECTED_FEATS", "1").run();

    // only normal+dev
    p.cargo("test").env("EXPECTED_FEATS", "5").run();
}

#[cargo_test]
fn required_features_host_dep() {
    // Check that required-features handles build-dependencies correctly.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [[bin]]
            name = "x"
            required-features = ["bdep/f1"]

            [build-dependencies]
            bdep = {path="bdep"}
            "#,
        )
        .file("build.rs", "fn main() {}")
        .file(
            "src/bin/x.rs",
            r#"
            fn main() {}
            "#,
        )
        .file(
            "bdep/Cargo.toml",
            r#"
            [package]
            name = "bdep"
            version = "0.1.0"

            [features]
            f1 = []
            "#,
        )
        .file("bdep/src/lib.rs", "")
        .build();

    p.cargo("run")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] target `x` in package `foo` requires the features: `bdep/f1`
Consider enabling them by passing, e.g., `--features=\"bdep/f1\"`
",
        )
        .run();

    // New behavior.
    switch_to_resolver_2(&p);
    p.cargo("run --features bdep/f1").run();
}

#[cargo_test]
fn disabled_shared_host_dep() {
    // Check for situation where an optional dep of a shared dep is enabled in
    // a normal dependency, but disabled in an optional one. The unit tree is:
    // foo
    // ├── foo build.rs
    // |   └── common (BUILD dependency, NO FEATURES)
    // └── common (Normal dependency, default features)
    //     └── somedep
    Package::new("somedep", "1.0.0")
        .file(
            "src/lib.rs",
            r#"
            pub fn f() { println!("hello from somedep"); }
            "#,
        )
        .publish();
    Package::new("common", "1.0.0")
        .feature("default", &["somedep"])
        .add_dep(Dependency::new("somedep", "1.0").optional(true))
        .file(
            "src/lib.rs",
            r#"
            pub fn check_somedep() -> bool {
                #[cfg(feature="somedep")]
                {
                    extern crate somedep;
                    somedep::f();
                    true
                }
                #[cfg(not(feature="somedep"))]
                {
                    println!("no somedep");
                    false
                }
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            edition = "2018"
            resolver = "2"

            [dependencies]
            common = "1.0"

            [build-dependencies]
            common = {version = "1.0", default-features = false}
            "#,
        )
        .file(
            "src/main.rs",
            "fn main() { assert!(common::check_somedep()); }",
        )
        .file(
            "build.rs",
            "fn main() { assert!(!common::check_somedep()); }",
        )
        .build();

    p.cargo("run -v").with_stdout("hello from somedep").run();
}

#[cargo_test]
fn required_features_inactive_dep() {
    // required-features with an inactivated dep.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            resolver = "2"

            [target.'cfg(whatever)'.dependencies]
            bar = {path="bar"}

            [[bin]]
            name = "foo"
            required-features = ["feat1"]

            [features]
            feat1 = []
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("check").with_stderr("[FINISHED] [..]").run();

    p.cargo("check --features=feat1")
        .with_stderr("[CHECKING] foo[..]\n[FINISHED] [..]")
        .run();
}

#[cargo_test]
fn decouple_proc_macro() {
    // proc macro features are not shared
    Package::new("common", "1.0.0")
        .feature("somefeat", &[])
        .file(
            "src/lib.rs",
            r#"
            pub const fn foo() -> bool { cfg!(feature="somefeat") }
            #[cfg(feature="somefeat")]
            pub const FEAT_ONLY_CONST: bool = true;
            "#,
        )
        .publish();
    Package::new("pm", "1.0.0")
        .proc_macro(true)
        .feature_dep("common", "1.0", &["somefeat"])
        .file(
            "src/lib.rs",
            r#"
            extern crate proc_macro;
            extern crate common;
            #[proc_macro]
            pub fn foo(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                assert!(common::foo());
                "".parse().unwrap()
            }
            "#,
        )
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            edition = "2018"

            [dependencies]
            pm = "1.0"
            common = "1.0"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            //! Test with docs.
            //!
            //! ```rust
            //! pm::foo!{}
            //! fn main() {
            //!   let expected = std::env::var_os("TEST_EXPECTS_ENABLED").is_some();
            //!   assert_eq!(expected, common::foo(), "common is wrong");
            //! }
            //! ```
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            pm::foo!{}
            fn main() {
                println!("it is {}", common::foo());
            }
            "#,
        )
        .build();

    p.cargo("run")
        .env("TEST_EXPECTS_ENABLED", "1")
        .with_stdout("it is true")
        .run();
    // Make sure the test is fallible.
    p.cargo("test --doc")
        .with_status(101)
        .with_stdout_contains("[..]common is wrong[..]")
        .run();
    p.cargo("test --doc").env("TEST_EXPECTS_ENABLED", "1").run();
    p.cargo("doc").run();
    assert!(p
        .build_dir()
        .join("doc/common/constant.FEAT_ONLY_CONST.html")
        .exists());
    // cargo doc should clean in-between runs, but it doesn't, and leaves stale files.
    // https://github.com/rust-lang/cargo/issues/6783 (same for removed items)
    p.build_dir().join("doc").rm_rf();

    // New behavior.
    switch_to_resolver_2(&p);
    p.cargo("run").with_stdout("it is false").run();

    p.cargo("test --doc").run();
    p.cargo("doc").run();
    assert!(!p
        .build_dir()
        .join("doc/common/constant.FEAT_ONLY_CONST.html")
        .exists());
}

#[cargo_test]
fn proc_macro_ws() {
    // Checks for bug with proc-macro in a workspace with dependency (shouldn't panic).
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["foo", "pm"]
            resolver = "2"
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [features]
            feat1 = []
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file(
            "pm/Cargo.toml",
            r#"
            [package]
            name = "pm"
            version = "0.1.0"

            [lib]
            proc-macro = true

            [dependencies]
            foo = { path = "../foo", features=["feat1"] }
            "#,
        )
        .file("pm/src/lib.rs", "")
        .build();

    p.cargo("check -p pm -v")
        .with_stderr_contains("[RUNNING] `rustc --crate-name foo [..]--cfg[..]feat1[..]")
        .run();
    // This may be surprising that `foo` doesn't get built separately. It is
    // because pm might have other units (binaries, tests, etc.), and so the
    // feature resolver must assume that normal deps get unified with it. This
    // is related to the bigger issue where the features selected in a
    // workspace depend on which packages are selected.
    p.cargo("check --workspace -v")
        .with_stderr(
            "\
[FRESH] foo v0.1.0 [..]
[FRESH] pm v0.1.0 [..]
[FINISHED] dev [..]
",
        )
        .run();
    // Selecting just foo will build without unification.
    p.cargo("check -p foo -v")
        // Make sure `foo` is built without feat1
        .with_stderr_line_without(&["[RUNNING] `rustc --crate-name foo"], &["--cfg[..]feat1"])
        .run();
}

#[cargo_test]
fn has_dev_dep_for_test() {
    // Check for a bug where the decision on whether or not "dev dependencies"
    // should be used did not consider `check --profile=test`.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dev-dependencies]
            dep = { path = 'dep', features = ['f1'] }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #[test]
            fn t1() {
                dep::f();
            }
            "#,
        )
        .file(
            "dep/Cargo.toml",
            r#"
            [package]
            name = "dep"
            version = "0.1.0"

            [features]
            f1 = []
            "#,
        )
        .file(
            "dep/src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            pub fn f() {}
            "#,
        )
        .build();

    p.cargo("check -v")
        .with_stderr(
            "\
[CHECKING] foo v0.1.0 [..]
[RUNNING] `rustc --crate-name foo [..]
[FINISHED] [..]
",
        )
        .run();
    p.cargo("check -v --profile=test")
        .with_stderr(
            "\
[CHECKING] dep v0.1.0 [..]
[RUNNING] `rustc --crate-name dep [..]
[CHECKING] foo v0.1.0 [..]
[RUNNING] `rustc --crate-name foo [..]
[FINISHED] [..]
",
        )
        .run();

    // New resolver should not be any different.
    switch_to_resolver_2(&p);
    p.cargo("check -v --profile=test")
        .with_stderr(
            "\
[FRESH] dep [..]
[FRESH] foo [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn build_dep_activated() {
    // Build dependencies always match the host for [target.*.build-dependencies].
    if cross_compile::disabled() {
        return;
    }
    Package::new("somedep", "1.0.0")
        .file("src/lib.rs", "")
        .publish();
    Package::new("targetdep", "1.0.0").publish();
    Package::new("hostdep", "1.0.0")
        // Check that "for_host" is sticky.
        .target_dep("somedep", "1.0", rustc_host())
        .feature("feat1", &[])
        .file(
            "src/lib.rs",
            r#"
            extern crate somedep;

            #[cfg(not(feature="feat1"))]
            compile_error!{"feat1 missing"}
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                [package]
                name = "foo"
                version = "0.1.0"

                # This should never be selected.
                [target.'{}'.build-dependencies]
                targetdep = "1.0"

                [target.'{}'.build-dependencies]
                hostdep = {{version="1.0", features=["feat1"]}}
                "#,
                alternate(),
                rustc_host()
            ),
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    p.cargo("check").run();
    p.cargo("check --target").arg(alternate()).run();

    // New behavior.
    switch_to_resolver_2(&p);
    p.cargo("check").run();
    p.cargo("check --target").arg(alternate()).run();
}

#[cargo_test]
fn resolver_bad_setting() {
    // Unknown setting in `resolver`
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            resolver = "foo"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  `resolver` setting `foo` is not valid, valid options are \"1\" or \"2\"
",
        )
        .run();
}

#[cargo_test]
fn resolver_original() {
    // resolver="1" uses old unification behavior.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            compile_error!("f1 should not activate");
            "#,
        )
        .publish();

    Package::new("bar", "1.0.0")
        .add_dep(
            Dependency::new("common", "1.0")
                .target("cfg(whatever)")
                .enable_features(&["f1"]),
        )
        .publish();

    let manifest = |resolver| {
        format!(
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                resolver = "{}"

                [dependencies]
                common = "1.0"
                bar = "1.0"
            "#,
            resolver
        )
    };

    let p = project()
        .file("Cargo.toml", &manifest("1"))
        .file("src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr_contains("[..]f1 should not activate[..]")
        .run();

    p.change_file("Cargo.toml", &manifest("2"));

    p.cargo("check").run();
}

#[cargo_test]
fn resolver_not_both() {
    // Can't specify resolver in both workspace and package.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            resolver = "2"
            [package]
            name = "foo"
            version = "0.1.0"
            resolver = "2"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  cannot specify `resolver` field in both `[workspace]` and `[package]`
",
        )
        .run();
}

#[cargo_test]
fn resolver_ws_member() {
    // Can't specify `resolver` in a ws member.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a"]
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.1.0"
            resolver = "2"
            "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
warning: resolver for the non root package will be ignored, specify resolver at the workspace root:
package:   [..]/foo/a/Cargo.toml
workspace: [..]/foo/Cargo.toml
[CHECKING] a v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn resolver_ws_root_and_member() {
    // Check when specified in both ws root and member.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a"]
            resolver = "2"
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.1.0"
            resolver = "2"
            "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    // Ignores if they are the same.
    p.cargo("check")
        .with_stderr(
            "\
[CHECKING] a v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn resolver_enables_new_features() {
    // resolver="2" enables all the things.
    Package::new("common", "1.0.0")
        .feature("normal", &[])
        .feature("build", &[])
        .feature("dev", &[])
        .feature("itarget", &[])
        .file(
            "src/lib.rs",
            r#"
            pub fn feats() -> u32 {
                let mut res = 0;
                if cfg!(feature="normal") { res |= 1; }
                if cfg!(feature="build") { res |= 2; }
                if cfg!(feature="dev") { res |= 4; }
                if cfg!(feature="itarget") { res |= 8; }
                res
            }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
            resolver = "2"
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.1.0"
            edition = "2018"

            [dependencies]
            common = {version = "1.0", features=["normal"]}

            [dev-dependencies]
            common = {version = "1.0", features=["dev"]}

            [build-dependencies]
            common = {version = "1.0", features=["build"]}

            [target.'cfg(whatever)'.dependencies]
            common = {version = "1.0", features=["itarget"]}
            "#,
        )
        .file(
            "a/src/main.rs",
            r#"
            fn main() {
                expect();
            }

            fn expect() {
                let expected: u32 = std::env::var("EXPECTED_FEATS").unwrap().parse().unwrap();
                assert_eq!(expected, common::feats());
            }

            #[test]
            fn from_test() {
                expect();
            }
            "#,
        )
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.1.0"

            [features]
            ping = []
            "#,
        )
        .file(
            "b/src/main.rs",
            r#"
            fn main() {
                if cfg!(feature="ping") {
                    println!("pong");
                }
            }
            "#,
        )
        .build();

    // Only normal.
    p.cargo("run --bin a")
        .env("EXPECTED_FEATS", "1")
        .with_stderr(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] common [..]
[COMPILING] common v1.0.0
[COMPILING] a v0.1.0 [..]
[FINISHED] [..]
[RUNNING] `target/debug/a[EXE]`
",
        )
        .run();

    // only normal+dev
    p.cargo("test").cwd("a").env("EXPECTED_FEATS", "5").run();

    // Can specify features of packages from a different directory.
    p.cargo("run -p b --features=ping")
        .cwd("a")
        .with_stdout("pong")
        .run();
}

#[cargo_test]
fn install_resolve_behavior() {
    // install honors the resolver behavior.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "f1")]
            compile_error!("f1 should not activate");
            "#,
        )
        .publish();

    Package::new("bar", "1.0.0").dep("common", "1.0").publish();

    Package::new("foo", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            resolver = "2"

            [target.'cfg(whatever)'.dependencies]
            common = {version="1.0", features=["f1"]}

            [dependencies]
            bar = "1.0"

            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .publish();

    cargo_process("install foo").run();
}

#[cargo_test]
fn package_includes_resolve_behavior() {
    // `cargo package` will inherit the correct resolve behavior.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a"]
            resolver = "2"
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.1.0"
            authors = ["Zzz"]
            description = "foo"
            license = "MIT"
            homepage = "https://example.com/"
            "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("package").cwd("a").run();

    let rewritten_toml = format!(
        r#"{}
[package]
name = "a"
version = "0.1.0"
authors = ["Zzz"]
description = "foo"
homepage = "https://example.com/"
license = "MIT"
resolver = "2"
"#,
        cargo::core::package::MANIFEST_PREAMBLE
    );

    let f = File::open(&p.root().join("target/package/a-0.1.0.crate")).unwrap();
    validate_crate_contents(
        f,
        "a-0.1.0.crate",
        &["Cargo.toml", "Cargo.toml.orig", "src/lib.rs"],
        &[("Cargo.toml", &rewritten_toml)],
    );
}

#[cargo_test]
fn tree_all() {
    // `cargo tree` with the new feature resolver.
    Package::new("log", "0.4.8").feature("serde", &[]).publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                resolver = "2"

                [target.'cfg(whatever)'.dependencies]
                log = {version="*", features=["serde"]}
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("tree --target=all")
        .with_stdout(
            "\
foo v0.1.0 ([..]/foo)
└── log v0.4.8
",
        )
        .run();
}

#[cargo_test]
fn shared_dep_same_but_dependencies() {
    // Checks for a bug of nondeterminism. This scenario creates a shared
    // dependency `dep` which needs to be built twice (once as normal, and
    // once as a build dep). However, in both cases the flags to `dep` are the
    // same, the only difference is what it links to. The normal dependency
    // should link to `subdep` with the feature disabled, and the build
    // dependency should link to it with it enabled. Crucially, the `--target`
    // flag should not be specified, otherwise Unit.kind would be different
    // and avoid the collision, and this bug won't manifest.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bin1", "bin2"]
                resolver = "2"
            "#,
        )
        .file(
            "bin1/Cargo.toml",
            r#"
                [package]
                name = "bin1"
                version = "0.1.0"

                [dependencies]
                dep = { path = "../dep" }
            "#,
        )
        .file("bin1/src/main.rs", "fn main() { dep::feat_func(); }")
        .file(
            "bin2/Cargo.toml",
            r#"
                [package]
                name = "bin2"
                version = "0.1.0"

                [build-dependencies]
                dep = { path = "../dep" }
                subdep = { path = "../subdep", features = ["feat"] }
            "#,
        )
        .file("bin2/build.rs", "fn main() { dep::feat_func(); }")
        .file("bin2/src/main.rs", "fn main() {}")
        .file(
            "dep/Cargo.toml",
            r#"
                [package]
                name = "dep"
                version = "0.1.0"

                [dependencies]
                subdep = { path = "../subdep" }
            "#,
        )
        .file(
            "dep/src/lib.rs",
            "pub fn feat_func() { subdep::feat_func(); }",
        )
        .file(
            "subdep/Cargo.toml",
            r#"
                [package]
                name = "subdep"
                version = "0.1.0"

                [features]
                feat = []
            "#,
        )
        .file(
            "subdep/src/lib.rs",
            r#"
                pub fn feat_func() {
                    #[cfg(feature = "feat")] println!("cargo:warning=feat: enabled");
                    #[cfg(not(feature = "feat"))] println!("cargo:warning=feat: not enabled");
                }
            "#,
        )
        .build();

    p.cargo("build --bin bin1 --bin bin2")
        // unordered because bin1 and bin2 build at the same time
        .with_stderr_unordered(
            "\
[COMPILING] subdep [..]
[COMPILING] dep [..]
[COMPILING] bin2 [..]
[COMPILING] bin1 [..]
warning: feat: enabled
[FINISHED] [..]
",
        )
        .run();
    p.process(p.bin("bin1"))
        .with_stdout("cargo:warning=feat: not enabled")
        .run();

    // Make sure everything stays cached.
    p.cargo("build -v --bin bin1 --bin bin2")
        .with_stderr_unordered(
            "\
[FRESH] subdep [..]
[FRESH] dep [..]
[FRESH] bin1 [..]
warning: feat: enabled
[FRESH] bin2 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn test_proc_macro() {
    // Running `cargo test` on a proc-macro, with a shared dependency that has
    // different features.
    //
    // There was a bug where `shared` was built twice (once with feature "B"
    // and once without), and both copies linked into the unit test. This
    // would cause a type failure when used in an intermediate dependency
    // (the-macro-support).
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "runtime"
                version = "0.1.0"
                resolver = "2"

                [dependencies]
                the-macro = { path = "the-macro", features = ['a'] }
                [build-dependencies]
                shared = { path = "shared", features = ['b'] }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "the-macro/Cargo.toml",
            r#"
                [package]
                name = "the-macro"
                version = "0.1.0"
                [lib]
                proc-macro = true
                test = false
                [dependencies]
                the-macro-support = { path = "../the-macro-support" }
                shared = { path = "../shared" }
                [dev-dependencies]
                runtime = { path = ".." }
                [features]
                a = []
            "#,
        )
        .file(
            "the-macro/src/lib.rs",
            "
                fn _test() {
                    the_macro_support::foo(shared::Foo);
                }
            ",
        )
        .file(
            "the-macro-support/Cargo.toml",
            r#"
                [package]
                name = "the-macro-support"
                version = "0.1.0"
                [dependencies]
                shared = { path = "../shared" }
            "#,
        )
        .file(
            "the-macro-support/src/lib.rs",
            "
                pub fn foo(_: shared::Foo) {}
            ",
        )
        .file(
            "shared/Cargo.toml",
            r#"
                [package]
                name = "shared"
                version = "0.1.0"
                [features]
                b = []
            "#,
        )
        .file("shared/src/lib.rs", "pub struct Foo;")
        .build();
    p.cargo("test --manifest-path the-macro/Cargo.toml").run();
}

#[cargo_test]
fn doc_optional() {
    // Checks for a bug where `cargo doc` was failing with an inactive target
    // that enables a shared optional dependency.
    Package::new("spin", "1.0.0").publish();
    Package::new("bar", "1.0.0")
        .add_dep(Dependency::new("spin", "1.0").optional(true))
        .publish();
    // The enabler package enables the `spin` feature, which we don't want.
    Package::new("enabler", "1.0.0")
        .feature_dep("bar", "1.0", &["spin"])
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                resolver = "2"

                [target.'cfg(whatever)'.dependencies]
                enabler = "1.0"

                [dependencies]
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_unordered(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] spin v1.0.0 [..]
[DOWNLOADED] bar v1.0.0 [..]
[DOCUMENTING] bar v1.0.0
[CHECKING] bar v1.0.0
[DOCUMENTING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn minimal_download() {
    // Various checks that it only downloads the minimum set of dependencies
    // needed in various situations.
    //
    // This checks several permutations of the different
    // host_dep/dev_dep/itarget settings. These 3 are planned to be stabilized
    // together, so there isn't much need to be concerned about how the behave
    // independently. However, there are some cases where they do behave
    // independently. Specifically:
    //
    // * `cargo test` forces dev_dep decoupling to be disabled.
    // * `cargo tree --target=all` forces ignore_inactive_targets off and decouple_dev_deps off.
    // * `cargo tree --target=all -e normal` forces ignore_inactive_targets off.
    //
    // However, `cargo tree` is a little weird because it downloads everything
    // anyways.
    //
    // So to summarize the different permutations:
    //
    // dev_dep | host_dep | itarget | Notes
    // --------|----------|---------|----------------------------
    //         |          |         | -Zfeatures=compare (new resolver should behave same as old)
    //         |          |    ✓    | This scenario should not happen.
    //         |     ✓    |         | `cargo tree --target=all -Zfeatures=all`†
    //         |     ✓    |    ✓    | `cargo test`
    //    ✓    |          |         | This scenario should not happen.
    //    ✓    |          |    ✓    | This scenario should not happen.
    //    ✓    |     ✓    |         | `cargo tree --target=all -e normal -Z features=all`†
    //    ✓    |     ✓    |    ✓    | A normal build.
    //
    // † — However, `cargo tree` downloads everything.
    Package::new("normal", "1.0.0").publish();
    Package::new("normal_pm", "1.0.0").publish();
    Package::new("normal_opt", "1.0.0").publish();
    Package::new("dev_dep", "1.0.0").publish();
    Package::new("dev_dep_pm", "1.0.0").publish();
    Package::new("build_dep", "1.0.0").publish();
    Package::new("build_dep_pm", "1.0.0").publish();
    Package::new("build_dep_opt", "1.0.0").publish();

    Package::new("itarget_normal", "1.0.0").publish();
    Package::new("itarget_normal_pm", "1.0.0").publish();
    Package::new("itarget_dev_dep", "1.0.0").publish();
    Package::new("itarget_dev_dep_pm", "1.0.0").publish();
    Package::new("itarget_build_dep", "1.0.0").publish();
    Package::new("itarget_build_dep_pm", "1.0.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                normal = "1.0"
                normal_pm = "1.0"
                normal_opt = { version = "1.0", optional = true }

                [dev-dependencies]
                dev_dep = "1.0"
                dev_dep_pm = "1.0"

                [build-dependencies]
                build_dep = "1.0"
                build_dep_pm = "1.0"
                build_dep_opt = { version = "1.0", optional = true }

                [target.'cfg(whatever)'.dependencies]
                itarget_normal = "1.0"
                itarget_normal_pm = "1.0"

                [target.'cfg(whatever)'.dev-dependencies]
                itarget_dev_dep = "1.0"
                itarget_dev_dep_pm = "1.0"

                [target.'cfg(whatever)'.build-dependencies]
                itarget_build_dep = "1.0"
                itarget_build_dep_pm = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    let clear = || {
        cargo_home().join("registry/cache").rm_rf();
        cargo_home().join("registry/src").rm_rf();
        p.build_dir().rm_rf();
    };

    // none
    // Should be the same as `-Zfeatures=all`
    p.cargo("check -Zfeatures=compare")
        .masquerade_as_nightly_cargo()
        .with_stderr_unordered(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] normal_pm v1.0.0 [..]
[DOWNLOADED] normal v1.0.0 [..]
[DOWNLOADED] build_dep_pm v1.0.0 [..]
[DOWNLOADED] build_dep v1.0.0 [..]
[COMPILING] build_dep v1.0.0
[COMPILING] build_dep_pm v1.0.0
[CHECKING] normal_pm v1.0.0
[CHECKING] normal v1.0.0
[COMPILING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
    clear();

    // New behavior
    switch_to_resolver_2(&p);

    // all
    p.cargo("check")
        .with_stderr_unordered(
            "\
[DOWNLOADING] crates ...
[DOWNLOADED] normal_pm v1.0.0 [..]
[DOWNLOADED] normal v1.0.0 [..]
[DOWNLOADED] build_dep_pm v1.0.0 [..]
[DOWNLOADED] build_dep v1.0.0 [..]
[COMPILING] build_dep v1.0.0
[COMPILING] build_dep_pm v1.0.0
[CHECKING] normal v1.0.0
[CHECKING] normal_pm v1.0.0
[COMPILING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
    clear();

    // This disables decouple_dev_deps.
    p.cargo("test --no-run")
        .with_stderr_unordered(
            "\
[DOWNLOADING] crates ...
[DOWNLOADED] normal_pm v1.0.0 [..]
[DOWNLOADED] normal v1.0.0 [..]
[DOWNLOADED] dev_dep_pm v1.0.0 [..]
[DOWNLOADED] dev_dep v1.0.0 [..]
[DOWNLOADED] build_dep_pm v1.0.0 [..]
[DOWNLOADED] build_dep v1.0.0 [..]
[COMPILING] build_dep v1.0.0
[COMPILING] build_dep_pm v1.0.0
[COMPILING] normal_pm v1.0.0
[COMPILING] normal v1.0.0
[COMPILING] dev_dep_pm v1.0.0
[COMPILING] dev_dep v1.0.0
[COMPILING] foo v0.1.0 [..]
[FINISHED] [..]
[EXECUTABLE] unittests src/lib.rs (target/debug/deps/foo-[..][EXE])
",
        )
        .run();
    clear();

    // This disables itarget, but leaves decouple_dev_deps enabled.
    p.cargo("tree -e normal --target=all")
        .with_stderr_unordered(
            "\
[DOWNLOADING] crates ...
[DOWNLOADED] normal v1.0.0 [..]
[DOWNLOADED] normal_pm v1.0.0 [..]
[DOWNLOADED] build_dep v1.0.0 [..]
[DOWNLOADED] build_dep_pm v1.0.0 [..]
[DOWNLOADED] itarget_normal v1.0.0 [..]
[DOWNLOADED] itarget_normal_pm v1.0.0 [..]
[DOWNLOADED] itarget_build_dep v1.0.0 [..]
[DOWNLOADED] itarget_build_dep_pm v1.0.0 [..]
",
        )
        .with_stdout(
            "\
foo v0.1.0 ([ROOT]/foo)
├── itarget_normal v1.0.0
├── itarget_normal_pm v1.0.0
├── normal v1.0.0
└── normal_pm v1.0.0
",
        )
        .run();
    clear();

    // This disables itarget and decouple_dev_deps.
    p.cargo("tree --target=all")
        .with_stderr_unordered(
            "\
[DOWNLOADING] crates ...
[DOWNLOADED] normal_pm v1.0.0 [..]
[DOWNLOADED] normal v1.0.0 [..]
[DOWNLOADED] itarget_normal_pm v1.0.0 [..]
[DOWNLOADED] itarget_normal v1.0.0 [..]
[DOWNLOADED] itarget_dev_dep_pm v1.0.0 [..]
[DOWNLOADED] itarget_dev_dep v1.0.0 [..]
[DOWNLOADED] itarget_build_dep_pm v1.0.0 [..]
[DOWNLOADED] itarget_build_dep v1.0.0 [..]
[DOWNLOADED] dev_dep_pm v1.0.0 [..]
[DOWNLOADED] dev_dep v1.0.0 [..]
[DOWNLOADED] build_dep_pm v1.0.0 [..]
[DOWNLOADED] build_dep v1.0.0 [..]
",
        )
        .with_stdout(
            "\
foo v0.1.0 ([ROOT]/foo)
├── itarget_normal v1.0.0
├── itarget_normal_pm v1.0.0
├── normal v1.0.0
└── normal_pm v1.0.0
[build-dependencies]
├── build_dep v1.0.0
├── build_dep_pm v1.0.0
├── itarget_build_dep v1.0.0
└── itarget_build_dep_pm v1.0.0
[dev-dependencies]
├── dev_dep v1.0.0
├── dev_dep_pm v1.0.0
├── itarget_dev_dep v1.0.0
└── itarget_dev_dep_pm v1.0.0
",
        )
        .run();
    clear();
}

#[test] fn pm_with_int_shared0() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared1() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared2() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared3() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared4() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared5() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared6() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared7() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared8() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared9() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared10() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared11() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared12() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared13() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared14() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared15() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared16() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared17() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared18() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared19() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared20() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared21() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared22() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared23() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared24() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared25() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared26() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared27() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared28() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared29() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared30() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared31() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared32() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared33() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared34() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared35() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared36() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared37() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared38() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared39() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared40() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared41() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared42() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared43() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared44() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared45() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared46() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared47() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared48() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared49() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared50() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared51() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared52() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared53() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared54() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared55() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared56() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared57() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared58() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared59() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared60() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared61() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared62() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared63() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared64() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared65() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared66() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared67() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared68() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared69() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared70() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared71() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared72() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared73() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared74() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared75() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared76() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared77() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared78() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared79() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared80() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared81() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared82() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared83() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared84() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared85() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared86() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared87() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared88() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared89() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared90() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared91() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared92() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared93() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared94() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared95() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared96() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared97() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared98() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared99() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared100() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared101() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared102() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared103() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared104() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared105() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared106() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared107() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared108() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared109() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared110() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared111() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared112() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared113() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared114() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared115() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared116() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared117() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared118() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared119() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared120() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared121() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared122() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared123() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared124() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared125() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared126() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared127() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared128() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared129() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared130() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared131() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared132() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared133() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared134() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared135() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared136() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared137() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared138() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared139() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared140() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared141() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared142() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared143() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared144() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared145() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared146() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared147() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared148() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared149() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared150() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared151() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared152() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared153() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared154() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared155() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared156() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared157() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared158() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared159() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared160() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared161() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared162() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared163() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared164() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared165() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared166() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared167() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared168() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared169() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared170() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared171() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared172() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared173() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared174() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared175() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared176() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared177() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared178() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared179() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared180() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared181() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared182() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared183() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared184() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared185() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared186() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared187() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared188() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared189() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared190() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared191() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared192() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared193() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared194() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared195() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared196() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared197() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared198() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared199() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared200() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared201() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared202() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared203() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared204() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared205() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared206() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared207() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared208() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared209() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared210() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared211() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared212() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared213() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared214() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared215() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared216() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared217() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared218() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared219() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared220() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared221() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared222() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared223() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared224() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared225() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared226() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared227() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared228() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared229() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared230() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared231() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared232() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared233() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared234() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared235() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared236() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared237() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared238() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared239() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared240() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared241() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared242() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared243() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared244() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared245() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared246() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared247() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared248() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared249() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared250() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared251() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared252() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared253() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared254() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared255() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared256() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared257() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared258() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared259() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared260() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared261() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared262() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared263() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared264() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared265() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared266() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared267() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared268() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared269() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared270() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared271() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared272() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared273() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared274() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared275() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared276() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared277() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared278() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared279() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared280() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared281() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared282() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared283() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared284() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared285() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared286() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared287() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared288() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared289() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared290() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared291() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared292() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared293() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared294() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared295() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared296() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared297() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared298() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared299() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared300() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared301() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared302() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared303() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared304() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared305() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared306() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared307() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared308() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared309() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared310() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared311() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared312() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared313() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared314() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared315() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared316() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared317() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared318() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared319() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared320() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared321() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared322() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared323() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared324() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared325() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared326() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared327() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared328() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared329() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared330() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared331() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared332() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared333() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared334() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared335() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared336() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared337() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared338() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared339() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared340() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared341() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared342() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared343() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared344() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared345() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared346() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared347() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared348() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared349() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared350() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared351() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared352() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared353() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared354() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared355() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared356() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared357() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared358() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared359() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared360() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared361() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared362() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared363() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared364() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared365() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared366() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared367() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared368() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared369() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared370() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared371() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared372() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared373() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared374() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared375() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared376() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared377() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared378() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared379() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared380() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared381() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared382() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared383() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared384() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared385() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared386() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared387() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared388() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared389() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared390() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared391() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared392() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared393() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared394() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared395() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared396() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared397() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared398() { pm_with_int_shared(); }
#[test] fn pm_with_int_shared399() { pm_with_int_shared(); }

#[cargo_test]
fn pm_with_int_shared() {
    // This is a somewhat complex scenario of a proc-macro in a workspace with
    // an integration test where the proc-macro is used for other things, and
    // *everything* is built at once (`--workspace --all-targets
    // --all-features`). There was a bug where the UnitFor settings were being
    // incorrectly computed based on the order that the graph was traversed.
    //
    // There are some uncertainties about exactly how proc-macros should behave
    // with `--workspace`, see https://github.com/rust-lang/cargo/issues/8312.
    //
    // This uses a const-eval hack to do compile-time feature checking.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "pm", "shared"]
                resolver = "2"
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                pm = { path = "../pm" }
                shared = { path = "../shared", features = ["norm-feat"] }
            "#,
        )
        .file(
            "foo/src/lib.rs",
            r#"
                // foo->shared always has both features set
                const _CHECK: [(); 0] = [(); 0-!(shared::FEATS==3) as usize];
            "#,
        )
        .file(
            "pm/Cargo.toml",
            r#"
                [package]
                name = "pm"
                version = "0.1.0"

                [lib]
                proc-macro = true

                [dependencies]
                shared = { path = "../shared", features = ["host-feat"] }
            "#,
        )
        .file(
            "pm/src/lib.rs",
            r#"
                // pm->shared always has just host
                const _CHECK: [(); 0] = [(); 0-!(shared::FEATS==1) as usize];
            "#,
        )
        .file(
            "pm/tests/pm_test.rs",
            r#"
                // integration test gets both set
                const _CHECK: [(); 0] = [(); 0-!(shared::FEATS==3) as usize];
            "#,
        )
        .file(
            "shared/Cargo.toml",
            r#"
                [package]
                name = "shared"
                version = "0.1.0"

                [features]
                norm-feat = []
                host-feat = []
            "#,
        )
        .file(
            "shared/src/lib.rs",
            r#"
                pub const FEATS: u32 = {
                    if cfg!(feature="norm-feat") && cfg!(feature="host-feat") {
                        3
                    } else if cfg!(feature="norm-feat") {
                        2
                    } else if cfg!(feature="host-feat") {
                        1
                    } else {
                        0
                    }
                };
            "#,
        )
        .build();

    p.cargo("build --workspace --all-targets --all-features -v")
        .with_stderr_unordered(
            "\
[COMPILING] shared [..]
[RUNNING] `rustc --crate-name shared [..]--crate-type lib [..]
[RUNNING] `rustc --crate-name shared [..]--crate-type lib [..]
[RUNNING] `rustc --crate-name shared [..]--test[..]
[COMPILING] pm [..]
[RUNNING] `rustc --crate-name pm [..]--crate-type proc-macro[..]
[RUNNING] `rustc --crate-name pm [..]--test[..]
[COMPILING] foo [..]
[RUNNING] `rustc --crate-name foo [..]--test[..]
[RUNNING] `rustc --crate-name pm_test [..]--test[..]
[RUNNING] `rustc --crate-name foo [..]--crate-type lib[..]
[FINISHED] [..]
",
        )
        .run();

    // And again, should stay fresh.
    p.cargo("build --workspace --all-targets --all-features -v")
        .with_stderr_unordered(
            "\
[FRESH] shared [..]
[FRESH] pm [..]
[FRESH] foo [..]
[FINISHED] [..]",
        )
        .run();
}

#[cargo_test]
fn doc_proc_macro() {
    // Checks for a bug when documenting a proc-macro with a dependency. The
    // doc unit builder was not carrying the "for host" setting through the
    // dependencies, and the `pm-dep` dependency was causing a panic because
    // it was looking for target features instead of host features.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                resolver = "2"

                [dependencies]
                pm = { path = "pm" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "pm/Cargo.toml",
            r#"
                [package]
                name = "pm"
                version = "0.1.0"

                [lib]
                proc-macro = true

                [dependencies]
                pm-dep = { path = "../pm-dep" }
            "#,
        )
        .file("pm/src/lib.rs", "")
        .file("pm-dep/Cargo.toml", &basic_manifest("pm-dep", "0.1.0"))
        .file("pm-dep/src/lib.rs", "")
        .build();

    // Unfortunately this cannot check the output because what it prints is
    // nondeterministic. Sometimes it says "Compiling pm-dep" and sometimes
    // "Checking pm-dep". This is because it is both building it and checking
    // it in parallel (building so it can build the proc-macro, and checking
    // so rustdoc can load it).
    p.cargo("doc").run();
}

#[cargo_test]
fn edition_2021_default_2() {
    // edition = 2021 defaults to v2 resolver.
    Package::new("common", "1.0.0")
        .feature("f1", &[])
        .file("src/lib.rs", "")
        .publish();

    Package::new("bar", "1.0.0")
        .add_dep(
            Dependency::new("common", "1.0")
                .target("cfg(whatever)")
                .enable_features(&["f1"]),
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                common = "1.0"
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // First without edition.
    p.cargo("tree -f")
        .arg("{p} feats:{f}")
        .with_stdout(
            "\
foo v0.1.0 [..]
├── bar v1.0.0 feats:
└── common v1.0.0 feats:f1
",
        )
        .run();

    p.change_file(
        "Cargo.toml",
        r#"
            cargo-features = ["edition2021"]

            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2021"

            [dependencies]
            common = "1.0"
            bar = "1.0"
        "#,
    );

    // Importantly, this does not include `f1` on `common`.
    p.cargo("tree -f")
        .arg("{p} feats:{f}")
        .masquerade_as_nightly_cargo()
        .with_stdout(
            "\
foo v0.1.0 [..]
├── bar v1.0.0 feats:
└── common v1.0.0 feats:
",
        )
        .run();
}

#[cargo_test]
fn all_features_merges_with_features() {
    Package::new("dep", "0.1.0")
        .feature("feat1", &[])
        .file(
            "src/lib.rs",
            r#"
                #[cfg(feature="feat1")]
                pub fn work() {
                    println!("it works");
                }
            "#,
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [features]
                a = []

                [dependencies]
                dep = "0.1"

                [[example]]
                name = "ex"
                required-features = ["a", "dep/feat1"]
            "#,
        )
        .file(
            "examples/ex.rs",
            r#"
            fn main() {
                dep::work();
            }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("run --example ex --all-features --features dep/feat1")
        .with_stderr(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] [..]
[COMPILING] dep v0.1.0
[COMPILING] foo v0.1.0 [..]
[FINISHED] [..]
[RUNNING] `target/debug/examples/ex[EXE]`
",
        )
        .with_stdout("it works")
        .run();

    switch_to_resolver_2(&p);

    p.cargo("run --example ex --all-features --features dep/feat1")
        .with_stderr(
            "\
[FINISHED] [..]
[RUNNING] `target/debug/examples/ex[EXE]`
",
        )
        .with_stdout("it works")
        .run();
}
