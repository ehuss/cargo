//! Tests for the `cargo build` command.

use cargo::{
    core::compiler::CompileMode,
    core::{Shell, Workspace},
    ops::CompileOptions,
    util::paths::dylib_path_envvar,
    Config,
};
use cargo_test_support::paths::{root, CargoPathExt};
use cargo_test_support::registry::Package;
use cargo_test_support::{
    basic_bin_manifest, basic_lib_manifest, basic_manifest, is_nightly, lines_match, main_file,
    paths, project, rustc_host, sleep_ms, symlink_supported, t, Execs, ProjectBuilder,
};
use std::env;
use std::fs;
use std::io::Read;
use std::process::Stdio;

#[cargo_test]
fn cargo_compile_simple() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("i am foo\n").run();
}

#[cargo_test]
fn cargo_fail_with_no_stderr() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &String::from("refusal"))
        .build();
    p.cargo("build --message-format=json")
        .with_status(101)
        .with_stderr_does_not_contain("--- stderr")
        .run();
}

/// Checks that the `CARGO_INCREMENTAL` environment variable results in
/// `rustc` getting `-C incremental` passed to it.
#[cargo_test]
fn cargo_compile_incremental() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    p.cargo("build -v")
        .env("CARGO_INCREMENTAL", "1")
        .with_stderr_contains(
            "[RUNNING] `rustc [..] -C incremental=[..]/target/debug/incremental[..]`\n",
        )
        .run();

    p.cargo("test -v")
        .env("CARGO_INCREMENTAL", "1")
        .with_stderr_contains(
            "[RUNNING] `rustc [..] -C incremental=[..]/target/debug/incremental[..]`\n",
        )
        .run();
}

#[cargo_test]
fn incremental_profile() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []

            [profile.dev]
            incremental = false

            [profile.release]
            incremental = true
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .env_remove("CARGO_INCREMENTAL")
        .with_stderr_does_not_contain("[..]C incremental=[..]")
        .run();

    p.cargo("build -v")
        .env("CARGO_INCREMENTAL", "1")
        .with_stderr_contains("[..]C incremental=[..]")
        .run();

    p.cargo("build --release -v")
        .env_remove("CARGO_INCREMENTAL")
        .with_stderr_contains("[..]C incremental=[..]")
        .run();

    p.cargo("build --release -v")
        .env("CARGO_INCREMENTAL", "0")
        .with_stderr_does_not_contain("[..]C incremental=[..]")
        .run();
}

#[cargo_test]
fn incremental_config() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file(
            ".cargo/config",
            r#"
            [build]
            incremental = false
        "#,
        )
        .build();

    p.cargo("build -v")
        .env_remove("CARGO_INCREMENTAL")
        .with_stderr_does_not_contain("[..]C incremental=[..]")
        .run();

    p.cargo("build -v")
        .env("CARGO_INCREMENTAL", "1")
        .with_stderr_contains("[..]C incremental=[..]")
        .run();
}

#[cargo_test]
fn cargo_compile_with_workspace_excluded() {
    let p = project().file("src/main.rs", "fn main() {}").build();

    p.cargo("build --workspace --exclude foo")
        .with_stderr_does_not_contain("[..]virtual[..]")
        .with_stderr_contains("[..]no packages to compile")
        .with_status(101)
        .run();
}

#[cargo_test]
fn cargo_compile_manifest_path() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    p.cargo("build --manifest-path foo/Cargo.toml")
        .cwd(p.root().parent().unwrap())
        .run();
    assert!(p.bin("foo").is_file());
}

#[cargo_test]
fn cargo_compile_with_invalid_manifest() {
    let p = project().file("Cargo.toml", "").build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  virtual manifests must be configured with [workspace]
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_manifest2() {
    let p = project()
        .file(
            "Cargo.toml",
            r"
            [project]
            foo = bar
        ",
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  could not parse input as TOML

Caused by:
  invalid number at line 3 column 19
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_manifest3() {
    let p = project().file("src/Cargo.toml", "a = bar").build();

    p.cargo("build --manifest-path src/Cargo.toml")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  could not parse input as TOML

Caused by:
  invalid number at line 1 column 5
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_duplicate_build_targets() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            name = "main"
            path = "src/main.rs"
            crate-type = ["dylib"]

            [dependencies]
        "#,
        )
        .file("src/main.rs", "#![allow(warnings)] fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
warning: file found to be present in multiple build targets: [..]main.rs
[COMPILING] foo v0.0.1 ([..])
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_version() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo", "1.0"))
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  Expected dot for key `package.version`
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_empty_package_name() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("", "0.0.0"))
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  package name cannot be an empty string
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_package_name() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo::bar", "0.0.0"))
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  invalid character `:` in package name: `foo::bar`, [..]
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_bin_target_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [[bin]]
            name = ""
        "#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  binary target names cannot be empty
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_forbidden_bin_target_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [[bin]]
            name = "build"
        "#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  the binary target name `build` is forbidden
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_bin_and_crate_type() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [[bin]]
            name = "the_foo_bin"
            path = "src/foo.rs"
            crate-type = ["cdylib", "rlib"]
        "#,
        )
        .file("src/foo.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  the target `the_foo_bin` is a binary and can't have any crate-types set \
(currently \"cdylib, rlib\")",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_api_exposes_artifact_paths() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [[bin]]
            name = "the_foo_bin"
            path = "src/bin.rs"

            [lib]
            name = "the_foo_lib"
            path = "src/foo.rs"
            crate-type = ["cdylib", "rlib"]
        "#,
        )
        .file("src/foo.rs", "pub fn bar() {}")
        .file("src/bin.rs", "pub fn main() {}")
        .build();

    let shell = Shell::from_write(Box::new(Vec::new()));
    let config = Config::new(shell, env::current_dir().unwrap(), paths::home());
    let ws = Workspace::new(&p.root().join("Cargo.toml"), &config).unwrap();
    let compile_options = CompileOptions::new(ws.config(), CompileMode::Build).unwrap();

    let result = cargo::ops::compile(&ws, &compile_options).unwrap();

    assert_eq!(1, result.binaries.len());
    assert!(result.binaries[0].1.exists());
    assert!(result.binaries[0]
        .1
        .to_str()
        .unwrap()
        .contains("the_foo_bin"));

    assert_eq!(1, result.cdylibs.len());
    // The exact library path varies by platform, but should certainly exist at least
    assert!(result.cdylibs[0].1.exists());
    assert!(result.cdylibs[0]
        .1
        .to_str()
        .unwrap()
        .contains("the_foo_lib"));
}

#[cargo_test]
fn cargo_compile_with_bin_and_proc() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [[bin]]
            name = "the_foo_bin"
            path = "src/foo.rs"
            proc-macro = true
        "#,
        )
        .file("src/foo.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  the target `the_foo_bin` is a binary and can't have `proc-macro` set `true`",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_lib_target_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"

            [lib]
            name = ""
        "#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  library target names cannot be empty
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_non_numeric_dep_version() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            crossbeam = "y"
        "#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  failed to parse the version requirement `y` for dependency `crossbeam`

Caused by:
  the given version requirement is invalid
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_without_manifest() {
    let p = project().no_manifest().build();

    p.cargo("build")
        .with_status(101)
        .with_stderr("[ERROR] could not find `Cargo.toml` in `[..]` or any parent directory")
        .run();
}

#[cargo_test]
fn cargo_compile_with_invalid_code() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", "invalid rust code!")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains(
            "\
[ERROR] could not compile `foo`.

To learn more, run the command again with --verbose.\n",
        )
        .run();
    assert!(p.root().join("Cargo.lock").is_file());
}

#[cargo_test]
fn cargo_compile_with_invalid_code_in_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "../bar"
            [dependencies.baz]
            path = "../baz"
        "#,
        )
        .file("src/main.rs", "invalid rust code!")
        .build();
    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "invalid rust code!")
        .build();
    let _baz = project()
        .at("baz")
        .file("Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("src/lib.rs", "invalid rust code!")
        .build();
    p.cargo("build")
        .with_status(101)
        .with_stderr_contains("[..]invalid rust code[..]")
        .with_stderr_contains("[ERROR] could not compile [..]")
        .run();
}

#[cargo_test]
fn cargo_compile_with_warnings_in_the_root_package() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", "fn main() {} fn dead() {}")
        .build();

    p.cargo("build")
        .with_stderr_contains("[..]function is never used: `dead`[..]")
        .run();
}

#[cargo_test]
fn cargo_compile_with_warnings_in_a_dep_package() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = "bar"

            [[bin]]

            name = "foo"
        "#,
        )
        .file("src/foo.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file("bar/Cargo.toml", &basic_lib_manifest("bar"))
        .file(
            "bar/src/bar.rs",
            r#"
            pub fn gimme() -> &'static str {
                "test passed"
            }

            fn dead() {}
        "#,
        )
        .build();

    p.cargo("build")
        .with_stderr_contains("[..]function is never used: `dead`[..]")
        .run();

    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("test passed\n").run();
}

#[cargo_test]
fn cargo_compile_with_nested_deps_inferred() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = 'bar'

            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/foo.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file(
            "bar/Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.baz]
            path = "../baz"
        "#,
        )
        .file(
            "bar/src/lib.rs",
            r#"
            extern crate baz;

            pub fn gimme() -> String {
                baz::gimme()
            }
        "#,
        )
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.5.0"))
        .file(
            "baz/src/lib.rs",
            r#"
            pub fn gimme() -> String {
                "test passed".to_string()
            }
        "#,
        )
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    assert!(!p.bin("libbar.rlib").is_file());
    assert!(!p.bin("libbaz.rlib").is_file());

    p.process(&p.bin("foo")).with_stdout("test passed\n").run();
}

#[cargo_test]
fn cargo_compile_with_nested_deps_correct_bin() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = "bar"

            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file(
            "bar/Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.baz]
            path = "../baz"
        "#,
        )
        .file(
            "bar/src/lib.rs",
            r#"
            extern crate baz;

            pub fn gimme() -> String {
                baz::gimme()
            }
        "#,
        )
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.5.0"))
        .file(
            "baz/src/lib.rs",
            r#"
            pub fn gimme() -> String {
                "test passed".to_string()
            }
        "#,
        )
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    assert!(!p.bin("libbar.rlib").is_file());
    assert!(!p.bin("libbaz.rlib").is_file());

    p.process(&p.bin("foo")).with_stdout("test passed\n").run();
}

#[cargo_test]
fn cargo_compile_with_nested_deps_shorthand() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/main.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file(
            "bar/Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.baz]
            path = "../baz"

            [lib]

            name = "bar"
        "#,
        )
        .file(
            "bar/src/bar.rs",
            r#"
            extern crate baz;

            pub fn gimme() -> String {
                baz::gimme()
            }
        "#,
        )
        .file("baz/Cargo.toml", &basic_lib_manifest("baz"))
        .file(
            "baz/src/baz.rs",
            r#"
            pub fn gimme() -> String {
                "test passed".to_string()
            }
        "#,
        )
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    assert!(!p.bin("libbar.rlib").is_file());
    assert!(!p.bin("libbaz.rlib").is_file());

    p.process(&p.bin("foo")).with_stdout("test passed\n").run();
}

#[cargo_test]
fn cargo_compile_with_nested_deps_longhand() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = "bar"
            version = "0.5.0"

            [[bin]]

            name = "foo"
        "#,
        )
        .file("src/foo.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file(
            "bar/Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.baz]
            path = "../baz"
            version = "0.5.0"

            [lib]

            name = "bar"
        "#,
        )
        .file(
            "bar/src/bar.rs",
            r#"
            extern crate baz;

            pub fn gimme() -> String {
                baz::gimme()
            }
        "#,
        )
        .file("baz/Cargo.toml", &basic_lib_manifest("baz"))
        .file(
            "baz/src/baz.rs",
            r#"
            pub fn gimme() -> String {
                "test passed".to_string()
            }
        "#,
        )
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    assert!(!p.bin("libbar.rlib").is_file());
    assert!(!p.bin("libbaz.rlib").is_file());

    p.process(&p.bin("foo")).with_stdout("test passed\n").run();
}

// Check that Cargo gives a sensible error if a dependency can't be found
// because of a name mismatch.
#[cargo_test]
fn cargo_compile_with_dep_name_mismatch() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]

            name = "foo"
            version = "0.0.1"
            authors = ["wycats@example.com"]

            [[bin]]

            name = "foo"

            [dependencies.notquitebar]

            path = "bar"
        "#,
        )
        .file("src/bin/foo.rs", &main_file(r#""i am foo""#, &["bar"]))
        .file("bar/Cargo.toml", &basic_bin_manifest("bar"))
        .file("bar/src/bar.rs", &main_file(r#""i am bar""#, &[]))
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            r#"error: no matching package named `notquitebar` found
location searched: [CWD]/bar
required by package `foo v0.0.1 ([CWD])`
"#,
        )
        .run();
}

// Ensure that renamed deps have a valid name
#[cargo_test]
fn cargo_compile_with_invalid_dep_rename() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "buggin"
            version = "0.1.0"

            [dependencies]
            "haha this isn't a valid name 🐛" = { package = "libc", version = "0.1" }
        "#,
        )
        .file("src/main.rs", &main_file(r#""What's good?""#, &[]))
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]`

Caused by:
  invalid character ` ` in dependency name: `haha this isn't a valid name 🐛`, characters must be Unicode XID characters (numbers, `-`, `_`, or most letters)
",
        )
        .run();
}

#[cargo_test]
fn cargo_compile_with_filename() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "src/bin/a.rs",
            r#"
            extern crate foo;
            fn main() { println!("hello a.rs"); }
        "#,
        )
        .file("examples/a.rs", r#"fn main() { println!("example"); }"#)
        .build();

    p.cargo("build --bin bin.rs")
        .with_status(101)
        .with_stderr("[ERROR] no bin target named `bin.rs`")
        .run();

    p.cargo("build --bin a.rs")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] no bin target named `a.rs`

<tab>Did you mean `a`?",
        )
        .run();

    p.cargo("build --example example.rs")
        .with_status(101)
        .with_stderr("[ERROR] no example target named `example.rs`")
        .run();

    p.cargo("build --example a.rs")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] no example target named `a.rs`

<tab>Did you mean `a`?",
        )
        .run();
}

#[cargo_test]
fn incompatible_dependencies() {
    Package::new("bad", "0.1.0").publish();
    Package::new("bad", "1.0.0").publish();
    Package::new("bad", "1.0.1").publish();
    Package::new("bad", "1.0.2").publish();
    Package::new("bar", "0.1.0").dep("bad", "0.1.0").publish();
    Package::new("baz", "0.1.1").dep("bad", "=1.0.0").publish();
    Package::new("baz", "0.1.0").dep("bad", "=1.0.0").publish();
    Package::new("qux", "0.1.2").dep("bad", ">=1.0.1").publish();
    Package::new("qux", "0.1.1").dep("bad", ">=1.0.1").publish();
    Package::new("qux", "0.1.0").dep("bad", ">=1.0.1").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            bar = "0.1.0"
            baz = "0.1.0"
            qux = "0.1.0"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains(
            "\
error: failed to select a version for `bad`.
    ... required by package `qux v0.1.0`
    ... which is depended on by `foo v0.0.1 ([..])`
versions that meet the requirements `>=1.0.1` are: 1.0.2, 1.0.1

all possible versions conflict with previously selected packages.

  previously selected package `bad v1.0.0`
    ... which is depended on by `baz v0.1.0`
    ... which is depended on by `foo v0.0.1 ([..])`

failed to select a version for `bad` which could resolve this conflict",
        )
        .run();
}

#[cargo_test]
fn incompatible_dependencies_with_multi_semver() {
    Package::new("bad", "1.0.0").publish();
    Package::new("bad", "1.0.1").publish();
    Package::new("bad", "2.0.0").publish();
    Package::new("bad", "2.0.1").publish();
    Package::new("bar", "0.1.0").dep("bad", "=1.0.0").publish();
    Package::new("baz", "0.1.0").dep("bad", ">=2.0.1").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            bar = "0.1.0"
            baz = "0.1.0"
            bad = ">=1.0.1, <=2.0.0"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains(
            "\
error: failed to select a version for `bad`.
    ... required by package `foo v0.0.1 ([..])`
versions that meet the requirements `>=1.0.1, <=2.0.0` are: 2.0.0, 1.0.1

all possible versions conflict with previously selected packages.

  previously selected package `bad v2.0.1`
    ... which is depended on by `baz v0.1.0`
    ... which is depended on by `foo v0.0.1 ([..])`

  previously selected package `bad v1.0.0`
    ... which is depended on by `bar v0.1.0`
    ... which is depended on by `foo v0.0.1 ([..])`

failed to select a version for `bad` which could resolve this conflict",
        )
        .run();
}

#[cargo_test]
fn compile_path_dep_then_change_version() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("build").run();

    p.change_file("bar/Cargo.toml", &basic_manifest("bar", "0.0.2"));

    p.cargo("build").run();
}

#[cargo_test]
fn ignores_carriage_return_in_lockfile() {
    let p = project()
        .file("src/main.rs", r"mod a; fn main() {}")
        .file("src/a.rs", "")
        .build();

    p.cargo("build").run();

    let lock = p.read_lockfile();
    p.change_file("Cargo.lock", &lock.replace("\n", "\r\n"));
    p.cargo("build").run();
}

