use std::fs::File;
use std::io::prelude::*;

use cargo;
use support::paths::CargoPathExt;
use support::registry::Package;
use support::{basic_bin_manifest, basic_lib_manifest, basic_manifest, cargo_exe, project};
use support::{is_nightly, rustc_host, sleep_ms};

#[test]
fn cargo_test_simple() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            fn hello() -> &'static str {
                "hello"
            }

            pub fn main() {
                println!("{}", hello())
            }

            #[test]
            fn test_hello() {
                assert_eq!(hello(), "hello")
            }"#,
        ).build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("hello\n").run();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]",
        ).with_stdout_contains("test test_hello ... ok")
        .run();
}

#[test]
fn cargo_test_release() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.1.0"

            [dependencies]
            bar = { path = "bar" }
        "#,
        ).file(
            "src/lib.rs",
            r#"
            extern crate bar;
            pub fn foo() { bar::bar(); }

            #[test]
            fn test() { foo(); }
        "#,
        ).file(
            "tests/test.rs",
            r#"
            extern crate foo;

            #[test]
            fn test() { foo::foo(); }
        "#,
        ).file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    p.cargo("test -v --release")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([CWD]/bar)
[RUNNING] [..] -C opt-level=3 [..]
[COMPILING] foo v0.1.0 ([CWD])
[RUNNING] [..] -C opt-level=3 [..]
[RUNNING] [..] -C opt-level=3 [..]
[RUNNING] [..] -C opt-level=3 [..]
[FINISHED] release [optimized] target(s) in [..]
[RUNNING] `[..]target/release/deps/foo-[..][EXE]`
[RUNNING] `[..]target/release/deps/test-[..][EXE]`
[DOCTEST] foo
[RUNNING] `rustdoc --test [..]lib.rs[..]`",
        ).with_stdout_contains_n("test test ... ok", 2)
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn cargo_test_overflow_checks() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.5.0"
            authors = []

            [[bin]]
            name = "foo"

            [profile.release]
            overflow-checks = true
            "#,
        ).file(
            "src/foo.rs",
            r#"
            use std::panic;
            pub fn main() {
                let r = panic::catch_unwind(|| {
                    [1, i32::max_value()].iter().sum::<i32>();
                });
                assert!(r.is_err());
            }"#,
        ).build();

    p.cargo("build --release").run();
    assert!(p.release_bin("foo").is_file());

    p.process(&p.release_bin("foo")).with_stdout("").run();
}

#[test]
fn cargo_test_verbose() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            fn main() {}
            #[test] fn test_hello() {}
        "#,
        ).build();

    p.cargo("test -v hello")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc [..] src/main.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/deps/foo-[..][EXE] hello`",
        ).with_stdout_contains("test test_hello ... ok")
        .run();
}

#[test]
fn many_similar_names() {
    let p = project()
        .file(
            "src/lib.rs",
            "
            pub fn foo() {}
            #[test] fn lib_test() {}
        ",
        ).file(
            "src/main.rs",
            "
            extern crate foo;
            fn main() {}
            #[test] fn bin_test() { foo::foo() }
        ",
        ).file(
            "tests/foo.rs",
            r#"
            extern crate foo;
            #[test] fn test_test() { foo::foo() }
        "#,
        ).build();

    p.cargo("test -v")
        .with_stdout_contains("test bin_test ... ok")
        .with_stdout_contains("test lib_test ... ok")
        .with_stdout_contains("test test_test ... ok")
        .run();
}

#[test]
fn cargo_test_failing_test_in_bin() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            fn hello() -> &'static str {
                "hello"
            }

            pub fn main() {
                println!("{}", hello())
            }

            #[test]
            fn test_hello() {
                assert_eq!(hello(), "nope")
            }"#,
        ).build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("hello\n").run();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[ERROR] test failed, to rerun pass '--bin foo'",
        ).with_stdout_contains(
            "
running 1 test
test test_hello ... FAILED

failures:

---- test_hello stdout ----
[..]thread 'test_hello' panicked at 'assertion failed:[..]",
        ).with_stdout_contains("[..]`(left == right)`[..]")
        .with_stdout_contains("[..]left: `\"hello\"`,[..]")
        .with_stdout_contains("[..]right: `\"nope\"`[..]")
        .with_stdout_contains("[..]src/main.rs:12[..]")
        .with_stdout_contains(
            "\
failures:
    test_hello
",
        ).with_status(101)
        .run();
}

#[test]
fn cargo_test_failing_test_in_test() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file("src/main.rs", r#"pub fn main() { println!("hello"); }"#)
        .file(
            "tests/footest.rs",
            "#[test] fn test_hello() { assert!(false) }",
        ).build();

    p.cargo("build").run();
    assert!(p.bin("foo").is_file());

    p.process(&p.bin("foo")).with_stdout("hello\n").run();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/footest-[..][EXE]
[ERROR] test failed, to rerun pass '--test footest'",
        ).with_stdout_contains("running 0 tests")
        .with_stdout_contains(
            "\
running 1 test
test test_hello ... FAILED

failures:

---- test_hello stdout ----
[..]thread 'test_hello' panicked at 'assertion failed: false', \
      tests/footest.rs:1[..]
",
        ).with_stdout_contains(
            "\
failures:
    test_hello
",
        ).with_status(101)
        .run();
}

#[test]
fn cargo_test_failing_test_in_lib() {
    let p = project()
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", "#[test] fn test_hello() { assert!(false) }")
        .build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[ERROR] test failed, to rerun pass '--lib'",
        ).with_stdout_contains(
            "\
test test_hello ... FAILED

failures:

---- test_hello stdout ----
[..]thread 'test_hello' panicked at 'assertion failed: false', \
      src/lib.rs:1[..]
",
        ).with_stdout_contains(
            "\
failures:
    test_hello
",
        ).with_status(101)
        .run();
}

#[test]
fn test_with_lib_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name = "baz"
            path = "src/main.rs"
        "#,
        ).file(
            "src/lib.rs",
            r#"
            ///
            /// ```rust
            /// extern crate foo;
            /// fn main() {
            ///     println!("{:?}", foo::foo());
            /// }
            /// ```
            ///
            pub fn foo(){}
            #[test] fn lib_test() {}
        "#,
        ).file(
            "src/main.rs",
            "
            #[allow(unused_extern_crates)]
            extern crate foo;

            fn main() {}

            #[test]
            fn bin_test() {}
        ",
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/baz-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test lib_test ... ok")
        .with_stdout_contains("test bin_test ... ok")
        .with_stdout_contains_n("test [..] ... ok", 3)
        .run();
}

#[test]
fn test_with_deep_lib_dep() {
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
        "#,
        ).file(
            "src/lib.rs",
            "
            #[cfg(test)]
            extern crate bar;
            /// ```
            /// foo::foo();
            /// ```
            pub fn foo() {}

            #[test]
            fn bar_test() {
                bar::bar();
            }
        ",
        ).build();
    let _p2 = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("src/lib.rs", "pub fn bar() {} #[test] fn foo_test() {}")
        .build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target[..]
[DOCTEST] foo",
        ).with_stdout_contains("test bar_test ... ok")
        .with_stdout_contains_n("test [..] ... ok", 2)
        .run();
}

#[test]
fn external_test_explicit() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[test]]
            name = "test"
            path = "src/test.rs"
        "#,
        ).file(
            "src/lib.rs",
            r#"
            pub fn get_hello() -> &'static str { "Hello" }

            #[test]
            fn internal_test() {}
        "#,
        ).file(
            "src/test.rs",
            r#"
            extern crate foo;

            #[test]
            fn external_test() { assert_eq!(foo::get_hello(), "Hello") }
        "#,
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/test-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test internal_test ... ok")
        .with_stdout_contains("test external_test ... ok")
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn external_test_named_test() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[test]]
            name = "test"
        "#,
        ).file("src/lib.rs", "")
        .file("tests/test.rs", "#[test] fn foo() {}")
        .build();

    p.cargo("test").run();
}

