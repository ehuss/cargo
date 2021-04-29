//! Tests for when multiple artifacts have the same output filename.
//! See https://github.com/rust-lang/cargo/issues/6313 for more details.
//! Ideally these should never happen, but I don't think we'll ever be able to
//! prevent all collisions.

use cargo_test_support::registry::Package;
use cargo_test_support::{basic_manifest, cross_compile, project};
use std::env;

#[cargo_test]
fn collision_dylib() {
    // Path dependencies don't include metadata hash in filename for dylibs.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "1.0.0"

            [lib]
            crate-type = ["dylib"]
            "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "1.0.0"

            [lib]
            crate-type = ["dylib"]
            name = "a"
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    // `j=1` is required because on Windows you'll get an error due to
    // two processes writing to the file at the same time.
    p.cargo("build -j=1")
        .with_stderr_contains(&format!("\
[WARNING] output filename collision.
The lib target `a` in package `b v1.0.0 ([..]/foo/b)` has the same output filename as the lib target `a` in package `a v1.0.0 ([..]/foo/a)`.
Colliding filename is: [..]/foo/target/debug/deps/{}a{}
The targets should have unique names.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future; see <https://github.com/rust-lang/cargo/issues/6313>.
", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX))
        .run();
}

#[cargo_test]
fn collision_example() {
    // Examples in a workspace can easily collide.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
            "#,
        )
        .file("a/Cargo.toml", &basic_manifest("a", "1.0.0"))
        .file("a/examples/ex1.rs", "fn main() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "1.0.0"))
        .file("b/examples/ex1.rs", "fn main() {}")
        .build();

    // `j=1` is required because on Windows you'll get an error due to
    // two processes writing to the file at the same time.
    p.cargo("build --examples -j=1")
        .with_stderr_contains("\
[WARNING] output filename collision.
The example target `ex1` in package `b v1.0.0 ([..]/foo/b)` has the same output filename as the example target `ex1` in package `a v1.0.0 ([..]/foo/a)`.
Colliding filename is: [..]/foo/target/debug/examples/ex1[EXE]
The targets should have unique names.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future; see <https://github.com/rust-lang/cargo/issues/6313>.
")
        .run();
}

#[cargo_test]
// --out-dir and examples are currently broken on MSVC and apple.
// See https://github.com/rust-lang/cargo/issues/7493
#[cfg_attr(any(target_env = "msvc", target_vendor = "apple"), ignore)]
fn collision_export() {
    // `--out-dir` combines some things which can cause conflicts.
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo", "1.0.0"))
        .file("examples/foo.rs", "fn main() {}")
        .file("src/main.rs", "fn main() {}")
        .build();

    // -j1 to avoid issues with two processes writing to the same file at the
    // same time.
    p.cargo("build -j1 --out-dir=out -Z unstable-options --bins --examples")
        .masquerade_as_nightly_cargo()
        .with_stderr_contains("\
[WARNING] `--out-dir` filename collision.
The example target `foo` in package `foo v1.0.0 ([..]/foo)` has the same output filename as the bin target `foo` in package `foo v1.0.0 ([..]/foo)`.
Colliding filename is: [..]/foo/out/foo[EXE]
The exported filenames should be unique.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future; see <https://github.com/rust-lang/cargo/issues/6313>.
")
        .run();
}

#[cargo_test]
fn collision_doc() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            foo2 = { path = "foo2" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "foo2/Cargo.toml",
            r#"
            [package]
            name = "foo2"
            version = "0.1.0"

            [lib]
            name = "foo"
            "#,
        )
        .file("foo2/src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_contains(
            "\
[WARNING] output filename collision.
The lib target `foo` in package `foo2 v0.1.0 ([..]/foo/foo2)` has the same output \
filename as the lib target `foo` in package `foo v0.1.0 ([..]/foo)`.
Colliding filename is: [..]/foo/target/doc/foo/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
",
        )
        .run();
}

#[cargo_test]
fn collision_doc_multiple_versions() {
    // Multiple versions of the same package.
    Package::new("old-dep", "1.0.0").publish();
    Package::new("bar", "1.0.0").dep("old-dep", "1.0").publish();
    // Note that this removes "old-dep". Just checking what happens when there
    // are orphans.
    Package::new("bar", "2.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = "1.0"
                bar2 = { package="bar", version="2.0" }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // Should only document bar 2.0, should not document old-dep.
    p.cargo("doc")
        .with_stderr_unordered(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v2.0.0 [..]
[DOWNLOADED] bar v1.0.0 [..]
[DOWNLOADED] old-dep v1.0.0 [..]
[CHECKING] old-dep v1.0.0
[CHECKING] bar v2.0.0
[CHECKING] bar v1.0.0
[DOCUMENTING] bar v2.0.0
[FINISHED] [..]
[DOCUMENTING] foo v0.1.0 [..]
",
        )
        .run();
}

#[cargo_test]
fn collision_doc_host_target_feature_split() {
    // Same dependency built twice due to different features.
    //
    // foo v0.1.0
    // ├── common v1.0.0
    // │   └── common-dep v1.0.0
    // └── pm v0.1.0 (proc-macro)
    //     └── common v1.0.0
    //         └── common-dep v1.0.0
    // [build-dependencies]
    // └── common-dep v1.0.0
    //
    // Here `common` and `common-dep` are built twice. `common-dep` has
    // different features for host versus target.
    Package::new("common-dep", "1.0.0")
        .feature("bdep-feat", &[])
        .file(
            "src/lib.rs",
            r#"
                /// Some doc
                pub fn f() {}

                /// Another doc
                #[cfg(feature = "bdep-feat")]
                pub fn bdep_func() {}
            "#,
        )
        .publish();
    Package::new("common", "1.0.0")
        .dep("common-dep", "1.0")
        .file(
            "src/lib.rs",
            r#"
                /// Some doc
                pub fn f() {}
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
                resolver = "2"

                [dependencies]
                pm = { path = "pm" }
                common = "1.0"

                [build-dependencies]
                common-dep = { version = "1.0", features = ["bdep-feat"] }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                /// Some doc
                pub fn f() {}
            "#,
        )
        .file("build.rs", "fn main() {}")
        .file(
            "pm/Cargo.toml",
            r#"
                [package]
                name = "pm"
                version = "0.1.0"
                edition = "2018"

                [lib]
                proc-macro = true

                [dependencies]
                common = "1.0"
            "#,
        )
        .file(
            "pm/src/lib.rs",
            r#"
                use proc_macro::TokenStream;

                /// Some doc
                #[proc_macro]
                pub fn pm(_input: TokenStream) -> TokenStream {
                    "".parse().unwrap()
                }
            "#,
        )
        .build();

    // No warnings, no duplicates, common and common-dep only documented once.
    p.cargo("doc")
        // Cannot check full output due to https://github.com/rust-lang/cargo/issues/9076
        .with_stderr_does_not_contain("[WARNING][..]")
        .run();

    assert!(p.build_dir().join("doc/common_dep/fn.f.html").exists());
    assert!(!p
        .build_dir()
        .join("doc/common_dep/fn.bdep_func.html")
        .exists());
    assert!(p.build_dir().join("doc/common/fn.f.html").exists());
    assert!(p.build_dir().join("doc/pm/macro.pm.html").exists());
    assert!(p.build_dir().join("doc/foo/fn.f.html").exists());
}

#[cargo_test]
fn collision_doc_profile_split() {
    // Same dependency built twice due to different profile settings.
    Package::new("common", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                pm = { path = "pm" }
                common = "1.0"

                [profile.dev]
                opt-level = 2
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "pm/Cargo.toml",
            r#"
                [package]
                name = "pm"
                version = "0.1.0"

                [dependencies]
                common = "1.0"

                [lib]
                proc-macro = true
            "#,
        )
        .file("pm/src/lib.rs", "")
        .build();

    // Just to verify that common is normally built twice.
    p.cargo("build -v")
        .with_stderr(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] common v1.0.0 [..]
[COMPILING] common v1.0.0
[RUNNING] `rustc --crate-name common [..]
[RUNNING] `rustc --crate-name common [..]
[COMPILING] pm v0.1.0 [..]
[RUNNING] `rustc --crate-name pm [..]
[COMPILING] foo v0.1.0 [..]
[RUNNING] `rustc --crate-name foo [..]
[FINISHED] [..]
",
        )
        .run();

    // Should only document common once, no warnings.
    p.cargo("doc")
        .with_stderr_unordered(
            "\
[CHECKING] common v1.0.0
[DOCUMENTING] common v1.0.0
[DOCUMENTING] pm v0.1.0 [..]
[DOCUMENTING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[test] fn collision_doc_sources0() { collision_doc_sources(); }
#[test] fn collision_doc_sources1() { collision_doc_sources(); }
#[test] fn collision_doc_sources2() { collision_doc_sources(); }
#[test] fn collision_doc_sources3() { collision_doc_sources(); }
#[test] fn collision_doc_sources4() { collision_doc_sources(); }
#[test] fn collision_doc_sources5() { collision_doc_sources(); }
#[test] fn collision_doc_sources6() { collision_doc_sources(); }
#[test] fn collision_doc_sources7() { collision_doc_sources(); }
#[test] fn collision_doc_sources8() { collision_doc_sources(); }
#[test] fn collision_doc_sources9() { collision_doc_sources(); }
#[test] fn collision_doc_sources10() { collision_doc_sources(); }
#[test] fn collision_doc_sources11() { collision_doc_sources(); }
#[test] fn collision_doc_sources12() { collision_doc_sources(); }
#[test] fn collision_doc_sources13() { collision_doc_sources(); }
#[test] fn collision_doc_sources14() { collision_doc_sources(); }
#[test] fn collision_doc_sources15() { collision_doc_sources(); }
#[test] fn collision_doc_sources16() { collision_doc_sources(); }
#[test] fn collision_doc_sources17() { collision_doc_sources(); }
#[test] fn collision_doc_sources18() { collision_doc_sources(); }
#[test] fn collision_doc_sources19() { collision_doc_sources(); }
#[test] fn collision_doc_sources20() { collision_doc_sources(); }
#[test] fn collision_doc_sources21() { collision_doc_sources(); }
#[test] fn collision_doc_sources22() { collision_doc_sources(); }
#[test] fn collision_doc_sources23() { collision_doc_sources(); }
#[test] fn collision_doc_sources24() { collision_doc_sources(); }
#[test] fn collision_doc_sources25() { collision_doc_sources(); }
#[test] fn collision_doc_sources26() { collision_doc_sources(); }
#[test] fn collision_doc_sources27() { collision_doc_sources(); }
#[test] fn collision_doc_sources28() { collision_doc_sources(); }
#[test] fn collision_doc_sources29() { collision_doc_sources(); }
#[test] fn collision_doc_sources30() { collision_doc_sources(); }
#[test] fn collision_doc_sources31() { collision_doc_sources(); }
#[test] fn collision_doc_sources32() { collision_doc_sources(); }
#[test] fn collision_doc_sources33() { collision_doc_sources(); }
#[test] fn collision_doc_sources34() { collision_doc_sources(); }
#[test] fn collision_doc_sources35() { collision_doc_sources(); }
#[test] fn collision_doc_sources36() { collision_doc_sources(); }
#[test] fn collision_doc_sources37() { collision_doc_sources(); }
#[test] fn collision_doc_sources38() { collision_doc_sources(); }
#[test] fn collision_doc_sources39() { collision_doc_sources(); }
#[test] fn collision_doc_sources40() { collision_doc_sources(); }
#[test] fn collision_doc_sources41() { collision_doc_sources(); }
#[test] fn collision_doc_sources42() { collision_doc_sources(); }
#[test] fn collision_doc_sources43() { collision_doc_sources(); }
#[test] fn collision_doc_sources44() { collision_doc_sources(); }
#[test] fn collision_doc_sources45() { collision_doc_sources(); }
#[test] fn collision_doc_sources46() { collision_doc_sources(); }
#[test] fn collision_doc_sources47() { collision_doc_sources(); }
#[test] fn collision_doc_sources48() { collision_doc_sources(); }
#[test] fn collision_doc_sources49() { collision_doc_sources(); }
#[test] fn collision_doc_sources50() { collision_doc_sources(); }
#[test] fn collision_doc_sources51() { collision_doc_sources(); }
#[test] fn collision_doc_sources52() { collision_doc_sources(); }
#[test] fn collision_doc_sources53() { collision_doc_sources(); }
#[test] fn collision_doc_sources54() { collision_doc_sources(); }
#[test] fn collision_doc_sources55() { collision_doc_sources(); }
#[test] fn collision_doc_sources56() { collision_doc_sources(); }
#[test] fn collision_doc_sources57() { collision_doc_sources(); }
#[test] fn collision_doc_sources58() { collision_doc_sources(); }
#[test] fn collision_doc_sources59() { collision_doc_sources(); }
#[test] fn collision_doc_sources60() { collision_doc_sources(); }
#[test] fn collision_doc_sources61() { collision_doc_sources(); }
#[test] fn collision_doc_sources62() { collision_doc_sources(); }
#[test] fn collision_doc_sources63() { collision_doc_sources(); }
#[test] fn collision_doc_sources64() { collision_doc_sources(); }
#[test] fn collision_doc_sources65() { collision_doc_sources(); }
#[test] fn collision_doc_sources66() { collision_doc_sources(); }
#[test] fn collision_doc_sources67() { collision_doc_sources(); }
#[test] fn collision_doc_sources68() { collision_doc_sources(); }
#[test] fn collision_doc_sources69() { collision_doc_sources(); }
#[test] fn collision_doc_sources70() { collision_doc_sources(); }
#[test] fn collision_doc_sources71() { collision_doc_sources(); }
#[test] fn collision_doc_sources72() { collision_doc_sources(); }
#[test] fn collision_doc_sources73() { collision_doc_sources(); }
#[test] fn collision_doc_sources74() { collision_doc_sources(); }
#[test] fn collision_doc_sources75() { collision_doc_sources(); }
#[test] fn collision_doc_sources76() { collision_doc_sources(); }
#[test] fn collision_doc_sources77() { collision_doc_sources(); }
#[test] fn collision_doc_sources78() { collision_doc_sources(); }
#[test] fn collision_doc_sources79() { collision_doc_sources(); }
#[test] fn collision_doc_sources80() { collision_doc_sources(); }
#[test] fn collision_doc_sources81() { collision_doc_sources(); }
#[test] fn collision_doc_sources82() { collision_doc_sources(); }
#[test] fn collision_doc_sources83() { collision_doc_sources(); }
#[test] fn collision_doc_sources84() { collision_doc_sources(); }
#[test] fn collision_doc_sources85() { collision_doc_sources(); }
#[test] fn collision_doc_sources86() { collision_doc_sources(); }
#[test] fn collision_doc_sources87() { collision_doc_sources(); }
#[test] fn collision_doc_sources88() { collision_doc_sources(); }
#[test] fn collision_doc_sources89() { collision_doc_sources(); }
#[test] fn collision_doc_sources90() { collision_doc_sources(); }
#[test] fn collision_doc_sources91() { collision_doc_sources(); }
#[test] fn collision_doc_sources92() { collision_doc_sources(); }
#[test] fn collision_doc_sources93() { collision_doc_sources(); }
#[test] fn collision_doc_sources94() { collision_doc_sources(); }
#[test] fn collision_doc_sources95() { collision_doc_sources(); }
#[test] fn collision_doc_sources96() { collision_doc_sources(); }
#[test] fn collision_doc_sources97() { collision_doc_sources(); }
#[test] fn collision_doc_sources98() { collision_doc_sources(); }
#[test] fn collision_doc_sources99() { collision_doc_sources(); }
#[test] fn collision_doc_sources100() { collision_doc_sources(); }
#[test] fn collision_doc_sources101() { collision_doc_sources(); }
#[test] fn collision_doc_sources102() { collision_doc_sources(); }
#[test] fn collision_doc_sources103() { collision_doc_sources(); }
#[test] fn collision_doc_sources104() { collision_doc_sources(); }
#[test] fn collision_doc_sources105() { collision_doc_sources(); }
#[test] fn collision_doc_sources106() { collision_doc_sources(); }
#[test] fn collision_doc_sources107() { collision_doc_sources(); }
#[test] fn collision_doc_sources108() { collision_doc_sources(); }
#[test] fn collision_doc_sources109() { collision_doc_sources(); }
#[test] fn collision_doc_sources110() { collision_doc_sources(); }
#[test] fn collision_doc_sources111() { collision_doc_sources(); }
#[test] fn collision_doc_sources112() { collision_doc_sources(); }
#[test] fn collision_doc_sources113() { collision_doc_sources(); }
#[test] fn collision_doc_sources114() { collision_doc_sources(); }
#[test] fn collision_doc_sources115() { collision_doc_sources(); }
#[test] fn collision_doc_sources116() { collision_doc_sources(); }
#[test] fn collision_doc_sources117() { collision_doc_sources(); }
#[test] fn collision_doc_sources118() { collision_doc_sources(); }
#[test] fn collision_doc_sources119() { collision_doc_sources(); }
#[test] fn collision_doc_sources120() { collision_doc_sources(); }
#[test] fn collision_doc_sources121() { collision_doc_sources(); }
#[test] fn collision_doc_sources122() { collision_doc_sources(); }
#[test] fn collision_doc_sources123() { collision_doc_sources(); }
#[test] fn collision_doc_sources124() { collision_doc_sources(); }
#[test] fn collision_doc_sources125() { collision_doc_sources(); }
#[test] fn collision_doc_sources126() { collision_doc_sources(); }
#[test] fn collision_doc_sources127() { collision_doc_sources(); }
#[test] fn collision_doc_sources128() { collision_doc_sources(); }
#[test] fn collision_doc_sources129() { collision_doc_sources(); }
#[test] fn collision_doc_sources130() { collision_doc_sources(); }
#[test] fn collision_doc_sources131() { collision_doc_sources(); }
#[test] fn collision_doc_sources132() { collision_doc_sources(); }
#[test] fn collision_doc_sources133() { collision_doc_sources(); }
#[test] fn collision_doc_sources134() { collision_doc_sources(); }
#[test] fn collision_doc_sources135() { collision_doc_sources(); }
#[test] fn collision_doc_sources136() { collision_doc_sources(); }
#[test] fn collision_doc_sources137() { collision_doc_sources(); }
#[test] fn collision_doc_sources138() { collision_doc_sources(); }
#[test] fn collision_doc_sources139() { collision_doc_sources(); }
#[test] fn collision_doc_sources140() { collision_doc_sources(); }
#[test] fn collision_doc_sources141() { collision_doc_sources(); }
#[test] fn collision_doc_sources142() { collision_doc_sources(); }
#[test] fn collision_doc_sources143() { collision_doc_sources(); }
#[test] fn collision_doc_sources144() { collision_doc_sources(); }
#[test] fn collision_doc_sources145() { collision_doc_sources(); }
#[test] fn collision_doc_sources146() { collision_doc_sources(); }
#[test] fn collision_doc_sources147() { collision_doc_sources(); }
#[test] fn collision_doc_sources148() { collision_doc_sources(); }
#[test] fn collision_doc_sources149() { collision_doc_sources(); }
#[test] fn collision_doc_sources150() { collision_doc_sources(); }
#[test] fn collision_doc_sources151() { collision_doc_sources(); }
#[test] fn collision_doc_sources152() { collision_doc_sources(); }
#[test] fn collision_doc_sources153() { collision_doc_sources(); }
#[test] fn collision_doc_sources154() { collision_doc_sources(); }
#[test] fn collision_doc_sources155() { collision_doc_sources(); }
#[test] fn collision_doc_sources156() { collision_doc_sources(); }
#[test] fn collision_doc_sources157() { collision_doc_sources(); }
#[test] fn collision_doc_sources158() { collision_doc_sources(); }
#[test] fn collision_doc_sources159() { collision_doc_sources(); }
#[test] fn collision_doc_sources160() { collision_doc_sources(); }
#[test] fn collision_doc_sources161() { collision_doc_sources(); }
#[test] fn collision_doc_sources162() { collision_doc_sources(); }
#[test] fn collision_doc_sources163() { collision_doc_sources(); }
#[test] fn collision_doc_sources164() { collision_doc_sources(); }
#[test] fn collision_doc_sources165() { collision_doc_sources(); }
#[test] fn collision_doc_sources166() { collision_doc_sources(); }
#[test] fn collision_doc_sources167() { collision_doc_sources(); }
#[test] fn collision_doc_sources168() { collision_doc_sources(); }
#[test] fn collision_doc_sources169() { collision_doc_sources(); }
#[test] fn collision_doc_sources170() { collision_doc_sources(); }
#[test] fn collision_doc_sources171() { collision_doc_sources(); }
#[test] fn collision_doc_sources172() { collision_doc_sources(); }
#[test] fn collision_doc_sources173() { collision_doc_sources(); }
#[test] fn collision_doc_sources174() { collision_doc_sources(); }
#[test] fn collision_doc_sources175() { collision_doc_sources(); }
#[test] fn collision_doc_sources176() { collision_doc_sources(); }
#[test] fn collision_doc_sources177() { collision_doc_sources(); }
#[test] fn collision_doc_sources178() { collision_doc_sources(); }
#[test] fn collision_doc_sources179() { collision_doc_sources(); }
#[test] fn collision_doc_sources180() { collision_doc_sources(); }
#[test] fn collision_doc_sources181() { collision_doc_sources(); }
#[test] fn collision_doc_sources182() { collision_doc_sources(); }
#[test] fn collision_doc_sources183() { collision_doc_sources(); }
#[test] fn collision_doc_sources184() { collision_doc_sources(); }
#[test] fn collision_doc_sources185() { collision_doc_sources(); }
#[test] fn collision_doc_sources186() { collision_doc_sources(); }
#[test] fn collision_doc_sources187() { collision_doc_sources(); }
#[test] fn collision_doc_sources188() { collision_doc_sources(); }
#[test] fn collision_doc_sources189() { collision_doc_sources(); }
#[test] fn collision_doc_sources190() { collision_doc_sources(); }
#[test] fn collision_doc_sources191() { collision_doc_sources(); }
#[test] fn collision_doc_sources192() { collision_doc_sources(); }
#[test] fn collision_doc_sources193() { collision_doc_sources(); }
#[test] fn collision_doc_sources194() { collision_doc_sources(); }
#[test] fn collision_doc_sources195() { collision_doc_sources(); }
#[test] fn collision_doc_sources196() { collision_doc_sources(); }
#[test] fn collision_doc_sources197() { collision_doc_sources(); }
#[test] fn collision_doc_sources198() { collision_doc_sources(); }
#[test] fn collision_doc_sources199() { collision_doc_sources(); }
#[test] fn collision_doc_sources200() { collision_doc_sources(); }
#[test] fn collision_doc_sources201() { collision_doc_sources(); }
#[test] fn collision_doc_sources202() { collision_doc_sources(); }
#[test] fn collision_doc_sources203() { collision_doc_sources(); }
#[test] fn collision_doc_sources204() { collision_doc_sources(); }
#[test] fn collision_doc_sources205() { collision_doc_sources(); }
#[test] fn collision_doc_sources206() { collision_doc_sources(); }
#[test] fn collision_doc_sources207() { collision_doc_sources(); }
#[test] fn collision_doc_sources208() { collision_doc_sources(); }
#[test] fn collision_doc_sources209() { collision_doc_sources(); }
#[test] fn collision_doc_sources210() { collision_doc_sources(); }
#[test] fn collision_doc_sources211() { collision_doc_sources(); }
#[test] fn collision_doc_sources212() { collision_doc_sources(); }
#[test] fn collision_doc_sources213() { collision_doc_sources(); }
#[test] fn collision_doc_sources214() { collision_doc_sources(); }
#[test] fn collision_doc_sources215() { collision_doc_sources(); }
#[test] fn collision_doc_sources216() { collision_doc_sources(); }
#[test] fn collision_doc_sources217() { collision_doc_sources(); }
#[test] fn collision_doc_sources218() { collision_doc_sources(); }
#[test] fn collision_doc_sources219() { collision_doc_sources(); }
#[test] fn collision_doc_sources220() { collision_doc_sources(); }
#[test] fn collision_doc_sources221() { collision_doc_sources(); }
#[test] fn collision_doc_sources222() { collision_doc_sources(); }
#[test] fn collision_doc_sources223() { collision_doc_sources(); }
#[test] fn collision_doc_sources224() { collision_doc_sources(); }
#[test] fn collision_doc_sources225() { collision_doc_sources(); }
#[test] fn collision_doc_sources226() { collision_doc_sources(); }
#[test] fn collision_doc_sources227() { collision_doc_sources(); }
#[test] fn collision_doc_sources228() { collision_doc_sources(); }
#[test] fn collision_doc_sources229() { collision_doc_sources(); }
#[test] fn collision_doc_sources230() { collision_doc_sources(); }
#[test] fn collision_doc_sources231() { collision_doc_sources(); }
#[test] fn collision_doc_sources232() { collision_doc_sources(); }
#[test] fn collision_doc_sources233() { collision_doc_sources(); }
#[test] fn collision_doc_sources234() { collision_doc_sources(); }
#[test] fn collision_doc_sources235() { collision_doc_sources(); }
#[test] fn collision_doc_sources236() { collision_doc_sources(); }
#[test] fn collision_doc_sources237() { collision_doc_sources(); }
#[test] fn collision_doc_sources238() { collision_doc_sources(); }
#[test] fn collision_doc_sources239() { collision_doc_sources(); }
#[test] fn collision_doc_sources240() { collision_doc_sources(); }
#[test] fn collision_doc_sources241() { collision_doc_sources(); }
#[test] fn collision_doc_sources242() { collision_doc_sources(); }
#[test] fn collision_doc_sources243() { collision_doc_sources(); }
#[test] fn collision_doc_sources244() { collision_doc_sources(); }
#[test] fn collision_doc_sources245() { collision_doc_sources(); }
#[test] fn collision_doc_sources246() { collision_doc_sources(); }
#[test] fn collision_doc_sources247() { collision_doc_sources(); }
#[test] fn collision_doc_sources248() { collision_doc_sources(); }
#[test] fn collision_doc_sources249() { collision_doc_sources(); }
#[test] fn collision_doc_sources250() { collision_doc_sources(); }
#[test] fn collision_doc_sources251() { collision_doc_sources(); }
#[test] fn collision_doc_sources252() { collision_doc_sources(); }
#[test] fn collision_doc_sources253() { collision_doc_sources(); }
#[test] fn collision_doc_sources254() { collision_doc_sources(); }
#[test] fn collision_doc_sources255() { collision_doc_sources(); }
#[test] fn collision_doc_sources256() { collision_doc_sources(); }
#[test] fn collision_doc_sources257() { collision_doc_sources(); }
#[test] fn collision_doc_sources258() { collision_doc_sources(); }
#[test] fn collision_doc_sources259() { collision_doc_sources(); }
#[test] fn collision_doc_sources260() { collision_doc_sources(); }
#[test] fn collision_doc_sources261() { collision_doc_sources(); }
#[test] fn collision_doc_sources262() { collision_doc_sources(); }
#[test] fn collision_doc_sources263() { collision_doc_sources(); }
#[test] fn collision_doc_sources264() { collision_doc_sources(); }
#[test] fn collision_doc_sources265() { collision_doc_sources(); }
#[test] fn collision_doc_sources266() { collision_doc_sources(); }
#[test] fn collision_doc_sources267() { collision_doc_sources(); }
#[test] fn collision_doc_sources268() { collision_doc_sources(); }
#[test] fn collision_doc_sources269() { collision_doc_sources(); }
#[test] fn collision_doc_sources270() { collision_doc_sources(); }
#[test] fn collision_doc_sources271() { collision_doc_sources(); }
#[test] fn collision_doc_sources272() { collision_doc_sources(); }
#[test] fn collision_doc_sources273() { collision_doc_sources(); }
#[test] fn collision_doc_sources274() { collision_doc_sources(); }
#[test] fn collision_doc_sources275() { collision_doc_sources(); }
#[test] fn collision_doc_sources276() { collision_doc_sources(); }
#[test] fn collision_doc_sources277() { collision_doc_sources(); }
#[test] fn collision_doc_sources278() { collision_doc_sources(); }
#[test] fn collision_doc_sources279() { collision_doc_sources(); }
#[test] fn collision_doc_sources280() { collision_doc_sources(); }
#[test] fn collision_doc_sources281() { collision_doc_sources(); }
#[test] fn collision_doc_sources282() { collision_doc_sources(); }
#[test] fn collision_doc_sources283() { collision_doc_sources(); }
#[test] fn collision_doc_sources284() { collision_doc_sources(); }
#[test] fn collision_doc_sources285() { collision_doc_sources(); }
#[test] fn collision_doc_sources286() { collision_doc_sources(); }
#[test] fn collision_doc_sources287() { collision_doc_sources(); }
#[test] fn collision_doc_sources288() { collision_doc_sources(); }
#[test] fn collision_doc_sources289() { collision_doc_sources(); }
#[test] fn collision_doc_sources290() { collision_doc_sources(); }
#[test] fn collision_doc_sources291() { collision_doc_sources(); }
#[test] fn collision_doc_sources292() { collision_doc_sources(); }
#[test] fn collision_doc_sources293() { collision_doc_sources(); }
#[test] fn collision_doc_sources294() { collision_doc_sources(); }
#[test] fn collision_doc_sources295() { collision_doc_sources(); }
#[test] fn collision_doc_sources296() { collision_doc_sources(); }
#[test] fn collision_doc_sources297() { collision_doc_sources(); }
#[test] fn collision_doc_sources298() { collision_doc_sources(); }
#[test] fn collision_doc_sources299() { collision_doc_sources(); }
#[test] fn collision_doc_sources300() { collision_doc_sources(); }
#[test] fn collision_doc_sources301() { collision_doc_sources(); }
#[test] fn collision_doc_sources302() { collision_doc_sources(); }
#[test] fn collision_doc_sources303() { collision_doc_sources(); }
#[test] fn collision_doc_sources304() { collision_doc_sources(); }
#[test] fn collision_doc_sources305() { collision_doc_sources(); }
#[test] fn collision_doc_sources306() { collision_doc_sources(); }
#[test] fn collision_doc_sources307() { collision_doc_sources(); }
#[test] fn collision_doc_sources308() { collision_doc_sources(); }
#[test] fn collision_doc_sources309() { collision_doc_sources(); }
#[test] fn collision_doc_sources310() { collision_doc_sources(); }
#[test] fn collision_doc_sources311() { collision_doc_sources(); }
#[test] fn collision_doc_sources312() { collision_doc_sources(); }
#[test] fn collision_doc_sources313() { collision_doc_sources(); }
#[test] fn collision_doc_sources314() { collision_doc_sources(); }
#[test] fn collision_doc_sources315() { collision_doc_sources(); }
#[test] fn collision_doc_sources316() { collision_doc_sources(); }
#[test] fn collision_doc_sources317() { collision_doc_sources(); }
#[test] fn collision_doc_sources318() { collision_doc_sources(); }
#[test] fn collision_doc_sources319() { collision_doc_sources(); }
#[test] fn collision_doc_sources320() { collision_doc_sources(); }
#[test] fn collision_doc_sources321() { collision_doc_sources(); }
#[test] fn collision_doc_sources322() { collision_doc_sources(); }
#[test] fn collision_doc_sources323() { collision_doc_sources(); }
#[test] fn collision_doc_sources324() { collision_doc_sources(); }
#[test] fn collision_doc_sources325() { collision_doc_sources(); }
#[test] fn collision_doc_sources326() { collision_doc_sources(); }
#[test] fn collision_doc_sources327() { collision_doc_sources(); }
#[test] fn collision_doc_sources328() { collision_doc_sources(); }
#[test] fn collision_doc_sources329() { collision_doc_sources(); }
#[test] fn collision_doc_sources330() { collision_doc_sources(); }
#[test] fn collision_doc_sources331() { collision_doc_sources(); }
#[test] fn collision_doc_sources332() { collision_doc_sources(); }
#[test] fn collision_doc_sources333() { collision_doc_sources(); }
#[test] fn collision_doc_sources334() { collision_doc_sources(); }
#[test] fn collision_doc_sources335() { collision_doc_sources(); }
#[test] fn collision_doc_sources336() { collision_doc_sources(); }
#[test] fn collision_doc_sources337() { collision_doc_sources(); }
#[test] fn collision_doc_sources338() { collision_doc_sources(); }
#[test] fn collision_doc_sources339() { collision_doc_sources(); }
#[test] fn collision_doc_sources340() { collision_doc_sources(); }
#[test] fn collision_doc_sources341() { collision_doc_sources(); }
#[test] fn collision_doc_sources342() { collision_doc_sources(); }
#[test] fn collision_doc_sources343() { collision_doc_sources(); }
#[test] fn collision_doc_sources344() { collision_doc_sources(); }
#[test] fn collision_doc_sources345() { collision_doc_sources(); }
#[test] fn collision_doc_sources346() { collision_doc_sources(); }
#[test] fn collision_doc_sources347() { collision_doc_sources(); }
#[test] fn collision_doc_sources348() { collision_doc_sources(); }
#[test] fn collision_doc_sources349() { collision_doc_sources(); }
#[test] fn collision_doc_sources350() { collision_doc_sources(); }
#[test] fn collision_doc_sources351() { collision_doc_sources(); }
#[test] fn collision_doc_sources352() { collision_doc_sources(); }
#[test] fn collision_doc_sources353() { collision_doc_sources(); }
#[test] fn collision_doc_sources354() { collision_doc_sources(); }
#[test] fn collision_doc_sources355() { collision_doc_sources(); }
#[test] fn collision_doc_sources356() { collision_doc_sources(); }
#[test] fn collision_doc_sources357() { collision_doc_sources(); }
#[test] fn collision_doc_sources358() { collision_doc_sources(); }
#[test] fn collision_doc_sources359() { collision_doc_sources(); }
#[test] fn collision_doc_sources360() { collision_doc_sources(); }
#[test] fn collision_doc_sources361() { collision_doc_sources(); }
#[test] fn collision_doc_sources362() { collision_doc_sources(); }
#[test] fn collision_doc_sources363() { collision_doc_sources(); }
#[test] fn collision_doc_sources364() { collision_doc_sources(); }
#[test] fn collision_doc_sources365() { collision_doc_sources(); }
#[test] fn collision_doc_sources366() { collision_doc_sources(); }
#[test] fn collision_doc_sources367() { collision_doc_sources(); }
#[test] fn collision_doc_sources368() { collision_doc_sources(); }
#[test] fn collision_doc_sources369() { collision_doc_sources(); }
#[test] fn collision_doc_sources370() { collision_doc_sources(); }
#[test] fn collision_doc_sources371() { collision_doc_sources(); }
#[test] fn collision_doc_sources372() { collision_doc_sources(); }
#[test] fn collision_doc_sources373() { collision_doc_sources(); }
#[test] fn collision_doc_sources374() { collision_doc_sources(); }
#[test] fn collision_doc_sources375() { collision_doc_sources(); }
#[test] fn collision_doc_sources376() { collision_doc_sources(); }
#[test] fn collision_doc_sources377() { collision_doc_sources(); }
#[test] fn collision_doc_sources378() { collision_doc_sources(); }
#[test] fn collision_doc_sources379() { collision_doc_sources(); }
#[test] fn collision_doc_sources380() { collision_doc_sources(); }
#[test] fn collision_doc_sources381() { collision_doc_sources(); }
#[test] fn collision_doc_sources382() { collision_doc_sources(); }
#[test] fn collision_doc_sources383() { collision_doc_sources(); }
#[test] fn collision_doc_sources384() { collision_doc_sources(); }
#[test] fn collision_doc_sources385() { collision_doc_sources(); }
#[test] fn collision_doc_sources386() { collision_doc_sources(); }
#[test] fn collision_doc_sources387() { collision_doc_sources(); }
#[test] fn collision_doc_sources388() { collision_doc_sources(); }
#[test] fn collision_doc_sources389() { collision_doc_sources(); }
#[test] fn collision_doc_sources390() { collision_doc_sources(); }
#[test] fn collision_doc_sources391() { collision_doc_sources(); }
#[test] fn collision_doc_sources392() { collision_doc_sources(); }
#[test] fn collision_doc_sources393() { collision_doc_sources(); }
#[test] fn collision_doc_sources394() { collision_doc_sources(); }
#[test] fn collision_doc_sources395() { collision_doc_sources(); }
#[test] fn collision_doc_sources396() { collision_doc_sources(); }
#[test] fn collision_doc_sources397() { collision_doc_sources(); }
#[test] fn collision_doc_sources398() { collision_doc_sources(); }
#[test] fn collision_doc_sources399() { collision_doc_sources(); }

#[cargo_test]
fn collision_doc_sources() {
    // Different sources with the same package.
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
                bar2 = { path = "bar", package = "bar" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "1.0.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_unordered(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 [..]
[WARNING] output filename collision.
The lib target `bar` in package `bar v1.0.0` has the same output filename as \
the lib target `bar` in package `bar v1.0.0 ([..]/foo/bar)`.
Colliding filename is: [..]/foo/target/doc/bar/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[CHECKING] bar v1.0.0 [..]
[DOCUMENTING] bar v1.0.0 [..]
[DOCUMENTING] bar v1.0.0
[CHECKING] bar v1.0.0
[DOCUMENTING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn collision_doc_target() {
    // collision in doc with --target, doesn't fail due to orphans
    if cross_compile::disabled() {
        return;
    }

    Package::new("orphaned", "1.0.0").publish();
    Package::new("bar", "1.0.0")
        .dep("orphaned", "1.0")
        .publish();
    Package::new("bar", "2.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar2 = { version = "2.0", package="bar" }
                bar = "1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc --target")
        .arg(cross_compile::alternate())
        .with_stderr_unordered(
            "\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] orphaned v1.0.0 [..]
[DOWNLOADED] bar v2.0.0 [..]
[DOWNLOADED] bar v1.0.0 [..]
[CHECKING] orphaned v1.0.0
[DOCUMENTING] bar v2.0.0
[CHECKING] bar v2.0.0
[CHECKING] bar v1.0.0
[DOCUMENTING] foo v0.1.0 [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn collision_with_root() {
    // Check for a doc collision between a root package and a dependency.
    // In this case, `foo-macro` comes from both the workspace and crates.io.
    // This checks that the duplicate correction code doesn't choke on this
    // by removing the root unit.
    Package::new("foo-macro", "1.0.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["abc", "foo-macro"]
            "#,
        )
        .file(
            "abc/Cargo.toml",
            r#"
                [package]
                name = "abc"
                version = "1.0.0"

                [dependencies]
                foo-macro = "1.0"
            "#,
        )
        .file("abc/src/lib.rs", "")
        .file(
            "foo-macro/Cargo.toml",
            r#"
                [package]
                name = "foo-macro"
                version = "1.0.0"

                [lib]
                proc-macro = true

                [dependencies]
                abc = {path="../abc"}
            "#,
        )
        .file("foo-macro/src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_unordered("\
[UPDATING] [..]
[DOWNLOADING] crates ...
[DOWNLOADED] foo-macro v1.0.0 [..]
warning: output filename collision.
The lib target `foo-macro` in package `foo-macro v1.0.0` has the same output filename as the lib target `foo-macro` in package `foo-macro v1.0.0 [..]`.
Colliding filename is: [CWD]/target/doc/foo_macro/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[CHECKING] foo-macro v1.0.0
[DOCUMENTING] foo-macro v1.0.0
[CHECKING] abc v1.0.0 [..]
[DOCUMENTING] foo-macro v1.0.0 [..]
[DOCUMENTING] abc v1.0.0 [..]
[FINISHED] [..]
")
        .run();
}