#[cargo_test]
fn cargo_default_env_metadata_env_var() {
    // Ensure that path dep + dylib + env_var get metadata
    // (even though path_dep + dylib should not)
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/lib.rs", "// hi")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []

            [lib]
            name = "bar"
            crate_type = ["dylib"]
        "#,
        )
        .file("bar/src/lib.rs", "// hello")
        .build();

    // No metadata on libbar since it's a dylib path dependency
    p.cargo("build -v")
        .with_stderr(&format!(
            "\
[COMPILING] bar v0.0.1 ([CWD]/bar)
[RUNNING] `rustc --crate-name bar bar/src/lib.rs [..]--crate-type dylib \
        --emit=[..]link \
        -C prefer-dynamic[..]-C debuginfo=2 \
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps`
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]-C debuginfo=2 \
        -C metadata=[..] \
        -C extra-filename=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps \
        --extern bar=[CWD]/target/debug/deps/{prefix}bar{suffix}`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
            prefix = env::consts::DLL_PREFIX,
            suffix = env::consts::DLL_SUFFIX,
        ))
        .run();

    p.cargo("clean").run();

    // If you set the env-var, then we expect metadata on libbar
    p.cargo("build -v")
        .env("__CARGO_DEFAULT_LIB_METADATA", "stable")
        .with_stderr(&format!(
            "\
[COMPILING] bar v0.0.1 ([CWD]/bar)
[RUNNING] `rustc --crate-name bar bar/src/lib.rs [..]--crate-type dylib \
        --emit=[..]link \
        -C prefer-dynamic[..]-C debuginfo=2 \
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps`
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]-C debuginfo=2 \
        -C metadata=[..] \
        -C extra-filename=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps \
        --extern bar=[CWD]/target/debug/deps/{prefix}bar-[..]{suffix}`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            prefix = env::consts::DLL_PREFIX,
            suffix = env::consts::DLL_SUFFIX,
        ))
        .run();
}

#[cargo_test]
fn crate_env_vars() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
        [project]
        name = "foo"
        version = "0.5.1-alpha.1"
        description = "This is foo"
        homepage = "https://example.com"
        repository = "https://example.com/repo.git"
        authors = ["wycats@example.com"]
        license = "MIT OR Apache-2.0"
        license_file = "license.txt"

        [[bin]]
        name = "foo-bar"
        path = "src/main.rs"
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            extern crate foo;


            static VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
            static VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");
            static VERSION_PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");
            static VERSION_PRE: &'static str = env!("CARGO_PKG_VERSION_PRE");
            static VERSION: &'static str = env!("CARGO_PKG_VERSION");
            static CARGO_MANIFEST_DIR: &'static str = env!("CARGO_MANIFEST_DIR");
            static PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
            static HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
            static REPOSITORY: &'static str = env!("CARGO_PKG_REPOSITORY");
            static LICENSE: &'static str = env!("CARGO_PKG_LICENSE");
            static LICENSE_FILE: &'static str = env!("CARGO_PKG_LICENSE_FILE");
            static DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
            static BIN_NAME: &'static str = env!("CARGO_BIN_NAME");
            static CRATE_NAME: &'static str = env!("CARGO_CRATE_NAME");


            fn main() {
                let s = format!("{}-{}-{} @ {} in {}", VERSION_MAJOR,
                                VERSION_MINOR, VERSION_PATCH, VERSION_PRE,
                                CARGO_MANIFEST_DIR);
                 assert_eq!(s, foo::version());
                 println!("{}", s);
                 assert_eq!("foo", PKG_NAME);
                 assert_eq!("foo-bar", BIN_NAME);
                 assert_eq!("foo_bar", CRATE_NAME);
                 assert_eq!("https://example.com", HOMEPAGE);
                 assert_eq!("https://example.com/repo.git", REPOSITORY);
                 assert_eq!("MIT OR Apache-2.0", LICENSE);
                 assert_eq!("This is foo", DESCRIPTION);
                let s = format!("{}.{}.{}-{}", VERSION_MAJOR,
                                VERSION_MINOR, VERSION_PATCH, VERSION_PRE);
                assert_eq!(s, VERSION);
            }
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn version() -> String {
                format!("{}-{}-{} @ {} in {}",
                        env!("CARGO_PKG_VERSION_MAJOR"),
                        env!("CARGO_PKG_VERSION_MINOR"),
                        env!("CARGO_PKG_VERSION_PATCH"),
                        env!("CARGO_PKG_VERSION_PRE"),
                        env!("CARGO_MANIFEST_DIR"))
            }
        "#,
        )
        .build();

    println!("build");
    p.cargo("build -v").run();

    println!("bin");
    p.process(&p.bin("foo-bar"))
        .with_stdout("0-5-1 @ alpha.1 in [CWD]")
        .run();

    println!("test");
    p.cargo("test -v").run();
}

#[cargo_test]
fn crate_authors_env_vars() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.1-alpha.1"
            authors = ["wycats@example.com", "neikos@example.com"]
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            extern crate foo;

            static AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

            fn main() {
                let s = "wycats@example.com:neikos@example.com";
                assert_eq!(AUTHORS, foo::authors());
                println!("{}", AUTHORS);
                assert_eq!(s, AUTHORS);
            }
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn authors() -> String {
                format!("{}", env!("CARGO_PKG_AUTHORS"))
            }
        "#,
        )
        .build();

    println!("build");
    p.cargo("build -v").run();

    println!("bin");
    p.process(&p.bin("foo"))
        .with_stdout("wycats@example.com:neikos@example.com")
        .run();

    println!("test");
    p.cargo("test -v").run();
}

#[cargo_test]
fn vv_prints_rustc_env_vars() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = ["escape='\"@example.com"]
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    let mut b = p.cargo("build -vv");

    if cfg!(windows) {
        b.with_stderr_contains(
            "[RUNNING] `[..]set CARGO_PKG_NAME=foo&& [..]rustc [..]`"
        ).with_stderr_contains(
            r#"[RUNNING] `[..]set CARGO_PKG_AUTHORS="escape='\"@example.com"&& [..]rustc [..]`"#
        )
    } else {
        b.with_stderr_contains("[RUNNING] `[..]CARGO_PKG_NAME=foo [..]rustc [..]`")
            .with_stderr_contains(
                r#"[RUNNING] `[..]CARGO_PKG_AUTHORS='escape='\''"@example.com' [..]rustc [..]`"#,
            )
    };

    b.run();
}

// The tester may already have LD_LIBRARY_PATH=::/foo/bar which leads to a false positive error
fn setenv_for_removing_empty_component(mut execs: Execs) -> Execs {
    let v = dylib_path_envvar();
    if let Ok(search_path) = env::var(v) {
        let new_search_path =
            env::join_paths(env::split_paths(&search_path).filter(|e| !e.as_os_str().is_empty()))
                .expect("join_paths");
        execs.env(v, new_search_path); // build_command() will override LD_LIBRARY_PATH accordingly
    }
    execs
}

// Regression test for #4277
#[cargo_test]
fn crate_library_path_env_var() {
    let p = project()
        .file(
            "src/main.rs",
            &format!(
                r##"
            fn main() {{
                let search_path = env!("{}");
                let paths = std::env::split_paths(&search_path).collect::<Vec<_>>();
                assert!(!paths.contains(&"".into()));
            }}
        "##,
                dylib_path_envvar()
            ),
        )
        .build();

    setenv_for_removing_empty_component(p.cargo("run")).run();
}

// Regression test for #4277
#[cargo_test]
fn build_with_fake_libc_not_loading() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", r#" "#)
        .file("libc.so.6", r#""#)
        .build();

    setenv_for_removing_empty_component(p.cargo("build")).run();
}

// this is testing that src/<pkg-name>.rs still works (for now)
#[cargo_test]
fn many_crate_types_old_style_lib_location() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [lib]

            name = "foo"
            crate_type = ["rlib", "dylib"]
        "#,
        )
        .file("src/foo.rs", "pub fn foo() {}")
        .build();
    p.cargo("build")
        .with_stderr_contains(
            "\
[WARNING] path `[..]src/foo.rs` was erroneously implicitly accepted for library `foo`,
please rename the file to `src/lib.rs` or set lib.path in Cargo.toml",
        )
        .run();

    assert!(p.root().join("target/debug/libfoo.rlib").is_file());
    let fname = format!("{}foo{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
    assert!(p.root().join("target/debug").join(&fname).is_file());
}

#[cargo_test]
fn many_crate_types_correct() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [lib]

            name = "foo"
            crate_type = ["rlib", "dylib"]
        "#,
        )
        .file("src/lib.rs", "pub fn foo() {}")
        .build();
    p.cargo("build").run();

    assert!(p.root().join("target/debug/libfoo.rlib").is_file());
    let fname = format!("{}foo{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
    assert!(p.root().join("target/debug").join(&fname).is_file());
}

#[cargo_test]
fn self_dependency() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]

            name = "test"
            version = "0.0.0"
            authors = []

            [dependencies.test]

            path = "."

            [lib]
            name = "test"
            path = "src/test.rs"
        "#,
        )
        .file("src/test.rs", "fn main() {}")
        .build();
    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] cyclic package dependency: package `test v0.0.0 ([CWD])` depends on itself. Cycle:
package `test v0.0.0 ([CWD])`
    ... which is depended on by `test v0.0.0 ([..])`",
        )
        .run();
}

#[cargo_test]
/// Make sure broken symlinks don't break the build
///
/// This test requires you to be able to make symlinks.
/// For windows, this may require you to enable developer mode.
fn ignore_broken_symlinks() {
    if !symlink_supported() {
        return;
    }

    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .symlink("Notafile", "bar")
        .build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("i am foo\n").run();
}

#[cargo_test]
fn missing_lib_and_bin() {
    let p = project().build();
    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]Cargo.toml`

Caused by:
  no targets specified in the manifest
  either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present\n",
        )
        .run();
}

#[cargo_test]
fn lto_build() {
    // FIXME: currently this hits a linker bug on 32-bit MSVC
    if cfg!(all(target_env = "msvc", target_pointer_width = "32")) {
        return;
    }

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]

            name = "test"
            version = "0.0.0"
            authors = []

            [profile.release]
            lto = true
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("build -v --release")
        .with_stderr(
            "\
[COMPILING] test v0.0.0 ([CWD])
[RUNNING] `rustc --crate-name test src/main.rs [..]--crate-type bin \
        --emit=[..]link \
        -C opt-level=3 \
        -C lto \
        -C metadata=[..] \
        --out-dir [CWD]/target/release/deps \
        -L dependency=[CWD]/target/release/deps`
[FINISHED] release [optimized] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn verbose_build() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]-C debuginfo=2 \
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn verbose_release_build() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v --release")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]\
        -C opt-level=3[..]\
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/release/deps`
[FINISHED] release [optimized] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn verbose_release_build_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]

            name = "test"
            version = "0.0.0"
            authors = []

            [dependencies.foo]
            path = "foo"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "foo/Cargo.toml",
            r#"
            [package]

            name = "foo"
            version = "0.0.0"
            authors = []

            [lib]
            name = "foo"
            crate_type = ["dylib", "rlib"]
        "#,
        )
        .file("foo/src/lib.rs", "")
        .build();
    p.cargo("build -v --release")
        .with_stderr(&format!(
            "\
[COMPILING] foo v0.0.0 ([CWD]/foo)
[RUNNING] `rustc --crate-name foo foo/src/lib.rs [..]\
        --crate-type dylib --crate-type rlib \
        --emit=[..]link \
        -C prefer-dynamic[..]\
        -C opt-level=3[..]\
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/release/deps`
[COMPILING] test v0.0.0 ([CWD])
[RUNNING] `rustc --crate-name test src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]\
        -C opt-level=3[..]\
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/release/deps \
        --extern foo=[CWD]/target/release/deps/{prefix}foo{suffix} \
        --extern foo=[CWD]/target/release/deps/libfoo.rlib`
[FINISHED] release [optimized] target(s) in [..]
",
            prefix = env::consts::DLL_PREFIX,
            suffix = env::consts::DLL_SUFFIX
        ))
        .run();
}

#[cargo_test]
fn explicit_examples() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            authors = []

            [lib]
            name = "foo"
            path = "src/lib.rs"

            [[example]]
            name = "hello"
            path = "examples/ex-hello.rs"

            [[example]]
            name = "goodbye"
            path = "examples/ex-goodbye.rs"
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn get_hello() -> &'static str { "Hello" }
            pub fn get_goodbye() -> &'static str { "Goodbye" }
            pub fn get_world() -> &'static str { "World" }
        "#,
        )
        .file(
            "examples/ex-hello.rs",
            r#"
            extern crate foo;
            fn main() { println!("{}, {}!", foo::get_hello(), foo::get_world()); }
        "#,
        )
        .file(
            "examples/ex-goodbye.rs",
            r#"
            extern crate foo;
            fn main() { println!("{}, {}!", foo::get_goodbye(), foo::get_world()); }
        "#,
        )
        .build();

    p.cargo("build --examples").run();
    p.process(&p.bin("examples/hello"))
        .with_stdout("Hello, World!\n")
        .run();
    p.process(&p.bin("examples/goodbye"))
        .with_stdout("Goodbye, World!\n")
        .run();
}

#[cargo_test]
fn non_existing_example() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            authors = []

            [lib]
            name = "foo"
            path = "src/lib.rs"

            [[example]]
            name = "hello"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("test -v")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  can't find `hello` example, specify example.path",
        )
        .run();
}

#[cargo_test]
fn non_existing_binary() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/lib.rs", "")
        .file("src/bin/ehlo.rs", "")
        .build();

    p.cargo("build -v")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  can't find `foo` bin, specify bin.path",
        )
        .run();
}

#[cargo_test]
fn legacy_binary_paths_warnings() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            authors = []

            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr_contains(
            "\
[WARNING] path `[..]src/main.rs` was erroneously implicitly accepted for binary `bar`,
please set bin.path in Cargo.toml",
        )
        .run();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            authors = []

            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/bin/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr_contains(
            "\
[WARNING] path `[..]src/bin/main.rs` was erroneously implicitly accepted for binary `bar`,
please set bin.path in Cargo.toml",
        )
        .run();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.0.0"
            authors = []

            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/bar.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr_contains(
            "\
[WARNING] path `[..]src/bar.rs` was erroneously implicitly accepted for binary `bar`,
please set bin.path in Cargo.toml",
        )
        .run();
}

#[cargo_test]
fn implicit_examples() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
            pub fn get_hello() -> &'static str { "Hello" }
            pub fn get_goodbye() -> &'static str { "Goodbye" }
            pub fn get_world() -> &'static str { "World" }
        "#,
        )
        .file(
            "examples/hello.rs",
            r#"
            extern crate foo;
            fn main() {
                println!("{}, {}!", foo::get_hello(), foo::get_world());
            }
        "#,
        )
        .file(
            "examples/goodbye.rs",
            r#"
            extern crate foo;
            fn main() {
                println!("{}, {}!", foo::get_goodbye(), foo::get_world());
            }
        "#,
        )
        .build();

    p.cargo("build --examples").run();
    p.process(&p.bin("examples/hello"))
        .with_stdout("Hello, World!\n")
        .run();
    p.process(&p.bin("examples/goodbye"))
        .with_stdout("Goodbye, World!\n")
        .run();
}

#[cargo_test]
fn standard_build_no_ndebug() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/foo.rs",
            r#"
            fn main() {
                if cfg!(debug_assertions) {
                    println!("slow")
                } else {
                    println!("fast")
                }
            }
        "#,
        )
        .build();

    p.cargo("build").run();
    p.process(&p.bin("foo")).with_stdout("slow\n").run();
}

#[cargo_test]
fn release_build_ndebug() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/foo.rs",
            r#"
            fn main() {
                if cfg!(debug_assertions) {
                    println!("slow")
                } else {
                    println!("fast")
                }
            }
        "#,
        )
        .build();

    p.cargo("build --release").run();
    p.process(&p.release_bin("foo")).with_stdout("fast\n").run();
}

#[cargo_test]
fn inferred_main_bin() {
    let p = project().file("src/main.rs", "fn main() {}").build();

    p.cargo("build").run();
    p.process(&p.bin("foo")).run();
}

#[cargo_test]
fn deletion_causes_failure() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/main.rs", "extern crate bar; fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.change_file("Cargo.toml", &basic_manifest("foo", "0.0.1"));
    p.cargo("build")
        .with_status(101)
        .with_stderr_contains("[..]can't find crate for `bar`")
        .run();
}

#[cargo_test]
fn bad_cargo_toml_in_target_dir() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("target/Cargo.toml", "bad-toml")
        .build();

    p.cargo("build").run();
    p.process(&p.bin("foo")).run();
}

#[cargo_test]
fn lib_with_standard_name() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("syntax", "0.0.1"))
        .file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "extern crate syntax; fn main() { syntax::foo() }",
        )
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] syntax v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn simple_staticlib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
              [package]
              name = "foo"
              authors = []
              version = "0.0.1"

              [lib]
              name = "foo"
              crate-type = ["staticlib"]
        "#,
        )
        .file("src/lib.rs", "pub fn foo() {}")
        .build();

    // env var is a test for #1381
    p.cargo("build").env("CARGO_LOG", "nekoneko=trace").run();
}

#[cargo_test]
fn staticlib_rlib_and_bin() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
              [package]
              name = "foo"
              authors = []
              version = "0.0.1"

              [lib]
              name = "foo"
              crate-type = ["staticlib", "rlib"]
        "#,
        )
        .file("src/lib.rs", "pub fn foo() {}")
        .file("src/main.rs", "extern crate foo; fn main() { foo::foo(); }")
        .build();

    p.cargo("build -v").run();
}

#[cargo_test]
fn opt_out_of_bin() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
              bin = []

              [package]
              name = "foo"
              authors = []
              version = "0.0.1"
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/main.rs", "bad syntax")
        .build();
    p.cargo("build").run();
}

#[cargo_test]
fn single_lib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
              [package]
              name = "foo"
              authors = []
              version = "0.0.1"

              [lib]
              name = "foo"
              path = "src/bar.rs"
        "#,
        )
        .file("src/bar.rs", "")
        .build();
    p.cargo("build").run();
}

#[cargo_test]
fn freshness_ignores_excluded() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.0"
            authors = []
            build = "build.rs"
            exclude = ["src/b*.rs"]
        "#,
        )
        .file("build.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn bar() -> i32 { 1 }")
        .build();
    foo.root().move_into_the_past();

    foo.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    // Smoke test to make sure it doesn't compile again
    println!("first pass");
    foo.cargo("build").with_stdout("").run();

    // Modify an ignored file and make sure we don't rebuild
    println!("second pass");
    foo.change_file("src/bar.rs", "");
    foo.cargo("build").with_stdout("").run();
}