#[test]
fn external_test_implicit() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
            pub fn get_hello() -> &'static str { "Hello" }

            #[test]
            fn internal_test() {}
        "#,
        ).file(
            "tests/external.rs",
            r#"
            extern crate foo;

            #[test]
            fn external_test() { assert_eq!(foo::get_hello(), "Hello") }
        "#,
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/external-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test internal_test ... ok")
        .with_stdout_contains("test external_test ... ok")
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn dont_run_examples() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "examples/dont-run-me-i-will-fail.rs",
            r#"
            fn main() { panic!("Examples should not be run by 'cargo test'"); }
        "#,
        ).build();
    p.cargo("test").run();
}

#[test]
fn pass_through_command_line() {
    let p = project()
        .file(
            "src/lib.rs",
            "
            #[test] fn foo() {}
            #[test] fn bar() {}
        ",
        ).build();

    p.cargo("test bar")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test bar ... ok")
        .with_stdout_contains("running 0 tests")
        .run();

    p.cargo("test foo")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test foo ... ok")
        .with_stdout_contains("running 0 tests")
        .run();
}

// Regression test for running cargo-test twice with
// tests in an rlib
#[test]
fn cargo_test_twice() {
    let p = project()
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file(
            "src/foo.rs",
            r#"
            #![crate_type = "rlib"]

            #[test]
            fn dummy_test() { }
            "#,
        ).build();

    for _ in 0..2 {
        p.cargo("test").run();
    }
}

#[test]
fn lib_bin_same_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            name = "foo"
            [[bin]]
            name = "foo"
        "#,
        ).file("src/lib.rs", "#[test] fn lib_test() {}")
        .file(
            "src/main.rs",
            "
            #[allow(unused_extern_crates)]
            extern crate foo;

            #[test]
            fn bin_test() {}
        ",
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains_n("test [..] ... ok", 2)
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn lib_with_standard_name() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("syntax", "0.0.1"))
        .file(
            "src/lib.rs",
            "
            /// ```
            /// syntax::foo();
            /// ```
            pub fn foo() {}

            #[test]
            fn foo_test() {}
        ",
        ).file(
            "tests/test.rs",
            "
            extern crate syntax;

            #[test]
            fn test() { syntax::foo() }
        ",
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] syntax v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/syntax-[..][EXE]
[RUNNING] target/debug/deps/test-[..][EXE]
[DOCTEST] syntax",
        ).with_stdout_contains("test foo_test ... ok")
        .with_stdout_contains("test test ... ok")
        .with_stdout_contains_n("test [..] ... ok", 3)
        .run();
}

#[test]
fn lib_with_standard_name2() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            name = "syntax"
            test = false
            doctest = false
        "#,
        ).file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] syntax v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/syntax-[..][EXE]",
        ).with_stdout_contains("test test ... ok")
        .run();
}

#[test]
fn lib_without_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            test = false
            doctest = false
        "#,
        ).file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] syntax v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/syntax-[..][EXE]",
        ).with_stdout_contains("test test ... ok")
        .run();
}

#[test]
fn bin_without_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            test = false
            doctest = false

            [[bin]]
            path = "src/main.rs"
        "#,
        ).file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).build();

    p.cargo("test")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  binary target bin.name is required",
        ).run();
}

#[test]
fn bench_without_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            test = false
            doctest = false

            [[bench]]
            path = "src/bench.rs"
        "#,
        ).file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).file(
            "src/bench.rs",
            "
            #![feature(test)]
            extern crate syntax;
            extern crate test;

            #[bench]
            fn external_bench(_b: &mut test::Bencher) {}
        ",
        ).build();

    p.cargo("test")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  benchmark target bench.name is required",
        ).run();
}

#[test]
fn test_without_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            test = false
            doctest = false

            [[test]]
            path = "src/test.rs"
        "#,
        ).file(
            "src/lib.rs",
            r#"
            pub fn foo() {}
            pub fn get_hello() -> &'static str { "Hello" }
        "#,
        ).file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).file(
            "src/test.rs",
            r#"
            extern crate syntax;

            #[test]
            fn external_test() { assert_eq!(syntax::get_hello(), "Hello") }
        "#,
        ).build();

    p.cargo("test")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  test target test.name is required",
        ).run();
}

#[test]
fn example_without_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "syntax"
            version = "0.0.1"
            authors = []

            [lib]
            test = false
            doctest = false

            [[example]]
            path = "examples/example.rs"
        "#,
        ).file("src/lib.rs", "pub fn foo() {}")
        .file(
            "src/main.rs",
            "
            extern crate syntax;

            fn main() {}

            #[test]
            fn test() { syntax::foo() }
        ",
        ).file(
            "examples/example.rs",
            r#"
            extern crate syntax;

            fn main() {
                println!("example1");
            }
        "#,
        ).build();

    p.cargo("test")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  example target example.name is required",
        ).run();
}

#[test]
fn bin_there_for_integration() {
    let p = project()
        .file(
            "src/main.rs",
            "
            fn main() { std::process::exit(101); }
            #[test] fn main_test() {}
        ",
        ).file(
            "tests/foo.rs",
            r#"
            use std::process::Command;
            #[test]
            fn test_test() {
                let status = Command::new("target/debug/foo").status().unwrap();
                assert_eq!(status.code(), Some(101));
            }
        "#,
        ).build();

    p.cargo("test -v")
        .with_stdout_contains("test main_test ... ok")
        .with_stdout_contains("test test_test ... ok")
        .run();
}

#[test]
fn test_dylib() {
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
            crate_type = ["dylib"]

            [dependencies.bar]
            path = "bar"
        "#,
        ).file(
            "src/lib.rs",
            r#"
            extern crate bar as the_bar;

            pub fn bar() { the_bar::baz(); }

            #[test]
            fn foo() { bar(); }
        "#,
        ).file(
            "tests/test.rs",
            r#"
            extern crate foo as the_foo;

            #[test]
            fn foo() { the_foo::bar(); }
        "#,
        ).file(
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
        ).file("bar/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([CWD]/bar)
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/test-[..][EXE]",
        ).with_stdout_contains_n("test foo ... ok", 2)
        .run();

    p.root().move_into_the_past();
    p.cargo("test")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/test-[..][EXE]",
        ).with_stdout_contains_n("test foo ... ok", 2)
        .run();
}

#[test]
fn test_twice_with_build_cmd() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"
        "#,
        ).file("build.rs", "fn main() {}")
        .file("src/lib.rs", "#[test] fn foo() {}")
        .build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test foo ... ok")
        .with_stdout_contains("running 0 tests")
        .run();

    p.cargo("test")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test foo ... ok")
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn test_then_build() {
    let p = project().file("src/lib.rs", "#[test] fn foo() {}").build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test foo ... ok")
        .with_stdout_contains("running 0 tests")
        .run();

    p.cargo("build").with_stdout("").run();
}

#[test]
fn test_no_run() {
    let p = project()
        .file("src/lib.rs", "#[test] fn foo() { panic!() }")
        .build();

    p.cargo("test --no-run")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ).run();
}

#[test]
fn test_run_specific_bin_target() {
    let prj = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name="bin1"
            path="src/bin1.rs"

            [[bin]]
            name="bin2"
            path="src/bin2.rs"
        "#,
        ).file("src/bin1.rs", "#[test] fn test1() { }")
        .file("src/bin2.rs", "#[test] fn test2() { }")
        .build();

    prj.cargo("test --bin bin2")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/bin2-[..][EXE]",
        ).with_stdout_contains("test test2 ... ok")
        .run();
}

