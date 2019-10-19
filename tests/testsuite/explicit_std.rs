//! Tests for the explicit-std feature.
use cargo_test_support::{cross_compile, is_nightly, project};

#[cargo_test]
fn explicit_gated() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                std = { stdlib = true }
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("build")
        .masquerade_as_nightly_cargo()
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  feature `explicit-std` is required

consider adding `cargo-features = [\"explicit-std\"]` to the manifest
",
        )
        .with_status(101)
        .run();
}

#[cargo_test]
fn explicit_alloc() {
    if !is_nightly() {
        // Pathless --extern is unstable.
        return;
    }

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                alloc = { stdlib = true }
            "#,
        )
        .file(
            "src/lib.rs",
            "pub fn f() -> String { alloc::string::String::new() }",
        )
        .build();
    p.cargo("build")
        .masquerade_as_nightly_cargo()
        .with_stderr(
            "\
[COMPILING] foo [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn explicit_test() {
    if !is_nightly() {
        // Pathless --extern is unstable.
        // libtest is unstable.
        return;
    }

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dev-dependencies]
                test = { stdlib = true }
            "#,
        )
        .file(
            "src/lib.rs",
            "
            #![feature(test)]
            #[cfg(test)]
            use test::black_box;
            ",
        )
        .build();
    // TODO: remove --lib when doc tests are supported
    p.cargo("test --lib").masquerade_as_nightly_cargo().run();
}

#[cargo_test]
fn explicit_incompatible() {
    // Some settings are currently not supported.
    let expected = &[
        ("stdlib=false", "`stdlib` cannot be `false` (dependency `std`)"),
        ("stdlib=true, features=[\"panic-unwind\"]", "`stdlib` currently does not support features (dependency `std`)"),
        ("stdlib=true, default-features=false", "`stdlib` currently does not support features (dependency `std`)"),
        ("stdlib=true, default_features=true", "`stdlib` currently does not support features (dependency `std`)"),
        ("stdlib=true, public=true", "`stdlib` currently does not support public/private (dependency `std`)"),
        ("stdlib=true, package=\"std\"", "`package` renaming for stdlib dependencies is currently not supported (dependency `std`)"),
    ];

    for (dep, err_msg) in expected {
        let p = project()
            .file(
                "Cargo.toml",
                &format!(
                    r#"
                    cargo-features = ["explicit-std"]
                    [package]
                    name = "foo"
                    version = "0.1.0"

                    [dependencies]
                    std = {{ {} }}
                    "#,
                    dep
                ),
            )
            .file("src/lib.rs", "")
            .build();

        p.cargo("build")
            .masquerade_as_nightly_cargo()
            .with_status(101)
            .with_stderr(&format!(
                "\
[ERROR] failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  {}
",
                err_msg
            ))
            .run();
    }
}

#[cargo_test]
fn doc_explicit() {
    // Test explicit dep for rustdoc and rustdoc --test.
    if !is_nightly() {
        // Pathless --extern is unstable.
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                alloc = { stdlib = true }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                /// Example
                /// ```
                /// alloc::string::String::new();
                /// ```
                pub fn f() -> alloc::string::String {
                    alloc::string::String::new()
                }
            "#,
        )
        .build();

    // TODO: Disabled until pathless extern is stabilized or rustdoc bug is fixed.
    // https://github.com/rust-lang/rust/pull/65314
    // p.cargo("test --doc")
    //     .masquerade_as_nightly_cargo()
    //     .run();

    p.cargo("doc").masquerade_as_nightly_cargo().run();
}

#[cargo_test]
fn explicit_build_dependency() {
    // Test explicit dep in build-dependencies.
    // This maybe isn't useful, but supported for completeness.
    if !is_nightly() {
        // Pathless --extern is unstable.
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [build-dependencies]
                alloc = {stdlib = true}

                [features]
                break-me = []
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                // Verifies it only applies to build-dependencies.
                #[cfg(feature="break-me")]
                pub fn f() {
                    let _ = alloc::string::String::new();
                }
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    let _ = alloc::string::String::new();
                }
            "#,
        )
        .build();

    p.cargo("build").masquerade_as_nightly_cargo().run();
    p.cargo("build --features=break-me")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .run();
}

#[cargo_test]
fn explicit_target_dependency() {
    // Test explicit dep as a target dependency.
    if !is_nightly() {
        // Pathless --extern is unstable.
        return;
    }
    if cross_compile::disabled() {
        return;
    }
    let alt = cross_compile::alternate();
    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [target.{}.dependencies]
                alloc = {{stdlib = true}}
                "#,
                alt
            ),
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    let _ = alloc::string::String::new();
                }
            "#,
        )
        .build();

    p.cargo("build")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .run();
    p.cargo("build --target")
        .arg(alt)
        .masquerade_as_nightly_cargo()
        .run();
}

#[cargo_test]
fn explicit_optional() {
    // Test `optional` dependency.
    if !is_nightly() {
        // Pathless --extern is unstable.
        return;
    }

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                alloc = { stdlib = true, optional = true }
                dep = { path="dep", optional = true }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    let _ = alloc::string::String::new();
                    #[cfg(feature = "dep")]
                    dep::f();
                }
            "#,
        )
        .file(
            "dep/Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "dep"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                alloc = { stdlib = true, optional = true }
            "#,
        )
        .file(
            "dep/src/lib.rs",
            r#"
                pub fn f() {
                    let _ = alloc::string::String::new();
                }
            "#,
        )
        .build();

    p.cargo("build")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .run();

    // dep's "alloc" is still not enabled.
    p.cargo("build --features=alloc,dep")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .run();

    p.cargo("build --features=alloc,dep/alloc")
        .masquerade_as_nightly_cargo()
        .run();
}