#[cargo_test]
fn rebuild_preserves_out_dir() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.0"
            authors = []
            build = 'build.rs'
        "#,
        )
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::fs::File;
            use std::path::Path;

            fn main() {
                let path = Path::new(&env::var("OUT_DIR").unwrap()).join("foo");
                if env::var_os("FIRST").is_some() {
                    File::create(&path).unwrap();
                } else {
                    File::create(&path).unwrap();
                }
            }
        "#,
        )
        .file("src/lib.rs", "pub fn bar() -> i32 { 1 }")
        .build();
    foo.root().move_into_the_past();

    foo.cargo("build")
        .env("FIRST", "1")
        .with_stderr(
            "\
[COMPILING] foo v0.0.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    foo.change_file("src/bar.rs", "");
    foo.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn dep_no_libs() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.0"
            authors = []

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/lib.rs", "pub fn bar() -> i32 { 1 }")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.0"))
        .file("bar/src/main.rs", "")
        .build();
    foo.cargo("build").run();
}

#[cargo_test]
fn recompile_space_in_name() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.0"
            authors = []

            [lib]
            name = "foo"
            path = "src/my lib.rs"
        "#,
        )
        .file("src/my lib.rs", "")
        .build();
    foo.cargo("build").run();
    foo.root().move_into_the_past();
    foo.cargo("build").with_stdout("").run();
}

#[cfg(unix)]
#[cargo_test]
fn credentials_is_unreadable() {
    use cargo_test_support::paths::home;
    use std::os::unix::prelude::*;
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo", "0.1.0"))
        .file("src/lib.rs", "")
        .build();

    let credentials = home().join(".cargo/credentials");
    t!(fs::create_dir_all(credentials.parent().unwrap()));
    t!(fs::write(
        &credentials,
        r#"
            [registry]
            token = "api-token"
        "#
    ));
    let stat = fs::metadata(credentials.as_path()).unwrap();
    let mut perms = stat.permissions();
    perms.set_mode(0o000);
    fs::set_permissions(credentials, perms).unwrap();

    p.cargo("build").run();
}

#[cfg(unix)]
#[cargo_test]
fn ignore_bad_directories() {
    use std::os::unix::prelude::*;
    let foo = project()
        .file("Cargo.toml", &basic_manifest("foo", "0.0.0"))
        .file("src/lib.rs", "")
        .build();
    let dir = foo.root().join("tmp");
    fs::create_dir(&dir).unwrap();
    let stat = fs::metadata(&dir).unwrap();
    let mut perms = stat.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&dir, perms.clone()).unwrap();
    foo.cargo("build").run();
    perms.set_mode(0o755);
    fs::set_permissions(&dir, perms).unwrap();
}

#[cargo_test]
fn bad_cargo_config() {
    let foo = project()
        .file("Cargo.toml", &basic_manifest("foo", "0.0.0"))
        .file("src/lib.rs", "")
        .file(".cargo/config", "this is not valid toml")
        .build();
    foo.cargo("build -v")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] could not load Cargo configuration

Caused by:
  could not parse TOML configuration in `[..]`

Caused by:
  could not parse input as TOML

Caused by:
  expected an equals, found an identifier at line 1 column 6
",
        )
        .run();
}

#[cargo_test]
fn cargo_platform_specific_dependency() {
    let host = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]
            build = "build.rs"

            [target.{host}.dependencies]
            dep = {{ path = "dep" }}
            [target.{host}.build-dependencies]
            build = {{ path = "build" }}
            [target.{host}.dev-dependencies]
            dev = {{ path = "dev" }}
        "#,
                host = host
            ),
        )
        .file("src/main.rs", "extern crate dep; fn main() { dep::dep() }")
        .file(
            "tests/foo.rs",
            "extern crate dev; #[test] fn foo() { dev::dev() }",
        )
        .file(
            "build.rs",
            "extern crate build; fn main() { build::build(); }",
        )
        .file("dep/Cargo.toml", &basic_manifest("dep", "0.5.0"))
        .file("dep/src/lib.rs", "pub fn dep() {}")
        .file("build/Cargo.toml", &basic_manifest("build", "0.5.0"))
        .file("build/src/lib.rs", "pub fn build() {}")
        .file("dev/Cargo.toml", &basic_manifest("dev", "0.5.0"))
        .file("dev/src/lib.rs", "pub fn dev() {}")
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    p.cargo("test").run();
}

#[cargo_test]
fn bad_platform_specific_dependency() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [target.wrong-target.dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/main.rs", &main_file(r#""{}", bar::gimme()"#, &["bar"]))
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file(
            "bar/src/lib.rs",
            r#"pub fn gimme() -> String { format!("") }"#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains("[..]can't find crate for `bar`")
        .run();
}

#[cargo_test]
fn cargo_platform_specific_dependency_wrong_platform() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [target.non-existing-triplet.dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file(
            "bar/src/lib.rs",
            "invalid rust file, should not be compiled",
        )
        .build();

    p.cargo("build").run();

    assert!(p.bin("foo").is_file());
    p.process(&p.bin("foo")).run();

    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("bar"));
}

#[cargo_test]
fn example_as_lib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[example]]
            name = "ex"
            crate-type = ["lib"]
        "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "")
        .build();

    p.cargo("build --example=ex").run();
    assert!(p.example_lib("ex", "lib").is_file());
}

#[cargo_test]
fn example_as_rlib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[example]]
            name = "ex"
            crate-type = ["rlib"]
        "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "")
        .build();

    p.cargo("build --example=ex").run();
    assert!(p.example_lib("ex", "rlib").is_file());
}

#[cargo_test]
fn example_as_dylib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[example]]
            name = "ex"
            crate-type = ["dylib"]
        "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "")
        .build();

    p.cargo("build --example=ex").run();
    assert!(p.example_lib("ex", "dylib").is_file());
}

#[cargo_test]
fn example_as_proc_macro() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[example]]
            name = "ex"
            crate-type = ["proc-macro"]
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "examples/ex.rs",
            r#"
            extern crate proc_macro;
            use proc_macro::TokenStream;

            #[proc_macro]
            pub fn eat(_item: TokenStream) -> TokenStream {
                "".parse().unwrap()
            }
            "#,
        )
        .build();

    p.cargo("build --example=ex").run();
    assert!(p.example_lib("ex", "proc-macro").is_file());
}

#[cargo_test]
fn example_bin_same_name() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("examples/foo.rs", "fn main() {}")
        .build();

    p.cargo("build --examples").run();

    assert!(!p.bin("foo").is_file());
    // We expect a file of the form bin/foo-{metadata_hash}
    assert!(p.bin("examples/foo").is_file());

    p.cargo("build --examples").run();

    assert!(!p.bin("foo").is_file());
    // We expect a file of the form bin/foo-{metadata_hash}
    assert!(p.bin("examples/foo").is_file());
}

#[cargo_test]
fn compile_then_delete() {
    let p = project().file("src/main.rs", "fn main() {}").build();

    p.cargo("run -v").run();
    assert!(p.bin("foo").is_file());
    if cfg!(windows) {
        // On windows unlinking immediately after running often fails, so sleep
        sleep_ms(100);
    }
    fs::remove_file(&p.bin("foo")).unwrap();
    p.cargo("run -v").run();
}

#[cargo_test]
fn transitive_dependencies_not_available() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.aaaaa]
            path = "a"
        "#,
        )
        .file(
            "src/main.rs",
            "extern crate bbbbb; extern crate aaaaa; fn main() {}",
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "aaaaa"
            version = "0.0.1"
            authors = []

            [dependencies.bbbbb]
            path = "../b"
        "#,
        )
        .file("a/src/lib.rs", "extern crate bbbbb;")
        .file("b/Cargo.toml", &basic_manifest("bbbbb", "0.0.1"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_status(101)
        .with_stderr_contains("[..] can't find crate for `bbbbb`[..]")
        .run();
}

#[cargo_test]
fn cyclic_deps_rejected() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            authors = []

            [dependencies.foo]
            path = ".."
        "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_status(101)
        .with_stderr(
"[ERROR] cyclic package dependency: package `a v0.0.1 ([CWD]/a)` depends on itself. Cycle:
package `a v0.0.1 ([CWD]/a)`
    ... which is depended on by `foo v0.0.1 ([CWD])`
    ... which is depended on by `a v0.0.1 ([..])`",
        ).run();
}

#[cargo_test]
fn predictable_filenames() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            name = "foo"
            crate-type = ["dylib", "rlib"]
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build -v").run();
    assert!(p.root().join("target/debug/libfoo.rlib").is_file());
    let dylib_name = format!("{}foo{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
    assert!(p.root().join("target/debug").join(dylib_name).is_file());
}

#[cargo_test]
fn dashes_to_underscores() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo-bar", "0.0.1"))
        .file("src/lib.rs", "")
        .file("src/main.rs", "extern crate foo_bar; fn main() {}")
        .build();

    p.cargo("build -v").run();
    assert!(p.bin("foo-bar").is_file());
}

#[cargo_test]
fn dashes_in_crate_name_bad() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            name = "foo-bar"
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/main.rs", "extern crate foo_bar; fn main() {}")
        .build();

    p.cargo("build -v")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  library target names cannot contain hyphens: foo-bar
",
        )
        .run();
}

#[cargo_test]
fn rustc_env_var() {
    let p = project().file("src/lib.rs", "").build();

    p.cargo("build -v")
        .env("RUSTC", "rustc-that-does-not-exist")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] could not execute process `rustc-that-does-not-exist -vV` ([..])

Caused by:
[..]
",
        )
        .run();
    assert!(!p.bin("a").is_file());
}

#[cargo_test]
fn filtering() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", "fn main() {}")
        .file("src/bin/b.rs", "fn main() {}")
        .file("examples/a.rs", "fn main() {}")
        .file("examples/b.rs", "fn main() {}")
        .build();

    p.cargo("build --lib").run();
    assert!(!p.bin("a").is_file());

    p.cargo("build --bin=a --example=a").run();
    assert!(p.bin("a").is_file());
    assert!(!p.bin("b").is_file());
    assert!(p.bin("examples/a").is_file());
    assert!(!p.bin("examples/b").is_file());
}

#[cargo_test]
fn filtering_implicit_bins() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", "fn main() {}")
        .file("src/bin/b.rs", "fn main() {}")
        .file("examples/a.rs", "fn main() {}")
        .file("examples/b.rs", "fn main() {}")
        .build();

    p.cargo("build --bins").run();
    assert!(p.bin("a").is_file());
    assert!(p.bin("b").is_file());
    assert!(!p.bin("examples/a").is_file());
    assert!(!p.bin("examples/b").is_file());
}

#[cargo_test]
fn filtering_implicit_examples() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", "fn main() {}")
        .file("src/bin/b.rs", "fn main() {}")
        .file("examples/a.rs", "fn main() {}")
        .file("examples/b.rs", "fn main() {}")
        .build();

    p.cargo("build --examples").run();
    assert!(!p.bin("a").is_file());
    assert!(!p.bin("b").is_file());
    assert!(p.bin("examples/a").is_file());
    assert!(p.bin("examples/b").is_file());
}

#[cargo_test]
fn ignore_dotfile() {
    let p = project()
        .file("src/bin/.a.rs", "")
        .file("src/bin/a.rs", "fn main() {}")
        .build();

    p.cargo("build").run();
}

#[cargo_test]
fn ignore_dotdirs() {
    let p = project()
        .file("src/bin/a.rs", "fn main() {}")
        .file(".git/Cargo.toml", "")
        .file(".pc/dummy-fix.patch/Cargo.toml", "")
        .build();

    p.cargo("build").run();
}

#[cargo_test]
fn dotdir_root() {
    let p = ProjectBuilder::new(root().join(".foo"))
        .file("src/bin/a.rs", "fn main() {}")
        .build();
    p.cargo("build").run();
}

#[cargo_test]
fn custom_target_dir_env() {
    let p = project().file("src/main.rs", "fn main() {}").build();

    let exe_name = format!("foo{}", env::consts::EXE_SUFFIX);

    p.cargo("build").env("CARGO_TARGET_DIR", "foo/target").run();
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(!p.root().join("target/debug").join(&exe_name).is_file());

    p.cargo("build").run();
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("target/debug").join(&exe_name).is_file());

    p.cargo("build")
        .env("CARGO_BUILD_TARGET_DIR", "foo2/target")
        .run();
    assert!(p.root().join("foo2/target/debug").join(&exe_name).is_file());

    p.change_file(
        ".cargo/config",
        r#"
            [build]
            target-dir = "foo/target"
        "#,
    );
    p.cargo("build").env("CARGO_TARGET_DIR", "bar/target").run();
    assert!(p.root().join("bar/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("target/debug").join(&exe_name).is_file());
}

#[cargo_test]
fn custom_target_dir_line_parameter() {
    let p = project().file("src/main.rs", "fn main() {}").build();

    let exe_name = format!("foo{}", env::consts::EXE_SUFFIX);

    p.cargo("build --target-dir foo/target").run();
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(!p.root().join("target/debug").join(&exe_name).is_file());

    p.cargo("build").run();
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("target/debug").join(&exe_name).is_file());

    p.change_file(
        ".cargo/config",
        r#"
            [build]
            target-dir = "foo/target"
        "#,
    );
    p.cargo("build --target-dir bar/target").run();
    assert!(p.root().join("bar/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("target/debug").join(&exe_name).is_file());

    p.cargo("build --target-dir foobar/target")
        .env("CARGO_TARGET_DIR", "bar/target")
        .run();
    assert!(p
        .root()
        .join("foobar/target/debug")
        .join(&exe_name)
        .is_file());
    assert!(p.root().join("bar/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("foo/target/debug").join(&exe_name).is_file());
    assert!(p.root().join("target/debug").join(&exe_name).is_file());
}

#[cargo_test]
fn build_multiple_packages() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.d1]
                path = "d1"
            [dependencies.d2]
                path = "d2"

            [[bin]]
                name = "foo"
        "#,
        )
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .file("d1/Cargo.toml", &basic_bin_manifest("d1"))
        .file("d1/src/lib.rs", "")
        .file("d1/src/main.rs", "fn main() { println!(\"d1\"); }")
        .file(
            "d2/Cargo.toml",
            r#"
            [package]
            name = "d2"
            version = "0.0.1"
            authors = []

            [[bin]]
                name = "d2"
                doctest = false
        "#,
        )
        .file("d2/src/main.rs", "fn main() { println!(\"d2\"); }")
        .build();

    p.cargo("build -p d1 -p d2 -p foo").run();

    assert!(p.bin("foo").is_file());
    p.process(&p.bin("foo")).with_stdout("i am foo\n").run();

    let d1_path = &p
        .build_dir()
        .join("debug")
        .join(format!("d1{}", env::consts::EXE_SUFFIX));
    let d2_path = &p
        .build_dir()
        .join("debug")
        .join(format!("d2{}", env::consts::EXE_SUFFIX));

    assert!(d1_path.is_file());
    p.process(d1_path).with_stdout("d1").run();

    assert!(d2_path.is_file());
    p.process(d2_path).with_stdout("d2").run();
}

#[cargo_test]
fn invalid_spec() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.d1]
                path = "d1"

            [[bin]]
                name = "foo"
        "#,
        )
        .file("src/bin/foo.rs", &main_file(r#""i am foo""#, &[]))
        .file("d1/Cargo.toml", &basic_bin_manifest("d1"))
        .file("d1/src/lib.rs", "")
        .file("d1/src/main.rs", "fn main() { println!(\"d1\"); }")
        .build();

    p.cargo("build -p notAValidDep")
        .with_status(101)
        .with_stderr("[ERROR] package ID specification `notAValidDep` matched no packages")
        .run();

    p.cargo("build -p d1 -p notAValidDep")
        .with_status(101)
        .with_stderr("[ERROR] package ID specification `notAValidDep` matched no packages")
        .run();
}

#[cargo_test]
fn manifest_with_bom_is_ok() {
    let p = project()
        .file(
            "Cargo.toml",
            "\u{FEFF}
            [package]
            name = \"foo\"
            version = \"0.0.1\"
            authors = []
        ",
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("build -v").run();
}

#[cargo_test]
fn panic_abort_compiles_with_panic_abort() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [profile.dev]
            panic = 'abort'
        "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("build -v")
        .with_stderr_contains("[..] -C panic=abort [..]")
        .run();
}

#[cargo_test]
fn compiler_json_error_format() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file(
            "build.rs",
            "fn main() { println!(\"cargo:rustc-cfg=xyz\") }",
        )
        .file("src/main.rs", "fn main() { let unused = 92; }")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file("bar/src/lib.rs", r#"fn dead() {}"#)
        .build();

    let output = |fresh| {
        r#"
    {
        "reason":"compiler-artifact",
        "package_id":"foo 0.5.0 ([..])",
        "target":{
            "kind":["custom-build"],
            "crate_types":["bin"],
            "doctest": false,
            "edition": "2015",
            "name":"build-script-build",
            "src_path":"[..]build.rs",
            "test": false
        },
        "profile": {
            "debug_assertions": true,
            "debuginfo": 2,
            "opt_level": "0",
            "overflow_checks": true,
            "test": false
        },
        "executable": null,
        "features": [],
        "filenames": "{...}",
        "fresh": $FRESH
    }

    {
        "reason":"compiler-message",
        "package_id":"bar 0.5.0 ([..])",
        "target":{
            "kind":["lib"],
            "crate_types":["lib"],
            "doctest": true,
            "edition": "2015",
            "name":"bar",
            "src_path":"[..]lib.rs",
            "test": true
        },
        "message":"{...}"
    }

    {
        "reason":"compiler-artifact",
        "profile": {
            "debug_assertions": true,
            "debuginfo": 2,
            "opt_level": "0",
            "overflow_checks": true,
            "test": false
        },
        "executable": null,
        "features": [],
        "package_id":"bar 0.5.0 ([..])",
        "target":{
            "kind":["lib"],
            "crate_types":["lib"],
            "doctest": true,
            "edition": "2015",
            "name":"bar",
            "src_path":"[..]lib.rs",
            "test": true
        },
        "filenames":[
            "[..].rlib",
            "[..].rmeta"
        ],
        "fresh": $FRESH
    }

    {
        "reason":"build-script-executed",
        "package_id":"foo 0.5.0 ([..])",
        "linked_libs":[],
        "linked_paths":[],
        "env":[],
        "cfgs":["xyz"],
        "out_dir": "[..]target/debug/build/foo-[..]/out"
    }

    {
        "reason":"compiler-message",
        "package_id":"foo 0.5.0 ([..])",
        "target":{
            "kind":["bin"],
            "crate_types":["bin"],
            "doctest": false,
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]main.rs",
            "test": true
        },
        "message":"{...}"
    }

    {
        "reason":"compiler-artifact",
        "package_id":"foo 0.5.0 ([..])",
        "target":{
            "kind":["bin"],
            "crate_types":["bin"],
            "doctest": false,
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]main.rs",
            "test": true
        },
        "profile": {
            "debug_assertions": true,
            "debuginfo": 2,
            "opt_level": "0",
            "overflow_checks": true,
            "test": false
        },
        "executable": "[..]/foo/target/debug/foo[EXE]",
        "features": [],
        "filenames": "{...}",
        "fresh": $FRESH
    }

    {"reason": "build-finished", "success": true}
"#
        .replace("$FRESH", fresh)
    };

    // Use `jobs=1` to ensure that the order of messages is consistent.
    p.cargo("build -v --message-format=json --jobs=1")
        .with_json_contains_unordered(&output("false"))
        .run();

    // With fresh build, we should repeat the artifacts,
    // and replay the cached compiler warnings.
    p.cargo("build -v --message-format=json --jobs=1")
        .with_json_contains_unordered(&output("true"))
        .run();
}