#[test]
fn test_run_implicit_bin_target() {
    let prj = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name="mybin"
            path="src/mybin.rs"
        "#,
        ).file(
            "src/mybin.rs",
            "#[test] fn test_in_bin() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).file("tests/mytest.rs", "#[test] fn test_in_test() { }")
        .file("benches/mybench.rs", "#[test] fn test_in_bench() { }")
        .file(
            "examples/myexm.rs",
            "#[test] fn test_in_exm() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).build();

    prj.cargo("test --bins")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/mybin-[..][EXE]",
        ).with_stdout_contains("test test_in_bin ... ok")
        .run();
}

#[test]
fn test_run_specific_test_target() {
    let prj = project()
        .file("src/bin/a.rs", "fn main() { }")
        .file("src/bin/b.rs", "#[test] fn test_b() { } fn main() { }")
        .file("tests/a.rs", "#[test] fn test_a() { }")
        .file("tests/b.rs", "#[test] fn test_b() { }")
        .build();

    prj.cargo("test --test b")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/b-[..][EXE]",
        ).with_stdout_contains("test test_b ... ok")
        .run();
}

#[test]
fn test_run_implicit_test_target() {
    let prj = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name="mybin"
            path="src/mybin.rs"
        "#,
        ).file(
            "src/mybin.rs",
            "#[test] fn test_in_bin() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).file("tests/mytest.rs", "#[test] fn test_in_test() { }")
        .file("benches/mybench.rs", "#[test] fn test_in_bench() { }")
        .file(
            "examples/myexm.rs",
            "fn main() { compile_error!(\"Don't build me!\"); }",
        ).build();

    prj.cargo("test --tests")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/mybin-[..][EXE]
[RUNNING] target/debug/deps/mytest-[..][EXE]",
        ).with_stdout_contains("test test_in_test ... ok")
        .run();
}

#[test]
fn test_run_implicit_bench_target() {
    let prj = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name="mybin"
            path="src/mybin.rs"
        "#,
        ).file(
            "src/mybin.rs",
            "#[test] fn test_in_bin() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).file("tests/mytest.rs", "#[test] fn test_in_test() { }")
        .file("benches/mybench.rs", "#[test] fn test_in_bench() { }")
        .file(
            "examples/myexm.rs",
            "fn main() { compile_error!(\"Don't build me!\"); }",
        ).build();

    prj.cargo("test --benches")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/mybin-[..][EXE]
[RUNNING] target/debug/deps/mybench-[..][EXE]",
        ).with_stdout_contains("test test_in_bench ... ok")
        .run();
}

#[test]
fn test_run_implicit_example_target() {
    let prj = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name = "mybin"
            path = "src/mybin.rs"

            [[example]]
            name = "myexm1"

            [[example]]
            name = "myexm2"
            test = true
        "#,
        ).file(
            "src/mybin.rs",
            "#[test] fn test_in_bin() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).file("tests/mytest.rs", "#[test] fn test_in_test() { }")
        .file("benches/mybench.rs", "#[test] fn test_in_bench() { }")
        .file(
            "examples/myexm1.rs",
            "#[test] fn test_in_exm() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).file(
            "examples/myexm2.rs",
            "#[test] fn test_in_exm() { }
               fn main() { panic!(\"Don't execute me!\"); }",
        ).build();

    // Compiles myexm1 as normal, but does not run it.
    prj.cargo("test -v")
        .with_stderr_contains("[RUNNING] `rustc [..]myexm1.rs [..]--crate-type bin[..]")
        .with_stderr_contains("[RUNNING] `rustc [..]myexm2.rs [..]--test[..]")
        .with_stderr_does_not_contain("[RUNNING] [..]myexm1-[..]")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm2-[..]")
        .run();

    // Only tests myexm2.
    prj.cargo("test --tests")
        .with_stderr_does_not_contain("[RUNNING] [..]myexm1-[..]")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm2-[..]")
        .run();

    // Tests all examples.
    prj.cargo("test --examples")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm1-[..]")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm2-[..]")
        .run();

    // Test an example, even without `test` set.
    prj.cargo("test --example myexm1")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm1-[..]")
        .run();

    // Tests all examples.
    prj.cargo("test --all-targets")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm1-[..]")
        .with_stderr_contains("[RUNNING] [..]target/debug/examples/myexm2-[..]")
        .run();
}

#[test]
fn test_no_harness() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name = "foo"
            test = false

            [[test]]
            name = "bar"
            path = "foo.rs"
            harness = false
        "#,
        ).file("src/main.rs", "fn main() {}")
        .file("foo.rs", "fn main() {}")
        .build();

    p.cargo("test -- --nocapture")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/bar-[..][EXE]
",
        ).run();
}

#[test]
fn selective_testing() {
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

            [lib]
                name = "foo"
                doctest = false
        "#,
        ).file("src/lib.rs", "")
        .file(
            "d1/Cargo.toml",
            r#"
            [package]
            name = "d1"
            version = "0.0.1"
            authors = []

            [lib]
                name = "d1"
                doctest = false
        "#,
        ).file("d1/src/lib.rs", "")
        .file(
            "d1/src/main.rs",
            "#[allow(unused_extern_crates)] extern crate d1; fn main() {}",
        ).file(
            "d2/Cargo.toml",
            r#"
            [package]
            name = "d2"
            version = "0.0.1"
            authors = []

            [lib]
                name = "d2"
                doctest = false
        "#,
        ).file("d2/src/lib.rs", "")
        .file(
            "d2/src/main.rs",
            "#[allow(unused_extern_crates)] extern crate d2; fn main() {}",
        );
    let p = p.build();

    println!("d1");
    p.cargo("test -p d1")
        .with_stderr(
            "\
[COMPILING] d1 v0.0.1 ([CWD]/d1)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/d1-[..][EXE]
[RUNNING] target/debug/deps/d1-[..][EXE]",
        ).with_stdout_contains_n("running 0 tests", 2)
        .run();

    println!("d2");
    p.cargo("test -p d2")
        .with_stderr(
            "\
[COMPILING] d2 v0.0.1 ([CWD]/d2)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/d2-[..][EXE]
[RUNNING] target/debug/deps/d2-[..][EXE]",
        ).with_stdout_contains_n("running 0 tests", 2)
        .run();

    println!("whole");
    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]",
        ).with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn almost_cyclic_but_not_quite() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies.b]
            path = "b"
            [dev-dependencies.c]
            path = "c"
        "#,
        ).file(
            "src/lib.rs",
            r#"
            #[cfg(test)] extern crate b;
            #[cfg(test)] extern crate c;
        "#,
        ).file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.0.1"
            authors = []

            [dependencies.foo]
            path = ".."
        "#,
        ).file(
            "b/src/lib.rs",
            r#"
            #[allow(unused_extern_crates)]
            extern crate foo;
        "#,
        ).file("c/Cargo.toml", &basic_manifest("c", "0.0.1"))
        .file("c/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("test").run();
}

#[test]
fn build_then_selective_test() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.b]
            path = "b"
        "#,
        ).file(
            "src/lib.rs",
            "#[allow(unused_extern_crates)] extern crate b;",
        ).file(
            "src/main.rs",
            r#"
            #[allow(unused_extern_crates)]
            extern crate b;
            #[allow(unused_extern_crates)]
            extern crate foo;
            fn main() {}
        "#,
        ).file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.root().move_into_the_past();
    p.cargo("test -p b").run();
}

