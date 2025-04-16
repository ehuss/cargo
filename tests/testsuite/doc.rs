//! Tests for the `cargo doc` command.

use std::fs;
use std::str;

use cargo::core::compiler::RustDocFingerprint;
use cargo_test_support::prelude::*;
use cargo_test_support::registry::Package;
use cargo_test_support::str;
use cargo_test_support::{basic_lib_manifest, basic_manifest, git, project};
use cargo_test_support::{rustc_host, symlink_supported, tools};

#[cargo_test]
fn simple() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []
                build = "build.rs"
            "#,
        )
        .file("build.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn foo() {}")
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[COMPILING] foo v0.0.1 ([ROOT]/foo)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/index.html").is_file());
}

#[cargo_test]
fn doc_no_libs() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [[bin]]
                name = "foo"
                doc = false
            "#,
        )
        .file("src/main.rs", "bad code")
        .build();

    p.cargo("doc").run();
}

#[cargo_test]
fn doc_twice() {
    let p = project().file("src/lib.rs", "pub fn foo() {}").build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"
            "#,
        )
        .file("src/lib.rs", "extern crate bar; pub fn foo() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("doc")
        .with_stderr_data(
            str![[r#"
[LOCKING] 1 package to latest compatible version
[DOCUMENTING] bar v0.0.1 ([ROOT]/foo/bar)
[CHECKING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();

    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/index.html").is_file());
    assert!(p.root().join("target/doc/bar/index.html").is_file());

    // Verify that it only emits rmeta for the dependency.
    assert_eq!(p.glob("target/debug/**/*.rlib").count(), 0);
    assert_eq!(p.glob("target/debug/deps/libbar-*.rmeta").count(), 1);

    // Make sure it doesn't recompile.
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/index.html").is_file());
    assert!(p.root().join("target/doc/bar/index.html").is_file());
}

#[cargo_test]
fn doc_no_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"
            "#,
        )
        .file("src/lib.rs", "extern crate bar; pub fn foo() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("doc --no-deps")
        .with_stderr_data(str![[r#"
[LOCKING] 1 package to latest compatible version
[CHECKING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/index.html").is_file());
    assert!(!p.root().join("target/doc/bar/index.html").is_file());
}

#[cargo_test]
fn doc_only_bin() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"
            "#,
        )
        .file("src/main.rs", "extern crate bar; pub fn foo() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("doc -v").run();

    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/bar/index.html").is_file());
    assert!(p.root().join("target/doc/foo/index.html").is_file());
}

#[cargo_test]
fn doc_multiple_targets_same_name_lib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [lib]
                name = "foo_lib"
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"
                [lib]
                name = "foo_lib"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("doc --workspace")
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] document output filename collision
The lib `foo_lib` in package `foo v0.1.0 ([ROOT]/foo/foo)` has the same name as the lib `foo_lib` in package `bar v0.1.0 ([ROOT]/foo/bar)`.
Only one may be documented at once since they output to the same path.
Consider documenting only one, renaming one, or marking one with `doc = false` in Cargo.toml.

"#]])
        .run();
}

#[cargo_test]
fn doc_multiple_targets_same_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [[bin]]
                name = "foo_lib"
                path = "src/foo_lib.rs"
            "#,
        )
        .file("foo/src/foo_lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"
                [lib]
                name = "foo_lib"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("doc --workspace")
        .with_stderr_data(str![[r#"
[WARNING] output filename collision.
The bin target `foo_lib` in package `foo v0.1.0 ([ROOT]/foo/foo)` has the same output filename as the lib target `foo_lib` in package `bar v0.1.0 ([ROOT]/foo/bar)`.
Colliding filename is: [ROOT]/foo/target/doc/foo_lib/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo_lib/index.html and 1 other file

"#]].unordered())
        .run();
}

#[cargo_test]
fn doc_multiple_targets_same_name_bin() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file("foo/src/bin/foo-cli.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file("bar/src/bin/foo-cli.rs", "")
        .build();

    p.cargo("doc --workspace")
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] document output filename collision
The bin `foo-cli` in package `foo v0.1.0 ([ROOT]/foo/foo)` has the same name as the bin `foo-cli` in package `bar v0.1.0 ([ROOT]/foo/bar)`.
Only one may be documented at once since they output to the same path.
Consider documenting only one, renaming one, or marking one with `doc = false` in Cargo.toml.

"#]])
        .run();
}

#[cargo_test]
fn doc_multiple_targets_same_name_undoced() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [[bin]]
                name = "foo-cli"
            "#,
        )
        .file("foo/src/foo-cli.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"
                [[bin]]
                name = "foo-cli"
                doc = false
            "#,
        )
        .file("bar/src/foo-cli.rs", "")
        .build();

    p.cargo("doc --workspace").run();
}

#[cargo_test]
fn doc_lib_bin_same_name_documents_lib() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("Library"));
    assert!(!doc_html.contains("Binary"));
}

#[cargo_test]
fn doc_lib_bin_same_name_documents_lib_when_requested() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("doc --lib")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("Library"));
    assert!(!doc_html.contains("Binary"));
}

#[cargo_test]
fn doc_lib_bin_same_name_with_dash() {
    // Checks `doc` behavior when there is a dash in the package name, and
    // there is a lib and bin, and the lib name is inferred.
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo-bar", "1.0.0"))
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo-bar v1.0.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo_bar/index.html

"#]])
        .run();
    assert!(p.build_dir().join("doc/foo_bar/index.html").exists());
    assert!(!p.build_dir().join("doc/foo_bar/fn.main.html").exists());
}

#[cargo_test]
fn doc_lib_bin_same_name_documents_named_bin_when_requested() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("doc --bin foo")
        // The checking/documenting lines are sometimes swapped since they run
        // concurrently.
        .with_stderr_data(str![[r#"
[WARNING] output filename collision.
The bin target `foo` in package `foo v0.0.1 ([ROOT]/foo)` has the same output filename as the lib target `foo` in package `foo v0.0.1 ([ROOT]/foo)`.
Colliding filename is: [ROOT]/foo/target/doc/foo/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]].unordered())
        .run();
    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(!doc_html.contains("Library"));
    assert!(doc_html.contains("Binary"));
}

#[cargo_test]
fn doc_lib_bin_same_name_documents_bins_when_requested() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("doc --bins")
        // The checking/documenting lines are sometimes swapped since they run
        // concurrently.
        .with_stderr_data(str![[r#"
[WARNING] output filename collision.
The bin target `foo` in package `foo v0.0.1 ([ROOT]/foo)` has the same output filename as the lib target `foo` in package `foo v0.0.1 ([ROOT]/foo)`.
Colliding filename is: [ROOT]/foo/target/doc/foo/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]].unordered())
        .run();
    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(!doc_html.contains("Library"));
    assert!(doc_html.contains("Binary"));
}