#[cargo_test]
fn wrong_message_format_option() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build --message-format XML")
        .with_status(101)
        .with_stderr_contains(
            "\
error: invalid message format specifier: `xml`
",
        )
        .run();
}

#[cargo_test]
fn message_format_json_forward_stderr() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", "fn main() { let unused = 0; }")
        .build();

    p.cargo("rustc --release --bin foo --message-format JSON")
        .with_json_contains_unordered(
            r#"
    {
        "reason":"compiler-message",
        "package_id":"foo 0.5.0 ([..])",
        "target":{
            "kind":["bin"],
            "crate_types":["bin"],
            "doctest": false,
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]",
            "test": true
        },
        "message":"{...}"
    }

    {
        "reason":"compiler-artifact",
        "package_id":"foo 0.5.0 ([..])",
        "target":{
            "kind":["bin"],
            "crate_types":["bin"],
            "doctest": false,
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]",
            "test": true
        },
        "profile":{
            "debug_assertions":false,
            "debuginfo":null,
            "opt_level":"3",
            "overflow_checks": false,
            "test":false
        },
        "executable": "{...}",
        "features":[],
        "filenames": "{...}",
        "fresh": false
    }

    {"reason": "build-finished", "success": true}
"#,
        )
        .run();
}

#[cargo_test]
fn no_warn_about_package_metadata() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [package.metadata]
            foo = "bar"
            a = true
            b = 3

            [package.metadata.another]
            bar = 3
        "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("build")
        .with_stderr(
            "[..] foo v0.0.1 ([..])\n\
             [FINISHED] dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn no_warn_about_workspace_metadata() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["foo"]

            [workspace.metadata]
            something = "something_else"
            x = 1
            y = 2

            [workspace.metadata.another]
            bar = 12
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            "#,
        )
        .file("foo/src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "[..] foo v0.0.1 ([..])\n\
             [FINISHED] dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn cargo_build_empty_target() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build --target")
        .arg("")
        .with_status(101)
        .with_stderr_contains("[..] target was empty")
        .run();
}

#[cargo_test]
fn build_all_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = { path = "bar" }

            [workspace]
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("build --workspace")
        .with_stderr(
            "[..] Compiling bar v0.1.0 ([..])\n\
             [..] Compiling foo v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn build_all_exclude() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"

            [workspace]
            members = ["bar", "baz"]
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("build --workspace --exclude baz")
        .with_stderr_contains("[..]Compiling foo v0.1.0 [..]")
        .with_stderr_contains("[..]Compiling bar v0.1.0 [..]")
        .with_stderr_does_not_contain("[..]Compiling baz v0.1.0 [..]")
        .run();
}

#[cargo_test]
fn build_all_workspace_implicit_examples() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            bar = { path = "bar" }

            [workspace]
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", "fn main() {}")
        .file("src/bin/b.rs", "fn main() {}")
        .file("examples/c.rs", "fn main() {}")
        .file("examples/d.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .file("bar/src/bin/e.rs", "fn main() {}")
        .file("bar/src/bin/f.rs", "fn main() {}")
        .file("bar/examples/g.rs", "fn main() {}")
        .file("bar/examples/h.rs", "fn main() {}")
        .build();

    p.cargo("build --workspace --examples")
        .with_stderr(
            "[..] Compiling bar v0.1.0 ([..])\n\
             [..] Compiling foo v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
    assert!(!p.bin("a").is_file());
    assert!(!p.bin("b").is_file());
    assert!(p.bin("examples/c").is_file());
    assert!(p.bin("examples/d").is_file());
    assert!(!p.bin("e").is_file());
    assert!(!p.bin("f").is_file());
    assert!(p.bin("examples/g").is_file());
    assert!(p.bin("examples/h").is_file());
}

#[cargo_test]
fn build_all_virtual_manifest() {
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

    // The order in which bar and baz are built is not guaranteed
    p.cargo("build --workspace")
        .with_stderr_contains("[..] Compiling baz v0.1.0 ([..])")
        .with_stderr_contains("[..] Compiling bar v0.1.0 ([..])")
        .with_stderr(
            "[..] Compiling [..] v0.1.0 ([..])\n\
             [..] Compiling [..] v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn build_virtual_manifest_all_implied() {
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

    // The order in which `bar` and `baz` are built is not guaranteed.
    p.cargo("build")
        .with_stderr_contains("[..] Compiling baz v0.1.0 ([..])")
        .with_stderr_contains("[..] Compiling bar v0.1.0 ([..])")
        .with_stderr(
            "[..] Compiling [..] v0.1.0 ([..])\n\
             [..] Compiling [..] v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn build_virtual_manifest_one_project() {
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

    p.cargo("build -p bar")
        .with_stderr_does_not_contain("[..]baz[..]")
        .with_stderr_contains("[..] Compiling bar v0.1.0 ([..])")
        .with_stderr(
            "[..] Compiling [..] v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn build_all_virtual_manifest_implicit_examples() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar", "baz"]
        "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .file("bar/src/bin/a.rs", "fn main() {}")
        .file("bar/src/bin/b.rs", "fn main() {}")
        .file("bar/examples/c.rs", "fn main() {}")
        .file("bar/examples/d.rs", "fn main() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "")
        .file("baz/src/bin/e.rs", "fn main() {}")
        .file("baz/src/bin/f.rs", "fn main() {}")
        .file("baz/examples/g.rs", "fn main() {}")
        .file("baz/examples/h.rs", "fn main() {}")
        .build();

    // The order in which bar and baz are built is not guaranteed
    p.cargo("build --workspace --examples")
        .with_stderr_contains("[..] Compiling baz v0.1.0 ([..])")
        .with_stderr_contains("[..] Compiling bar v0.1.0 ([..])")
        .with_stderr(
            "[..] Compiling [..] v0.1.0 ([..])\n\
             [..] Compiling [..] v0.1.0 ([..])\n\
             [..] Finished dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
    assert!(!p.bin("a").is_file());
    assert!(!p.bin("b").is_file());
    assert!(p.bin("examples/c").is_file());
    assert!(p.bin("examples/d").is_file());
    assert!(!p.bin("e").is_file());
    assert!(!p.bin("f").is_file());
    assert!(p.bin("examples/g").is_file());
    assert!(p.bin("examples/h").is_file());
}

#[cargo_test]
fn build_all_member_dependency_same_name() {
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
            [project]
            name = "a"
            version = "0.1.0"

            [dependencies]
            a = "0.1.0"
        "#,
        )
        .file("a/src/lib.rs", "pub fn a() {}")
        .build();

    Package::new("a", "0.1.0").publish();

    p.cargo("build --workspace")
        .with_stderr(
            "[UPDATING] `[..]` index\n\
             [DOWNLOADING] crates ...\n\
             [DOWNLOADED] a v0.1.0 ([..])\n\
             [COMPILING] a v0.1.0\n\
             [COMPILING] a v0.1.0 ([..])\n\
             [FINISHED] dev [unoptimized + debuginfo] target(s) in [..]\n",
        )
        .run();
}

#[cargo_test]
fn run_proper_binary() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"
            [[bin]]
            name = "main"
            [[bin]]
            name = "other"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "src/bin/main.rs",
            r#"fn main() { panic!("This should never be run."); }"#,
        )
        .file("src/bin/other.rs", "fn main() {}")
        .build();

    p.cargo("run --bin other").run();
}

#[cargo_test]
fn run_proper_binary_main_rs() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/lib.rs", "")
        .file("src/bin/main.rs", "fn main() {}")
        .build();

    p.cargo("run --bin foo").run();
}

#[cargo_test]
fn run_proper_alias_binary_from_src() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"
            [[bin]]
            name = "foo"
            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/foo.rs", r#"fn main() { println!("foo"); }"#)
        .file("src/bar.rs", r#"fn main() { println!("bar"); }"#)
        .build();

    p.cargo("build --workspace").run();
    p.process(&p.bin("foo")).with_stdout("foo\n").run();
    p.process(&p.bin("bar")).with_stdout("bar\n").run();
}

#[cargo_test]
fn run_proper_alias_binary_main_rs() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"
            [[bin]]
            name = "foo"
            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/main.rs", r#"fn main() { println!("main"); }"#)
        .build();

    p.cargo("build --workspace").run();
    p.process(&p.bin("foo")).with_stdout("main\n").run();
    p.process(&p.bin("bar")).with_stdout("main\n").run();
}

#[cargo_test]
fn run_proper_binary_main_rs_as_foo() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/foo.rs",
            r#" fn main() { panic!("This should never be run."); }"#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("run --bin foo").run();
}

#[cargo_test]
// NOTE: we don't have `/usr/bin/env` on Windows.
#[cfg(not(windows))]
fn rustc_wrapper() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v")
        .env("RUSTC_WRAPPER", "/usr/bin/env")
        .with_stderr_contains("[RUNNING] `/usr/bin/env rustc --crate-name foo [..]")
        .run();
}

#[cargo_test]
#[cfg(not(windows))]
fn rustc_wrapper_relative() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v")
        .env("RUSTC_WRAPPER", "./sccache")
        .with_status(101)
        .with_stderr_contains("[..]/foo/./sccache rustc[..]")
        .run();
}

#[cargo_test]
#[cfg(not(windows))]
fn rustc_wrapper_from_path() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v")
        .env("RUSTC_WRAPPER", "wannabe_sccache")
        .with_status(101)
        .with_stderr_contains("[..]`wannabe_sccache rustc [..]")
        .run();
}

#[cargo_test]
// NOTE: we don't have `/usr/bin/env` on Windows.
#[cfg(not(windows))]
fn rustc_workspace_wrapper() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v -Zunstable-options")
        .env("RUSTC_WORKSPACE_WRAPPER", "/usr/bin/env")
        .masquerade_as_nightly_cargo()
        .with_stderr_contains("[RUNNING] `/usr/bin/env rustc --crate-name foo [..]")
        .run();
}

#[cargo_test]
#[cfg(not(windows))]
fn rustc_workspace_wrapper_relative() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v -Zunstable-options")
        .env("RUSTC_WORKSPACE_WRAPPER", "./sccache")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        .with_stderr_contains("[..]/foo/./sccache rustc[..]")
        .run();
}

#[cargo_test]
#[cfg(not(windows))]
fn rustc_workspace_wrapper_from_path() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v -Zunstable-options")
        .env("RUSTC_WORKSPACE_WRAPPER", "wannabe_sccache")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        .with_stderr_contains("[..]`wannabe_sccache rustc [..]")
        .run();
}

#[cargo_test]
fn cdylib_not_lifted() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            authors = []
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build").run();

    let files = if cfg!(windows) {
        if cfg!(target_env = "msvc") {
            vec!["foo.dll.lib", "foo.dll.exp", "foo.dll"]
        } else {
            vec!["libfoo.dll.a", "foo.dll"]
        }
    } else if cfg!(target_os = "macos") {
        vec!["libfoo.dylib"]
    } else {
        vec!["libfoo.so"]
    };

    for file in files {
        println!("checking: {}", file);
        assert!(p.root().join("target/debug/deps").join(&file).is_file());
    }
}

#[cargo_test]
fn cdylib_final_outputs() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo-bar"
            authors = []
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build").run();

    let files = if cfg!(windows) {
        if cfg!(target_env = "msvc") {
            vec!["foo_bar.dll.lib", "foo_bar.dll"]
        } else {
            vec!["foo_bar.dll", "libfoo_bar.dll.a"]
        }
    } else if cfg!(target_os = "macos") {
        vec!["libfoo_bar.dylib"]
    } else {
        vec!["libfoo_bar.so"]
    };

    for file in files {
        println!("checking: {}", file);
        assert!(p.root().join("target/debug").join(&file).is_file());
    }
}

#[cargo_test]
fn deterministic_cfg_flags() {
    // This bug is non-deterministic.

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            build = "build.rs"

            [features]
            default = ["f_a", "f_b", "f_c", "f_d"]
            f_a = []
            f_b = []
            f_c = []
            f_d = []
        "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:rustc-cfg=cfg_a");
                    println!("cargo:rustc-cfg=cfg_b");
                    println!("cargo:rustc-cfg=cfg_c");
                    println!("cargo:rustc-cfg=cfg_d");
                    println!("cargo:rustc-cfg=cfg_e");
                }
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.1.0 [..]
[RUNNING] [..]
[RUNNING] [..]
[RUNNING] `rustc --crate-name foo [..] \
--cfg[..]default[..]--cfg[..]f_a[..]--cfg[..]f_b[..]\
--cfg[..]f_c[..]--cfg[..]f_d[..] \
--cfg cfg_a --cfg cfg_b --cfg cfg_c --cfg cfg_d --cfg cfg_e`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

#[cargo_test]
fn explicit_bins_without_paths() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []

            [[bin]]
            name = "foo"

            [[bin]]
            name = "bar"
        "#,
        )
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .file("src/bin/bar.rs", "fn main() {}")
        .build();

    p.cargo("build").run();
}

#[cargo_test]
fn no_bin_in_src_with_lib() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/lib.rs", "")
        .file("src/foo.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  can't find `foo` bin, specify bin.path",
        )
        .run();
}

#[cargo_test]
fn inferred_bins() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/bin/bar.rs", "fn main() {}")
        .file("src/bin/baz/main.rs", "fn main() {}")
        .build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());
    assert!(p.bin("bar").is_file());
    assert!(p.bin("baz").is_file());
}

#[cargo_test]
fn inferred_bins_duplicate_name() {
    // this should fail, because we have two binaries with the same name
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/bin/bar.rs", "fn main() {}")
        .file("src/bin/bar/main.rs", "fn main() {}")
        .build();

    p.cargo("build").with_status(101).with_stderr_contains(
            "[..]found duplicate binary name bar, but all binary targets must have a unique name[..]",
        )
        .run();
}

#[cargo_test]
fn inferred_bin_path() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
        [package]
        name = "foo"
        version = "0.1.0"
        authors = []

        [[bin]]
        name = "bar"
        # Note, no `path` key!
        "#,
        )
        .file("src/bin/bar/main.rs", "fn main() {}")
        .build();

    p.cargo("build").run();
    assert!(p.bin("bar").is_file());
}

#[cargo_test]
fn inferred_examples() {
    let p = project()
        .file("src/lib.rs", "fn main() {}")
        .file("examples/bar.rs", "fn main() {}")
        .file("examples/baz/main.rs", "fn main() {}")
        .build();

    p.cargo("build --examples").run();
    assert!(p.bin("examples/bar").is_file());
    assert!(p.bin("examples/baz").is_file());
}

#[cargo_test]
fn inferred_tests() {
    let p = project()
        .file("src/lib.rs", "fn main() {}")
        .file("tests/bar.rs", "fn main() {}")
        .file("tests/baz/main.rs", "fn main() {}")
        .build();

    p.cargo("test --test=bar --test=baz").run();
}

#[cargo_test]
fn inferred_benchmarks() {
    let p = project()
        .file("src/lib.rs", "fn main() {}")
        .file("benches/bar.rs", "fn main() {}")
        .file("benches/baz/main.rs", "fn main() {}")
        .build();

    p.cargo("bench --bench=bar --bench=baz").run();
}