#[test]
fn example_dev_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies.bar]
            path = "bar"
        "#,
        ).file("src/lib.rs", "")
        .file("examples/e1.rs", "extern crate bar; fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file(
            "bar/src/lib.rs",
            r#"
            // make sure this file takes awhile to compile
            macro_rules! f0( () => (1) );
            macro_rules! f1( () => ({(f0!()) + (f0!())}) );
            macro_rules! f2( () => ({(f1!()) + (f1!())}) );
            macro_rules! f3( () => ({(f2!()) + (f2!())}) );
            macro_rules! f4( () => ({(f3!()) + (f3!())}) );
            macro_rules! f5( () => ({(f4!()) + (f4!())}) );
            macro_rules! f6( () => ({(f5!()) + (f5!())}) );
            macro_rules! f7( () => ({(f6!()) + (f6!())}) );
            macro_rules! f8( () => ({(f7!()) + (f7!())}) );
            pub fn bar() {
                f8!();
            }
        "#,
        ).build();
    p.cargo("test").run();
    p.cargo("run --example e1 --release -v").run();
}

#[test]
fn selective_testing_with_docs() {
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
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```
            /// not valid rust
            /// ```
            pub fn foo() {}
        "#,
        ).file(
            "d1/Cargo.toml",
            r#"
            [package]
            name = "d1"
            version = "0.0.1"
            authors = []

            [lib]
            name = "d1"
            path = "d1.rs"
        "#,
        ).file("d1/d1.rs", "");
    let p = p.build();

    p.cargo("test -p d1")
        .with_stderr(
            "\
[COMPILING] d1 v0.0.1 ([CWD]/d1)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/d1[..][EXE]
[DOCTEST] d1",
        ).with_stdout_contains_n("running 0 tests", 2)
        .run();
}

#[test]
fn example_bin_same_name() {
    let p = project()
        .file("src/bin/foo.rs", r#"fn main() { println!("bin"); }"#)
        .file("examples/foo.rs", r#"fn main() { println!("example"); }"#)
        .build();

    p.cargo("test --no-run -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `rustc [..]`
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ).run();

    assert!(!p.bin("foo").is_file());
    assert!(p.bin("examples/foo").is_file());

    p.process(&p.bin("examples/foo"))
        .with_stdout("example\n")
        .run();

    p.cargo("run")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] [..]",
        ).with_stdout("bin")
        .run();
    assert!(p.bin("foo").is_file());
}

#[test]
fn test_with_example_twice() {
    let p = project()
        .file("src/bin/foo.rs", r#"fn main() { println!("bin"); }"#)
        .file("examples/foo.rs", r#"fn main() { println!("example"); }"#)
        .build();

    println!("first");
    p.cargo("test -v").run();
    assert!(p.bin("examples/foo").is_file());
    println!("second");
    p.cargo("test -v").run();
    assert!(p.bin("examples/foo").is_file());
}

#[test]
fn example_with_dev_dep() {
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
            test = false
            doctest = false

            [dev-dependencies.a]
            path = "a"
        "#,
        ).file("src/lib.rs", "")
        .file(
            "examples/ex.rs",
            "#[allow(unused_extern_crates)] extern crate a; fn main() {}",
        ).file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("test -v")
        .with_stderr(
            "\
[..]
[..]
[..]
[..]
[RUNNING] `rustc --crate-name ex [..] --extern a=[..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ).run();
}

#[test]
fn bin_is_preserved() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v").run();
    assert!(p.bin("foo").is_file());

    println!("testing");
    p.cargo("test -v").run();
    assert!(p.bin("foo").is_file());
}

#[test]
fn bad_example() {
    let p = project().file("src/lib.rs", "");
    let p = p.build();

    p.cargo("run --example foo")
        .with_status(101)
        .with_stderr("[ERROR] no example target named `foo`")
        .run();
    p.cargo("run --bin foo")
        .with_status(101)
        .with_stderr("[ERROR] no bin target named `foo`")
        .run();
}

#[test]
fn doctest_feature() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            [features]
            bar = []
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```rust
            /// assert_eq!(foo::foo(), 1);
            /// ```
            #[cfg(feature = "bar")]
            pub fn foo() -> i32 { 1 }
        "#,
        ).build();

    p.cargo("test --features bar")
        .with_stderr(
            "\
[COMPILING] foo [..]
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("running 0 tests")
        .with_stdout_contains("test [..] ... ok")
        .run();
}

#[test]
fn dashes_to_underscores() {
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo-bar", "0.0.1"))
        .file(
            "src/lib.rs",
            r#"
            /// ```
            /// assert_eq!(foo_bar::foo(), 1);
            /// ```
            pub fn foo() -> i32 { 1 }
        "#,
        ).build();

    p.cargo("test -v").run();
}

#[test]
fn doctest_dev_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies]
            b = { path = "b" }
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```
            /// extern crate b;
            /// ```
            pub fn foo() {}
        "#,
        ).file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("test -v").run();
}

#[test]
fn filter_no_doc_tests() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
            /// ```
            /// extern crate b;
            /// ```
            pub fn foo() {}
        "#,
        ).file("tests/foo.rs", "")
        .build();

    p.cargo("test --test=foo")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo[..][EXE]",
        ).with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn dylib_doctest() {
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
            crate-type = ["rlib", "dylib"]
            test = false
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```
            /// foo::foo();
            /// ```
            pub fn foo() {}
        "#,
        ).build();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[DOCTEST] foo",
        ).with_stdout_contains("test [..] ... ok")
        .run();
}

#[test]
fn dylib_doctest2() {
    // can't doctest dylibs as they're statically linked together
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
            crate-type = ["dylib"]
            test = false
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```
            /// foo::foo();
            /// ```
            pub fn foo() {}
        "#,
        ).build();

    p.cargo("test").with_stdout("").run();
}

#[test]
fn cyclic_dev_dep_doc_test() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies]
            bar = { path = "bar" }
        "#,
        ).file(
            "src/lib.rs",
            r#"
            //! ```
            //! extern crate bar;
            //! ```
        "#,
        ).file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []

            [dependencies]
            foo = { path = ".." }
        "#,
        ).file(
            "bar/src/lib.rs",
            r#"
            #[allow(unused_extern_crates)]
            extern crate foo;
        "#,
        ).build();
    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[COMPILING] bar v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("running 0 tests")
        .with_stdout_contains("test [..] ... ok")
        .run();
}

#[test]
fn dev_dep_with_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies]
            bar = { path = "bar" }
        "#,
        ).file("src/lib.rs", "")
        .file("examples/foo.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []
            build = "build.rs"
        "#,
        ).file("bar/src/lib.rs", "")
        .file("bar/build.rs", "fn main() {}")
        .build();
    p.cargo("test").run();
}

#[test]
fn no_fail_fast() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
        pub fn add_one(x: i32) -> i32{
            x + 1
        }

        /// ```rust
        /// use foo::sub_one;
        /// assert_eq!(sub_one(101), 100);
        /// ```
        pub fn sub_one(x: i32) -> i32{
            x - 1
        }
        "#,
        ).file(
            "tests/test_add_one.rs",
            r#"
        extern crate foo;
        use foo::*;

        #[test]
        fn add_one_test() {
            assert_eq!(add_one(1), 2);
        }

        #[test]
        fn fail_add_one_test() {
            assert_eq!(add_one(1), 1);
        }
        "#,
        ).file(
            "tests/test_sub_one.rs",
            r#"
        extern crate foo;
        use foo::*;

        #[test]
        fn sub_one_test() {
            assert_eq!(sub_one(1), 0);
        }
        "#,
        ).build();
    p.cargo("test --no-fail-fast")
        .with_status(101)
        .with_stderr_contains(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..][EXE]
[RUNNING] target/debug/deps/test_add_one-[..][EXE]",
        ).with_stdout_contains("running 0 tests")
        .with_stderr_contains(
            "\
[RUNNING] target/debug/deps/test_sub_one-[..][EXE]
[DOCTEST] foo",
        ).with_stdout_contains("test result: FAILED. [..]")
        .with_stdout_contains("test sub_one_test ... ok")
        .with_stdout_contains_n("test [..] ... ok", 3)
        .run();
}