#[cargo_test]
fn doc_lib_bin_example_same_name_documents_named_example_when_requested() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .file(
            "examples/ex1.rs",
            r#"
                //! Example1 documentation
                pub fn x() { f(); }
            "#,
        )
        .build();

    p.cargo("doc --example ex1")
        // The checking/documenting lines are sometimes swapped since they run
        // concurrently.
        .with_stderr_data(
            str![[r#"
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/ex1/index.html

"#]]
            .unordered(),
        )
        .run();

    let doc_html = p.read_file("target/doc/ex1/index.html");
    assert!(!doc_html.contains("Library"));
    assert!(!doc_html.contains("Binary"));
    assert!(doc_html.contains("Example1"));
}

#[cargo_test]
fn doc_lib_bin_example_same_name_documents_examples_when_requested() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                //! Binary documentation
                extern crate foo;
                fn main() {
                    foo::foo();
                }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                //! Library documentation
                pub fn foo() {}
            "#,
        )
        .file(
            "examples/ex1.rs",
            r#"
                //! Example1 documentation
                pub fn example1() { f(); }
            "#,
        )
        .file(
            "examples/ex2.rs",
            r#"
                //! Example2 documentation
                pub fn example2() { f(); }
            "#,
        )
        .build();

    p.cargo("doc --examples")
        // The checking/documenting lines are sometimes swapped since they run
        // concurrently.
        .with_stderr_data(
            str![[r#"
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/ex1/index.html and 1 other file

"#]]
            .unordered(),
        )
        .run();

    let example_doc_html_1 = p.read_file("target/doc/ex1/index.html");
    let example_doc_html_2 = p.read_file("target/doc/ex2/index.html");

    assert!(!example_doc_html_1.contains("Library"));
    assert!(!example_doc_html_1.contains("Binary"));

    assert!(!example_doc_html_2.contains("Library"));
    assert!(!example_doc_html_2.contains("Binary"));

    assert!(example_doc_html_1.contains("Example1"));
    assert!(example_doc_html_2.contains("Example2"));
}

#[cargo_test]
fn doc_dash_p() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.a]
                path = "a"
            "#,
        )
        .file("src/lib.rs", "extern crate a;")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.b]
                path = "../b"
            "#,
        )
        .file("a/src/lib.rs", "extern crate b;")
        .file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("doc -p a")
        .with_stderr_data(
            str![[r#"
[LOCKING] 2 packages to latest compatible versions
[DOCUMENTING] b v0.0.1 ([ROOT]/foo/b)
[CHECKING] b v0.0.1 ([ROOT]/foo/b)
[DOCUMENTING] a v0.0.1 ([ROOT]/foo/a)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/a/index.html

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_all_exclude() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("doc --workspace --exclude baz")
        .with_stderr_data(str![[r#"
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_all_exclude_glob() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("doc --workspace --exclude '*z'")
        .with_stderr_data(str![[r#"
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_same_name() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin/main.rs", "fn main() {}")
        .file("examples/main.rs", "fn main() {}")
        .file("tests/main.rs", "fn main() {}")
        .build();

    p.cargo("doc").run();
}

#[cargo_test(nightly, reason = "no_core, lang_items requires nightly")]
fn doc_target() {
    const TARGET: &str = "arm-unknown-linux-gnueabihf";

    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![allow(internal_features)]
                #![feature(no_core, lang_items)]
                #![no_core]

                #[lang = "sized"]
                trait Sized {}

                extern {
                    pub static A: u32;
                }
            "#,
        )
        .build();

    p.cargo("doc --verbose --target").arg(TARGET).run();
    assert!(p.root().join(&format!("target/{}/doc", TARGET)).is_dir());
    assert!(p
        .root()
        .join(&format!("target/{}/doc/foo/index.html", TARGET))
        .is_file());
}

#[cargo_test]
fn target_specific_not_documented() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [target.foo.dependencies]
                a = { path = "a" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "not rust")
        .build();

    p.cargo("doc").run();
}

#[cargo_test]
fn output_not_captured() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies]
                a = { path = "a" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file(
            "a/src/lib.rs",
            "
            /// ```
            /// `
            /// ```
            pub fn foo() {}
        ",
        )
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
...
[..]unknown start of token: `
...
"#]])
        .run();
}

#[cargo_test]
fn target_specific_documented() {
    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    edition = "2015"
                    authors = []

                    [target.foo.dependencies]
                    a = {{ path = "a" }}
                    [target.{}.dependencies]
                    a = {{ path = "a" }}
                "#,
                rustc_host()
            ),
        )
        .file(
            "src/lib.rs",
            "
            extern crate a;

            /// test
            pub fn foo() {}
        ",
        )
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file(
            "a/src/lib.rs",
            "
            /// test
            pub fn foo() {}
        ",
        )
        .build();

    p.cargo("doc").run();
}

#[cargo_test]
fn no_document_build_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [build-dependencies]
                a = { path = "a" }
            "#,
        )
        .file("src/lib.rs", "pub fn foo() {}")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file(
            "a/src/lib.rs",
            "
            /// ```
            /// ☃
            /// ```
            pub fn foo() {}
        ",
        )
        .build();

    p.cargo("doc").run();
}

#[cargo_test]
fn doc_release() {
    let p = project().file("src/lib.rs", "").build();

    p.cargo("check --release").run();
    p.cargo("doc --release -v")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustdoc [..] src/lib.rs [..]`
[FINISHED] `release` profile [optimized] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_multiple_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"

                [dependencies.baz]
                path = "baz"
            "#,
        )
        .file("src/lib.rs", "extern crate bar; pub fn foo() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.0.1"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("doc -p bar -p baz -v").run();

    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/bar/index.html").is_file());
    assert!(p.root().join("target/doc/baz/index.html").is_file());
}

#[cargo_test]
fn features() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"

                [features]
                foo = ["bar/bar"]
            "#,
        )
        .file("src/lib.rs", r#"#[cfg(feature = "foo")] pub fn foo() {}"#)
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [features]
                bar = []
            "#,
        )
        .file(
            "bar/build.rs",
            r#"
                fn main() {
                    println!("cargo::rustc-cfg=bar");
                }
            "#,
        )
        .file(
            "bar/src/lib.rs",
            r#"#[cfg(feature = "bar")] pub fn bar() {}"#,
        )
        .build();
    p.cargo("doc --features foo")
        .with_stderr_data(str![[r#"
[LOCKING] 1 package to latest compatible version
[COMPILING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/fn.foo.html").is_file());
    assert!(p.root().join("target/doc/bar/fn.bar.html").is_file());
    // Check that turning the feature off will remove the files.
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[COMPILING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    assert!(!p.root().join("target/doc/foo/fn.foo.html").is_file());
    assert!(!p.root().join("target/doc/bar/fn.bar.html").is_file());
    // And switching back will rebuild and bring them back.
    p.cargo("doc --features foo")
        .with_stderr_data(str![[r#"
[DOCUMENTING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    assert!(p.root().join("target/doc/foo/fn.foo.html").is_file());
    assert!(p.root().join("target/doc/bar/fn.bar.html").is_file());
}

#[cargo_test]
fn rerun_when_dir_removed() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                /// dox
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("doc").run();
    assert!(p.root().join("target/doc/foo/index.html").is_file());

    fs::remove_dir_all(p.root().join("target/doc/foo")).unwrap();

    p.cargo("doc").run();
    assert!(p.root().join("target/doc/foo/index.html").is_file());
}

#[cargo_test]
fn document_only_lib() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                /// dox
                pub fn foo() {}
            "#,
        )
        .file(
            "src/bin/bar.rs",
            r#"
                /// ```
                /// ☃
                /// ```
                pub fn foo() {}
                fn main() { foo(); }
            "#,
        )
        .build();
    p.cargo("doc --lib").run();
    assert!(p.root().join("target/doc/foo/index.html").is_file());
}

#[cargo_test]
fn plugins_no_use_target() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [lib]
                proc-macro = true
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("doc --target=x86_64-unknown-openbsd -v").run();
}

#[test] fn doc_all_workspace0() { doc_all_workspace(); }
#[test] fn doc_all_workspace1() { doc_all_workspace(); }
#[test] fn doc_all_workspace2() { doc_all_workspace(); }
#[test] fn doc_all_workspace3() { doc_all_workspace(); }
#[test] fn doc_all_workspace4() { doc_all_workspace(); }
#[test] fn doc_all_workspace5() { doc_all_workspace(); }
#[test] fn doc_all_workspace6() { doc_all_workspace(); }
#[test] fn doc_all_workspace7() { doc_all_workspace(); }
#[test] fn doc_all_workspace8() { doc_all_workspace(); }
#[test] fn doc_all_workspace9() { doc_all_workspace(); }
#[test] fn doc_all_workspace10() { doc_all_workspace(); }
#[test] fn doc_all_workspace11() { doc_all_workspace(); }
#[test] fn doc_all_workspace12() { doc_all_workspace(); }
#[test] fn doc_all_workspace13() { doc_all_workspace(); }
#[test] fn doc_all_workspace14() { doc_all_workspace(); }
#[test] fn doc_all_workspace15() { doc_all_workspace(); }
#[test] fn doc_all_workspace16() { doc_all_workspace(); }
#[test] fn doc_all_workspace17() { doc_all_workspace(); }
#[test] fn doc_all_workspace18() { doc_all_workspace(); }
#[test] fn doc_all_workspace19() { doc_all_workspace(); }
#[test] fn doc_all_workspace20() { doc_all_workspace(); }
#[test] fn doc_all_workspace21() { doc_all_workspace(); }
#[test] fn doc_all_workspace22() { doc_all_workspace(); }
#[test] fn doc_all_workspace23() { doc_all_workspace(); }
#[test] fn doc_all_workspace24() { doc_all_workspace(); }
#[test] fn doc_all_workspace25() { doc_all_workspace(); }
#[test] fn doc_all_workspace26() { doc_all_workspace(); }
#[test] fn doc_all_workspace27() { doc_all_workspace(); }
#[test] fn doc_all_workspace28() { doc_all_workspace(); }
#[test] fn doc_all_workspace29() { doc_all_workspace(); }
#[test] fn doc_all_workspace30() { doc_all_workspace(); }
#[test] fn doc_all_workspace31() { doc_all_workspace(); }
#[test] fn doc_all_workspace32() { doc_all_workspace(); }
#[test] fn doc_all_workspace33() { doc_all_workspace(); }
#[test] fn doc_all_workspace34() { doc_all_workspace(); }
#[test] fn doc_all_workspace35() { doc_all_workspace(); }
#[test] fn doc_all_workspace36() { doc_all_workspace(); }
#[test] fn doc_all_workspace37() { doc_all_workspace(); }
#[test] fn doc_all_workspace38() { doc_all_workspace(); }
#[test] fn doc_all_workspace39() { doc_all_workspace(); }
#[test] fn doc_all_workspace40() { doc_all_workspace(); }
#[test] fn doc_all_workspace41() { doc_all_workspace(); }
#[test] fn doc_all_workspace42() { doc_all_workspace(); }
#[test] fn doc_all_workspace43() { doc_all_workspace(); }
#[test] fn doc_all_workspace44() { doc_all_workspace(); }
#[test] fn doc_all_workspace45() { doc_all_workspace(); }
#[test] fn doc_all_workspace46() { doc_all_workspace(); }
#[test] fn doc_all_workspace47() { doc_all_workspace(); }
#[test] fn doc_all_workspace48() { doc_all_workspace(); }
#[test] fn doc_all_workspace49() { doc_all_workspace(); }
#[test] fn doc_all_workspace50() { doc_all_workspace(); }
#[test] fn doc_all_workspace51() { doc_all_workspace(); }
#[test] fn doc_all_workspace52() { doc_all_workspace(); }
#[test] fn doc_all_workspace53() { doc_all_workspace(); }
#[test] fn doc_all_workspace54() { doc_all_workspace(); }
#[test] fn doc_all_workspace55() { doc_all_workspace(); }
#[test] fn doc_all_workspace56() { doc_all_workspace(); }
#[test] fn doc_all_workspace57() { doc_all_workspace(); }
#[test] fn doc_all_workspace58() { doc_all_workspace(); }
#[test] fn doc_all_workspace59() { doc_all_workspace(); }
#[test] fn doc_all_workspace60() { doc_all_workspace(); }
#[test] fn doc_all_workspace61() { doc_all_workspace(); }
#[test] fn doc_all_workspace62() { doc_all_workspace(); }
#[test] fn doc_all_workspace63() { doc_all_workspace(); }
#[test] fn doc_all_workspace64() { doc_all_workspace(); }
#[test] fn doc_all_workspace65() { doc_all_workspace(); }
#[test] fn doc_all_workspace66() { doc_all_workspace(); }
#[test] fn doc_all_workspace67() { doc_all_workspace(); }
#[test] fn doc_all_workspace68() { doc_all_workspace(); }
#[test] fn doc_all_workspace69() { doc_all_workspace(); }
#[test] fn doc_all_workspace70() { doc_all_workspace(); }
#[test] fn doc_all_workspace71() { doc_all_workspace(); }
#[test] fn doc_all_workspace72() { doc_all_workspace(); }
#[test] fn doc_all_workspace73() { doc_all_workspace(); }
#[test] fn doc_all_workspace74() { doc_all_workspace(); }
#[test] fn doc_all_workspace75() { doc_all_workspace(); }
#[test] fn doc_all_workspace76() { doc_all_workspace(); }
#[test] fn doc_all_workspace77() { doc_all_workspace(); }
#[test] fn doc_all_workspace78() { doc_all_workspace(); }
#[test] fn doc_all_workspace79() { doc_all_workspace(); }
#[test] fn doc_all_workspace80() { doc_all_workspace(); }
#[test] fn doc_all_workspace81() { doc_all_workspace(); }
#[test] fn doc_all_workspace82() { doc_all_workspace(); }
#[test] fn doc_all_workspace83() { doc_all_workspace(); }
#[test] fn doc_all_workspace84() { doc_all_workspace(); }
#[test] fn doc_all_workspace85() { doc_all_workspace(); }
#[test] fn doc_all_workspace86() { doc_all_workspace(); }
#[test] fn doc_all_workspace87() { doc_all_workspace(); }
#[test] fn doc_all_workspace88() { doc_all_workspace(); }
#[test] fn doc_all_workspace89() { doc_all_workspace(); }
#[test] fn doc_all_workspace90() { doc_all_workspace(); }
#[test] fn doc_all_workspace91() { doc_all_workspace(); }
#[test] fn doc_all_workspace92() { doc_all_workspace(); }
#[test] fn doc_all_workspace93() { doc_all_workspace(); }
#[test] fn doc_all_workspace94() { doc_all_workspace(); }
#[test] fn doc_all_workspace95() { doc_all_workspace(); }
#[test] fn doc_all_workspace96() { doc_all_workspace(); }
#[test] fn doc_all_workspace97() { doc_all_workspace(); }
#[test] fn doc_all_workspace98() { doc_all_workspace(); }
#[test] fn doc_all_workspace99() { doc_all_workspace(); }
#[test] fn doc_all_workspace100() { doc_all_workspace(); }
#[test] fn doc_all_workspace101() { doc_all_workspace(); }
#[test] fn doc_all_workspace102() { doc_all_workspace(); }
#[test] fn doc_all_workspace103() { doc_all_workspace(); }
#[test] fn doc_all_workspace104() { doc_all_workspace(); }
#[test] fn doc_all_workspace105() { doc_all_workspace(); }
#[test] fn doc_all_workspace106() { doc_all_workspace(); }
#[test] fn doc_all_workspace107() { doc_all_workspace(); }
#[test] fn doc_all_workspace108() { doc_all_workspace(); }
#[test] fn doc_all_workspace109() { doc_all_workspace(); }
#[test] fn doc_all_workspace110() { doc_all_workspace(); }
#[test] fn doc_all_workspace111() { doc_all_workspace(); }
#[test] fn doc_all_workspace112() { doc_all_workspace(); }
#[test] fn doc_all_workspace113() { doc_all_workspace(); }
#[test] fn doc_all_workspace114() { doc_all_workspace(); }
#[test] fn doc_all_workspace115() { doc_all_workspace(); }
#[test] fn doc_all_workspace116() { doc_all_workspace(); }
#[test] fn doc_all_workspace117() { doc_all_workspace(); }
#[test] fn doc_all_workspace118() { doc_all_workspace(); }
#[test] fn doc_all_workspace119() { doc_all_workspace(); }
#[test] fn doc_all_workspace120() { doc_all_workspace(); }
#[test] fn doc_all_workspace121() { doc_all_workspace(); }
#[test] fn doc_all_workspace122() { doc_all_workspace(); }
#[test] fn doc_all_workspace123() { doc_all_workspace(); }
#[test] fn doc_all_workspace124() { doc_all_workspace(); }
#[test] fn doc_all_workspace125() { doc_all_workspace(); }
#[test] fn doc_all_workspace126() { doc_all_workspace(); }
#[test] fn doc_all_workspace127() { doc_all_workspace(); }
#[test] fn doc_all_workspace128() { doc_all_workspace(); }
#[test] fn doc_all_workspace129() { doc_all_workspace(); }
#[test] fn doc_all_workspace130() { doc_all_workspace(); }
#[test] fn doc_all_workspace131() { doc_all_workspace(); }
#[test] fn doc_all_workspace132() { doc_all_workspace(); }
#[test] fn doc_all_workspace133() { doc_all_workspace(); }
#[test] fn doc_all_workspace134() { doc_all_workspace(); }
#[test] fn doc_all_workspace135() { doc_all_workspace(); }
#[test] fn doc_all_workspace136() { doc_all_workspace(); }
#[test] fn doc_all_workspace137() { doc_all_workspace(); }
#[test] fn doc_all_workspace138() { doc_all_workspace(); }
#[test] fn doc_all_workspace139() { doc_all_workspace(); }
#[test] fn doc_all_workspace140() { doc_all_workspace(); }
#[test] fn doc_all_workspace141() { doc_all_workspace(); }
#[test] fn doc_all_workspace142() { doc_all_workspace(); }
#[test] fn doc_all_workspace143() { doc_all_workspace(); }
#[test] fn doc_all_workspace144() { doc_all_workspace(); }
#[test] fn doc_all_workspace145() { doc_all_workspace(); }
#[test] fn doc_all_workspace146() { doc_all_workspace(); }
#[test] fn doc_all_workspace147() { doc_all_workspace(); }
#[test] fn doc_all_workspace148() { doc_all_workspace(); }
#[test] fn doc_all_workspace149() { doc_all_workspace(); }
#[test] fn doc_all_workspace150() { doc_all_workspace(); }
#[test] fn doc_all_workspace151() { doc_all_workspace(); }
#[test] fn doc_all_workspace152() { doc_all_workspace(); }
#[test] fn doc_all_workspace153() { doc_all_workspace(); }
#[test] fn doc_all_workspace154() { doc_all_workspace(); }
#[test] fn doc_all_workspace155() { doc_all_workspace(); }
#[test] fn doc_all_workspace156() { doc_all_workspace(); }
#[test] fn doc_all_workspace157() { doc_all_workspace(); }
#[test] fn doc_all_workspace158() { doc_all_workspace(); }
#[test] fn doc_all_workspace159() { doc_all_workspace(); }
#[test] fn doc_all_workspace160() { doc_all_workspace(); }
#[test] fn doc_all_workspace161() { doc_all_workspace(); }
#[test] fn doc_all_workspace162() { doc_all_workspace(); }
#[test] fn doc_all_workspace163() { doc_all_workspace(); }
#[test] fn doc_all_workspace164() { doc_all_workspace(); }
#[test] fn doc_all_workspace165() { doc_all_workspace(); }
#[test] fn doc_all_workspace166() { doc_all_workspace(); }
#[test] fn doc_all_workspace167() { doc_all_workspace(); }
#[test] fn doc_all_workspace168() { doc_all_workspace(); }
#[test] fn doc_all_workspace169() { doc_all_workspace(); }
#[test] fn doc_all_workspace170() { doc_all_workspace(); }
#[test] fn doc_all_workspace171() { doc_all_workspace(); }
#[test] fn doc_all_workspace172() { doc_all_workspace(); }
#[test] fn doc_all_workspace173() { doc_all_workspace(); }
#[test] fn doc_all_workspace174() { doc_all_workspace(); }
#[test] fn doc_all_workspace175() { doc_all_workspace(); }
#[test] fn doc_all_workspace176() { doc_all_workspace(); }
#[test] fn doc_all_workspace177() { doc_all_workspace(); }
#[test] fn doc_all_workspace178() { doc_all_workspace(); }
#[test] fn doc_all_workspace179() { doc_all_workspace(); }
#[test] fn doc_all_workspace180() { doc_all_workspace(); }
#[test] fn doc_all_workspace181() { doc_all_workspace(); }
#[test] fn doc_all_workspace182() { doc_all_workspace(); }
#[test] fn doc_all_workspace183() { doc_all_workspace(); }
#[test] fn doc_all_workspace184() { doc_all_workspace(); }
#[test] fn doc_all_workspace185() { doc_all_workspace(); }
#[test] fn doc_all_workspace186() { doc_all_workspace(); }
#[test] fn doc_all_workspace187() { doc_all_workspace(); }
#[test] fn doc_all_workspace188() { doc_all_workspace(); }
#[test] fn doc_all_workspace189() { doc_all_workspace(); }
#[test] fn doc_all_workspace190() { doc_all_workspace(); }
#[test] fn doc_all_workspace191() { doc_all_workspace(); }
#[test] fn doc_all_workspace192() { doc_all_workspace(); }
#[test] fn doc_all_workspace193() { doc_all_workspace(); }
#[test] fn doc_all_workspace194() { doc_all_workspace(); }
#[test] fn doc_all_workspace195() { doc_all_workspace(); }
#[test] fn doc_all_workspace196() { doc_all_workspace(); }
#[test] fn doc_all_workspace197() { doc_all_workspace(); }
#[test] fn doc_all_workspace198() { doc_all_workspace(); }
#[test] fn doc_all_workspace199() { doc_all_workspace(); }
#[test] fn doc_all_workspace200() { doc_all_workspace(); }
#[test] fn doc_all_workspace201() { doc_all_workspace(); }
#[test] fn doc_all_workspace202() { doc_all_workspace(); }
#[test] fn doc_all_workspace203() { doc_all_workspace(); }
#[test] fn doc_all_workspace204() { doc_all_workspace(); }
#[test] fn doc_all_workspace205() { doc_all_workspace(); }
#[test] fn doc_all_workspace206() { doc_all_workspace(); }
#[test] fn doc_all_workspace207() { doc_all_workspace(); }
#[test] fn doc_all_workspace208() { doc_all_workspace(); }
#[test] fn doc_all_workspace209() { doc_all_workspace(); }
#[test] fn doc_all_workspace210() { doc_all_workspace(); }
#[test] fn doc_all_workspace211() { doc_all_workspace(); }
#[test] fn doc_all_workspace212() { doc_all_workspace(); }
#[test] fn doc_all_workspace213() { doc_all_workspace(); }
#[test] fn doc_all_workspace214() { doc_all_workspace(); }
#[test] fn doc_all_workspace215() { doc_all_workspace(); }
#[test] fn doc_all_workspace216() { doc_all_workspace(); }
#[test] fn doc_all_workspace217() { doc_all_workspace(); }
#[test] fn doc_all_workspace218() { doc_all_workspace(); }
#[test] fn doc_all_workspace219() { doc_all_workspace(); }
#[test] fn doc_all_workspace220() { doc_all_workspace(); }
#[test] fn doc_all_workspace221() { doc_all_workspace(); }
#[test] fn doc_all_workspace222() { doc_all_workspace(); }
#[test] fn doc_all_workspace223() { doc_all_workspace(); }
#[test] fn doc_all_workspace224() { doc_all_workspace(); }
#[test] fn doc_all_workspace225() { doc_all_workspace(); }
#[test] fn doc_all_workspace226() { doc_all_workspace(); }
#[test] fn doc_all_workspace227() { doc_all_workspace(); }
#[test] fn doc_all_workspace228() { doc_all_workspace(); }
#[test] fn doc_all_workspace229() { doc_all_workspace(); }
#[test] fn doc_all_workspace230() { doc_all_workspace(); }
#[test] fn doc_all_workspace231() { doc_all_workspace(); }
#[test] fn doc_all_workspace232() { doc_all_workspace(); }
#[test] fn doc_all_workspace233() { doc_all_workspace(); }
#[test] fn doc_all_workspace234() { doc_all_workspace(); }
#[test] fn doc_all_workspace235() { doc_all_workspace(); }
#[test] fn doc_all_workspace236() { doc_all_workspace(); }
#[test] fn doc_all_workspace237() { doc_all_workspace(); }
#[test] fn doc_all_workspace238() { doc_all_workspace(); }
#[test] fn doc_all_workspace239() { doc_all_workspace(); }
#[test] fn doc_all_workspace240() { doc_all_workspace(); }
#[test] fn doc_all_workspace241() { doc_all_workspace(); }
#[test] fn doc_all_workspace242() { doc_all_workspace(); }
#[test] fn doc_all_workspace243() { doc_all_workspace(); }
#[test] fn doc_all_workspace244() { doc_all_workspace(); }
#[test] fn doc_all_workspace245() { doc_all_workspace(); }
#[test] fn doc_all_workspace246() { doc_all_workspace(); }
#[test] fn doc_all_workspace247() { doc_all_workspace(); }
#[test] fn doc_all_workspace248() { doc_all_workspace(); }
#[test] fn doc_all_workspace249() { doc_all_workspace(); }
#[test] fn doc_all_workspace250() { doc_all_workspace(); }
#[test] fn doc_all_workspace251() { doc_all_workspace(); }
#[test] fn doc_all_workspace252() { doc_all_workspace(); }
#[test] fn doc_all_workspace253() { doc_all_workspace(); }
#[test] fn doc_all_workspace254() { doc_all_workspace(); }
#[test] fn doc_all_workspace255() { doc_all_workspace(); }
#[test] fn doc_all_workspace256() { doc_all_workspace(); }
#[test] fn doc_all_workspace257() { doc_all_workspace(); }
#[test] fn doc_all_workspace258() { doc_all_workspace(); }
#[test] fn doc_all_workspace259() { doc_all_workspace(); }
#[test] fn doc_all_workspace260() { doc_all_workspace(); }
#[test] fn doc_all_workspace261() { doc_all_workspace(); }
#[test] fn doc_all_workspace262() { doc_all_workspace(); }
#[test] fn doc_all_workspace263() { doc_all_workspace(); }
#[test] fn doc_all_workspace264() { doc_all_workspace(); }
#[test] fn doc_all_workspace265() { doc_all_workspace(); }
#[test] fn doc_all_workspace266() { doc_all_workspace(); }
#[test] fn doc_all_workspace267() { doc_all_workspace(); }
#[test] fn doc_all_workspace268() { doc_all_workspace(); }
#[test] fn doc_all_workspace269() { doc_all_workspace(); }
#[test] fn doc_all_workspace270() { doc_all_workspace(); }
#[test] fn doc_all_workspace271() { doc_all_workspace(); }
#[test] fn doc_all_workspace272() { doc_all_workspace(); }
#[test] fn doc_all_workspace273() { doc_all_workspace(); }
#[test] fn doc_all_workspace274() { doc_all_workspace(); }
#[test] fn doc_all_workspace275() { doc_all_workspace(); }
#[test] fn doc_all_workspace276() { doc_all_workspace(); }
#[test] fn doc_all_workspace277() { doc_all_workspace(); }
#[test] fn doc_all_workspace278() { doc_all_workspace(); }
#[test] fn doc_all_workspace279() { doc_all_workspace(); }
#[test] fn doc_all_workspace280() { doc_all_workspace(); }
#[test] fn doc_all_workspace281() { doc_all_workspace(); }
#[test] fn doc_all_workspace282() { doc_all_workspace(); }
#[test] fn doc_all_workspace283() { doc_all_workspace(); }
#[test] fn doc_all_workspace284() { doc_all_workspace(); }
#[test] fn doc_all_workspace285() { doc_all_workspace(); }
#[test] fn doc_all_workspace286() { doc_all_workspace(); }
#[test] fn doc_all_workspace287() { doc_all_workspace(); }
#[test] fn doc_all_workspace288() { doc_all_workspace(); }
#[test] fn doc_all_workspace289() { doc_all_workspace(); }
#[test] fn doc_all_workspace290() { doc_all_workspace(); }
#[test] fn doc_all_workspace291() { doc_all_workspace(); }
#[test] fn doc_all_workspace292() { doc_all_workspace(); }
#[test] fn doc_all_workspace293() { doc_all_workspace(); }
#[test] fn doc_all_workspace294() { doc_all_workspace(); }
#[test] fn doc_all_workspace295() { doc_all_workspace(); }
#[test] fn doc_all_workspace296() { doc_all_workspace(); }
#[test] fn doc_all_workspace297() { doc_all_workspace(); }
#[test] fn doc_all_workspace298() { doc_all_workspace(); }
#[test] fn doc_all_workspace299() { doc_all_workspace(); }
#[test] fn doc_all_workspace300() { doc_all_workspace(); }
#[test] fn doc_all_workspace301() { doc_all_workspace(); }
#[test] fn doc_all_workspace302() { doc_all_workspace(); }
#[test] fn doc_all_workspace303() { doc_all_workspace(); }
#[test] fn doc_all_workspace304() { doc_all_workspace(); }
#[test] fn doc_all_workspace305() { doc_all_workspace(); }
#[test] fn doc_all_workspace306() { doc_all_workspace(); }
#[test] fn doc_all_workspace307() { doc_all_workspace(); }
#[test] fn doc_all_workspace308() { doc_all_workspace(); }
#[test] fn doc_all_workspace309() { doc_all_workspace(); }
#[test] fn doc_all_workspace310() { doc_all_workspace(); }
#[test] fn doc_all_workspace311() { doc_all_workspace(); }
#[test] fn doc_all_workspace312() { doc_all_workspace(); }
#[test] fn doc_all_workspace313() { doc_all_workspace(); }
#[test] fn doc_all_workspace314() { doc_all_workspace(); }
#[test] fn doc_all_workspace315() { doc_all_workspace(); }
#[test] fn doc_all_workspace316() { doc_all_workspace(); }
#[test] fn doc_all_workspace317() { doc_all_workspace(); }
#[test] fn doc_all_workspace318() { doc_all_workspace(); }
#[test] fn doc_all_workspace319() { doc_all_workspace(); }
#[test] fn doc_all_workspace320() { doc_all_workspace(); }
#[test] fn doc_all_workspace321() { doc_all_workspace(); }
#[test] fn doc_all_workspace322() { doc_all_workspace(); }
#[test] fn doc_all_workspace323() { doc_all_workspace(); }
#[test] fn doc_all_workspace324() { doc_all_workspace(); }
#[test] fn doc_all_workspace325() { doc_all_workspace(); }
#[test] fn doc_all_workspace326() { doc_all_workspace(); }
#[test] fn doc_all_workspace327() { doc_all_workspace(); }
#[test] fn doc_all_workspace328() { doc_all_workspace(); }
#[test] fn doc_all_workspace329() { doc_all_workspace(); }
#[test] fn doc_all_workspace330() { doc_all_workspace(); }
#[test] fn doc_all_workspace331() { doc_all_workspace(); }
#[test] fn doc_all_workspace332() { doc_all_workspace(); }
#[test] fn doc_all_workspace333() { doc_all_workspace(); }
#[test] fn doc_all_workspace334() { doc_all_workspace(); }
#[test] fn doc_all_workspace335() { doc_all_workspace(); }
#[test] fn doc_all_workspace336() { doc_all_workspace(); }
#[test] fn doc_all_workspace337() { doc_all_workspace(); }
#[test] fn doc_all_workspace338() { doc_all_workspace(); }
#[test] fn doc_all_workspace339() { doc_all_workspace(); }
#[test] fn doc_all_workspace340() { doc_all_workspace(); }
#[test] fn doc_all_workspace341() { doc_all_workspace(); }
#[test] fn doc_all_workspace342() { doc_all_workspace(); }
#[test] fn doc_all_workspace343() { doc_all_workspace(); }
#[test] fn doc_all_workspace344() { doc_all_workspace(); }
#[test] fn doc_all_workspace345() { doc_all_workspace(); }
#[test] fn doc_all_workspace346() { doc_all_workspace(); }
#[test] fn doc_all_workspace347() { doc_all_workspace(); }
#[test] fn doc_all_workspace348() { doc_all_workspace(); }
#[test] fn doc_all_workspace349() { doc_all_workspace(); }
#[test] fn doc_all_workspace350() { doc_all_workspace(); }
#[test] fn doc_all_workspace351() { doc_all_workspace(); }
#[test] fn doc_all_workspace352() { doc_all_workspace(); }
#[test] fn doc_all_workspace353() { doc_all_workspace(); }
#[test] fn doc_all_workspace354() { doc_all_workspace(); }
#[test] fn doc_all_workspace355() { doc_all_workspace(); }
#[test] fn doc_all_workspace356() { doc_all_workspace(); }
#[test] fn doc_all_workspace357() { doc_all_workspace(); }
#[test] fn doc_all_workspace358() { doc_all_workspace(); }
#[test] fn doc_all_workspace359() { doc_all_workspace(); }
#[test] fn doc_all_workspace360() { doc_all_workspace(); }
#[test] fn doc_all_workspace361() { doc_all_workspace(); }
#[test] fn doc_all_workspace362() { doc_all_workspace(); }
#[test] fn doc_all_workspace363() { doc_all_workspace(); }
#[test] fn doc_all_workspace364() { doc_all_workspace(); }
#[test] fn doc_all_workspace365() { doc_all_workspace(); }
#[test] fn doc_all_workspace366() { doc_all_workspace(); }
#[test] fn doc_all_workspace367() { doc_all_workspace(); }
#[test] fn doc_all_workspace368() { doc_all_workspace(); }
#[test] fn doc_all_workspace369() { doc_all_workspace(); }
#[test] fn doc_all_workspace370() { doc_all_workspace(); }
#[test] fn doc_all_workspace371() { doc_all_workspace(); }
#[test] fn doc_all_workspace372() { doc_all_workspace(); }
#[test] fn doc_all_workspace373() { doc_all_workspace(); }
#[test] fn doc_all_workspace374() { doc_all_workspace(); }
#[test] fn doc_all_workspace375() { doc_all_workspace(); }
#[test] fn doc_all_workspace376() { doc_all_workspace(); }
#[test] fn doc_all_workspace377() { doc_all_workspace(); }
#[test] fn doc_all_workspace378() { doc_all_workspace(); }
#[test] fn doc_all_workspace379() { doc_all_workspace(); }
#[test] fn doc_all_workspace380() { doc_all_workspace(); }
#[test] fn doc_all_workspace381() { doc_all_workspace(); }
#[test] fn doc_all_workspace382() { doc_all_workspace(); }
#[test] fn doc_all_workspace383() { doc_all_workspace(); }
#[test] fn doc_all_workspace384() { doc_all_workspace(); }
#[test] fn doc_all_workspace385() { doc_all_workspace(); }
#[test] fn doc_all_workspace386() { doc_all_workspace(); }
#[test] fn doc_all_workspace387() { doc_all_workspace(); }
#[test] fn doc_all_workspace388() { doc_all_workspace(); }
#[test] fn doc_all_workspace389() { doc_all_workspace(); }
#[test] fn doc_all_workspace390() { doc_all_workspace(); }
#[test] fn doc_all_workspace391() { doc_all_workspace(); }
#[test] fn doc_all_workspace392() { doc_all_workspace(); }
#[test] fn doc_all_workspace393() { doc_all_workspace(); }
#[test] fn doc_all_workspace394() { doc_all_workspace(); }
#[test] fn doc_all_workspace395() { doc_all_workspace(); }
#[test] fn doc_all_workspace396() { doc_all_workspace(); }
#[test] fn doc_all_workspace397() { doc_all_workspace(); }
#[test] fn doc_all_workspace398() { doc_all_workspace(); }
#[test] fn doc_all_workspace399() { doc_all_workspace(); }

#[cargo_test]
fn doc_all_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"

                [dependencies]
                bar = { path = "bar" }

                [workspace]
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    // The order in which bar is compiled or documented is not deterministic
    p.cargo("doc --workspace")
        .with_stderr_data(
            str![[r#"
[CHECKING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html and 1 other file

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_all_workspace_verbose() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"

                [dependencies]
                bar = { path = "bar" }

                [workspace]
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    // The order in which bar is compiled or documented is not deterministic
    p.cargo("doc --workspace -v")
        .with_stderr_data(
            str![[r#"
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo)
[RUNNING] `rustdoc [..]
[CHECKING] bar v0.1.0 ([ROOT]/foo/bar)
[RUNNING] `rustc [..]
[RUNNING] `rustdoc [..]
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_all_virtual_manifest() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    // The order in which bar and baz are documented is not guaranteed
    p.cargo("doc --workspace")
        .with_stderr_data(
            str![[r#"
[DOCUMENTING] baz v0.1.0 ([ROOT]/foo/baz)
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html and 1 other file

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_virtual_manifest_all_implied() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    // The order in which bar and baz are documented is not guaranteed
    p.cargo("doc")
        .with_stderr_data(
            str![[r#"
[GENERATED] [ROOT]/foo/target/doc/bar/index.html and 1 other file
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] baz v0.1.0 ([ROOT]/foo/baz)

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_virtual_manifest_one_project() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("doc -p bar")
        .with_stderr_data(str![[r#"
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_virtual_manifest_glob() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {  break_the_build(); }")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("doc -p '*z'")
        .with_stderr_data(str![[r#"
[DOCUMENTING] baz v0.1.0 ([ROOT]/foo/baz)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/baz/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_all_member_dependency_same_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar"]
            "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"

                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    Package::new("bar", "0.1.0").publish();

    p.cargo("doc --workspace")
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 (registry `dummy-registry`)
[WARNING] output filename collision.
The lib target `bar` in package `bar v0.1.0` has the same output filename as the lib target `bar` in package `bar v0.1.0 ([ROOT]/foo/bar)`.
Colliding filename is: [ROOT]/foo/target/doc/bar/index.html
The targets should have unique names.
This is a known bug where multiple crates with the same name use
the same path; see <https://github.com/rust-lang/cargo/issues/6313>.
[DOCUMENTING] bar v0.1.0
[CHECKING] bar v0.1.0
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/bar/index.html

"#]].unordered())
        .run();
}

#[cargo_test]
fn doc_workspace_open_help_message() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file("foo/Cargo.toml", &basic_manifest("foo", "0.1.0"))
        .file("foo/src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    // The order in which bar is compiled or documented is not deterministic
    p.cargo("doc --workspace --open")
        .env("BROWSER", tools::echo())
        .with_stderr_data(
            str![[r#"
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo/foo)
[DOCUMENTING] bar v0.1.0 ([ROOT]/foo/bar)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[OPENING] [ROOT]/foo/target/doc/bar/index.html

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test(nightly, reason = "-Zextern-html-root-url is unstable")]
fn doc_extern_map_local() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file("src/lib.rs", "")
        .file(".cargo/config.toml", "doc.extern-map.std = 'local'")
        .build();

    p.cargo("doc -v --no-deps -Zrustdoc-map --open")
        .env("BROWSER", tools::echo())
        .masquerade_as_nightly_cargo(&["rustdoc-map"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo)
[RUNNING] `rustdoc --edition=2015 --crate-type lib --crate-name foo src/lib.rs [..]--crate-version 0.1.0`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[OPENING] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
}

#[cargo_test]
fn open_no_doc_crate() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            edition = "2015"
            authors = []

            [lib]
            doc = false
        "#,
        )
        .file("src/lib.rs", "#[cfg(feature)] pub fn f();")
        .build();

    p.cargo("doc --open")
        .env("BROWSER", "do_not_run_me")
        .with_status(101)
        .with_stderr_data(str![[r#"
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[ERROR] cannot open specified crate's documentation: no documentation generated

"#]])
        .run();
}

#[cargo_test]
fn doc_workspace_open_different_library_and_package_names() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [lib]
                name = "foolib"
            "#,
        )
        .file("foo/src/lib.rs", "")
        .build();

    p.cargo("doc --open")
        .env("BROWSER", tools::echo())
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[OPENING] [ROOT]/foo/target/doc/foolib/index.html

"#]])
        .with_stdout_data(str![[r#"
[ROOT]/foo/target/doc/foolib/index.html

"#]])
        .run();

    p.change_file(
        ".cargo/config.toml",
        &format!(
            r#"
                [doc]
                browser = ["{}", "a"]
            "#,
            tools::echo().display().to_string().replace('\\', "\\\\")
        ),
    );

    // check that the cargo config overrides the browser env var
    p.cargo("doc --open")
        .env("BROWSER", "do_not_run_me")
        .with_stdout_data(str![[r#"
a [ROOT]/foo/target/doc/foolib/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_workspace_open_binary() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [[bin]]
                name = "foobin"
                path = "src/main.rs"
            "#,
        )
        .file("foo/src/main.rs", "")
        .build();

    p.cargo("doc --open")
        .env("BROWSER", tools::echo())
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[OPENING] [ROOT]/foo/target/doc/foobin/index.html

"#]])
        .run();
}

#[cargo_test]
fn doc_workspace_open_binary_and_library() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"
                [lib]
                name = "foolib"
                [[bin]]
                name = "foobin"
                path = "src/main.rs"
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file("foo/src/main.rs", "")
        .build();

    p.cargo("doc --open")
        .env("BROWSER", tools::echo())
        .with_stderr_data(
            str![[r#"
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo/foo)
[CHECKING] foo v0.1.0 ([ROOT]/foo/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[OPENING] [ROOT]/foo/target/doc/foolib/index.html

"#]]
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn doc_edition() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
                edition = "2018"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc -v")
        .with_stderr_data(str![[r#"
...
[RUNNING] `rustdoc [..]--edition=2018[..]
...
"#]])
        .run();

    p.cargo("test -v")
        .with_stderr_data(str![[r#"
...
[RUNNING] `rustdoc [..]--edition=2018[..]
...
"#]])
        .run();
}

#[cargo_test]
fn doc_target_edition() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [lib]
                edition = "2018"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc -v")
        .with_stderr_data(str![[r#"
...
[RUNNING] `rustdoc [..]--edition=2018[..]
...
"#]])
        .run();

    p.cargo("test -v")
        .with_stderr_data(str![[r#"
...
[RUNNING] `rustdoc [..]--edition=2018[..]
...
"#]])
        .run();
}

// Tests an issue where depending on different versions of the same crate depending on `cfg`s
// caused `cargo doc` to fail.
#[cargo_test]
fn issue_5345() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [target.'cfg(all(windows, target_arch = "x86"))'.dependencies]
                bar = "0.1"

                [target.'cfg(not(all(windows, target_arch = "x86")))'.dependencies]
                bar = "0.2"
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .build();
    Package::new("bar", "0.1.0").publish();
    Package::new("bar", "0.2.0").publish();

    foo.cargo("check").run();
    foo.cargo("doc").run();
}

#[cargo_test]
fn doc_private_items() {
    let foo = project()
        .file("src/lib.rs", "mod private { fn private_item() {} }")
        .build();
    foo.cargo("doc --document-private-items").run();

    assert!(foo.root().join("target/doc").is_dir());
    assert!(foo
        .root()
        .join("target/doc/foo/private/index.html")
        .is_file());
}

#[cargo_test]
fn doc_private_ws() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["a", "b"]
            "#,
        )
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "fn p() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/lib.rs", "fn p2() {}")
        .file("b/src/bin/b-cli.rs", "fn main() {}")
        .build();
    p.cargo("doc --workspace --bins --lib --document-private-items -v")
        .with_stderr_data(
            str![[r#"
[DOCUMENTING] b v0.0.1 ([ROOT]/foo/b)
[CHECKING] b v0.0.1 ([ROOT]/foo/b)
[DOCUMENTING] a v0.0.1 ([ROOT]/foo/a)
[RUNNING] `rustdoc [..] a/src/lib.rs [..]--document-private-items[..]
[RUNNING] `rustc [..]
[WARNING] function `p2` is never used
...
[RUNNING] `rustdoc [..] b/src/lib.rs [..]--document-private-items[..]
[WARNING] `b` (lib) generated 1 warning
[RUNNING] `rustdoc [..] b/src/bin/b-cli.rs [..]--document-private-items[..]
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/a/index.html
[GENERATED] [ROOT]/foo/target/doc/b/index.html
[GENERATED] [ROOT]/foo/target/doc/b_cli/index.html

"#]]
            .unordered(),
        )
        .run();
}

const BAD_INTRA_LINK_LIB: &str = r#"
#![deny(rustdoc::broken_intra_doc_links)]

/// [bad_link]
pub fn foo() {}
"#;

#[test] fn doc_cap_lints0() { doc_cap_lints(); }
#[test] fn doc_cap_lints1() { doc_cap_lints(); }
#[test] fn doc_cap_lints2() { doc_cap_lints(); }
#[test] fn doc_cap_lints3() { doc_cap_lints(); }
#[test] fn doc_cap_lints4() { doc_cap_lints(); }
#[test] fn doc_cap_lints5() { doc_cap_lints(); }
#[test] fn doc_cap_lints6() { doc_cap_lints(); }
#[test] fn doc_cap_lints7() { doc_cap_lints(); }
#[test] fn doc_cap_lints8() { doc_cap_lints(); }
#[test] fn doc_cap_lints9() { doc_cap_lints(); }
#[test] fn doc_cap_lints10() { doc_cap_lints(); }
#[test] fn doc_cap_lints11() { doc_cap_lints(); }
#[test] fn doc_cap_lints12() { doc_cap_lints(); }
#[test] fn doc_cap_lints13() { doc_cap_lints(); }
#[test] fn doc_cap_lints14() { doc_cap_lints(); }
#[test] fn doc_cap_lints15() { doc_cap_lints(); }
#[test] fn doc_cap_lints16() { doc_cap_lints(); }
#[test] fn doc_cap_lints17() { doc_cap_lints(); }
#[test] fn doc_cap_lints18() { doc_cap_lints(); }
#[test] fn doc_cap_lints19() { doc_cap_lints(); }
#[test] fn doc_cap_lints20() { doc_cap_lints(); }
#[test] fn doc_cap_lints21() { doc_cap_lints(); }
#[test] fn doc_cap_lints22() { doc_cap_lints(); }
#[test] fn doc_cap_lints23() { doc_cap_lints(); }
#[test] fn doc_cap_lints24() { doc_cap_lints(); }
#[test] fn doc_cap_lints25() { doc_cap_lints(); }
#[test] fn doc_cap_lints26() { doc_cap_lints(); }
#[test] fn doc_cap_lints27() { doc_cap_lints(); }
#[test] fn doc_cap_lints28() { doc_cap_lints(); }
#[test] fn doc_cap_lints29() { doc_cap_lints(); }
#[test] fn doc_cap_lints30() { doc_cap_lints(); }
#[test] fn doc_cap_lints31() { doc_cap_lints(); }
#[test] fn doc_cap_lints32() { doc_cap_lints(); }
#[test] fn doc_cap_lints33() { doc_cap_lints(); }
#[test] fn doc_cap_lints34() { doc_cap_lints(); }
#[test] fn doc_cap_lints35() { doc_cap_lints(); }
#[test] fn doc_cap_lints36() { doc_cap_lints(); }
#[test] fn doc_cap_lints37() { doc_cap_lints(); }
#[test] fn doc_cap_lints38() { doc_cap_lints(); }
#[test] fn doc_cap_lints39() { doc_cap_lints(); }
#[test] fn doc_cap_lints40() { doc_cap_lints(); }
#[test] fn doc_cap_lints41() { doc_cap_lints(); }
#[test] fn doc_cap_lints42() { doc_cap_lints(); }
#[test] fn doc_cap_lints43() { doc_cap_lints(); }
#[test] fn doc_cap_lints44() { doc_cap_lints(); }
#[test] fn doc_cap_lints45() { doc_cap_lints(); }
#[test] fn doc_cap_lints46() { doc_cap_lints(); }
#[test] fn doc_cap_lints47() { doc_cap_lints(); }
#[test] fn doc_cap_lints48() { doc_cap_lints(); }
#[test] fn doc_cap_lints49() { doc_cap_lints(); }
#[test] fn doc_cap_lints50() { doc_cap_lints(); }
#[test] fn doc_cap_lints51() { doc_cap_lints(); }
#[test] fn doc_cap_lints52() { doc_cap_lints(); }
#[test] fn doc_cap_lints53() { doc_cap_lints(); }
#[test] fn doc_cap_lints54() { doc_cap_lints(); }
#[test] fn doc_cap_lints55() { doc_cap_lints(); }
#[test] fn doc_cap_lints56() { doc_cap_lints(); }
#[test] fn doc_cap_lints57() { doc_cap_lints(); }
#[test] fn doc_cap_lints58() { doc_cap_lints(); }
#[test] fn doc_cap_lints59() { doc_cap_lints(); }
#[test] fn doc_cap_lints60() { doc_cap_lints(); }
#[test] fn doc_cap_lints61() { doc_cap_lints(); }
#[test] fn doc_cap_lints62() { doc_cap_lints(); }
#[test] fn doc_cap_lints63() { doc_cap_lints(); }
#[test] fn doc_cap_lints64() { doc_cap_lints(); }
#[test] fn doc_cap_lints65() { doc_cap_lints(); }
#[test] fn doc_cap_lints66() { doc_cap_lints(); }
#[test] fn doc_cap_lints67() { doc_cap_lints(); }
#[test] fn doc_cap_lints68() { doc_cap_lints(); }
#[test] fn doc_cap_lints69() { doc_cap_lints(); }
#[test] fn doc_cap_lints70() { doc_cap_lints(); }
#[test] fn doc_cap_lints71() { doc_cap_lints(); }
#[test] fn doc_cap_lints72() { doc_cap_lints(); }
#[test] fn doc_cap_lints73() { doc_cap_lints(); }
#[test] fn doc_cap_lints74() { doc_cap_lints(); }
#[test] fn doc_cap_lints75() { doc_cap_lints(); }
#[test] fn doc_cap_lints76() { doc_cap_lints(); }
#[test] fn doc_cap_lints77() { doc_cap_lints(); }
#[test] fn doc_cap_lints78() { doc_cap_lints(); }
#[test] fn doc_cap_lints79() { doc_cap_lints(); }
#[test] fn doc_cap_lints80() { doc_cap_lints(); }
#[test] fn doc_cap_lints81() { doc_cap_lints(); }
#[test] fn doc_cap_lints82() { doc_cap_lints(); }
#[test] fn doc_cap_lints83() { doc_cap_lints(); }
#[test] fn doc_cap_lints84() { doc_cap_lints(); }
#[test] fn doc_cap_lints85() { doc_cap_lints(); }
#[test] fn doc_cap_lints86() { doc_cap_lints(); }
#[test] fn doc_cap_lints87() { doc_cap_lints(); }
#[test] fn doc_cap_lints88() { doc_cap_lints(); }
#[test] fn doc_cap_lints89() { doc_cap_lints(); }
#[test] fn doc_cap_lints90() { doc_cap_lints(); }
#[test] fn doc_cap_lints91() { doc_cap_lints(); }
#[test] fn doc_cap_lints92() { doc_cap_lints(); }
#[test] fn doc_cap_lints93() { doc_cap_lints(); }
#[test] fn doc_cap_lints94() { doc_cap_lints(); }
#[test] fn doc_cap_lints95() { doc_cap_lints(); }
#[test] fn doc_cap_lints96() { doc_cap_lints(); }
#[test] fn doc_cap_lints97() { doc_cap_lints(); }
#[test] fn doc_cap_lints98() { doc_cap_lints(); }
#[test] fn doc_cap_lints99() { doc_cap_lints(); }
#[test] fn doc_cap_lints100() { doc_cap_lints(); }
#[test] fn doc_cap_lints101() { doc_cap_lints(); }
#[test] fn doc_cap_lints102() { doc_cap_lints(); }
#[test] fn doc_cap_lints103() { doc_cap_lints(); }
#[test] fn doc_cap_lints104() { doc_cap_lints(); }
#[test] fn doc_cap_lints105() { doc_cap_lints(); }
#[test] fn doc_cap_lints106() { doc_cap_lints(); }
#[test] fn doc_cap_lints107() { doc_cap_lints(); }
#[test] fn doc_cap_lints108() { doc_cap_lints(); }
#[test] fn doc_cap_lints109() { doc_cap_lints(); }
#[test] fn doc_cap_lints110() { doc_cap_lints(); }
#[test] fn doc_cap_lints111() { doc_cap_lints(); }
#[test] fn doc_cap_lints112() { doc_cap_lints(); }
#[test] fn doc_cap_lints113() { doc_cap_lints(); }
#[test] fn doc_cap_lints114() { doc_cap_lints(); }
#[test] fn doc_cap_lints115() { doc_cap_lints(); }
#[test] fn doc_cap_lints116() { doc_cap_lints(); }
#[test] fn doc_cap_lints117() { doc_cap_lints(); }
#[test] fn doc_cap_lints118() { doc_cap_lints(); }
#[test] fn doc_cap_lints119() { doc_cap_lints(); }
#[test] fn doc_cap_lints120() { doc_cap_lints(); }
#[test] fn doc_cap_lints121() { doc_cap_lints(); }
#[test] fn doc_cap_lints122() { doc_cap_lints(); }
#[test] fn doc_cap_lints123() { doc_cap_lints(); }
#[test] fn doc_cap_lints124() { doc_cap_lints(); }
#[test] fn doc_cap_lints125() { doc_cap_lints(); }
#[test] fn doc_cap_lints126() { doc_cap_lints(); }
#[test] fn doc_cap_lints127() { doc_cap_lints(); }
#[test] fn doc_cap_lints128() { doc_cap_lints(); }
#[test] fn doc_cap_lints129() { doc_cap_lints(); }
#[test] fn doc_cap_lints130() { doc_cap_lints(); }
#[test] fn doc_cap_lints131() { doc_cap_lints(); }
#[test] fn doc_cap_lints132() { doc_cap_lints(); }
#[test] fn doc_cap_lints133() { doc_cap_lints(); }
#[test] fn doc_cap_lints134() { doc_cap_lints(); }
#[test] fn doc_cap_lints135() { doc_cap_lints(); }
#[test] fn doc_cap_lints136() { doc_cap_lints(); }
#[test] fn doc_cap_lints137() { doc_cap_lints(); }
#[test] fn doc_cap_lints138() { doc_cap_lints(); }
#[test] fn doc_cap_lints139() { doc_cap_lints(); }
#[test] fn doc_cap_lints140() { doc_cap_lints(); }
#[test] fn doc_cap_lints141() { doc_cap_lints(); }
#[test] fn doc_cap_lints142() { doc_cap_lints(); }
#[test] fn doc_cap_lints143() { doc_cap_lints(); }
#[test] fn doc_cap_lints144() { doc_cap_lints(); }
#[test] fn doc_cap_lints145() { doc_cap_lints(); }
#[test] fn doc_cap_lints146() { doc_cap_lints(); }
#[test] fn doc_cap_lints147() { doc_cap_lints(); }
#[test] fn doc_cap_lints148() { doc_cap_lints(); }
#[test] fn doc_cap_lints149() { doc_cap_lints(); }
#[test] fn doc_cap_lints150() { doc_cap_lints(); }
#[test] fn doc_cap_lints151() { doc_cap_lints(); }
#[test] fn doc_cap_lints152() { doc_cap_lints(); }
#[test] fn doc_cap_lints153() { doc_cap_lints(); }
#[test] fn doc_cap_lints154() { doc_cap_lints(); }
#[test] fn doc_cap_lints155() { doc_cap_lints(); }
#[test] fn doc_cap_lints156() { doc_cap_lints(); }
#[test] fn doc_cap_lints157() { doc_cap_lints(); }
#[test] fn doc_cap_lints158() { doc_cap_lints(); }
#[test] fn doc_cap_lints159() { doc_cap_lints(); }
#[test] fn doc_cap_lints160() { doc_cap_lints(); }
#[test] fn doc_cap_lints161() { doc_cap_lints(); }
#[test] fn doc_cap_lints162() { doc_cap_lints(); }
#[test] fn doc_cap_lints163() { doc_cap_lints(); }
#[test] fn doc_cap_lints164() { doc_cap_lints(); }
#[test] fn doc_cap_lints165() { doc_cap_lints(); }
#[test] fn doc_cap_lints166() { doc_cap_lints(); }
#[test] fn doc_cap_lints167() { doc_cap_lints(); }
#[test] fn doc_cap_lints168() { doc_cap_lints(); }
#[test] fn doc_cap_lints169() { doc_cap_lints(); }
#[test] fn doc_cap_lints170() { doc_cap_lints(); }
#[test] fn doc_cap_lints171() { doc_cap_lints(); }
#[test] fn doc_cap_lints172() { doc_cap_lints(); }
#[test] fn doc_cap_lints173() { doc_cap_lints(); }
#[test] fn doc_cap_lints174() { doc_cap_lints(); }
#[test] fn doc_cap_lints175() { doc_cap_lints(); }
#[test] fn doc_cap_lints176() { doc_cap_lints(); }
#[test] fn doc_cap_lints177() { doc_cap_lints(); }
#[test] fn doc_cap_lints178() { doc_cap_lints(); }
#[test] fn doc_cap_lints179() { doc_cap_lints(); }
#[test] fn doc_cap_lints180() { doc_cap_lints(); }
#[test] fn doc_cap_lints181() { doc_cap_lints(); }
#[test] fn doc_cap_lints182() { doc_cap_lints(); }
#[test] fn doc_cap_lints183() { doc_cap_lints(); }
#[test] fn doc_cap_lints184() { doc_cap_lints(); }
#[test] fn doc_cap_lints185() { doc_cap_lints(); }
#[test] fn doc_cap_lints186() { doc_cap_lints(); }
#[test] fn doc_cap_lints187() { doc_cap_lints(); }
#[test] fn doc_cap_lints188() { doc_cap_lints(); }
#[test] fn doc_cap_lints189() { doc_cap_lints(); }
#[test] fn doc_cap_lints190() { doc_cap_lints(); }
#[test] fn doc_cap_lints191() { doc_cap_lints(); }
#[test] fn doc_cap_lints192() { doc_cap_lints(); }
#[test] fn doc_cap_lints193() { doc_cap_lints(); }
#[test] fn doc_cap_lints194() { doc_cap_lints(); }
#[test] fn doc_cap_lints195() { doc_cap_lints(); }
#[test] fn doc_cap_lints196() { doc_cap_lints(); }
#[test] fn doc_cap_lints197() { doc_cap_lints(); }
#[test] fn doc_cap_lints198() { doc_cap_lints(); }
#[test] fn doc_cap_lints199() { doc_cap_lints(); }
#[test] fn doc_cap_lints200() { doc_cap_lints(); }
#[test] fn doc_cap_lints201() { doc_cap_lints(); }
#[test] fn doc_cap_lints202() { doc_cap_lints(); }
#[test] fn doc_cap_lints203() { doc_cap_lints(); }
#[test] fn doc_cap_lints204() { doc_cap_lints(); }
#[test] fn doc_cap_lints205() { doc_cap_lints(); }
#[test] fn doc_cap_lints206() { doc_cap_lints(); }
#[test] fn doc_cap_lints207() { doc_cap_lints(); }
#[test] fn doc_cap_lints208() { doc_cap_lints(); }
#[test] fn doc_cap_lints209() { doc_cap_lints(); }
#[test] fn doc_cap_lints210() { doc_cap_lints(); }
#[test] fn doc_cap_lints211() { doc_cap_lints(); }
#[test] fn doc_cap_lints212() { doc_cap_lints(); }
#[test] fn doc_cap_lints213() { doc_cap_lints(); }
#[test] fn doc_cap_lints214() { doc_cap_lints(); }
#[test] fn doc_cap_lints215() { doc_cap_lints(); }
#[test] fn doc_cap_lints216() { doc_cap_lints(); }
#[test] fn doc_cap_lints217() { doc_cap_lints(); }
#[test] fn doc_cap_lints218() { doc_cap_lints(); }
#[test] fn doc_cap_lints219() { doc_cap_lints(); }
#[test] fn doc_cap_lints220() { doc_cap_lints(); }
#[test] fn doc_cap_lints221() { doc_cap_lints(); }
#[test] fn doc_cap_lints222() { doc_cap_lints(); }
#[test] fn doc_cap_lints223() { doc_cap_lints(); }
#[test] fn doc_cap_lints224() { doc_cap_lints(); }
#[test] fn doc_cap_lints225() { doc_cap_lints(); }
#[test] fn doc_cap_lints226() { doc_cap_lints(); }
#[test] fn doc_cap_lints227() { doc_cap_lints(); }
#[test] fn doc_cap_lints228() { doc_cap_lints(); }
#[test] fn doc_cap_lints229() { doc_cap_lints(); }
#[test] fn doc_cap_lints230() { doc_cap_lints(); }
#[test] fn doc_cap_lints231() { doc_cap_lints(); }
#[test] fn doc_cap_lints232() { doc_cap_lints(); }
#[test] fn doc_cap_lints233() { doc_cap_lints(); }
#[test] fn doc_cap_lints234() { doc_cap_lints(); }
#[test] fn doc_cap_lints235() { doc_cap_lints(); }
#[test] fn doc_cap_lints236() { doc_cap_lints(); }
#[test] fn doc_cap_lints237() { doc_cap_lints(); }
#[test] fn doc_cap_lints238() { doc_cap_lints(); }
#[test] fn doc_cap_lints239() { doc_cap_lints(); }
#[test] fn doc_cap_lints240() { doc_cap_lints(); }
#[test] fn doc_cap_lints241() { doc_cap_lints(); }
#[test] fn doc_cap_lints242() { doc_cap_lints(); }
#[test] fn doc_cap_lints243() { doc_cap_lints(); }
#[test] fn doc_cap_lints244() { doc_cap_lints(); }
#[test] fn doc_cap_lints245() { doc_cap_lints(); }
#[test] fn doc_cap_lints246() { doc_cap_lints(); }
#[test] fn doc_cap_lints247() { doc_cap_lints(); }
#[test] fn doc_cap_lints248() { doc_cap_lints(); }
#[test] fn doc_cap_lints249() { doc_cap_lints(); }
#[test] fn doc_cap_lints250() { doc_cap_lints(); }
#[test] fn doc_cap_lints251() { doc_cap_lints(); }
#[test] fn doc_cap_lints252() { doc_cap_lints(); }
#[test] fn doc_cap_lints253() { doc_cap_lints(); }
#[test] fn doc_cap_lints254() { doc_cap_lints(); }
#[test] fn doc_cap_lints255() { doc_cap_lints(); }
#[test] fn doc_cap_lints256() { doc_cap_lints(); }
#[test] fn doc_cap_lints257() { doc_cap_lints(); }
#[test] fn doc_cap_lints258() { doc_cap_lints(); }
#[test] fn doc_cap_lints259() { doc_cap_lints(); }
#[test] fn doc_cap_lints260() { doc_cap_lints(); }
#[test] fn doc_cap_lints261() { doc_cap_lints(); }
#[test] fn doc_cap_lints262() { doc_cap_lints(); }
#[test] fn doc_cap_lints263() { doc_cap_lints(); }
#[test] fn doc_cap_lints264() { doc_cap_lints(); }
#[test] fn doc_cap_lints265() { doc_cap_lints(); }
#[test] fn doc_cap_lints266() { doc_cap_lints(); }
#[test] fn doc_cap_lints267() { doc_cap_lints(); }
#[test] fn doc_cap_lints268() { doc_cap_lints(); }
#[test] fn doc_cap_lints269() { doc_cap_lints(); }
#[test] fn doc_cap_lints270() { doc_cap_lints(); }
#[test] fn doc_cap_lints271() { doc_cap_lints(); }
#[test] fn doc_cap_lints272() { doc_cap_lints(); }
#[test] fn doc_cap_lints273() { doc_cap_lints(); }
#[test] fn doc_cap_lints274() { doc_cap_lints(); }
#[test] fn doc_cap_lints275() { doc_cap_lints(); }
#[test] fn doc_cap_lints276() { doc_cap_lints(); }
#[test] fn doc_cap_lints277() { doc_cap_lints(); }
#[test] fn doc_cap_lints278() { doc_cap_lints(); }
#[test] fn doc_cap_lints279() { doc_cap_lints(); }
#[test] fn doc_cap_lints280() { doc_cap_lints(); }
#[test] fn doc_cap_lints281() { doc_cap_lints(); }
#[test] fn doc_cap_lints282() { doc_cap_lints(); }
#[test] fn doc_cap_lints283() { doc_cap_lints(); }
#[test] fn doc_cap_lints284() { doc_cap_lints(); }
#[test] fn doc_cap_lints285() { doc_cap_lints(); }
#[test] fn doc_cap_lints286() { doc_cap_lints(); }
#[test] fn doc_cap_lints287() { doc_cap_lints(); }
#[test] fn doc_cap_lints288() { doc_cap_lints(); }
#[test] fn doc_cap_lints289() { doc_cap_lints(); }
#[test] fn doc_cap_lints290() { doc_cap_lints(); }
#[test] fn doc_cap_lints291() { doc_cap_lints(); }
#[test] fn doc_cap_lints292() { doc_cap_lints(); }
#[test] fn doc_cap_lints293() { doc_cap_lints(); }
#[test] fn doc_cap_lints294() { doc_cap_lints(); }
#[test] fn doc_cap_lints295() { doc_cap_lints(); }
#[test] fn doc_cap_lints296() { doc_cap_lints(); }
#[test] fn doc_cap_lints297() { doc_cap_lints(); }
#[test] fn doc_cap_lints298() { doc_cap_lints(); }
#[test] fn doc_cap_lints299() { doc_cap_lints(); }
#[test] fn doc_cap_lints300() { doc_cap_lints(); }
#[test] fn doc_cap_lints301() { doc_cap_lints(); }
#[test] fn doc_cap_lints302() { doc_cap_lints(); }
#[test] fn doc_cap_lints303() { doc_cap_lints(); }
#[test] fn doc_cap_lints304() { doc_cap_lints(); }
#[test] fn doc_cap_lints305() { doc_cap_lints(); }
#[test] fn doc_cap_lints306() { doc_cap_lints(); }
#[test] fn doc_cap_lints307() { doc_cap_lints(); }
#[test] fn doc_cap_lints308() { doc_cap_lints(); }
#[test] fn doc_cap_lints309() { doc_cap_lints(); }
#[test] fn doc_cap_lints310() { doc_cap_lints(); }
#[test] fn doc_cap_lints311() { doc_cap_lints(); }
#[test] fn doc_cap_lints312() { doc_cap_lints(); }
#[test] fn doc_cap_lints313() { doc_cap_lints(); }
#[test] fn doc_cap_lints314() { doc_cap_lints(); }
#[test] fn doc_cap_lints315() { doc_cap_lints(); }
#[test] fn doc_cap_lints316() { doc_cap_lints(); }
#[test] fn doc_cap_lints317() { doc_cap_lints(); }
#[test] fn doc_cap_lints318() { doc_cap_lints(); }
#[test] fn doc_cap_lints319() { doc_cap_lints(); }
#[test] fn doc_cap_lints320() { doc_cap_lints(); }
#[test] fn doc_cap_lints321() { doc_cap_lints(); }
#[test] fn doc_cap_lints322() { doc_cap_lints(); }
#[test] fn doc_cap_lints323() { doc_cap_lints(); }
#[test] fn doc_cap_lints324() { doc_cap_lints(); }
#[test] fn doc_cap_lints325() { doc_cap_lints(); }
#[test] fn doc_cap_lints326() { doc_cap_lints(); }
#[test] fn doc_cap_lints327() { doc_cap_lints(); }
#[test] fn doc_cap_lints328() { doc_cap_lints(); }
#[test] fn doc_cap_lints329() { doc_cap_lints(); }
#[test] fn doc_cap_lints330() { doc_cap_lints(); }
#[test] fn doc_cap_lints331() { doc_cap_lints(); }
#[test] fn doc_cap_lints332() { doc_cap_lints(); }
#[test] fn doc_cap_lints333() { doc_cap_lints(); }
#[test] fn doc_cap_lints334() { doc_cap_lints(); }
#[test] fn doc_cap_lints335() { doc_cap_lints(); }
#[test] fn doc_cap_lints336() { doc_cap_lints(); }
#[test] fn doc_cap_lints337() { doc_cap_lints(); }
#[test] fn doc_cap_lints338() { doc_cap_lints(); }
#[test] fn doc_cap_lints339() { doc_cap_lints(); }
#[test] fn doc_cap_lints340() { doc_cap_lints(); }
#[test] fn doc_cap_lints341() { doc_cap_lints(); }
#[test] fn doc_cap_lints342() { doc_cap_lints(); }
#[test] fn doc_cap_lints343() { doc_cap_lints(); }
#[test] fn doc_cap_lints344() { doc_cap_lints(); }
#[test] fn doc_cap_lints345() { doc_cap_lints(); }
#[test] fn doc_cap_lints346() { doc_cap_lints(); }
#[test] fn doc_cap_lints347() { doc_cap_lints(); }
#[test] fn doc_cap_lints348() { doc_cap_lints(); }
#[test] fn doc_cap_lints349() { doc_cap_lints(); }
#[test] fn doc_cap_lints350() { doc_cap_lints(); }
#[test] fn doc_cap_lints351() { doc_cap_lints(); }
#[test] fn doc_cap_lints352() { doc_cap_lints(); }
#[test] fn doc_cap_lints353() { doc_cap_lints(); }
#[test] fn doc_cap_lints354() { doc_cap_lints(); }
#[test] fn doc_cap_lints355() { doc_cap_lints(); }
#[test] fn doc_cap_lints356() { doc_cap_lints(); }
#[test] fn doc_cap_lints357() { doc_cap_lints(); }
#[test] fn doc_cap_lints358() { doc_cap_lints(); }
#[test] fn doc_cap_lints359() { doc_cap_lints(); }
#[test] fn doc_cap_lints360() { doc_cap_lints(); }
#[test] fn doc_cap_lints361() { doc_cap_lints(); }
#[test] fn doc_cap_lints362() { doc_cap_lints(); }
#[test] fn doc_cap_lints363() { doc_cap_lints(); }
#[test] fn doc_cap_lints364() { doc_cap_lints(); }
#[test] fn doc_cap_lints365() { doc_cap_lints(); }
#[test] fn doc_cap_lints366() { doc_cap_lints(); }
#[test] fn doc_cap_lints367() { doc_cap_lints(); }
#[test] fn doc_cap_lints368() { doc_cap_lints(); }
#[test] fn doc_cap_lints369() { doc_cap_lints(); }
#[test] fn doc_cap_lints370() { doc_cap_lints(); }
#[test] fn doc_cap_lints371() { doc_cap_lints(); }
#[test] fn doc_cap_lints372() { doc_cap_lints(); }
#[test] fn doc_cap_lints373() { doc_cap_lints(); }
#[test] fn doc_cap_lints374() { doc_cap_lints(); }
#[test] fn doc_cap_lints375() { doc_cap_lints(); }
#[test] fn doc_cap_lints376() { doc_cap_lints(); }
#[test] fn doc_cap_lints377() { doc_cap_lints(); }
#[test] fn doc_cap_lints378() { doc_cap_lints(); }
#[test] fn doc_cap_lints379() { doc_cap_lints(); }
#[test] fn doc_cap_lints380() { doc_cap_lints(); }
#[test] fn doc_cap_lints381() { doc_cap_lints(); }
#[test] fn doc_cap_lints382() { doc_cap_lints(); }
#[test] fn doc_cap_lints383() { doc_cap_lints(); }
#[test] fn doc_cap_lints384() { doc_cap_lints(); }
#[test] fn doc_cap_lints385() { doc_cap_lints(); }
#[test] fn doc_cap_lints386() { doc_cap_lints(); }
#[test] fn doc_cap_lints387() { doc_cap_lints(); }
#[test] fn doc_cap_lints388() { doc_cap_lints(); }
#[test] fn doc_cap_lints389() { doc_cap_lints(); }
#[test] fn doc_cap_lints390() { doc_cap_lints(); }
#[test] fn doc_cap_lints391() { doc_cap_lints(); }
#[test] fn doc_cap_lints392() { doc_cap_lints(); }
#[test] fn doc_cap_lints393() { doc_cap_lints(); }
#[test] fn doc_cap_lints394() { doc_cap_lints(); }
#[test] fn doc_cap_lints395() { doc_cap_lints(); }
#[test] fn doc_cap_lints396() { doc_cap_lints(); }
#[test] fn doc_cap_lints397() { doc_cap_lints(); }
#[test] fn doc_cap_lints398() { doc_cap_lints(); }
#[test] fn doc_cap_lints399() { doc_cap_lints(); }

#[cargo_test]
fn doc_cap_lints() {
    let a = git::new("a", |p| {
        p.file("Cargo.toml", &basic_lib_manifest("a"))
            .file("src/lib.rs", BAD_INTRA_LINK_LIB)
    });

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.0.1"
                    edition = "2015"
                    authors = []

                    [dependencies]
                    a = {{ git = '{}' }}
                "#,
                a.url()
            ),
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_data(
            str![[r#"
[LOCKING] 1 package to latest compatible version
[UPDATING] git repository `[..]`
[DOCUMENTING] a v0.5.0 ([..])
[CHECKING] a v0.5.0 ([..])
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();

    p.root().join("target").rm_rf();

    p.cargo("doc -vv")
        .with_stderr_data(str![[r#"
...
[WARNING] [..]`bad_link`[..]
...
"#]])
        .run();
}

#[cargo_test]
fn doc_message_format() {
    let p = project().file("src/lib.rs", BAD_INTRA_LINK_LIB).build();

    p.cargo("doc --message-format=json")
        .with_status(101)
        .with_stdout_data(
            str![[r##"
[
  {
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "message": {
      "$message_type": "diagnostic",
      "level": "error",
      "...": "{...}"
    },
    "package_id": "path+[ROOTURL]/foo#0.0.1",
    "reason": "compiler-message",
    "target": "{...}"
  },
  "{...}"
]
"##]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn doc_json_artifacts() {
    // Checks the output of json artifact messages.
    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin/somebin.rs", "fn main() {}")
        .build();

    p.cargo("doc --message-format=json")
        .with_stdout_data(
            str![[r#"
[
  {
    "executable": null,
    "features": [],
    "filenames": [
      "[ROOT]/foo/target/debug/deps/libfoo-[HASH].rmeta"
    ],
    "fresh": false,
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "package_id": "path+[ROOTURL]/foo#0.0.1",
    "profile": "{...}",
    "reason": "compiler-artifact",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doc": true,
      "doctest": true,
      "edition": "2015",
      "kind": [
        "lib"
      ],
      "name": "foo",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
    }
  },
  {
    "executable": null,
    "features": [],
    "filenames": [
      "[ROOT]/foo/target/doc/foo/index.html"
    ],
    "fresh": false,
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "package_id": "path+[ROOTURL]/foo#0.0.1",
    "profile": "{...}",
    "reason": "compiler-artifact",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doc": true,
      "doctest": true,
      "edition": "2015",
      "kind": [
        "lib"
      ],
      "name": "foo",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
    }
  },
  {
    "executable": null,
    "features": [],
    "filenames": [
      "[ROOT]/foo/target/doc/somebin/index.html"
    ],
    "fresh": false,
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "package_id": "path+[ROOTURL]/foo#0.0.1",
    "profile": "{...}",
    "reason": "compiler-artifact",
    "target": {
      "crate_types": [
        "bin"
      ],
      "doc": true,
      "doctest": false,
      "edition": "2015",
      "kind": [
        "bin"
      ],
      "name": "somebin",
      "src_path": "[ROOT]/foo/src/bin/somebin.rs",
      "test": true
    }
  },
  {
    "reason": "build-finished",
    "success": true
  }
]
"#]]
            .is_json()
            .against_jsonlines()
            .unordered(),
        )
        .run();
}

#[cargo_test]
fn short_message_format() {
    let p = project().file("src/lib.rs", BAD_INTRA_LINK_LIB).build();
    p.cargo("doc --message-format=short")
        .with_status(101)
        .with_stderr_data(str![[r#"
...
src/lib.rs:4:6: [ERROR] [..]`bad_link`[..]
...
"#]])
        .run();
}

#[cargo_test]
fn doc_example() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [[example]]
            crate-type = ["lib"]
            name = "ex1"
            doc = true
            "#,
        )
        .file("src/lib.rs", "pub fn f() {}")
        .file(
            "examples/ex1.rs",
            r#"
            use foo::f;

            /// Example
            pub fn x() { f(); }
            "#,
        )
        .build();

    p.cargo("doc").run();
    assert!(p
        .build_dir()
        .join("doc")
        .join("ex1")
        .join("fn.x.html")
        .exists());
}

#[cargo_test]
fn doc_example_with_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            edition = "2018"

            [[example]]
            crate-type = ["lib"]
            name = "ex"
            doc = true

            [dev-dependencies]
            a = {path = "a"}
            b = {path = "b"}
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "examples/ex.rs",
            r#"
            use a::fun;

            /// Example
            pub fn x() { fun(); }
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            b = {path = "../b"}
            "#,
        )
        .file("a/src/fun.rs", "pub fn fun() {}")
        .file("a/src/lib.rs", "pub mod fun;")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.0.1"
            edition = "2015"
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("doc --examples").run();
    assert!(p
        .build_dir()
        .join("doc")
        .join("ex")
        .join("fn.x.html")
        .exists());
}

#[cargo_test]
fn bin_private_items() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []
            "#,
        )
        .file(
            "src/main.rs",
            "
            pub fn foo_pub() {}
            fn foo_priv() {}
            struct FooStruct;
            enum FooEnum {}
            trait FooTrait {}
            type FooType = u32;
            mod foo_mod {}

        ",
        )
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    assert!(p.root().join("target/doc/foo/index.html").is_file());
    assert!(p.root().join("target/doc/foo/fn.foo_pub.html").is_file());
    assert!(p.root().join("target/doc/foo/fn.foo_priv.html").is_file());
    assert!(p
        .root()
        .join("target/doc/foo/struct.FooStruct.html")
        .is_file());
    assert!(p.root().join("target/doc/foo/enum.FooEnum.html").is_file());
    assert!(p
        .root()
        .join("target/doc/foo/trait.FooTrait.html")
        .is_file());
    assert!(p.root().join("target/doc/foo/type.FooType.html").is_file());
    assert!(p.root().join("target/doc/foo/foo_mod/index.html").is_file());
}

#[cargo_test]
fn bin_private_items_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                edition = "2015"
                authors = []

                [dependencies.bar]
                path = "bar"
            "#,
        )
        .file(
            "src/main.rs",
            "
            fn foo_priv() {}
            pub fn foo_pub() {}
        ",
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file(
            "bar/src/lib.rs",
            "
            #[allow(dead_code)]
            fn bar_priv() {}
            pub fn bar_pub() {}
        ",
        )
        .build();

    p.cargo("doc")
        .with_stderr_data(
            str![[r#"
[LOCKING] 1 package to latest compatible version
[DOCUMENTING] bar v0.0.1 ([ROOT]/foo/bar)
[CHECKING] bar v0.0.1 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();

    assert!(p.root().join("target/doc/foo/index.html").is_file());
    assert!(p.root().join("target/doc/foo/fn.foo_pub.html").is_file());
    assert!(p.root().join("target/doc/foo/fn.foo_priv.html").is_file());

    assert!(p.root().join("target/doc/bar/index.html").is_file());
    assert!(p.root().join("target/doc/bar/fn.bar_pub.html").is_file());
    assert!(!p.root().join("target/doc/bar/fn.bar_priv.html").exists());
}

#[cargo_test]
fn crate_versions() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "1.2.4"
                edition = "2015"
                authors = []
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc -v")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v1.2.4 ([ROOT]/foo)
[RUNNING] `rustdoc --edition=2015 --crate-type lib --crate-name foo src/lib.rs [..]--crate-version 1.2.4`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    let output_path = p.root().join("target/doc/foo/index.html");
    let output_documentation = fs::read_to_string(&output_path).unwrap();

    assert!(output_documentation.contains("1.2.4"));
}

#[cargo_test]
fn crate_versions_flag_is_overridden() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "1.2.4"
                edition = "2015"
                authors = []
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    let output_documentation = || {
        let output_path = p.root().join("target/doc/foo/index.html");
        fs::read_to_string(&output_path).unwrap()
    };
    let asserts = |html: String| {
        assert!(!html.contains("1.2.4"));
        assert!(html.contains("2.0.3"));
    };

    p.cargo("doc")
        .env("RUSTDOCFLAGS", "--crate-version 2.0.3")
        .run();
    asserts(output_documentation());

    p.build_dir().rm_rf();

    p.cargo("rustdoc -- --crate-version 2.0.3").run();
    asserts(output_documentation());
}

#[cargo_test]
fn doc_test_in_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = [
                    "crate-a",
                    "crate-b",
                ]
            "#,
        )
        .file(
            "crate-a/Cargo.toml",
            r#"
                [package]
                name = "crate-a"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file(
            "crate-a/src/lib.rs",
            "\
                //! ```
                //! assert_eq!(1, 1);
                //! ```
            ",
        )
        .file(
            "crate-b/Cargo.toml",
            r#"
                [package]
                name = "crate-b"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file(
            "crate-b/src/lib.rs",
            "\
                //! ```
                //! assert_eq!(1, 1);
                //! ```
            ",
        )
        .build();
    p.cargo("test --doc -vv")
        .with_stderr_data(str![[r#"
...
[DOCTEST] crate_a
...
"#]])
        .with_stdout_data(str![[r#"

running 1 test
test crate-a/src/lib.rs - (line 1) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in [ELAPSED]s


running 1 test
test crate-b/src/lib.rs - (line 1) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in [ELAPSED]s


"#]])
        .run();
}

/// This is a test for <https://github.com/rust-lang/rust/issues/46372>.
/// The `file!()` macro inside of an `include!()` should output
/// workspace-relative paths, just like it does in other cases.
#[cargo_test]
fn doc_test_include_file() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = [
                    "child",
                ]
                [package]
                name = "root"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                /// ```
                /// assert_eq!("src/lib.rs", file!().replace("\\", "/"))
                /// ```
                pub mod included {
                    include!(concat!("../", file!(), ".included.rs"));
                }
            "#,
        )
        .file(
            "src/lib.rs.included.rs",
            r#"
                /// ```
                /// assert_eq!(1, 1)
                /// ```
                pub fn foo() {}
            "#,
        )
        .file(
            "child/Cargo.toml",
            r#"
                [package]
                name = "child"
                version = "0.1.0"
                edition = "2015"
            "#,
        )
        .file(
            "child/src/lib.rs",
            r#"
                /// ```
                /// assert_eq!("child/src/lib.rs", file!().replace("\\", "/"))
                /// ```
                pub mod included {
                    include!(concat!("../../", file!(), ".included.rs"));
                }
            "#,
        )
        .file(
            "child/src/lib.rs.included.rs",
            r#"
                /// ```
                /// assert_eq!(1, 1)
                /// ```
                pub fn foo() {}
            "#,
        )
        .build();

    p.cargo("test --workspace --doc -vv -- --test-threads=1")
        .with_stderr_data(str![[r#"
...
[DOCTEST] child
...
[DOCTEST] root
...
"#]])
        .with_stdout_data(str![[r#"

running 2 tests
test child/src/../../child/src/lib.rs.included.rs - included::foo (line 2) ... ok
test child/src/lib.rs - included (line 2) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in [ELAPSED]s


running 2 tests
test src/../src/lib.rs.included.rs - included::foo (line 2) ... ok
test src/lib.rs - included (line 2) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in [ELAPSED]s


"#]])
        .run();
}

#[cargo_test]
fn doc_fingerprint_is_versioning_consistent() {
    // Random rustc verbose version
    let old_rustc_verbose_version = format!(
        "\
rustc 1.41.1 (f3e1a954d 2020-02-24)
binary: rustc
commit-hash: f3e1a954d2ead4e2fc197c7da7d71e6c61bad196
commit-date: 2020-02-24
host: {}
release: 1.41.1
LLVM version: 9.0
",
        rustc_host()
    );

    // Create the dummy project.
    let dummy_project = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.2.4"
            edition = "2015"
            authors = []
        "#,
        )
        .file("src/lib.rs", "//! These are the docs!")
        .build();

    dummy_project.cargo("doc").run();

    let fingerprint: RustDocFingerprint =
        serde_json::from_str(&dummy_project.read_file("target/.rustdoc_fingerprint.json"))
            .expect("JSON Serde fail");

    // Check that the fingerprint contains the actual rustc version
    // which has been used to compile the docs.
    let output = std::process::Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to get actual rustc verbose version");
    assert_eq!(
        fingerprint.rustc_vv,
        (String::from_utf8_lossy(&output.stdout).as_ref())
    );

    // As the test shows above. Now we have generated the `doc/` folder and inside
    // the rustdoc fingerprint file is located with the correct rustc version.
    // So we will remove it and create a new fingerprint with an old rustc version
    // inside it. We will also place a bogus file inside of the `doc/` folder to ensure
    // it gets removed as we expect on the next doc compilation.
    dummy_project.change_file(
        "target/.rustdoc_fingerprint.json",
        &old_rustc_verbose_version,
    );

    fs::write(
        dummy_project.build_dir().join("doc/bogus_file"),
        String::from("This is a bogus file and should be removed!"),
    )
    .expect("Error writing test bogus file");

    // Now if we trigger another compilation, since the fingerprint contains an old version
    // of rustc, cargo should remove the entire `/doc` folder (including the fingerprint)
    // and generating another one with the actual version.
    // It should also remove the bogus file we created above.
    dummy_project.cargo("doc").run();

    assert!(!dummy_project.build_dir().join("doc/bogus_file").exists());

    let fingerprint: RustDocFingerprint =
        serde_json::from_str(&dummy_project.read_file("target/.rustdoc_fingerprint.json"))
            .expect("JSON Serde fail");

    // Check that the fingerprint contains the actual rustc version
    // which has been used to compile the docs.
    assert_eq!(
        fingerprint.rustc_vv,
        (String::from_utf8_lossy(&output.stdout).as_ref())
    );
}

#[cargo_test]
fn doc_fingerprint_respects_target_paths() {
    // Random rustc verbose version
    let old_rustc_verbose_version = format!(
        "\
rustc 1.41.1 (f3e1a954d 2020-02-24)
binary: rustc
commit-hash: f3e1a954d2ead4e2fc197c7da7d71e6c61bad196
commit-date: 2020-02-24
host: {}
release: 1.41.1
LLVM version: 9.0
",
        rustc_host()
    );

    // Create the dummy project.
    let dummy_project = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.2.4"
            edition = "2015"
            authors = []
        "#,
        )
        .file("src/lib.rs", "//! These are the docs!")
        .build();

    dummy_project.cargo("doc --target").arg(rustc_host()).run();

    let fingerprint: RustDocFingerprint =
        serde_json::from_str(&dummy_project.read_file("target/.rustdoc_fingerprint.json"))
            .expect("JSON Serde fail");

    // Check that the fingerprint contains the actual rustc version
    // which has been used to compile the docs.
    let output = std::process::Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to get actual rustc verbose version");
    assert_eq!(
        fingerprint.rustc_vv,
        (String::from_utf8_lossy(&output.stdout).as_ref())
    );

    // As the test shows above. Now we have generated the `doc/` folder and inside
    // the rustdoc fingerprint file is located with the correct rustc version.
    // So we will remove it and create a new fingerprint with an old rustc version
    // inside it. We will also place a bogus file inside of the `doc/` folder to ensure
    // it gets removed as we expect on the next doc compilation.
    dummy_project.change_file(
        "target/.rustdoc_fingerprint.json",
        &old_rustc_verbose_version,
    );

    fs::write(
        dummy_project
            .build_dir()
            .join(rustc_host())
            .join("doc/bogus_file"),
        String::from("This is a bogus file and should be removed!"),
    )
    .expect("Error writing test bogus file");

    // Now if we trigger another compilation, since the fingerprint contains an old version
    // of rustc, cargo should remove the entire `/doc` folder (including the fingerprint)
    // and generating another one with the actual version.
    // It should also remove the bogus file we created above.
    dummy_project.cargo("doc --target").arg(rustc_host()).run();

    assert!(!dummy_project
        .build_dir()
        .join(rustc_host())
        .join("doc/bogus_file")
        .exists());

    let fingerprint: RustDocFingerprint =
        serde_json::from_str(&dummy_project.read_file("target/.rustdoc_fingerprint.json"))
            .expect("JSON Serde fail");

    // Check that the fingerprint contains the actual rustc version
    // which has been used to compile the docs.
    assert_eq!(
        fingerprint.rustc_vv,
        (String::from_utf8_lossy(&output.stdout).as_ref())
    );
}

#[cargo_test]
fn doc_fingerprint_unusual_behavior() {
    // Checks for some unusual circumstances with clearing the doc directory.
    if !symlink_supported() {
        return;
    }
    let p = project().file("src/lib.rs", "").build();
    p.build_dir().mkdir_p();
    let real_doc = p.root().join("doc");
    real_doc.mkdir_p();
    let build_doc = p.build_dir().join("doc");
    p.symlink(&real_doc, &build_doc);
    fs::write(real_doc.join("somefile"), "test").unwrap();
    fs::write(real_doc.join(".hidden"), "test").unwrap();
    p.cargo("doc").run();
    // Make sure for the first run, it does not delete any files and does not
    // break the symlink.
    assert!(build_doc.join("somefile").exists());
    assert!(real_doc.join("somefile").exists());
    assert!(real_doc.join(".hidden").exists());
    assert!(real_doc.join("foo/index.html").exists());
    // Pretend that the last build was generated by an older version.
    p.change_file(
        "target/.rustdoc_fingerprint.json",
        "{\"rustc_vv\": \"I am old\"}",
    );
    // Change file to trigger a new build.
    p.change_file("src/lib.rs", "// changed");
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    // This will delete somefile, but not .hidden.
    assert!(!real_doc.join("somefile").exists());
    assert!(real_doc.join(".hidden").exists());
    assert!(real_doc.join("foo/index.html").exists());
    // And also check the -Z flag behavior.
    p.change_file(
        "target/.rustdoc_fingerprint.json",
        "{\"rustc_vv\": \"I am old\"}",
    );
    // Change file to trigger a new build.
    p.change_file("src/lib.rs", "// changed2");
    fs::write(real_doc.join("somefile"), "test").unwrap();
    p.cargo("doc -Z skip-rustdoc-fingerprint")
        .masquerade_as_nightly_cargo(&["skip-rustdoc-fingerprint"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
    // Should not have deleted anything.
    assert!(build_doc.join("somefile").exists());
    assert!(real_doc.join("somefile").exists());
}

#[cargo_test]
fn lib_before_bin() {
    // Checks that the library is documented before the binary.
    // Previously they were built concurrently, which can cause issues
    // if the bin has intra-doc links to the lib.
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                /// Hi
                pub fn abc() {}
            "#,
        )
        .file(
            "src/bin/somebin.rs",
            r#"
                //! See [`foo::abc`]
                fn main() {}
            "#,
        )
        .build();

    // Run check first. This just helps ensure that the test clearly shows the
    // order of the rustdoc commands.
    p.cargo("check").run();

    // The order of output here should be deterministic.
    p.cargo("doc -v")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustdoc --edition=2015 --crate-type lib --crate-name foo src/lib.rs [..]
[RUNNING] `rustdoc --edition=2015 --crate-type bin --crate-name somebin src/bin/somebin.rs [..]
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html
[GENERATED] [ROOT]/foo/target/doc/somebin/index.html

"#]])
        .run();

    // And the link should exist.
    let bin_html = p.read_file("target/doc/somebin/index.html");
    assert!(bin_html.contains("../foo/fn.abc.html"));
}

#[cargo_test]
fn doc_lib_false() {
    // doc = false for a library
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"

                [lib]
                doc = false

                [dependencies]
                bar = {path = "bar"}
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file("src/bin/some-bin.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"

                [lib]
                doc = false
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[LOCKING] 1 package to latest compatible version
[CHECKING] bar v0.1.0 ([ROOT]/foo/bar)
[CHECKING] foo v0.1.0 ([ROOT]/foo)
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/some_bin/index.html

"#]])
        .run();

    assert!(!p.build_dir().join("doc/foo").exists());
    assert!(!p.build_dir().join("doc/bar").exists());
    assert!(p.build_dir().join("doc/some_bin").exists());
}

#[cargo_test]
fn doc_lib_false_dep() {
    // doc = false for a dependency
    // Ensures that the rmeta gets produced
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2015"

                [dependencies]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                edition = "2015"

                [lib]
                doc = false
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("doc")
        .with_stderr_data(str![[r#"
[LOCKING] 1 package to latest compatible version
[CHECKING] bar v0.1.0 ([ROOT]/foo/bar)
[DOCUMENTING] foo v0.1.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    assert!(p.build_dir().join("doc/foo").exists());
    assert!(!p.build_dir().join("doc/bar").exists());
}

#[cargo_test]
fn link_to_private_item() {
    let main = r#"
    //! [bar]
    #[allow(dead_code)]
    fn bar() {}
    "#;
    let p = project().file("src/lib.rs", main).build();
    p.cargo("doc")
        .with_stderr_data(str![[r#"
...
[..]documentation for `foo` links to private item `bar`
...
"#]])
        .run();
    // Check that binaries don't emit a private_intra_doc_links warning.
    fs::rename(p.root().join("src/lib.rs"), p.root().join("src/main.rs")).unwrap();
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();
}

#[cargo_test]
fn rustdoc_failure_hides_command_line_by_default() {
    let p = project().file("src/lib.rs", "invalid rust code").build();

    // `cargo doc` doesn't print the full command line on failures by default
    p.cargo("doc")
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.1 ([ROOT]/foo)
[ERROR] expected one of `!` or `::`, found `rust`
 --> src/lib.rs:1:9
  |
1 | invalid rust code
  |         ^^^^ expected one of `!` or `::`

[ERROR] could not document `foo`

"#]])
        .with_status(101)
        .run();

    // ... but it still does so if requested with `--verbose`.
    p.cargo("doc --verbose")
        .with_stderr_data(str![[r#"
...
Caused by:
  process didn't exit successfully[..]rustdoc[..]

"#]])
        .with_status(101)
        .run();
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rustdoc_depinfo_gated() {
    let p = project()
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] the `-Z` flag is only accepted on the nightly channel of Cargo, but this is the `stable` channel
See https://doc.rust-lang.org/book/appendix-07-nightly-rust.html for more information about Rust release channels.

"#]])
        .run();
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rebuild_tracks_target_src_outside_package_root() {
    let p = cargo_test_support::project_in("parent")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                edition = "2015"
                [lib]
                path = "../lib.rs"
            "#,
        )
        .file("../lib.rs", "//! # depinfo-before")
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.0.0 ([ROOT]/parent/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.change_file("../lib.rs", "//! # depinfo-after");

    p.cargo("doc --verbose -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DIRTY] foo v0.0.0 ([ROOT]/parent/foo): the file `../lib.rs` has changed ([TIME_DIFF_AFTER_LAST_BUILD])
[DOCUMENTING] foo v0.0.0 ([ROOT]/parent/foo)
[RUNNING] `rustdoc [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-after"));
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rebuild_tracks_include_str() {
    let p = cargo_test_support::project_in("parent")
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", r#"#![doc = include_str!("../../README")]"#)
        .file("../README", "# depinfo-before")
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.change_file("../README", "# depinfo-after");

    p.cargo("doc --verbose -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DIRTY] foo v0.5.0 ([ROOT]/parent/foo): the file `src/../../README` has changed ([TIME_DIFF_AFTER_LAST_BUILD])
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[RUNNING] `rustdoc [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-after"));
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rebuild_tracks_path_attr() {
    let p = cargo_test_support::project_in("parent")
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", r#"#[path = "../../bar.rs"] pub mod bar;"#)
        .file("../bar.rs", "//! # depinfo-before")
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.change_file("../bar.rs", "//! # depinfo-after");

    p.cargo("doc --verbose -Zrustdoc-depinfo")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DIRTY] foo v0.5.0 ([ROOT]/parent/foo): the file `src/../../bar.rs` has changed ([TIME_DIFF_AFTER_LAST_BUILD])
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[RUNNING] `rustdoc [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-after"));
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rebuild_tracks_env() {
    let env = "__RUSTDOC_INJECTED";
    let p = project()
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", &format!(r#"#![doc = env!("{env}")]"#))
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .env(env, "# depinfo-before")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.5.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.cargo("doc --verbose -Zrustdoc-depinfo")
        .env(env, "# depinfo-after")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DIRTY] foo v0.5.0 ([ROOT]/foo): the environment variable __RUSTDOC_INJECTED changed
[DOCUMENTING] foo v0.5.0 ([ROOT]/foo)
[RUNNING] `rustdoc [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-after"));
}

#[cargo_test(nightly, reason = "`rustdoc --emit` is unstable")]
fn rebuild_tracks_env_in_dep() {
    let env = "__RUSTDOC_INJECTED";
    Package::new("bar", "0.1.0")
        .file("src/lib.rs", &format!(r#"#![doc = env!("{env}")]"#))
        .publish();

    let env = "__RUSTDOC_INJECTED";
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                edition = "2015"
                [dependencies]
                bar = "0.1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("doc -Zrustdoc-depinfo")
        .env(env, "# depinfo-before")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(
            str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 (registry `dummy-registry`)
[CHECKING] bar v0.1.0
[DOCUMENTING] bar v0.1.0
[DOCUMENTING] foo v0.0.0 ([ROOT]/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();

    let doc_html = p.read_file("target/doc/bar/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.cargo("doc --verbose -Zrustdoc-depinfo")
        .env(env, "# depinfo-after")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(
            str![[r#"
[DIRTY] bar v0.1.0: the environment variable __RUSTDOC_INJECTED changed
[DOCUMENTING] bar v0.1.0
[DIRTY] bar v0.1.0: the environment variable __RUSTDOC_INJECTED changed
[CHECKING] bar v0.1.0
[RUNNING] `rustc --crate-name bar [..]`
[RUNNING] `rustdoc [..]--crate-name bar [..]`
[DIRTY] foo v0.0.0 ([ROOT]/foo): the dependency bar was rebuilt
[DOCUMENTING] foo v0.0.0 ([ROOT]/foo)
[RUNNING] `rustdoc [..]--crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/foo/target/doc/foo/index.html

"#]]
            .unordered(),
        )
        .run();

    let doc_html = p.read_file("target/doc/bar/index.html");
    assert!(doc_html.contains("depinfo-after"));
}

#[cargo_test(
    nightly,
    reason = "`rustdoc --emit` is unstable; requires -Zchecksum-hash-algorithm"
)]
fn rebuild_tracks_checksum() {
    let p = cargo_test_support::project_in("parent")
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", r#"#![doc = include_str!("../../README")]"#)
        .file("../README", "# depinfo-before")
        .build();

    p.cargo("doc -Zrustdoc-depinfo -Zchecksum-freshness")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo", "checksum-freshness"])
        .with_stderr_data(str![[r#"
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-before"));

    p.change_file("../README", "# depinfo-after");
    // Change mtime into the future
    p.root().move_into_the_future();

    p.cargo("doc --verbose -Zrustdoc-depinfo -Zchecksum-freshness")
        .masquerade_as_nightly_cargo(&["rustdoc-depinfo"])
        .with_stderr_data(str![[r#"
[DIRTY] foo v0.5.0 ([ROOT]/parent/foo): file size changed (16 != 15) for `src/../../README`
[DOCUMENTING] foo v0.5.0 ([ROOT]/parent/foo)
[RUNNING] `rustdoc [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
[GENERATED] [ROOT]/parent/foo/target/doc/foo/index.html

"#]])
        .run();

    let doc_html = p.read_file("target/doc/foo/index.html");
    assert!(doc_html.contains("depinfo-after"));
}