#[cargo_test]
fn target_edition() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [lib]
                edition = "2018"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build -v")
        // Passes on nightly, fails on stable, since `--edition` is nightly-only.
        .without_status()
        .with_stderr_contains(
            "\
[COMPILING] foo v0.0.1 ([..])
[RUNNING] `rustc [..]--edition=2018 [..]
",
        )
        .run();
}

#[cargo_test]
fn target_edition_override() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
                edition = "2018"

                [lib]
                edition = "2015"
            "#,
        )
        .file(
            "src/lib.rs",
            "
                pub fn async() {}
                pub fn try() {}
                pub fn await() {}
            ",
        )
        .build();

    p.cargo("build -v").run();
}

#[cargo_test]
fn same_metadata_different_directory() {
    // A top-level crate built in two different workspaces should have the
    // same metadata hash.
    let p = project()
        .at("foo1")
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();
    let output = t!(String::from_utf8(
        t!(p.cargo("build -v").exec_with_output()).stderr,
    ));
    let metadata = output
        .split_whitespace()
        .find(|arg| arg.starts_with("metadata="))
        .unwrap();

    let p = project()
        .at("foo2")
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    p.cargo("build -v")
        .with_stderr_contains(format!("[..]{}[..]", metadata))
        .run();
}

#[cargo_test]
fn building_a_dependent_crate_witout_bin_should_fail() {
    Package::new("testless", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "testless"
            version = "0.1.0"

            [[bin]]
            name = "a_bin"
        "#,
        )
        .file("src/lib.rs", "")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            testless = "0.1.0"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains("[..]can't find `a_bin` bin, specify bin.path")
        .run();
}

#[cargo_test]
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn uplift_dsym_of_bin_on_mac() {
    let p = project()
        .file("src/main.rs", "fn main() { panic!(); }")
        .file("src/bin/b.rs", "fn main() { panic!(); }")
        .file("examples/c.rs", "fn main() { panic!(); }")
        .file("tests/d.rs", "fn main() { panic!(); }")
        .build();

    p.cargo("build --bins --examples --tests").run();
    assert!(p.target_debug_dir().join("foo.dSYM").is_dir());
    assert!(p.target_debug_dir().join("b.dSYM").is_dir());
    assert!(p.target_debug_dir().join("b.dSYM").is_symlink());
    assert!(p.target_debug_dir().join("examples/c.dSYM").is_dir());
    assert!(!p.target_debug_dir().join("c.dSYM").exists());
    assert!(!p.target_debug_dir().join("d.dSYM").exists());
}

#[cargo_test]
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn uplift_dsym_of_bin_on_mac_when_broken_link_exists() {
    let p = project()
        .file("src/main.rs", "fn main() { panic!(); }")
        .build();
    let dsym = p.target_debug_dir().join("foo.dSYM");

    p.cargo("build").run();
    assert!(dsym.is_dir());

    // Simulate the situation where the underlying dSYM bundle goes missing
    // but the uplifted symlink to it remains. This would previously cause
    // builds to permanently fail until the bad symlink was manually removed.
    dsym.rm_rf();
    p.symlink(
        p.target_debug_dir()
            .join("deps")
            .join("foo-baaaaaadbaaaaaad.dSYM"),
        &dsym,
    );
    assert!(dsym.is_symlink());
    assert!(!dsym.exists());

    p.cargo("build").run();
    assert!(dsym.is_dir());
}

#[cargo_test]
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn uplift_pdb_of_bin_on_windows() {
    let p = project()
        .file("src/main.rs", "fn main() { panic!(); }")
        .file("src/bin/b.rs", "fn main() { panic!(); }")
        .file("src/bin/foo-bar.rs", "fn main() { panic!(); }")
        .file("examples/c.rs", "fn main() { panic!(); }")
        .file("tests/d.rs", "fn main() { panic!(); }")
        .build();

    p.cargo("build --bins --examples --tests").run();
    assert!(p.target_debug_dir().join("foo.pdb").is_file());
    assert!(p.target_debug_dir().join("b.pdb").is_file());
    assert!(p.target_debug_dir().join("examples/c.pdb").exists());
    assert!(p.target_debug_dir().join("foo-bar.exe").is_file());
    assert!(p.target_debug_dir().join("foo_bar.pdb").is_file());
    assert!(!p.target_debug_dir().join("c.pdb").exists());
    assert!(!p.target_debug_dir().join("d.pdb").exists());
}

// Ensure that `cargo build` chooses the correct profile for building
// targets based on filters (assuming `--profile` is not specified).
#[cargo_test]
fn build_filter_infer_profile() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .file("tests/t1.rs", "")
        .file("benches/b1.rs", "")
        .file("examples/ex1.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
             --emit=[..]link[..]",
        )
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]",
        )
        .run();

    p.root().join("target").rm_rf();
    p.cargo("build -v --test=t1")
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
             --emit=[..]link[..]-C debuginfo=2 [..]",
        )
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name t1 tests/t1.rs [..]--emit=[..]link[..]\
             -C debuginfo=2 [..]",
        )
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]-C debuginfo=2 [..]",
        )
        .run();

    p.root().join("target").rm_rf();
    // Bench uses test profile without `--release`.
    p.cargo("build -v --bench=b1")
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
             --emit=[..]link[..]-C debuginfo=2 [..]",
        )
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name b1 benches/b1.rs [..]--emit=[..]link[..]\
             -C debuginfo=2 [..]",
        )
        .with_stderr_does_not_contain("opt-level")
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]-C debuginfo=2 [..]",
        )
        .run();
}

#[cargo_test]
fn targets_selected_default() {
    let p = project().file("src/main.rs", "fn main() {}").build();
    p.cargo("build -v")
        // Binaries.
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]",
        )
        // Benchmarks.
        .with_stderr_does_not_contain(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--emit=[..]link \
             -C opt-level=3 --test [..]",
        )
        // Unit tests.
        .with_stderr_does_not_contain(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--emit=[..]link[..]\
             -C debuginfo=2 --test [..]",
        )
        .run();
}

#[cargo_test]
fn targets_selected_all() {
    let p = project().file("src/main.rs", "fn main() {}").build();
    p.cargo("build -v --all-targets")
        // Binaries.
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]",
        )
        // Unit tests.
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--emit=[..]link[..]\
             -C debuginfo=2 --test [..]",
        )
        .run();
}

#[cargo_test]
fn all_targets_no_lib() {
    let p = project().file("src/main.rs", "fn main() {}").build();
    p.cargo("build -v --all-targets")
        // Binaries.
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--crate-type bin \
             --emit=[..]link[..]",
        )
        // Unit tests.
        .with_stderr_contains(
            "[RUNNING] `rustc --crate-name foo src/main.rs [..]--emit=[..]link[..]\
             -C debuginfo=2 --test [..]",
        )
        .run();
}

#[cargo_test]
fn no_linkable_target() {
    // Issue 3169: this is currently not an error as per discussion in PR #4797.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []
            [dependencies]
            the_lib = { path = "the_lib" }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "the_lib/Cargo.toml",
            r#"
            [package]
            name = "the_lib"
            version = "0.1.0"
            [lib]
            name = "the_lib"
            crate-type = ["staticlib"]
        "#,
        )
        .file("the_lib/src/lib.rs", "pub fn foo() {}")
        .build();
    p.cargo("build")
        .with_stderr_contains(
            "[WARNING] The package `the_lib` provides no linkable [..] \
             while compiling `foo`. [..] in `the_lib`'s Cargo.toml. [..]",
        )
        .run();
}

#[cargo_test]
fn avoid_dev_deps() {
    Package::new("foo", "1.0.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.1.0"
            authors = []

            [dev-dependencies]
            baz = "1.0.0"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[UPDATING] [..]
[ERROR] no matching package named `baz` found
location searched: registry `https://github.com/rust-lang/crates.io-index`
required by package `bar v0.1.0 ([..]/foo)`
",
        )
        .run();
    p.cargo("build -Zavoid-dev-deps")
        .masquerade_as_nightly_cargo()
        .run();
}

#[cargo_test]
fn default_cargo_config_jobs() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            r#"
            [build]
            jobs = 1
        "#,
        )
        .build();
    p.cargo("build -v").run();
}

#[cargo_test]
fn good_cargo_config_jobs() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            r#"
            [build]
            jobs = 4
        "#,
        )
        .build();
    p.cargo("build -v").run();
}

#[cargo_test]
fn invalid_jobs() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    p.cargo("build --jobs -1")
        .with_status(1)
        .with_stderr_contains(
            "error: Found argument '-1' which wasn't expected, or isn't valid in this context",
        )
        .run();

    p.cargo("build --jobs over9000")
        .with_status(1)
        .with_stderr("error: Invalid value: could not parse `over9000` as a number")
        .run();
}

#[cargo_test]
fn target_filters_workspace() {
    let ws = project()
        .at("ws")
        .file(
            "Cargo.toml",
            r#"
        [workspace]
        members = ["a", "b"]
        "#,
        )
        .file("a/Cargo.toml", &basic_lib_manifest("a"))
        .file("a/src/lib.rs", "")
        .file("a/examples/ex1.rs", "fn main() {}")
        .file("b/Cargo.toml", &basic_bin_manifest("b"))
        .file("b/src/lib.rs", "")
        .file("b/src/main.rs", "fn main() {}")
        .build();

    ws.cargo("build -v --example ex")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] no example target named `ex`

<tab>Did you mean `ex1`?",
        )
        .run();

    ws.cargo("build -v --lib")
        .with_stderr_contains("[RUNNING] `rustc [..]a/src/lib.rs[..]")
        .with_stderr_contains("[RUNNING] `rustc [..]b/src/lib.rs[..]")
        .run();

    ws.cargo("build -v --example ex1")
        .with_stderr_contains("[RUNNING] `rustc [..]a/examples/ex1.rs[..]")
        .run();
}

#[cargo_test]
fn target_filters_workspace_not_found() {
    let ws = project()
        .at("ws")
        .file(
            "Cargo.toml",
            r#"
        [workspace]
        members = ["a", "b"]
        "#,
        )
        .file("a/Cargo.toml", &basic_bin_manifest("a"))
        .file("a/src/main.rs", "fn main() {}")
        .file("b/Cargo.toml", &basic_bin_manifest("b"))
        .file("b/src/main.rs", "fn main() {}")
        .build();

    ws.cargo("build -v --lib")
        .with_status(101)
        .with_stderr("[ERROR] no library targets found in packages: a, b")
        .run();
}

#[cfg(unix)]
#[cargo_test]
fn signal_display() {
    // Cause the compiler to crash with a signal.
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            [dependencies]
            pm = { path = "pm" }
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #[macro_use]
            extern crate pm;

            #[derive(Foo)]
            pub struct S;
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
        "#,
        )
        .file(
            "pm/src/lib.rs",
            r#"
            extern crate proc_macro;
            use proc_macro::TokenStream;

            #[proc_macro_derive(Foo)]
            pub fn derive(_input: TokenStream) -> TokenStream {
                std::process::abort()
            }
        "#,
        )
        .build();

    foo.cargo("build")
        .with_stderr(
            "\
[COMPILING] pm [..]
[COMPILING] foo [..]
[ERROR] could not compile `foo`.

Caused by:
  process didn't exit successfully: `rustc [..]` (signal: 6, SIGABRT: process abort signal)
",
        )
        .with_status(101)
        .run();
}

#[cargo_test]
fn tricky_pipelining() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                [dependencies]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file("bar/Cargo.toml", &basic_lib_manifest("bar"))
        .file("bar/src/lib.rs", "")
        .build();

    foo.cargo("build -p bar")
        .env("CARGO_BUILD_PIPELINING", "true")
        .run();
    foo.cargo("build -p foo")
        .env("CARGO_BUILD_PIPELINING", "true")
        .run();
}