#[test]
fn test_multiple_packages() {
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

            [lib]
                name = "foo"
                doctest = false
        "#,
        ).file("src/lib.rs", "")
        .file(
            "d1/Cargo.toml",
            r#"
            [package]
            name = "d1"
            version = "0.0.1"
            authors = []

            [lib]
                name = "d1"
                doctest = false
        "#,
        ).file("d1/src/lib.rs", "")
        .file(
            "d2/Cargo.toml",
            r#"
            [package]
            name = "d2"
            version = "0.0.1"
            authors = []

            [lib]
                name = "d2"
                doctest = false
        "#,
        ).file("d2/src/lib.rs", "");
    let p = p.build();

    p.cargo("test -p d1 -p d2")
        .with_stderr_contains("[RUNNING] target/debug/deps/d1-[..][EXE]")
        .with_stderr_contains("[RUNNING] target/debug/deps/d2-[..][EXE]")
        .with_stdout_contains_n("running 0 tests", 2)
        .run();
}

#[test]
fn bin_does_not_rebuild_tests() {
    let p = project()
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .file("tests/foo.rs", "");
    let p = p.build();

    p.cargo("test -v").run();

    sleep_ms(1000);
    File::create(&p.root().join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() { 3; }")
        .unwrap();

    p.cargo("test -v --no-run")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[RUNNING] `rustc [..] src/main.rs [..]`
[RUNNING] `rustc [..] src/main.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ).run();
}

#[test]
fn selective_test_wonky_profile() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [profile.release]
            opt-level = 2

            [dependencies]
            a = { path = "a" }
        "#,
        ).file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "");
    let p = p.build();

    p.cargo("test -v --no-run --release -p foo -p a").run();
}

#[test]
fn selective_test_optional_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            a = { path = "a", optional = true }
        "#,
        ).file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "");
    let p = p.build();

    p.cargo("test -v --no-run --features a -p a")
        .with_stderr(
            "\
[COMPILING] a v0.0.1 ([..])
[RUNNING] `rustc [..] a/src/lib.rs [..]`
[RUNNING] `rustc [..] a/src/lib.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ).run();
}

#[test]
fn only_test_docs() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
            #[test]
            fn foo() {
                let a: u32 = "hello";
            }

            /// ```
            /// foo::bar();
            /// println!("ok");
            /// ```
            pub fn bar() {
            }
        "#,
        ).file("tests/foo.rs", "this is not rust");
    let p = p.build();

    p.cargo("test --doc")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[DOCTEST] foo",
        ).with_stdout_contains("test [..] ... ok")
        .run();
}

#[test]
fn test_panic_abort_with_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            bar = { path = "bar" }

            [profile.dev]
            panic = 'abort'
        "#,
        ).file(
            "src/lib.rs",
            r#"
            extern crate bar;

            #[test]
            fn foo() {}
        "#,
        ).file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();
    p.cargo("test -v").run();
}

#[test]
fn cfg_test_even_with_no_harness() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            harness = false
            doctest = false
        "#,
        ).file(
            "src/lib.rs",
            r#"#[cfg(test)] fn main() { println!("hello!"); }"#,
        ).build();
    p.cargo("test -v")
        .with_stdout("hello!\n")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]`
",
        ).run();
}

#[test]
fn panic_abort_multiple() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            a = { path = "a" }

            [profile.release]
            panic = 'abort'
        "#,
        ).file(
            "src/lib.rs",
            "#[allow(unused_extern_crates)] extern crate a;",
        ).file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "")
        .build();
    p.cargo("test --release -v -p foo -p a").run();
}

#[test]
fn pass_correct_cfgs_flags_to_rustdoc() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []

            [features]
            default = ["feature_a/default"]
            nightly = ["feature_a/nightly"]

            [dependencies.feature_a]
            path = "libs/feature_a"
            default-features = false
        "#,
        ).file(
            "src/lib.rs",
            r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn it_works() {
                  assert!(true);
                }
            }
        "#,
        ).file(
            "libs/feature_a/Cargo.toml",
            r#"
            [package]
            name = "feature_a"
            version = "0.1.0"
            authors = []

            [features]
            default = ["mock_serde_codegen"]
            nightly = ["mock_serde_derive"]

            [dependencies]
            mock_serde_derive = { path = "../mock_serde_derive", optional = true }

            [build-dependencies]
            mock_serde_codegen = { path = "../mock_serde_codegen", optional = true }
        "#,
        ).file(
            "libs/feature_a/src/lib.rs",
            r#"
            #[cfg(feature = "mock_serde_derive")]
            const MSG: &'static str = "This is safe";

            #[cfg(feature = "mock_serde_codegen")]
            const MSG: &'static str = "This is risky";

            pub fn get() -> &'static str {
                MSG
            }
        "#,
        ).file(
            "libs/mock_serde_derive/Cargo.toml",
            &basic_manifest("mock_serde_derive", "0.1.0"),
        ).file("libs/mock_serde_derive/src/lib.rs", "")
        .file(
            "libs/mock_serde_codegen/Cargo.toml",
            &basic_manifest("mock_serde_codegen", "0.1.0"),
        ).file("libs/mock_serde_codegen/src/lib.rs", "");
    let p = p.build();

    p.cargo("test --package feature_a --verbose")
        .with_stderr_contains(
            "\
[DOCTEST] feature_a
[RUNNING] `rustdoc --test [..]mock_serde_codegen[..]`",
        ).run();

    p.cargo("test --verbose")
        .with_stderr_contains(
            "\
[DOCTEST] foo
[RUNNING] `rustdoc --test [..]feature_a[..]`",
        ).run();
}

#[test]
fn test_release_ignore_panic() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            a = { path = "a" }

            [profile.test]
            panic = 'abort'
            [profile.release]
            panic = 'abort'
        "#,
        ).file(
            "src/lib.rs",
            "#[allow(unused_extern_crates)] extern crate a;",
        ).file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "");
    let p = p.build();
    println!("test");
    p.cargo("test -v").run();
    println!("bench");
    p.cargo("bench -v").run();
}

#[test]
fn test_many_with_features() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            a = { path = "a" }

            [features]
            foo = []

            [workspace]
        "#,
        ).file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("test -v -p a -p foo --features foo").run();
}

#[test]
fn test_all_workspace() {
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
        ).file("src/main.rs", "#[test] fn foo_test() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "#[test] fn bar_test() {}")
        .build();

    p.cargo("test --all")
        .with_stdout_contains("test foo_test ... ok")
        .with_stdout_contains("test bar_test ... ok")
        .run();
}

#[test]
fn test_all_exclude() {
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
        ).file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "#[test] pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "#[test] pub fn baz() { assert!(false); }")
        .build();

    p.cargo("test --all --exclude baz")
        .with_stdout_contains(
            "running 1 test
test bar ... ok",
        ).run();
}

#[test]
fn test_all_virtual_manifest() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
        "#,
        ).file("a/Cargo.toml", &basic_manifest("a", "0.1.0"))
        .file("a/src/lib.rs", "#[test] fn a() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "0.1.0"))
        .file("b/src/lib.rs", "#[test] fn b() {}")
        .build();

    p.cargo("test --all")
        .with_stdout_contains("test a ... ok")
        .with_stdout_contains("test b ... ok")
        .run();
}

#[test]
fn test_virtual_manifest_all_implied() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
        "#,
        ).file("a/Cargo.toml", &basic_manifest("a", "0.1.0"))
        .file("a/src/lib.rs", "#[test] fn a() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "0.1.0"))
        .file("b/src/lib.rs", "#[test] fn b() {}")
        .build();

    p.cargo("test")
        .with_stdout_contains("test a ... ok")
        .with_stdout_contains("test b ... ok")
        .run();
}

#[test]
fn test_all_member_dependency_same_name() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a"]
        "#,
        ).file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.1.0"

            [dependencies]
            a = "0.1.0"
        "#,
        ).file("a/src/lib.rs", "#[test] fn a() {}")
        .build();

    Package::new("a", "0.1.0").publish();

    p.cargo("test --all")
        .with_stdout_contains("test a ... ok")
        .run();
}

#[test]
fn doctest_only_with_dev_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.1.0"

            [dev-dependencies]
            b = { path = "b" }
        "#,
        ).file(
            "src/lib.rs",
            r#"
            /// ```
            /// extern crate b;
            ///
            /// b::b();
            /// ```
            pub fn a() {}
        "#,
        ).file("b/Cargo.toml", &basic_manifest("b", "0.1.0"))
        .file("b/src/lib.rs", "pub fn b() {}")
        .build();

    p.cargo("test --doc -v").run();
}

#[test]
fn test_many_targets() {
    let p = project()
        .file(
            "src/bin/a.rs",
            r#"
            fn main() {}
            #[test] fn bin_a() {}
        "#,
        ).file(
            "src/bin/b.rs",
            r#"
            fn main() {}
            #[test] fn bin_b() {}
        "#,
        ).file(
            "src/bin/c.rs",
            r#"
            fn main() {}
            #[test] fn bin_c() { panic!(); }
        "#,
        ).file(
            "examples/a.rs",
            r#"
            fn main() {}
            #[test] fn example_a() {}
        "#,
        ).file(
            "examples/b.rs",
            r#"
            fn main() {}
            #[test] fn example_b() {}
        "#,
        ).file("examples/c.rs", "#[test] fn example_c() { panic!(); }")
        .file("tests/a.rs", "#[test] fn test_a() {}")
        .file("tests/b.rs", "#[test] fn test_b() {}")
        .file("tests/c.rs", "does not compile")
        .build();

    p.cargo("test --verbose --bin a --bin b --example a --example b --test a --test b")
        .with_stdout_contains("test bin_a ... ok")
        .with_stdout_contains("test bin_b ... ok")
        .with_stdout_contains("test test_a ... ok")
        .with_stdout_contains("test test_b ... ok")
        .with_stderr_contains("[RUNNING] `rustc --crate-name a examples/a.rs [..]`")
        .with_stderr_contains("[RUNNING] `rustc --crate-name b examples/b.rs [..]`")
        .run();
}

#[test]
fn doctest_and_registry() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.1.0"

            [dependencies]
            b = { path = "b" }
            c = { path = "c" }

            [workspace]
        "#,
        ).file("src/lib.rs", "")
        .file("b/Cargo.toml", &basic_manifest("b", "0.1.0"))
        .file(
            "b/src/lib.rs",
            "
            /// ```
            /// b::foo();
            /// ```
            pub fn foo() {}
        ",
        ).file(
            "c/Cargo.toml",
            r#"
            [project]
            name = "c"
            version = "0.1.0"

            [dependencies]
            b = "0.1"
        "#,
        ).file("c/src/lib.rs", "")
        .build();

    Package::new("b", "0.1.0").publish();

    p.cargo("test --all -v").run();
}

#[test]
fn cargo_test_env() {
    let src = format!(
        r#"
        #![crate_type = "rlib"]

        #[test]
        fn env_test() {{
            use std::env;
            println!("{{}}", env::var("{}").unwrap());
        }}
        "#,
        cargo::CARGO_ENV
    );

    let p = project()
        .file("Cargo.toml", &basic_lib_manifest("foo"))
        .file("src/lib.rs", &src)
        .build();

    let cargo = cargo_exe().canonicalize().unwrap();
    p.cargo("test --lib -- --nocapture")
        .with_stdout_contains(format!(
            "\
{}
test env_test ... ok
",
            cargo.to_str().unwrap()
        )).run();
}

#[test]
fn test_order() {
    let p = project()
        .file("src/lib.rs", "#[test] fn test_lib() {}")
        .file("tests/a.rs", "#[test] fn test_a() {}")
        .file("tests/z.rs", "#[test] fn test_z() {}")
        .build();

    p.cargo("test --all")
        .with_stdout_contains(
            "
running 1 test
test test_lib ... ok

test result: ok. [..]


running 1 test
test test_a ... ok

test result: ok. [..]


running 1 test
test test_z ... ok

test result: ok. [..]
",
        ).run();
}

#[test]
fn cyclic_dev() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"

            [dev-dependencies]
            foo = { path = "." }
        "#,
        ).file("src/lib.rs", "#[test] fn test_lib() {}")
        .file("tests/foo.rs", "extern crate foo;")
        .build();

    p.cargo("test --all").run();
}

#[test]
fn publish_a_crate_without_tests() {
    Package::new("testless", "0.1.0")
        .file("Cargo.toml", r#"
            [project]
            name = "testless"
            version = "0.1.0"
            exclude = ["tests/*"]

            [[test]]
            name = "a_test"
        "#)
        .file("src/lib.rs", "")

        // In real life, the package will have a test,
        // which would be excluded from .crate file by the
        // `exclude` field. Our test harness does not honor
        // exclude though, so let's just not add the file!
        // .file("tests/a_test.rs", "")

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
        ).file("src/lib.rs", "")
        .build();

    p.cargo("test").run();
    p.cargo("test --package testless").run();
}

#[test]
fn find_dependency_of_proc_macro_dependency_with_target() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["root", "proc_macro_dep"]
        "#,
        ).file(
            "root/Cargo.toml",
            r#"
            [project]
            name = "root"
            version = "0.1.0"
            authors = []

            [dependencies]
            proc_macro_dep = { path = "../proc_macro_dep" }
        "#,
        ).file(
            "root/src/lib.rs",
            r#"
            #[macro_use]
            extern crate proc_macro_dep;

            #[derive(Noop)]
            pub struct X;
        "#,
        ).file(
            "proc_macro_dep/Cargo.toml",
            r#"
            [project]
            name = "proc_macro_dep"
            version = "0.1.0"
            authors = []

            [lib]
            proc-macro = true

            [dependencies]
            baz = "^0.1"
        "#,
        ).file(
            "proc_macro_dep/src/lib.rs",
            r#"
            extern crate baz;
            extern crate proc_macro;
            use proc_macro::TokenStream;

            #[proc_macro_derive(Noop)]
            pub fn noop(_input: TokenStream) -> TokenStream {
                "".parse().unwrap()
            }
        "#,
        ).build();
    Package::new("bar", "0.1.0").publish();
    Package::new("baz", "0.1.0")
        .dep("bar", "0.1")
        .file("src/lib.rs", "extern crate bar;")
        .publish();
    p.cargo("test --all --target").arg(rustc_host()).run();
}

#[test]
fn test_hint_not_masked_by_doctest() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
            /// ```
            /// assert_eq!(1, 1);
            /// ```
            pub fn this_works() {}
        "#,
        ).file(
            "tests/integ.rs",
            r#"
            #[test]
            fn this_fails() {
                panic!();
            }
        "#,
        ).build();
    p.cargo("test --no-fail-fast")
        .with_status(101)
        .with_stdout_contains("test this_fails ... FAILED")
        .with_stdout_contains("[..]this_works (line [..]ok")
        .with_stderr_contains(
            "[ERROR] test failed, to rerun pass \
             '--test integ'",
        ).run();
}

#[test]
fn test_hint_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
        "#,
        ).file("a/Cargo.toml", &basic_manifest("a", "0.1.0"))
        .file("a/src/lib.rs", "#[test] fn t1() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "0.1.0"))
        .file("b/src/lib.rs", "#[test] fn t1() {assert!(false)}")
        .build();

    p.cargo("test")
        .with_stderr_contains("[ERROR] test failed, to rerun pass '-p b --lib'")
        .with_status(101)
        .run();
}