#[cargo_test]
fn pipelining_works() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                [dependencies]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file("bar/Cargo.toml", &basic_lib_manifest("bar"))
        .file("bar/src/lib.rs", "")
        .build();

    foo.cargo("build")
        .env("CARGO_BUILD_PIPELINING", "true")
        .with_stdout("")
        .with_stderr(
            "\
[COMPILING] [..]
[COMPILING] [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn pipelining_big_graph() {
    // Create a crate graph of the form {a,b}{0..29}, where {a,b}(n) depend on {a,b}(n+1)
    // Then have `foo`, a binary crate, depend on the whole thing.
    let mut project = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                [dependencies]
                a1 = { path = "a1" }
                b1 = { path = "b1" }
            "#,
        )
        .file("src/main.rs", "fn main(){}");

    for n in 0..30 {
        for x in &["a", "b"] {
            project = project
                .file(
                    &format!("{x}{n}/Cargo.toml", x = x, n = n),
                    &format!(
                        r#"
                            [package]
                            name = "{x}{n}"
                            version = "0.1.0"
                            [dependencies]
                            a{np1} = {{ path = "../a{np1}" }}
                            b{np1} = {{ path = "../b{np1}" }}
                        "#,
                        x = x,
                        n = n,
                        np1 = n + 1
                    ),
                )
                .file(&format!("{x}{n}/src/lib.rs", x = x, n = n), "");
        }
    }

    let foo = project
        .file("a30/Cargo.toml", &basic_lib_manifest("a30"))
        .file(
            "a30/src/lib.rs",
            r#"compile_error!("don't actually build me");"#,
        )
        .file("b30/Cargo.toml", &basic_lib_manifest("b30"))
        .file("b30/src/lib.rs", "")
        .build();
    foo.cargo("build -p foo")
        .env("CARGO_BUILD_PIPELINING", "true")
        .with_status(101)
        .with_stderr_contains("[ERROR] could not compile `a30`[..]")
        .run();
}

#[cargo_test]
fn forward_rustc_output() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = '2018'
                [dependencies]
                bar = { path = "bar" }
            "#,
        )
        .file("src/lib.rs", "bar::foo!();")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                [lib]
                proc-macro = true
            "#,
        )
        .file(
            "bar/src/lib.rs",
            r#"
                extern crate proc_macro;
                use proc_macro::*;

                #[proc_macro]
                pub fn foo(input: TokenStream) -> TokenStream {
                    println!("a");
                    println!("b");
                    println!("{{}}");
                    eprintln!("c");
                    eprintln!("d");
                    eprintln!("{{a"); // "malformed json"
                    input
                }
            "#,
        )
        .build();

    foo.cargo("build")
        .with_stdout("a\nb\n{}")
        .with_stderr(
            "\
[COMPILING] [..]
[COMPILING] [..]
c
d
{a
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn build_lib_only() {
    let p = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", r#" "#)
        .build();

    p.cargo("build --lib -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--crate-type lib \
        --emit=[..]link[..]-C debuginfo=2 \
        -C metadata=[..] \
        --out-dir [..] \
        -L dependency=[CWD]/target/debug/deps`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

#[cargo_test]
fn build_with_no_lib() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build --lib")
        .with_status(101)
        .with_stderr("[ERROR] no library targets found in package `foo`")
        .run();
}

#[cargo_test]
fn build_with_relative_cargo_home_path() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]

            name = "foo"
            version = "0.0.1"
            authors = ["wycats@example.com"]

            [dependencies]

            "test-dependency" = { path = "src/test_dependency" }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("src/test_dependency/src/lib.rs", r#" "#)
        .file(
            "src/test_dependency/Cargo.toml",
            &basic_manifest("test-dependency", "0.0.1"),
        )
        .build();

    p.cargo("build").env("CARGO_HOME", "./cargo_home/").run();
}

#[cargo_test]
fn user_specific_cfgs_are_filtered_out() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", r#"fn main() {}"#)
        .file(
            "build.rs",
            r#"
            fn main() {
                assert!(std::env::var_os("CARGO_CFG_PROC_MACRO").is_none());
                assert!(std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none());
            }"#,
        )
        .build();

    p.cargo("rustc -- --cfg debug_assertions --cfg proc_macro")
        .run();
    p.process(&p.bin("foo")).run();
}

#[test]
fn close_output0() {
    close_output();
}
#[test]
fn close_output1() {
    close_output();
}
#[test]
fn close_output2() {
    close_output();
}
#[test]
fn close_output3() {
    close_output();
}
#[test]
fn close_output4() {
    close_output();
}
#[test]
fn close_output5() {
    close_output();
}
#[test]
fn close_output6() {
    close_output();
}
#[test]
fn close_output7() {
    close_output();
}
#[test]
fn close_output8() {
    close_output();
}
#[test]
fn close_output9() {
    close_output();
}
#[test]
fn close_output10() {
    close_output();
}
#[test]
fn close_output11() {
    close_output();
}
#[test]
fn close_output12() {
    close_output();
}
#[test]
fn close_output13() {
    close_output();
}
#[test]
fn close_output14() {
    close_output();
}
#[test]
fn close_output15() {
    close_output();
}
#[test]
fn close_output16() {
    close_output();
}
#[test]
fn close_output17() {
    close_output();
}
#[test]
fn close_output18() {
    close_output();
}
#[test]
fn close_output19() {
    close_output();
}
#[test]
fn close_output20() {
    close_output();
}
#[test]
fn close_output21() {
    close_output();
}
#[test]
fn close_output22() {
    close_output();
}
#[test]
fn close_output23() {
    close_output();
}
#[test]
fn close_output24() {
    close_output();
}
#[test]
fn close_output25() {
    close_output();
}
#[test]
fn close_output26() {
    close_output();
}
#[test]
fn close_output27() {
    close_output();
}
#[test]
fn close_output28() {
    close_output();
}
#[test]
fn close_output29() {
    close_output();
}
#[test]
fn close_output30() {
    close_output();
}
#[test]
fn close_output31() {
    close_output();
}
#[test]
fn close_output32() {
    close_output();
}
#[test]
fn close_output33() {
    close_output();
}
#[test]
fn close_output34() {
    close_output();
}
#[test]
fn close_output35() {
    close_output();
}
#[test]
fn close_output36() {
    close_output();
}
#[test]
fn close_output37() {
    close_output();
}
#[test]
fn close_output38() {
    close_output();
}
#[test]
fn close_output39() {
    close_output();
}
#[test]
fn close_output40() {
    close_output();
}
#[test]
fn close_output41() {
    close_output();
}
#[test]
fn close_output42() {
    close_output();
}
#[test]
fn close_output43() {
    close_output();
}
#[test]
fn close_output44() {
    close_output();
}
#[test]
fn close_output45() {
    close_output();
}
#[test]
fn close_output46() {
    close_output();
}
#[test]
fn close_output47() {
    close_output();
}
#[test]
fn close_output48() {
    close_output();
}
#[test]
fn close_output49() {
    close_output();
}
#[test]
fn close_output50() {
    close_output();
}
#[test]
fn close_output51() {
    close_output();
}
#[test]
fn close_output52() {
    close_output();
}
#[test]
fn close_output53() {
    close_output();
}
#[test]
fn close_output54() {
    close_output();
}
#[test]
fn close_output55() {
    close_output();
}
#[test]
fn close_output56() {
    close_output();
}
#[test]
fn close_output57() {
    close_output();
}
#[test]
fn close_output58() {
    close_output();
}
#[test]
fn close_output59() {
    close_output();
}
#[test]
fn close_output60() {
    close_output();
}
#[test]
fn close_output61() {
    close_output();
}
#[test]
fn close_output62() {
    close_output();
}
#[test]
fn close_output63() {
    close_output();
}
#[test]
fn close_output64() {
    close_output();
}
#[test]
fn close_output65() {
    close_output();
}
#[test]
fn close_output66() {
    close_output();
}
#[test]
fn close_output67() {
    close_output();
}
#[test]
fn close_output68() {
    close_output();
}
#[test]
fn close_output69() {
    close_output();
}
#[test]
fn close_output70() {
    close_output();
}
#[test]
fn close_output71() {
    close_output();
}
#[test]
fn close_output72() {
    close_output();
}
#[test]
fn close_output73() {
    close_output();
}
#[test]
fn close_output74() {
    close_output();
}
#[test]
fn close_output75() {
    close_output();
}
#[test]
fn close_output76() {
    close_output();
}
#[test]
fn close_output77() {
    close_output();
}
#[test]
fn close_output78() {
    close_output();
}
#[test]
fn close_output79() {
    close_output();
}
#[test]
fn close_output80() {
    close_output();
}
#[test]
fn close_output81() {
    close_output();
}
#[test]
fn close_output82() {
    close_output();
}
#[test]
fn close_output83() {
    close_output();
}
#[test]
fn close_output84() {
    close_output();
}
#[test]
fn close_output85() {
    close_output();
}
#[test]
fn close_output86() {
    close_output();
}
#[test]
fn close_output87() {
    close_output();
}
#[test]
fn close_output88() {
    close_output();
}
#[test]
fn close_output89() {
    close_output();
}
#[test]
fn close_output90() {
    close_output();
}
#[test]
fn close_output91() {
    close_output();
}
#[test]
fn close_output92() {
    close_output();
}
#[test]
fn close_output93() {
    close_output();
}
#[test]
fn close_output94() {
    close_output();
}
#[test]
fn close_output95() {
    close_output();
}
#[test]
fn close_output96() {
    close_output();
}
#[test]
fn close_output97() {
    close_output();
}
#[test]
fn close_output98() {
    close_output();
}
#[test]
fn close_output99() {
    close_output();
}
#[test]
fn close_output100() {
    close_output();
}
#[test]
fn close_output101() {
    close_output();
}
#[test]
fn close_output102() {
    close_output();
}
#[test]
fn close_output103() {
    close_output();
}
#[test]
fn close_output104() {
    close_output();
}
#[test]
fn close_output105() {
    close_output();
}
#[test]
fn close_output106() {
    close_output();
}
#[test]
fn close_output107() {
    close_output();
}
#[test]
fn close_output108() {
    close_output();
}
#[test]
fn close_output109() {
    close_output();
}
#[test]
fn close_output110() {
    close_output();
}
#[test]
fn close_output111() {
    close_output();
}
#[test]
fn close_output112() {
    close_output();
}
#[test]
fn close_output113() {
    close_output();
}
#[test]
fn close_output114() {
    close_output();
}
#[test]
fn close_output115() {
    close_output();
}
#[test]
fn close_output116() {
    close_output();
}
#[test]
fn close_output117() {
    close_output();
}
#[test]
fn close_output118() {
    close_output();
}
#[test]
fn close_output119() {
    close_output();
}
#[test]
fn close_output120() {
    close_output();
}
#[test]
fn close_output121() {
    close_output();
}
#[test]
fn close_output122() {
    close_output();
}
#[test]
fn close_output123() {
    close_output();
}
#[test]
fn close_output124() {
    close_output();
}
#[test]
fn close_output125() {
    close_output();
}
#[test]
fn close_output126() {
    close_output();
}
#[test]
fn close_output127() {
    close_output();
}
#[test]
fn close_output128() {
    close_output();
}
#[test]
fn close_output129() {
    close_output();
}
#[test]
fn close_output130() {
    close_output();
}
#[test]
fn close_output131() {
    close_output();
}
#[test]
fn close_output132() {
    close_output();
}
#[test]
fn close_output133() {
    close_output();
}
#[test]
fn close_output134() {
    close_output();
}
#[test]
fn close_output135() {
    close_output();
}
#[test]
fn close_output136() {
    close_output();
}
#[test]
fn close_output137() {
    close_output();
}
#[test]
fn close_output138() {
    close_output();
}
#[test]
fn close_output139() {
    close_output();
}
#[test]
fn close_output140() {
    close_output();
}
#[test]
fn close_output141() {
    close_output();
}
#[test]
fn close_output142() {
    close_output();
}
#[test]
fn close_output143() {
    close_output();
}
#[test]
fn close_output144() {
    close_output();
}
#[test]
fn close_output145() {
    close_output();
}
#[test]
fn close_output146() {
    close_output();
}
#[test]
fn close_output147() {
    close_output();
}
#[test]
fn close_output148() {
    close_output();
}
#[test]
fn close_output149() {
    close_output();
}
#[test]
fn close_output150() {
    close_output();
}
#[test]
fn close_output151() {
    close_output();
}
#[test]
fn close_output152() {
    close_output();
}
#[test]
fn close_output153() {
    close_output();
}
#[test]
fn close_output154() {
    close_output();
}
#[test]
fn close_output155() {
    close_output();
}
#[test]
fn close_output156() {
    close_output();
}
#[test]
fn close_output157() {
    close_output();
}
#[test]
fn close_output158() {
    close_output();
}
#[test]
fn close_output159() {
    close_output();
}
#[test]
fn close_output160() {
    close_output();
}
#[test]
fn close_output161() {
    close_output();
}
#[test]
fn close_output162() {
    close_output();
}
#[test]
fn close_output163() {
    close_output();
}
#[test]
fn close_output164() {
    close_output();
}
#[test]
fn close_output165() {
    close_output();
}
#[test]
fn close_output166() {
    close_output();
}
#[test]
fn close_output167() {
    close_output();
}
#[test]
fn close_output168() {
    close_output();
}
#[test]
fn close_output169() {
    close_output();
}
#[test]
fn close_output170() {
    close_output();
}
#[test]
fn close_output171() {
    close_output();
}
#[test]
fn close_output172() {
    close_output();
}
#[test]
fn close_output173() {
    close_output();
}
#[test]
fn close_output174() {
    close_output();
}
#[test]
fn close_output175() {
    close_output();
}
#[test]
fn close_output176() {
    close_output();
}
#[test]
fn close_output177() {
    close_output();
}
#[test]
fn close_output178() {
    close_output();
}
#[test]
fn close_output179() {
    close_output();
}
#[test]
fn close_output180() {
    close_output();
}
#[test]
fn close_output181() {
    close_output();
}
#[test]
fn close_output182() {
    close_output();
}
#[test]
fn close_output183() {
    close_output();
}
#[test]
fn close_output184() {
    close_output();
}
#[test]
fn close_output185() {
    close_output();
}
#[test]
fn close_output186() {
    close_output();
}
#[test]
fn close_output187() {
    close_output();
}
#[test]
fn close_output188() {
    close_output();
}
#[test]
fn close_output189() {
    close_output();
}
#[test]
fn close_output190() {
    close_output();
}
#[test]
fn close_output191() {
    close_output();
}
#[test]
fn close_output192() {
    close_output();
}
#[test]
fn close_output193() {
    close_output();
}
#[test]
fn close_output194() {
    close_output();
}
#[test]
fn close_output195() {
    close_output();
}
#[test]
fn close_output196() {
    close_output();
}
#[test]
fn close_output197() {
    close_output();
}
#[test]
fn close_output198() {
    close_output();
}
#[test]
fn close_output199() {
    close_output();
}
#[test]
fn close_output200() {
    close_output();
}
#[test]
fn close_output201() {
    close_output();
}
#[test]
fn close_output202() {
    close_output();
}
#[test]
fn close_output203() {
    close_output();
}
#[test]
fn close_output204() {
    close_output();
}
#[test]
fn close_output205() {
    close_output();
}
#[test]
fn close_output206() {
    close_output();
}
#[test]
fn close_output207() {
    close_output();
}
#[test]
fn close_output208() {
    close_output();
}
#[test]
fn close_output209() {
    close_output();
}
#[test]
fn close_output210() {
    close_output();
}
#[test]
fn close_output211() {
    close_output();
}
#[test]
fn close_output212() {
    close_output();
}
#[test]
fn close_output213() {
    close_output();
}
#[test]
fn close_output214() {
    close_output();
}
#[test]
fn close_output215() {
    close_output();
}
#[test]
fn close_output216() {
    close_output();
}
#[test]
fn close_output217() {
    close_output();
}
#[test]
fn close_output218() {
    close_output();
}
#[test]
fn close_output219() {
    close_output();
}
#[test]
fn close_output220() {
    close_output();
}
#[test]
fn close_output221() {
    close_output();
}
#[test]
fn close_output222() {
    close_output();
}
#[test]
fn close_output223() {
    close_output();
}
#[test]
fn close_output224() {
    close_output();
}
#[test]
fn close_output225() {
    close_output();
}
#[test]
fn close_output226() {
    close_output();
}
#[test]
fn close_output227() {
    close_output();
}
#[test]
fn close_output228() {
    close_output();
}
#[test]
fn close_output229() {
    close_output();
}
#[test]
fn close_output230() {
    close_output();
}
#[test]
fn close_output231() {
    close_output();
}
#[test]
fn close_output232() {
    close_output();
}
#[test]
fn close_output233() {
    close_output();
}
#[test]
fn close_output234() {
    close_output();
}
#[test]
fn close_output235() {
    close_output();
}
#[test]
fn close_output236() {
    close_output();
}
#[test]
fn close_output237() {
    close_output();
}
#[test]
fn close_output238() {
    close_output();
}
#[test]
fn close_output239() {
    close_output();
}
#[test]
fn close_output240() {
    close_output();
}
#[test]
fn close_output241() {
    close_output();
}
#[test]
fn close_output242() {
    close_output();
}
#[test]
fn close_output243() {
    close_output();
}
#[test]
fn close_output244() {
    close_output();
}
#[test]
fn close_output245() {
    close_output();
}
#[test]
fn close_output246() {
    close_output();
}
#[test]
fn close_output247() {
    close_output();
}
#[test]
fn close_output248() {
    close_output();
}
#[test]
fn close_output249() {
    close_output();
}
#[test]
fn close_output250() {
    close_output();
}
#[test]
fn close_output251() {
    close_output();
}
#[test]
fn close_output252() {
    close_output();
}
#[test]
fn close_output253() {
    close_output();
}
#[test]
fn close_output254() {
    close_output();
}
#[test]
fn close_output255() {
    close_output();
}
#[test]
fn close_output256() {
    close_output();
}
#[test]
fn close_output257() {
    close_output();
}
#[test]
fn close_output258() {
    close_output();
}
#[test]
fn close_output259() {
    close_output();
}
#[test]
fn close_output260() {
    close_output();
}
#[test]
fn close_output261() {
    close_output();
}
#[test]
fn close_output262() {
    close_output();
}
#[test]
fn close_output263() {
    close_output();
}
#[test]
fn close_output264() {
    close_output();
}
#[test]
fn close_output265() {
    close_output();
}
#[test]
fn close_output266() {
    close_output();
}
#[test]
fn close_output267() {
    close_output();
}
#[test]
fn close_output268() {
    close_output();
}
#[test]
fn close_output269() {
    close_output();
}
#[test]
fn close_output270() {
    close_output();
}
#[test]
fn close_output271() {
    close_output();
}
#[test]
fn close_output272() {
    close_output();
}
#[test]
fn close_output273() {
    close_output();
}
#[test]
fn close_output274() {
    close_output();
}
#[test]
fn close_output275() {
    close_output();
}
#[test]
fn close_output276() {
    close_output();
}
#[test]
fn close_output277() {
    close_output();
}
#[test]
fn close_output278() {
    close_output();
}
#[test]
fn close_output279() {
    close_output();
}
#[test]
fn close_output280() {
    close_output();
}
#[test]
fn close_output281() {
    close_output();
}
#[test]
fn close_output282() {
    close_output();
}
#[test]
fn close_output283() {
    close_output();
}
#[test]
fn close_output284() {
    close_output();
}
#[test]
fn close_output285() {
    close_output();
}
#[test]
fn close_output286() {
    close_output();
}
#[test]
fn close_output287() {
    close_output();
}
#[test]
fn close_output288() {
    close_output();
}
#[test]
fn close_output289() {
    close_output();
}
#[test]
fn close_output290() {
    close_output();
}
#[test]
fn close_output291() {
    close_output();
}
#[test]
fn close_output292() {
    close_output();
}
#[test]
fn close_output293() {
    close_output();
}
#[test]
fn close_output294() {
    close_output();
}
#[test]
fn close_output295() {
    close_output();
}
#[test]
fn close_output296() {
    close_output();
}
#[test]
fn close_output297() {
    close_output();
}
#[test]
fn close_output298() {
    close_output();
}
#[test]
fn close_output299() {
    close_output();
}
#[test]
fn close_output300() {
    close_output();
}
#[test]
fn close_output301() {
    close_output();
}
#[test]
fn close_output302() {
    close_output();
}
#[test]
fn close_output303() {
    close_output();
}
#[test]
fn close_output304() {
    close_output();
}
#[test]
fn close_output305() {
    close_output();
}
#[test]
fn close_output306() {
    close_output();
}
#[test]
fn close_output307() {
    close_output();
}
#[test]
fn close_output308() {
    close_output();
}
#[test]
fn close_output309() {
    close_output();
}
#[test]
fn close_output310() {
    close_output();
}
#[test]
fn close_output311() {
    close_output();
}
#[test]
fn close_output312() {
    close_output();
}
#[test]
fn close_output313() {
    close_output();
}
#[test]
fn close_output314() {
    close_output();
}
#[test]
fn close_output315() {
    close_output();
}
#[test]
fn close_output316() {
    close_output();
}
#[test]
fn close_output317() {
    close_output();
}
#[test]
fn close_output318() {
    close_output();
}
#[test]
fn close_output319() {
    close_output();
}
#[test]
fn close_output320() {
    close_output();
}
#[test]
fn close_output321() {
    close_output();
}
#[test]
fn close_output322() {
    close_output();
}
#[test]
fn close_output323() {
    close_output();
}
#[test]
fn close_output324() {
    close_output();
}
#[test]
fn close_output325() {
    close_output();
}
#[test]
fn close_output326() {
    close_output();
}
#[test]
fn close_output327() {
    close_output();
}
#[test]
fn close_output328() {
    close_output();
}
#[test]
fn close_output329() {
    close_output();
}
#[test]
fn close_output330() {
    close_output();
}
#[test]
fn close_output331() {
    close_output();
}
#[test]
fn close_output332() {
    close_output();
}
#[test]
fn close_output333() {
    close_output();
}
#[test]
fn close_output334() {
    close_output();
}
#[test]
fn close_output335() {
    close_output();
}
#[test]
fn close_output336() {
    close_output();
}
#[test]
fn close_output337() {
    close_output();
}
#[test]
fn close_output338() {
    close_output();
}
#[test]
fn close_output339() {
    close_output();
}
#[test]
fn close_output340() {
    close_output();
}
#[test]
fn close_output341() {
    close_output();
}
#[test]
fn close_output342() {
    close_output();
}
#[test]
fn close_output343() {
    close_output();
}
#[test]
fn close_output344() {
    close_output();
}
#[test]
fn close_output345() {
    close_output();
}
#[test]
fn close_output346() {
    close_output();
}
#[test]
fn close_output347() {
    close_output();
}
#[test]
fn close_output348() {
    close_output();
}
#[test]
fn close_output349() {
    close_output();
}
#[test]
fn close_output350() {
    close_output();
}
#[test]
fn close_output351() {
    close_output();
}
#[test]
fn close_output352() {
    close_output();
}
#[test]
fn close_output353() {
    close_output();
}
#[test]
fn close_output354() {
    close_output();
}
#[test]
fn close_output355() {
    close_output();
}
#[test]
fn close_output356() {
    close_output();
}
#[test]
fn close_output357() {
    close_output();
}
#[test]
fn close_output358() {
    close_output();
}
#[test]
fn close_output359() {
    close_output();
}
#[test]
fn close_output360() {
    close_output();
}
#[test]
fn close_output361() {
    close_output();
}
#[test]
fn close_output362() {
    close_output();
}
#[test]
fn close_output363() {
    close_output();
}
#[test]
fn close_output364() {
    close_output();
}
#[test]
fn close_output365() {
    close_output();
}
#[test]
fn close_output366() {
    close_output();
}
#[test]
fn close_output367() {
    close_output();
}
#[test]
fn close_output368() {
    close_output();
}
#[test]
fn close_output369() {
    close_output();
}
#[test]
fn close_output370() {
    close_output();
}
#[test]
fn close_output371() {
    close_output();
}
#[test]
fn close_output372() {
    close_output();
}
#[test]
fn close_output373() {
    close_output();
}
#[test]
fn close_output374() {
    close_output();
}
#[test]
fn close_output375() {
    close_output();
}
#[test]
fn close_output376() {
    close_output();
}
#[test]
fn close_output377() {
    close_output();
}
#[test]
fn close_output378() {
    close_output();
}
#[test]
fn close_output379() {
    close_output();
}
#[test]
fn close_output380() {
    close_output();
}
#[test]
fn close_output381() {
    close_output();
}
#[test]
fn close_output382() {
    close_output();
}
#[test]
fn close_output383() {
    close_output();
}
#[test]
fn close_output384() {
    close_output();
}
#[test]
fn close_output385() {
    close_output();
}
#[test]
fn close_output386() {
    close_output();
}
#[test]
fn close_output387() {
    close_output();
}
#[test]
fn close_output388() {
    close_output();
}
#[test]
fn close_output389() {
    close_output();
}
#[test]
fn close_output390() {
    close_output();
}
#[test]
fn close_output391() {
    close_output();
}
#[test]
fn close_output392() {
    close_output();
}
#[test]
fn close_output393() {
    close_output();
}
#[test]
fn close_output394() {
    close_output();
}
#[test]
fn close_output395() {
    close_output();
}
#[test]
fn close_output396() {
    close_output();
}
#[test]
fn close_output397() {
    close_output();
}
#[test]
fn close_output398() {
    close_output();
}
#[test]
fn close_output399() {
    close_output();
}
#[test]
fn close_output400() {
    close_output();
}
#[test]
fn close_output401() {
    close_output();
}
#[test]
fn close_output402() {
    close_output();
}
#[test]
fn close_output403() {
    close_output();
}
#[test]
fn close_output404() {
    close_output();
}
#[test]
fn close_output405() {
    close_output();
}
#[test]
fn close_output406() {
    close_output();
}
#[test]
fn close_output407() {
    close_output();
}
#[test]
fn close_output408() {
    close_output();
}
#[test]
fn close_output409() {
    close_output();
}
#[test]
fn close_output410() {
    close_output();
}
#[test]
fn close_output411() {
    close_output();
}
#[test]
fn close_output412() {
    close_output();
}
#[test]
fn close_output413() {
    close_output();
}
#[test]
fn close_output414() {
    close_output();
}
#[test]
fn close_output415() {
    close_output();
}
#[test]
fn close_output416() {
    close_output();
}
#[test]
fn close_output417() {
    close_output();
}
#[test]
fn close_output418() {
    close_output();
}
#[test]
fn close_output419() {
    close_output();
}
#[test]
fn close_output420() {
    close_output();
}
#[test]
fn close_output421() {
    close_output();
}
#[test]
fn close_output422() {
    close_output();
}
#[test]
fn close_output423() {
    close_output();
}
#[test]
fn close_output424() {
    close_output();
}
#[test]
fn close_output425() {
    close_output();
}
#[test]
fn close_output426() {
    close_output();
}
#[test]
fn close_output427() {
    close_output();
}
#[test]
fn close_output428() {
    close_output();
}
#[test]
fn close_output429() {
    close_output();
}
#[test]
fn close_output430() {
    close_output();
}
#[test]
fn close_output431() {
    close_output();
}
#[test]
fn close_output432() {
    close_output();
}
#[test]
fn close_output433() {
    close_output();
}
#[test]
fn close_output434() {
    close_output();
}
#[test]
fn close_output435() {
    close_output();
}
#[test]
fn close_output436() {
    close_output();
}
#[test]
fn close_output437() {
    close_output();
}
#[test]
fn close_output438() {
    close_output();
}
#[test]
fn close_output439() {
    close_output();
}
#[test]
fn close_output440() {
    close_output();
}
#[test]
fn close_output441() {
    close_output();
}
#[test]
fn close_output442() {
    close_output();
}
#[test]
fn close_output443() {
    close_output();
}
#[test]
fn close_output444() {
    close_output();
}
#[test]
fn close_output445() {
    close_output();
}
#[test]
fn close_output446() {
    close_output();
}
#[test]
fn close_output447() {
    close_output();
}
#[test]
fn close_output448() {
    close_output();
}
#[test]
fn close_output449() {
    close_output();
}
#[test]
fn close_output450() {
    close_output();
}
#[test]
fn close_output451() {
    close_output();
}
#[test]
fn close_output452() {
    close_output();
}
#[test]
fn close_output453() {
    close_output();
}
#[test]
fn close_output454() {
    close_output();
}
#[test]
fn close_output455() {
    close_output();
}
#[test]
fn close_output456() {
    close_output();
}
#[test]
fn close_output457() {
    close_output();
}
#[test]
fn close_output458() {
    close_output();
}
#[test]
fn close_output459() {
    close_output();
}
#[test]
fn close_output460() {
    close_output();
}
#[test]
fn close_output461() {
    close_output();
}
#[test]
fn close_output462() {
    close_output();
}
#[test]
fn close_output463() {
    close_output();
}
#[test]
fn close_output464() {
    close_output();
}
#[test]
fn close_output465() {
    close_output();
}
#[test]
fn close_output466() {
    close_output();
}
#[test]
fn close_output467() {
    close_output();
}
#[test]
fn close_output468() {
    close_output();
}
#[test]
fn close_output469() {
    close_output();
}
#[test]
fn close_output470() {
    close_output();
}
#[test]
fn close_output471() {
    close_output();
}
#[test]
fn close_output472() {
    close_output();
}
#[test]
fn close_output473() {
    close_output();
}
#[test]
fn close_output474() {
    close_output();
}
#[test]
fn close_output475() {
    close_output();
}
#[test]
fn close_output476() {
    close_output();
}
#[test]
fn close_output477() {
    close_output();
}
#[test]
fn close_output478() {
    close_output();
}
#[test]
fn close_output479() {
    close_output();
}
#[test]
fn close_output480() {
    close_output();
}
#[test]
fn close_output481() {
    close_output();
}
#[test]
fn close_output482() {
    close_output();
}
#[test]
fn close_output483() {
    close_output();
}
#[test]
fn close_output484() {
    close_output();
}
#[test]
fn close_output485() {
    close_output();
}
#[test]
fn close_output486() {
    close_output();
}
#[test]
fn close_output487() {
    close_output();
}
#[test]
fn close_output488() {
    close_output();
}
#[test]
fn close_output489() {
    close_output();
}
#[test]
fn close_output490() {
    close_output();
}
#[test]
fn close_output491() {
    close_output();
}
#[test]
fn close_output492() {
    close_output();
}
#[test]
fn close_output493() {
    close_output();
}
#[test]
fn close_output494() {
    close_output();
}
#[test]
fn close_output495() {
    close_output();
}
#[test]
fn close_output496() {
    close_output();
}
#[test]
fn close_output497() {
    close_output();
}
#[test]
fn close_output498() {
    close_output();
}
#[test]
fn close_output499() {
    close_output();
}
#[test]
fn close_output500() {
    close_output();
}
#[test]
fn close_output501() {
    close_output();
}
#[test]
fn close_output502() {
    close_output();
}
#[test]
fn close_output503() {
    close_output();
}
#[test]
fn close_output504() {
    close_output();
}
#[test]
fn close_output505() {
    close_output();
}
#[test]
fn close_output506() {
    close_output();
}
#[test]
fn close_output507() {
    close_output();
}
#[test]
fn close_output508() {
    close_output();
}
#[test]
fn close_output509() {
    close_output();
}
#[test]
fn close_output510() {
    close_output();
}
#[test]
fn close_output511() {
    close_output();
}
#[test]
fn close_output512() {
    close_output();
}
#[test]
fn close_output513() {
    close_output();
}
#[test]
fn close_output514() {
    close_output();
}
#[test]
fn close_output515() {
    close_output();
}
#[test]
fn close_output516() {
    close_output();
}
#[test]
fn close_output517() {
    close_output();
}
#[test]
fn close_output518() {
    close_output();
}
#[test]
fn close_output519() {
    close_output();
}
#[test]
fn close_output520() {
    close_output();
}
#[test]
fn close_output521() {
    close_output();
}
#[test]
fn close_output522() {
    close_output();
}
#[test]
fn close_output523() {
    close_output();
}
#[test]
fn close_output524() {
    close_output();
}
#[test]
fn close_output525() {
    close_output();
}
#[test]
fn close_output526() {
    close_output();
}
#[test]
fn close_output527() {
    close_output();
}
#[test]
fn close_output528() {
    close_output();
}
#[test]
fn close_output529() {
    close_output();
}
#[test]
fn close_output530() {
    close_output();
}
#[test]
fn close_output531() {
    close_output();
}
#[test]
fn close_output532() {
    close_output();
}
#[test]
fn close_output533() {
    close_output();
}
#[test]
fn close_output534() {
    close_output();
}
#[test]
fn close_output535() {
    close_output();
}
#[test]
fn close_output536() {
    close_output();
}
#[test]
fn close_output537() {
    close_output();
}
#[test]
fn close_output538() {
    close_output();
}
#[test]
fn close_output539() {
    close_output();
}
#[test]
fn close_output540() {
    close_output();
}
#[test]
fn close_output541() {
    close_output();
}
#[test]
fn close_output542() {
    close_output();
}
#[test]
fn close_output543() {
    close_output();
}
#[test]
fn close_output544() {
    close_output();
}
#[test]
fn close_output545() {
    close_output();
}
#[test]
fn close_output546() {
    close_output();
}
#[test]
fn close_output547() {
    close_output();
}
#[test]
fn close_output548() {
    close_output();
}
#[test]
fn close_output549() {
    close_output();
}
#[test]
fn close_output550() {
    close_output();
}
#[test]
fn close_output551() {
    close_output();
}
#[test]
fn close_output552() {
    close_output();
}
#[test]
fn close_output553() {
    close_output();
}
#[test]
fn close_output554() {
    close_output();
}
#[test]
fn close_output555() {
    close_output();
}
#[test]
fn close_output556() {
    close_output();
}
#[test]
fn close_output557() {
    close_output();
}
#[test]
fn close_output558() {
    close_output();
}
#[test]
fn close_output559() {
    close_output();
}
#[test]
fn close_output560() {
    close_output();
}
#[test]
fn close_output561() {
    close_output();
}
#[test]
fn close_output562() {
    close_output();
}
#[test]
fn close_output563() {
    close_output();
}
#[test]
fn close_output564() {
    close_output();
}
#[test]
fn close_output565() {
    close_output();
}
#[test]
fn close_output566() {
    close_output();
}
#[test]
fn close_output567() {
    close_output();
}
#[test]
fn close_output568() {
    close_output();
}
#[test]
fn close_output569() {
    close_output();
}
#[test]
fn close_output570() {
    close_output();
}
#[test]
fn close_output571() {
    close_output();
}
#[test]
fn close_output572() {
    close_output();
}
#[test]
fn close_output573() {
    close_output();
}
#[test]
fn close_output574() {
    close_output();
}
#[test]
fn close_output575() {
    close_output();
}
#[test]
fn close_output576() {
    close_output();
}
#[test]
fn close_output577() {
    close_output();
}
#[test]
fn close_output578() {
    close_output();
}
#[test]
fn close_output579() {
    close_output();
}
#[test]
fn close_output580() {
    close_output();
}
#[test]
fn close_output581() {
    close_output();
}
#[test]
fn close_output582() {
    close_output();
}
#[test]
fn close_output583() {
    close_output();
}
#[test]
fn close_output584() {
    close_output();
}
#[test]
fn close_output585() {
    close_output();
}
#[test]
fn close_output586() {
    close_output();
}
#[test]
fn close_output587() {
    close_output();
}
#[test]
fn close_output588() {
    close_output();
}
#[test]
fn close_output589() {
    close_output();
}
#[test]
fn close_output590() {
    close_output();
}
#[test]
fn close_output591() {
    close_output();
}
#[test]
fn close_output592() {
    close_output();
}
#[test]
fn close_output593() {
    close_output();
}
#[test]
fn close_output594() {
    close_output();
}
#[test]
fn close_output595() {
    close_output();
}
#[test]
fn close_output596() {
    close_output();
}
#[test]
fn close_output597() {
    close_output();
}
#[test]
fn close_output598() {
    close_output();
}
#[test]
fn close_output599() {
    close_output();
}
#[test]
fn close_output600() {
    close_output();
}
#[test]
fn close_output601() {
    close_output();
}
#[test]
fn close_output602() {
    close_output();
}
#[test]
fn close_output603() {
    close_output();
}
#[test]
fn close_output604() {
    close_output();
}
#[test]
fn close_output605() {
    close_output();
}
#[test]
fn close_output606() {
    close_output();
}
#[test]
fn close_output607() {
    close_output();
}
#[test]
fn close_output608() {
    close_output();
}
#[test]
fn close_output609() {
    close_output();
}
#[test]
fn close_output610() {
    close_output();
}
#[test]
fn close_output611() {
    close_output();
}
#[test]
fn close_output612() {
    close_output();
}
#[test]
fn close_output613() {
    close_output();
}
#[test]
fn close_output614() {
    close_output();
}
#[test]
fn close_output615() {
    close_output();
}
#[test]
fn close_output616() {
    close_output();
}
#[test]
fn close_output617() {
    close_output();
}
#[test]
fn close_output618() {
    close_output();
}
#[test]
fn close_output619() {
    close_output();
}
#[test]
fn close_output620() {
    close_output();
}
#[test]
fn close_output621() {
    close_output();
}
#[test]
fn close_output622() {
    close_output();
}
#[test]
fn close_output623() {
    close_output();
}
#[test]
fn close_output624() {
    close_output();
}
#[test]
fn close_output625() {
    close_output();
}
#[test]
fn close_output626() {
    close_output();
}
#[test]
fn close_output627() {
    close_output();
}
#[test]
fn close_output628() {
    close_output();
}
#[test]
fn close_output629() {
    close_output();
}
#[test]
fn close_output630() {
    close_output();
}
#[test]
fn close_output631() {
    close_output();
}
#[test]
fn close_output632() {
    close_output();
}
#[test]
fn close_output633() {
    close_output();
}
#[test]
fn close_output634() {
    close_output();
}
#[test]
fn close_output635() {
    close_output();
}
#[test]
fn close_output636() {
    close_output();
}
#[test]
fn close_output637() {
    close_output();
}
#[test]
fn close_output638() {
    close_output();
}
#[test]
fn close_output639() {
    close_output();
}
#[test]
fn close_output640() {
    close_output();
}
#[test]
fn close_output641() {
    close_output();
}
#[test]
fn close_output642() {
    close_output();
}
#[test]
fn close_output643() {
    close_output();
}
#[test]
fn close_output644() {
    close_output();
}
#[test]
fn close_output645() {
    close_output();
}
#[test]
fn close_output646() {
    close_output();
}
#[test]
fn close_output647() {
    close_output();
}
#[test]
fn close_output648() {
    close_output();
}
#[test]
fn close_output649() {
    close_output();
}
#[test]
fn close_output650() {
    close_output();
}
#[test]
fn close_output651() {
    close_output();
}
#[test]
fn close_output652() {
    close_output();
}
#[test]
fn close_output653() {
    close_output();
}
#[test]
fn close_output654() {
    close_output();
}
#[test]
fn close_output655() {
    close_output();
}
#[test]
fn close_output656() {
    close_output();
}
#[test]
fn close_output657() {
    close_output();
}
#[test]
fn close_output658() {
    close_output();
}
#[test]
fn close_output659() {
    close_output();
}
#[test]
fn close_output660() {
    close_output();
}
#[test]
fn close_output661() {
    close_output();
}
#[test]
fn close_output662() {
    close_output();
}
#[test]
fn close_output663() {
    close_output();
}
#[test]
fn close_output664() {
    close_output();
}
#[test]
fn close_output665() {
    close_output();
}
#[test]
fn close_output666() {
    close_output();
}
#[test]
fn close_output667() {
    close_output();
}
#[test]
fn close_output668() {
    close_output();
}
#[test]
fn close_output669() {
    close_output();
}
#[test]
fn close_output670() {
    close_output();
}
#[test]
fn close_output671() {
    close_output();
}
#[test]
fn close_output672() {
    close_output();
}
#[test]
fn close_output673() {
    close_output();
}
#[test]
fn close_output674() {
    close_output();
}
#[test]
fn close_output675() {
    close_output();
}
#[test]
fn close_output676() {
    close_output();
}
#[test]
fn close_output677() {
    close_output();
}
#[test]
fn close_output678() {
    close_output();
}
#[test]
fn close_output679() {
    close_output();
}
#[test]
fn close_output680() {
    close_output();
}
#[test]
fn close_output681() {
    close_output();
}
#[test]
fn close_output682() {
    close_output();
}
#[test]
fn close_output683() {
    close_output();
}
#[test]
fn close_output684() {
    close_output();
}
#[test]
fn close_output685() {
    close_output();
}
#[test]
fn close_output686() {
    close_output();
}
#[test]
fn close_output687() {
    close_output();
}
#[test]
fn close_output688() {
    close_output();
}
#[test]
fn close_output689() {
    close_output();
}
#[test]
fn close_output690() {
    close_output();
}
#[test]
fn close_output691() {
    close_output();
}
#[test]
fn close_output692() {
    close_output();
}
#[test]
fn close_output693() {
    close_output();
}
#[test]
fn close_output694() {
    close_output();
}
#[test]
fn close_output695() {
    close_output();
}
#[test]
fn close_output696() {
    close_output();
}
#[test]
fn close_output697() {
    close_output();
}
#[test]
fn close_output698() {
    close_output();
}
#[test]
fn close_output699() {
    close_output();
}
#[test]
fn close_output700() {
    close_output();
}
#[test]
fn close_output701() {
    close_output();
}
#[test]
fn close_output702() {
    close_output();
}
#[test]
fn close_output703() {
    close_output();
}
#[test]
fn close_output704() {
    close_output();
}
#[test]
fn close_output705() {
    close_output();
}
#[test]
fn close_output706() {
    close_output();
}
#[test]
fn close_output707() {
    close_output();
}
#[test]
fn close_output708() {
    close_output();
}
#[test]
fn close_output709() {
    close_output();
}
#[test]
fn close_output710() {
    close_output();
}
#[test]
fn close_output711() {
    close_output();
}
#[test]
fn close_output712() {
    close_output();
}
#[test]
fn close_output713() {
    close_output();
}
#[test]
fn close_output714() {
    close_output();
}
#[test]
fn close_output715() {
    close_output();
}
#[test]
fn close_output716() {
    close_output();
}
#[test]
fn close_output717() {
    close_output();
}
#[test]
fn close_output718() {
    close_output();
}
#[test]
fn close_output719() {
    close_output();
}
#[test]
fn close_output720() {
    close_output();
}
#[test]
fn close_output721() {
    close_output();
}
#[test]
fn close_output722() {
    close_output();
}
#[test]
fn close_output723() {
    close_output();
}
#[test]
fn close_output724() {
    close_output();
}
#[test]
fn close_output725() {
    close_output();
}
#[test]
fn close_output726() {
    close_output();
}
#[test]
fn close_output727() {
    close_output();
}
#[test]
fn close_output728() {
    close_output();
}
#[test]
fn close_output729() {
    close_output();
}
#[test]
fn close_output730() {
    close_output();
}
#[test]
fn close_output731() {
    close_output();
}
#[test]
fn close_output732() {
    close_output();
}
#[test]
fn close_output733() {
    close_output();
}
#[test]
fn close_output734() {
    close_output();
}
#[test]
fn close_output735() {
    close_output();
}
#[test]
fn close_output736() {
    close_output();
}
#[test]
fn close_output737() {
    close_output();
}
#[test]
fn close_output738() {
    close_output();
}
#[test]
fn close_output739() {
    close_output();
}
#[test]
fn close_output740() {
    close_output();
}
#[test]
fn close_output741() {
    close_output();
}
#[test]
fn close_output742() {
    close_output();
}
#[test]
fn close_output743() {
    close_output();
}
#[test]
fn close_output744() {
    close_output();
}
#[test]
fn close_output745() {
    close_output();
}
#[test]
fn close_output746() {
    close_output();
}
#[test]
fn close_output747() {
    close_output();
}
#[test]
fn close_output748() {
    close_output();
}
#[test]
fn close_output749() {
    close_output();
}
#[test]
fn close_output750() {
    close_output();
}
#[test]
fn close_output751() {
    close_output();
}
#[test]
fn close_output752() {
    close_output();
}
#[test]
fn close_output753() {
    close_output();
}
#[test]
fn close_output754() {
    close_output();
}
#[test]
fn close_output755() {
    close_output();
}
#[test]
fn close_output756() {
    close_output();
}
#[test]
fn close_output757() {
    close_output();
}
#[test]
fn close_output758() {
    close_output();
}
#[test]
fn close_output759() {
    close_output();
}
#[test]
fn close_output760() {
    close_output();
}
#[test]
fn close_output761() {
    close_output();
}
#[test]
fn close_output762() {
    close_output();
}
#[test]
fn close_output763() {
    close_output();
}
#[test]
fn close_output764() {
    close_output();
}
#[test]
fn close_output765() {
    close_output();
}
#[test]
fn close_output766() {
    close_output();
}
#[test]
fn close_output767() {
    close_output();
}
#[test]
fn close_output768() {
    close_output();
}
#[test]
fn close_output769() {
    close_output();
}
#[test]
fn close_output770() {
    close_output();
}
#[test]
fn close_output771() {
    close_output();
}
#[test]
fn close_output772() {
    close_output();
}
#[test]
fn close_output773() {
    close_output();
}
#[test]
fn close_output774() {
    close_output();
}
#[test]
fn close_output775() {
    close_output();
}
#[test]
fn close_output776() {
    close_output();
}
#[test]
fn close_output777() {
    close_output();
}
#[test]
fn close_output778() {
    close_output();
}
#[test]
fn close_output779() {
    close_output();
}
#[test]
fn close_output780() {
    close_output();
}
#[test]
fn close_output781() {
    close_output();
}
#[test]
fn close_output782() {
    close_output();
}
#[test]
fn close_output783() {
    close_output();
}
#[test]
fn close_output784() {
    close_output();
}
#[test]
fn close_output785() {
    close_output();
}
#[test]
fn close_output786() {
    close_output();
}
#[test]
fn close_output787() {
    close_output();
}
#[test]
fn close_output788() {
    close_output();
}
#[test]
fn close_output789() {
    close_output();
}
#[test]
fn close_output790() {
    close_output();
}
#[test]
fn close_output791() {
    close_output();
}
#[test]
fn close_output792() {
    close_output();
}
#[test]
fn close_output793() {
    close_output();
}
#[test]
fn close_output794() {
    close_output();
}
#[test]
fn close_output795() {
    close_output();
}
#[test]
fn close_output796() {
    close_output();
}
#[test]
fn close_output797() {
    close_output();
}
#[test]
fn close_output798() {
    close_output();
}
#[test]
fn close_output799() {
    close_output();
}
#[test]
fn close_output800() {
    close_output();
}
#[test]
fn close_output801() {
    close_output();
}
#[test]
fn close_output802() {
    close_output();
}
#[test]
fn close_output803() {
    close_output();
}
#[test]
fn close_output804() {
    close_output();
}
#[test]
fn close_output805() {
    close_output();
}
#[test]
fn close_output806() {
    close_output();
}
#[test]
fn close_output807() {
    close_output();
}
#[test]
fn close_output808() {
    close_output();
}
#[test]
fn close_output809() {
    close_output();
}
#[test]
fn close_output810() {
    close_output();
}
#[test]
fn close_output811() {
    close_output();
}
#[test]
fn close_output812() {
    close_output();
}
#[test]
fn close_output813() {
    close_output();
}
#[test]
fn close_output814() {
    close_output();
}
#[test]
fn close_output815() {
    close_output();
}
#[test]
fn close_output816() {
    close_output();
}
#[test]
fn close_output817() {
    close_output();
}
#[test]
fn close_output818() {
    close_output();
}
#[test]
fn close_output819() {
    close_output();
}
#[test]
fn close_output820() {
    close_output();
}
#[test]
fn close_output821() {
    close_output();
}
#[test]
fn close_output822() {
    close_output();
}
#[test]
fn close_output823() {
    close_output();
}
#[test]
fn close_output824() {
    close_output();
}
#[test]
fn close_output825() {
    close_output();
}
#[test]
fn close_output826() {
    close_output();
}
#[test]
fn close_output827() {
    close_output();
}
#[test]
fn close_output828() {
    close_output();
}
#[test]
fn close_output829() {
    close_output();
}
#[test]
fn close_output830() {
    close_output();
}
#[test]
fn close_output831() {
    close_output();
}
#[test]
fn close_output832() {
    close_output();
}
#[test]
fn close_output833() {
    close_output();
}
#[test]
fn close_output834() {
    close_output();
}
#[test]
fn close_output835() {
    close_output();
}
#[test]
fn close_output836() {
    close_output();
}
#[test]
fn close_output837() {
    close_output();
}
#[test]
fn close_output838() {
    close_output();
}
#[test]
fn close_output839() {
    close_output();
}
#[test]
fn close_output840() {
    close_output();
}
#[test]
fn close_output841() {
    close_output();
}
#[test]
fn close_output842() {
    close_output();
}
#[test]
fn close_output843() {
    close_output();
}
#[test]
fn close_output844() {
    close_output();
}
#[test]
fn close_output845() {
    close_output();
}
#[test]
fn close_output846() {
    close_output();
}
#[test]
fn close_output847() {
    close_output();
}
#[test]
fn close_output848() {
    close_output();
}
#[test]
fn close_output849() {
    close_output();
}
#[test]
fn close_output850() {
    close_output();
}
#[test]
fn close_output851() {
    close_output();
}
#[test]
fn close_output852() {
    close_output();
}
#[test]
fn close_output853() {
    close_output();
}
#[test]
fn close_output854() {
    close_output();
}
#[test]
fn close_output855() {
    close_output();
}
#[test]
fn close_output856() {
    close_output();
}
#[test]
fn close_output857() {
    close_output();
}
#[test]
fn close_output858() {
    close_output();
}
#[test]
fn close_output859() {
    close_output();
}
#[test]
fn close_output860() {
    close_output();
}
#[test]
fn close_output861() {
    close_output();
}
#[test]
fn close_output862() {
    close_output();
}
#[test]
fn close_output863() {
    close_output();
}
#[test]
fn close_output864() {
    close_output();
}
#[test]
fn close_output865() {
    close_output();
}
#[test]
fn close_output866() {
    close_output();
}
#[test]
fn close_output867() {
    close_output();
}
#[test]
fn close_output868() {
    close_output();
}
#[test]
fn close_output869() {
    close_output();
}
#[test]
fn close_output870() {
    close_output();
}
#[test]
fn close_output871() {
    close_output();
}
#[test]
fn close_output872() {
    close_output();
}
#[test]
fn close_output873() {
    close_output();
}
#[test]
fn close_output874() {
    close_output();
}
#[test]
fn close_output875() {
    close_output();
}
#[test]
fn close_output876() {
    close_output();
}
#[test]
fn close_output877() {
    close_output();
}
#[test]
fn close_output878() {
    close_output();
}
#[test]
fn close_output879() {
    close_output();
}
#[test]
fn close_output880() {
    close_output();
}
#[test]
fn close_output881() {
    close_output();
}
#[test]
fn close_output882() {
    close_output();
}
#[test]
fn close_output883() {
    close_output();
}
#[test]
fn close_output884() {
    close_output();
}
#[test]
fn close_output885() {
    close_output();
}
#[test]
fn close_output886() {
    close_output();
}
#[test]
fn close_output887() {
    close_output();
}
#[test]
fn close_output888() {
    close_output();
}
#[test]
fn close_output889() {
    close_output();
}
#[test]
fn close_output890() {
    close_output();
}
#[test]
fn close_output891() {
    close_output();
}
#[test]
fn close_output892() {
    close_output();
}
#[test]
fn close_output893() {
    close_output();
}
#[test]
fn close_output894() {
    close_output();
}
#[test]
fn close_output895() {
    close_output();
}
#[test]
fn close_output896() {
    close_output();
}
#[test]
fn close_output897() {
    close_output();
}
#[test]
fn close_output898() {
    close_output();
}
#[test]
fn close_output899() {
    close_output();
}
#[test]
fn close_output900() {
    close_output();
}
#[test]
fn close_output901() {
    close_output();
}
#[test]
fn close_output902() {
    close_output();
}
#[test]
fn close_output903() {
    close_output();
}
#[test]
fn close_output904() {
    close_output();
}
#[test]
fn close_output905() {
    close_output();
}
#[test]
fn close_output906() {
    close_output();
}
#[test]
fn close_output907() {
    close_output();
}
#[test]
fn close_output908() {
    close_output();
}
#[test]
fn close_output909() {
    close_output();
}
#[test]
fn close_output910() {
    close_output();
}
#[test]
fn close_output911() {
    close_output();
}
#[test]
fn close_output912() {
    close_output();
}
#[test]
fn close_output913() {
    close_output();
}
#[test]
fn close_output914() {
    close_output();
}
#[test]
fn close_output915() {
    close_output();
}
#[test]
fn close_output916() {
    close_output();
}
#[test]
fn close_output917() {
    close_output();
}
#[test]
fn close_output918() {
    close_output();
}
#[test]
fn close_output919() {
    close_output();
}
#[test]
fn close_output920() {
    close_output();
}
#[test]
fn close_output921() {
    close_output();
}
#[test]
fn close_output922() {
    close_output();
}
#[test]
fn close_output923() {
    close_output();
}
#[test]
fn close_output924() {
    close_output();
}
#[test]
fn close_output925() {
    close_output();
}
#[test]
fn close_output926() {
    close_output();
}
#[test]
fn close_output927() {
    close_output();
}
#[test]
fn close_output928() {
    close_output();
}
#[test]
fn close_output929() {
    close_output();
}
#[test]
fn close_output930() {
    close_output();
}
#[test]
fn close_output931() {
    close_output();
}
#[test]
fn close_output932() {
    close_output();
}
#[test]
fn close_output933() {
    close_output();
}
#[test]
fn close_output934() {
    close_output();
}
#[test]
fn close_output935() {
    close_output();
}
#[test]
fn close_output936() {
    close_output();
}
#[test]
fn close_output937() {
    close_output();
}
#[test]
fn close_output938() {
    close_output();
}
#[test]
fn close_output939() {
    close_output();
}
#[test]
fn close_output940() {
    close_output();
}
#[test]
fn close_output941() {
    close_output();
}
#[test]
fn close_output942() {
    close_output();
}
#[test]
fn close_output943() {
    close_output();
}
#[test]
fn close_output944() {
    close_output();
}
#[test]
fn close_output945() {
    close_output();
}
#[test]
fn close_output946() {
    close_output();
}
#[test]
fn close_output947() {
    close_output();
}
#[test]
fn close_output948() {
    close_output();
}
#[test]
fn close_output949() {
    close_output();
}
#[test]
fn close_output950() {
    close_output();
}
#[test]
fn close_output951() {
    close_output();
}
#[test]
fn close_output952() {
    close_output();
}
#[test]
fn close_output953() {
    close_output();
}
#[test]
fn close_output954() {
    close_output();
}
#[test]
fn close_output955() {
    close_output();
}
#[test]
fn close_output956() {
    close_output();
}
#[test]
fn close_output957() {
    close_output();
}
#[test]
fn close_output958() {
    close_output();
}
#[test]
fn close_output959() {
    close_output();
}
#[test]
fn close_output960() {
    close_output();
}
#[test]
fn close_output961() {
    close_output();
}
#[test]
fn close_output962() {
    close_output();
}
#[test]
fn close_output963() {
    close_output();
}
#[test]
fn close_output964() {
    close_output();
}
#[test]
fn close_output965() {
    close_output();
}
#[test]
fn close_output966() {
    close_output();
}
#[test]
fn close_output967() {
    close_output();
}
#[test]
fn close_output968() {
    close_output();
}
#[test]
fn close_output969() {
    close_output();
}
#[test]
fn close_output970() {
    close_output();
}
#[test]
fn close_output971() {
    close_output();
}
#[test]
fn close_output972() {
    close_output();
}
#[test]
fn close_output973() {
    close_output();
}
#[test]
fn close_output974() {
    close_output();
}
#[test]
fn close_output975() {
    close_output();
}
#[test]
fn close_output976() {
    close_output();
}
#[test]
fn close_output977() {
    close_output();
}
#[test]
fn close_output978() {
    close_output();
}
#[test]
fn close_output979() {
    close_output();
}
#[test]
fn close_output980() {
    close_output();
}
#[test]
fn close_output981() {
    close_output();
}
#[test]
fn close_output982() {
    close_output();
}
#[test]
fn close_output983() {
    close_output();
}
#[test]
fn close_output984() {
    close_output();
}
#[test]
fn close_output985() {
    close_output();
}
#[test]
fn close_output986() {
    close_output();
}
#[test]
fn close_output987() {
    close_output();
}
#[test]
fn close_output988() {
    close_output();
}
#[test]
fn close_output989() {
    close_output();
}
#[test]
fn close_output990() {
    close_output();
}
#[test]
fn close_output991() {
    close_output();
}
#[test]
fn close_output992() {
    close_output();
}
#[test]
fn close_output993() {
    close_output();
}
#[test]
fn close_output994() {
    close_output();
}
#[test]
fn close_output995() {
    close_output();
}
#[test]
fn close_output996() {
    close_output();
}
#[test]
fn close_output997() {
    close_output();
}
#[test]
fn close_output998() {
    close_output();
}
#[test]
fn close_output999() {
    close_output();
}

#[cargo_test]
fn close_output() {
    // What happens when stdout or stderr is closed during a build.

    // Server to know when rustc has spawned.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [lib]
                proc-macro = true

                [[bin]]
                name = "foobar"
            "#,
        )
        .file(
            "src/lib.rs",
            &r#"
                use proc_macro::TokenStream;
                use std::io::Read;

                #[proc_macro]
                pub fn repro(_input: TokenStream) -> TokenStream {
                    println!("hello stdout!");
                    eprintln!("hello stderr!");
                    // Tell the test we have started.
                    let mut socket = std::net::TcpStream::connect("__ADDR__").unwrap();
                    // Wait for the test to tell us to start printing.
                    let mut buf = [0];
                    drop(socket.read_exact(&mut buf));
                    let use_stderr = std::env::var("__CARGO_REPRO_STDERR").is_ok();
                    for i in 0..100000 {
                        if use_stderr {
                            eprintln!("0123456789{}", i);
                        } else {
                            println!("0123456789{}", i);
                        }
                    }
                    TokenStream::new()
                }
            "#
            .replace("__ADDR__", &addr.to_string()),
        )
        .file(
            "src/bin/foobar.rs",
            r#"
                foo::repro!();

                fn main() {}
            "#,
        )
        .build();

    // The `stderr` flag here indicates if this should forcefully close stderr or stdout.
    let spawn = |stderr: bool| {
        let mut cmd = p.cargo("build").build_command();
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        if stderr {
            cmd.env("__CARGO_REPRO_STDERR", "1");
        }
        let mut child = cmd.spawn().unwrap();
        // Wait for proc macro to start.
        let pm_conn = listener.accept().unwrap().0;
        // Close stderr or stdout.
        if stderr {
            drop(child.stderr.take());
        } else {
            drop(child.stdout.take());
        }
        // Tell the proc-macro to continue;
        drop(pm_conn);
        // Read the output from the other channel.
        let out: &mut dyn Read = if stderr {
            child.stdout.as_mut().unwrap()
        } else {
            child.stderr.as_mut().unwrap()
        };
        let mut result = String::new();
        out.read_to_string(&mut result).unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
        result
    };

    let stderr = spawn(false);
    assert!(
        lines_match(
            "\
[COMPILING] foo [..]
hello stderr!
[ERROR] [..]
[WARNING] build failed, waiting for other jobs to finish...
[ERROR] build failed
",
            &stderr,
        ),
        "lines differ:\n{}",
        stderr
    );

    // Try again with stderr.
    p.build_dir().rm_rf();
    let stdout = spawn(true);
    assert!(
        lines_match("hello stdout!\n", &stdout),
        "lines differ:\n{}",
        stdout
    );
}