#[test] fn json_artifact_includes_test_flag1() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag2() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag3() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag4() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag5() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag6() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag7() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag8() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag9() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag10() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag11() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag12() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag13() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag14() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag15() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag16() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag17() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag18() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag19() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag20() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag21() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag22() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag23() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag24() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag25() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag26() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag27() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag28() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag29() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag30() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag31() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag32() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag33() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag34() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag35() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag36() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag37() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag38() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag39() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag40() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag41() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag42() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag43() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag44() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag45() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag46() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag47() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag48() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag49() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag50() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag51() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag52() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag53() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag54() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag55() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag56() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag57() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag58() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag59() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag60() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag61() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag62() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag63() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag64() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag65() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag66() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag67() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag68() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag69() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag70() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag71() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag72() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag73() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag74() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag75() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag76() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag77() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag78() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag79() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag80() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag81() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag82() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag83() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag84() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag85() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag86() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag87() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag88() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag89() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag90() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag91() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag92() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag93() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag94() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag95() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag96() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag97() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag98() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag99() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag100() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag101() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag102() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag103() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag104() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag105() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag106() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag107() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag108() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag109() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag110() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag111() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag112() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag113() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag114() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag115() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag116() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag117() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag118() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag119() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag120() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag121() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag122() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag123() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag124() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag125() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag126() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag127() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag128() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag129() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag130() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag131() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag132() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag133() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag134() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag135() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag136() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag137() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag138() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag139() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag140() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag141() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag142() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag143() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag144() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag145() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag146() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag147() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag148() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag149() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag150() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag151() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag152() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag153() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag154() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag155() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag156() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag157() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag158() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag159() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag160() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag161() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag162() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag163() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag164() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag165() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag166() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag167() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag168() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag169() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag170() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag171() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag172() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag173() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag174() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag175() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag176() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag177() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag178() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag179() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag180() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag181() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag182() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag183() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag184() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag185() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag186() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag187() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag188() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag189() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag190() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag191() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag192() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag193() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag194() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag195() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag196() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag197() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag198() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag199() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag200() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag201() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag202() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag203() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag204() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag205() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag206() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag207() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag208() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag209() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag210() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag211() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag212() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag213() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag214() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag215() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag216() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag217() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag218() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag219() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag220() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag221() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag222() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag223() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag224() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag225() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag226() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag227() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag228() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag229() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag230() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag231() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag232() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag233() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag234() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag235() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag236() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag237() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag238() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag239() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag240() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag241() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag242() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag243() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag244() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag245() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag246() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag247() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag248() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag249() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag250() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag251() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag252() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag253() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag254() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag255() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag256() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag257() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag258() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag259() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag260() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag261() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag262() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag263() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag264() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag265() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag266() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag267() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag268() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag269() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag270() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag271() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag272() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag273() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag274() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag275() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag276() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag277() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag278() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag279() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag280() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag281() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag282() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag283() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag284() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag285() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag286() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag287() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag288() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag289() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag290() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag291() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag292() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag293() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag294() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag295() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag296() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag297() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag298() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag299() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag300() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag301() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag302() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag303() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag304() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag305() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag306() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag307() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag308() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag309() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag310() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag311() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag312() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag313() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag314() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag315() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag316() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag317() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag318() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag319() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag320() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag321() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag322() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag323() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag324() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag325() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag326() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag327() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag328() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag329() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag330() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag331() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag332() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag333() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag334() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag335() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag336() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag337() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag338() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag339() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag340() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag341() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag342() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag343() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag344() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag345() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag346() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag347() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag348() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag349() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag350() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag351() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag352() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag353() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag354() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag355() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag356() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag357() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag358() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag359() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag360() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag361() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag362() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag363() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag364() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag365() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag366() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag367() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag368() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag369() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag370() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag371() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag372() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag373() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag374() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag375() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag376() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag377() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag378() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag379() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag380() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag381() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag382() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag383() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag384() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag385() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag386() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag387() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag388() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag389() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag390() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag391() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag392() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag393() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag394() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag395() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag396() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag397() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag398() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag399() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag400() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag401() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag402() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag403() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag404() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag405() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag406() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag407() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag408() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag409() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag410() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag411() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag412() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag413() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag414() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag415() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag416() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag417() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag418() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag419() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag420() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag421() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag422() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag423() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag424() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag425() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag426() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag427() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag428() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag429() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag430() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag431() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag432() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag433() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag434() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag435() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag436() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag437() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag438() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag439() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag440() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag441() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag442() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag443() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag444() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag445() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag446() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag447() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag448() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag449() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag450() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag451() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag452() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag453() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag454() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag455() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag456() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag457() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag458() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag459() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag460() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag461() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag462() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag463() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag464() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag465() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag466() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag467() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag468() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag469() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag470() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag471() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag472() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag473() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag474() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag475() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag476() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag477() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag478() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag479() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag480() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag481() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag482() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag483() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag484() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag485() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag486() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag487() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag488() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag489() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag490() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag491() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag492() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag493() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag494() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag495() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag496() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag497() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag498() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag499() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag500() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag501() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag502() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag503() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag504() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag505() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag506() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag507() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag508() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag509() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag510() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag511() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag512() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag513() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag514() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag515() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag516() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag517() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag518() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag519() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag520() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag521() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag522() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag523() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag524() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag525() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag526() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag527() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag528() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag529() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag530() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag531() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag532() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag533() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag534() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag535() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag536() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag537() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag538() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag539() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag540() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag541() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag542() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag543() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag544() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag545() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag546() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag547() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag548() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag549() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag550() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag551() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag552() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag553() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag554() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag555() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag556() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag557() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag558() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag559() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag560() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag561() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag562() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag563() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag564() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag565() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag566() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag567() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag568() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag569() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag570() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag571() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag572() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag573() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag574() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag575() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag576() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag577() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag578() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag579() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag580() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag581() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag582() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag583() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag584() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag585() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag586() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag587() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag588() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag589() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag590() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag591() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag592() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag593() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag594() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag595() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag596() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag597() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag598() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag599() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag600() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag601() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag602() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag603() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag604() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag605() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag606() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag607() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag608() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag609() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag610() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag611() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag612() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag613() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag614() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag615() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag616() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag617() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag618() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag619() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag620() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag621() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag622() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag623() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag624() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag625() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag626() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag627() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag628() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag629() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag630() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag631() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag632() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag633() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag634() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag635() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag636() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag637() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag638() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag639() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag640() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag641() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag642() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag643() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag644() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag645() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag646() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag647() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag648() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag649() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag650() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag651() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag652() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag653() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag654() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag655() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag656() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag657() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag658() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag659() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag660() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag661() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag662() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag663() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag664() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag665() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag666() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag667() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag668() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag669() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag670() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag671() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag672() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag673() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag674() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag675() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag676() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag677() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag678() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag679() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag680() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag681() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag682() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag683() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag684() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag685() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag686() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag687() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag688() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag689() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag690() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag691() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag692() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag693() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag694() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag695() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag696() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag697() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag698() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag699() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag700() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag701() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag702() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag703() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag704() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag705() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag706() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag707() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag708() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag709() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag710() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag711() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag712() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag713() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag714() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag715() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag716() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag717() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag718() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag719() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag720() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag721() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag722() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag723() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag724() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag725() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag726() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag727() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag728() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag729() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag730() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag731() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag732() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag733() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag734() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag735() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag736() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag737() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag738() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag739() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag740() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag741() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag742() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag743() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag744() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag745() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag746() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag747() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag748() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag749() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag750() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag751() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag752() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag753() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag754() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag755() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag756() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag757() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag758() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag759() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag760() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag761() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag762() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag763() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag764() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag765() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag766() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag767() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag768() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag769() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag770() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag771() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag772() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag773() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag774() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag775() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag776() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag777() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag778() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag779() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag780() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag781() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag782() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag783() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag784() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag785() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag786() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag787() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag788() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag789() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag790() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag791() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag792() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag793() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag794() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag795() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag796() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag797() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag798() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag799() { json_artifact_includes_test_flag(); }
#[test] fn json_artifact_includes_test_flag800() { json_artifact_includes_test_flag(); }