use cargo_test_support::registry::Dependency;

#[cargo_test]
fn reduced_reproduction_8249() {
    // https://github.com/rust-lang/cargo/issues/8249
    Package::new("a-src", "0.1.0").links("a").publish();
    Package::new("a-src", "0.2.0").links("a").publish();

    Package::new("b", "0.1.0")
        .add_dep(Dependency::new("a-src", "0.1").optional(true))
        .publish();
    Package::new("b", "0.2.0")
        .add_dep(Dependency::new("a-src", "0.2").optional(true))
        .publish();

    Package::new("c", "1.0.0")
        .add_dep(&Dependency::new("b", "0.1.0"))
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                b = { version = "*", features = ["a-src"] }
                a-src = "*"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("generate-lockfile").run();
    cargo::util::paths::append(&p.root().join("Cargo.toml"), b"c = \"*\"").unwrap();
    p.cargo("check").run();
    p.cargo("check").run();
}

#[cargo_test]
fn target_directory_backup_exclusion() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/foo.rs", &main_file(r#""i am foo""#, &[]))
        .build();

    // Newly created target/ should have CACHEDIR.TAG inside...
    p.cargo("build").run();
    let cachedir_tag = p.build_dir().join("CACHEDIR.TAG");
    assert!(cachedir_tag.is_file());
    assert!(fs::read_to_string(&cachedir_tag)
        .unwrap()
        .starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
    // ...but if target/ already exists CACHEDIR.TAG should not be created in it.
    fs::remove_file(&cachedir_tag).unwrap();
    p.cargo("build").run();
    assert!(!&cachedir_tag.is_file());
}

#[cargo_test]
fn simple_terminal_width() {
    if !is_nightly() {
        // --terminal-width is unstable
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                fn main() {
                    let _: () = 42;
                }
            "#,
        )
        .build();

    p.cargo("build -Zterminal-width=20")
        .masquerade_as_nightly_cargo()
        .with_status(101)
        .with_stderr_contains("3 | ..._: () = 42;")
        .run();
}

#[cargo_test]
fn build_script_o0_default() {
    let p = project()
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    p.cargo("build -v --release")
        .with_stderr_does_not_contain("[..]build_script_build[..]opt-level[..]")
        .run();
}

#[cargo_test]
fn build_script_o0_default_even_with_release() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [profile.release]
                opt-level = 1
            "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    p.cargo("build -v --release")
        .with_stderr_does_not_contain("[..]build_script_build[..]opt-level[..]")
        .run();
}