#[test]
fn json_artifact_includes_test_flag() {
    // Verify that the JSON artifact output includes `test` flag.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [profile.test]
            opt-level = 1
        "#,
        ).file("src/lib.rs", "")
        .build();

    p.cargo("test -v --message-format=json")
        .with_json(
            r#"
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
        "package_id":"foo 0.0.1 ([..])",
        "target":{
            "kind":["lib"],
            "crate_types":["lib"],
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]lib.rs"
        },
        "filenames":["[..].rlib"],
        "fresh": false
    }

    {
        "reason":"compiler-artifact",
        "profile": {
            "debug_assertions": true,
            "debuginfo": 2,
            "opt_level": "1",
            "overflow_checks": true,
            "test": true
        },
        "executable": "[..]/foo-[..]",
        "features": [],
        "package_id":"foo 0.0.1 ([..])",
        "target":{
            "kind":["lib"],
            "crate_types":["lib"],
            "edition": "2015",
            "name":"foo",
            "src_path":"[..]lib.rs"
        },
        "filenames":["[..]/foo-[..]"],
        "fresh": false
    }
"#,
        ).run();
}

#[test]
fn json_artifact_includes_executable_for_library_tests() {
    let p = project()
        .file("src/main.rs", "fn main() { }")
        .file("src/lib.rs", r#"#[test] fn lib_test() {}"#)
        .build();

    p.cargo("test --lib -v --no-run --message-format=json")
        .with_json(r#"
            {
                "executable": "[..]/foo/target/debug/foo-[..][EXE]",
                "features": [],
                "filenames": "{...}",
                "fresh": false,
                "package_id": "foo 0.0.1 ([..])",
                "profile": "{...}",
                "reason": "compiler-artifact",
                "target": {
                    "crate_types": [ "lib" ],
                    "kind": [ "lib" ],
                    "edition": "2015",
                    "name": "foo",
                    "src_path": "[..]/foo/src/lib.rs"
                }
            }
        "#)
        .run();
}

#[test]
fn json_artifact_includes_executable_for_integration_tests() {
    let p = project()
        .file("tests/integration_test.rs", r#"#[test] fn integration_test() {}"#)
        .build();

    p.cargo("test -v --no-run --message-format=json --test integration_test")
        .with_json(r#"
            {
                "executable": "[..]/foo/target/debug/integration_test-[..][EXE]",
                "features": [],
                "filenames": "{...}",
                "fresh": false,
                "package_id": "foo 0.0.1 ([..])",
                "profile": "{...}",
                "reason": "compiler-artifact",
                "target": {
                    "crate_types": [ "bin" ],
                    "kind": [ "test" ],
                    "edition": "2015",
                    "name": "integration_test",
                    "src_path": "[..]/foo/tests/integration_test.rs"
                }
            }
        "#)
        .run();
}

#[test]
fn test_build_script_links() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                links = 'something'

                [lib]
                test = false
            "#,
        ).file("build.rs", "fn main() {}")
        .file("src/lib.rs", "")
        .build();

    p.cargo("test --no-run").run();
}

#[test]
fn doctest_skip_staticlib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [lib]
                crate-type = ["staticlib"]
            "#,
        ).file(
            "src/lib.rs",
            r#"
            //! ```
            //! assert_eq!(1,2);
            //! ```
            "#,
        ).build();

    p.cargo("test --doc")
        .with_status(101)
        .with_stderr(
            "\
[WARNING] doc tests are not supported for crate type(s) `staticlib` in package `foo`
[ERROR] no library targets found in package `foo`",
        ).run();

    p.cargo("test")
        .with_stderr(
            "\
[COMPILING] foo [..]
[FINISHED] dev [..]
[RUNNING] target/debug/deps/foo-[..]",
        ).run();
}

#[test]
fn can_not_mix_doc_tests_and_regular_tests() {
    let p = project()
        .file("src/lib.rs", "\
/// ```
/// assert_eq!(1, 1)
/// ```
pub fn foo() -> u8 { 1 }

#[cfg(test)] mod tests {
    #[test] fn it_works() { assert_eq!(2 + 2, 4); }
}
")
        .build();

    p.cargo("test")
        .with_stderr("\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..]
[DOCTEST] foo
")
        .with_stdout("
running 1 test
test tests::it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


running 1 test
test src/lib.rs - foo (line 1) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
\n")
        .run();

    p.cargo("test --lib")
        .with_stderr("\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target/debug/deps/foo-[..]\n")
        .with_stdout("
running 1 test
test tests::it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
\n")
        .run();

    p.cargo("test --doc")
        .with_stderr("\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[DOCTEST] foo
")
        .with_stdout("
running 1 test
test src/lib.rs - foo (line 1) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

").run();

    p.cargo("test --lib --doc")
        .with_status(101)
        .with_stderr("[ERROR] Can't mix --doc with other target selecting options\n")
        .run();
}

#[test]
fn test_all_targets_lib() {
    let p = project().file("src/lib.rs", "").build();

    p.cargo("test --all-targets")
        .with_stderr(
            "\
[COMPILING] foo [..]
[FINISHED] dev [..]
[RUNNING] [..]foo[..]
",
        ).run();
}


#[test]
fn test_dep_with_dev() {
    Package::new("devdep", "0.1.0").publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            bar = { path = "bar" }
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"

            [dev-dependencies]
            devdep = "0.1"
        "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("test -p bar")
        .with_status(101)
        .with_stderr(
            "[ERROR] package `bar` cannot be tested because it requires dev-dependencies \
             and is not a member of the workspace",
        )
        .run();
}
