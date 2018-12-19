use std::fs::File;

use git2;

use crate::support::git;
use crate::support::is_nightly;
use crate::support::{basic_manifest, project};

use std::io::Write;

#[test]
fn do_not_fix_broken_builds() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() {
                    let mut x = 3;
                    drop(x);
                }

                pub fn foo2() {
                    let _x: u32 = "a";
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_status(101)
        .run();
    assert!(p.read_file("src/lib.rs").contains("let mut x = 3;"));
}

#[test]
fn fix_broken_if_requested() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                fn foo(a: &u32) -> u32 { a + 1 }
                pub fn bar() {
                    foo(1);
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs --broken-code")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
}

#[test]
fn broken_fixes_backed_out() {
    // This works as follows:
    // - Create a `rustc` shim (the "foo" project) which will pretend that the
    //   verification step fails.
    // - There is an empty build script so `foo` has `OUT_DIR` to track the steps.
    // - The first "check", `foo` creates a file in OUT_DIR, and it completes
    //   successfully with a warning diagnostic to remove unused `mut`.
    // - rustfix removes the `mut`.
    // - The second "check" to verify the changes, `foo` swaps out the content
    //   with something that fails to compile. It creates a second file so it
    //   won't do anything in the third check.
    // - cargo fix discovers that the fix failed, and it backs out the changes.
    // - The third "check" is done to display the original diagnostics of the
    //   original code.
    let p = project()
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = 'foo'
                version = '0.1.0'
                [workspace]
            "#,
        )
        .file(
            "foo/src/main.rs",
            r##"
                use std::env;
                use std::fs;
                use std::io::Write;
                use std::path::{Path, PathBuf};
                use std::process::{self, Command};

                fn main() {
                    // Ignore calls to things like --print=file-names and compiling build.rs.
                    let is_lib_rs = env::args_os()
                        .map(PathBuf::from)
                        .any(|l| l == Path::new("src/lib.rs"));
                    if is_lib_rs {
                        let path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
                        let first = path.join("first");
                        let second = path.join("second");
                        if first.exists() && !second.exists() {
                            fs::write("src/lib.rs", b"not rust code").unwrap();
                            fs::File::create(&second).unwrap();
                        } else {
                            fs::File::create(&first).unwrap();
                        }
                    }

                    let status = Command::new("rustc")
                        .args(env::args().skip(1))
                        .status()
                        .expect("failed to run rustc");
                    process::exit(status.code().unwrap_or(2));
                }
            "##,
        )
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = 'bar'
                version = '0.1.0'
                [workspace]
            "#,
        )
        .file("bar/build.rs", "fn main() {}")
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() {
                    let mut x = 3;
                    drop(x);
                }
            "#,
        )
        .build();

    // Build our rustc shim
    p.cargo("build").cwd(p.root().join("foo")).run();

    // Attempt to fix code, but our shim will always fail the second compile
    p.cargo("fix --allow-no-vcs --lib")
        .cwd(p.root().join("bar"))
        .env("__CARGO_FIX_YOLO", "1")
        .env("RUSTC", p.root().join("foo/target/debug/foo"))
        .with_stderr_contains(
            "\
             warning: failed to automatically apply fixes suggested by rustc \
             to crate `bar`\n\
             \n\
             after fixes were automatically applied the compiler reported \
             errors within these files:\n\
             \n  \
             * src/lib.rs\n\
             \n\
             This likely indicates a bug in either rustc or cargo itself,\n\
             and we would appreciate a bug report! You're likely to see \n\
             a number of compiler warnings after this message which cargo\n\
             attempted to fix but failed. If you could open an issue at\n\
             https://github.com/rust-lang/cargo/issues\n\
             quoting the full output of this command we'd be very appreciative!\n\
             \n\
             The following errors were reported:\n\
             error: expected one of `!` or `::`, found `rust`\n\
             ",
        )
        .with_stderr_contains("Original diagnostics will follow.")
        .with_stderr_contains("[WARNING] variable does not need to be mutable")
        .with_stderr_does_not_contain("[..][FIXING][..]")
        .run();

    // Make sure the fix which should have been applied was backed out
    assert!(p.read_file("bar/src/lib.rs").contains("let mut x = 3;"));
}

#[test]
fn fix_path_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = { path = 'bar' }

                [workspace]
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate bar;

                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs -p foo -p bar")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr_unordered(
            "\
[CHECKING] bar v0.1.0 ([..])
[FIXING] bar/src/lib.rs (1 fix)
[CHECKING] foo v0.1.0 ([..])
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn do_not_fix_non_relevant_deps() {
    let p = project()
        .no_manifest()
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = { path = '../bar' }

                [workspace]
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .cwd(p.root().join("foo"))
        .run();

    assert!(p.read_file("bar/src/lib.rs").contains("mut"));
}

#[test]
fn prepare_for_2018() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![allow(unused)]
                #![feature(rust_2018_preview)]

                mod foo {
                    pub const FOO: &str = "fooo";
                }

                mod bar {
                    use ::foo::FOO;
                }

                fn main() {
                    let x = ::foo::FOO;
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (2 fixes)
[FINISHED] [..]
";
    p.cargo("fix --edition --allow-no-vcs")
        .with_stderr(stderr)
        .with_stdout("")
        .run();

    println!("{}", p.read_file("src/lib.rs"));
    assert!(p.read_file("src/lib.rs").contains("use crate::foo::FOO;"));
    assert!(p
        .read_file("src/lib.rs")
        .contains("let x = crate::foo::FOO;"));
}

#[test]
fn local_paths() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![feature(rust_2018_preview)]

                use test::foo;

                mod test {
                    pub fn foo() {}
                }

                pub fn f() {
                    foo();
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
";

    p.cargo("fix --edition --allow-no-vcs")
        .with_stderr(stderr)
        .with_stdout("")
        .run();

    println!("{}", p.read_file("src/lib.rs"));
    assert!(p.read_file("src/lib.rs").contains("use crate::test::foo;"));
}

#[test]
fn upgrade_extern_crate() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = '2018'

                [workspace]

                [dependencies]
                bar = { path = 'bar' }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![warn(rust_2018_idioms)]
                extern crate bar;

                use bar::bar;

                pub fn foo() {
                    ::bar::bar();
                    bar();
                }
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    let stderr = "\
[CHECKING] bar v0.1.0 ([..])
[CHECKING] foo v0.1.0 ([..])
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
";
    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
    println!("{}", p.read_file("src/lib.rs"));
    assert!(!p.read_file("src/lib.rs").contains("extern crate"));
}

#[test]
fn specify_rustflags() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![allow(unused)]
                #![feature(rust_2018_preview)]

                mod foo {
                    pub const FOO: &str = "fooo";
                }

                fn main() {
                    let x = ::foo::FOO;
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
";
    p.cargo("fix --edition --allow-no-vcs")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
}

#[test]
fn no_changes_necessary() {
    let p = project().file("src/lib.rs", "").build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FINISHED] [..]
";
    p.cargo("fix --allow-no-vcs")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
}

#[test]
fn fixes_extra_mut() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
";
    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
}

#[test]
fn fixes_two_missing_ampersands() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    let mut y = 3;
                    x + y
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (2 fixes)
[FINISHED] [..]
";
    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
}

#[test]
fn tricky() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3; let mut y = 3;
                    x + y
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
[FIXING] src/lib.rs (2 fixes)
[FINISHED] [..]
";
    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
}

#[test]
fn preserve_line_endings() {
    let p = project()
        .file(
            "src/lib.rs",
            "\
             fn add(a: &u32) -> u32 { a + 1 }\r\n\
             pub fn foo() -> u32 { let mut x = 3; add(&x) }\r\n\
             ",
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
    assert!(p.read_file("src/lib.rs").contains("\r\n"));
}

#[test]
fn fix_deny_warnings() {
    let p = project()
        .file(
            "src/lib.rs",
            "\
                #![deny(warnings)]
                pub fn foo() { let mut x = 3; drop(x); }
            ",
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
}

#[test]
fn fix_deny_warnings_but_not_others() {
    let p = project()
        .file(
            "src/lib.rs",
            "
                #![deny(warnings)]

                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }

                fn bar() {}
            ",
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
    assert!(!p.read_file("src/lib.rs").contains("let mut x = 3;"));
    assert!(p.read_file("src/lib.rs").contains("fn bar() {}"));
}

#[test]
fn fix_two_files() {
    let p = project()
        .file(
            "src/lib.rs",
            "
                pub mod bar;

                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            ",
        )
        .file(
            "src/bar.rs",
            "
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }

            ",
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stderr_contains("[FIXING] src/bar.rs (1 fix)")
        .with_stderr_contains("[FIXING] src/lib.rs (1 fix)")
        .run();
    assert!(!p.read_file("src/lib.rs").contains("let mut x = 3;"));
    assert!(!p.read_file("src/bar.rs").contains("let mut x = 3;"));
}

#[test]
fn fixes_missing_ampersand() {
    let p = project()
        .file("src/main.rs", "fn main() { let mut x = 3; drop(x); }")
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() { let mut x = 3; drop(x); }

                #[test]
                pub fn foo2() { let mut x = 3; drop(x); }
            "#,
        )
        .file(
            "tests/a.rs",
            r#"
                #[test]
                pub fn foo() { let mut x = 3; drop(x); }
            "#,
        )
        .file("examples/foo.rs", "fn main() { let mut x = 3; drop(x); }")
        .file("build.rs", "fn main() { let mut x = 3; drop(x); }")
        .build();

    p.cargo("fix --all-targets --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr_contains("[COMPILING] foo v0.0.1 ([..])")
        .with_stderr_contains("[FIXING] build.rs (1 fix)")
        // Don't assert number of fixes for this one, as we don't know if we're
        // fixing it once or twice! We run this all concurrently, and if we
        // compile (and fix) in `--test` mode first, we get two fixes. Otherwise
        // we'll fix one non-test thing, and then fix another one later in
        // test mode.
        .with_stderr_contains("[FIXING] src/lib.rs[..]")
        .with_stderr_contains("[FIXING] src/main.rs (1 fix)")
        .with_stderr_contains("[FIXING] examples/foo.rs (1 fix)")
        .with_stderr_contains("[FIXING] tests/a.rs (1 fix)")
        .with_stderr_contains("[FINISHED] [..]")
        .run();
    p.cargo("build").run();
    p.cargo("test").run();
}

#[test]
fn fix_features() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [features]
                bar = []

                [workspace]
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "bar")]
            pub fn foo() -> u32 { let mut x = 3; x }
        "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs").run();
    p.cargo("build").run();
    p.cargo("fix --features bar --allow-no-vcs").run();
    p.cargo("build --features bar").run();
}

#[test]
fn shows_warnings() {
    let p = project()
        .file("src/lib.rs", "use std::default::Default; pub fn foo() {}")
        .build();

    p.cargo("fix --allow-no-vcs")
        .with_stderr_contains("[..]warning: unused import[..]")
        .run();
}

#[test]
fn warns_if_no_vcs_detected() {
    let p = project().file("src/lib.rs", "pub fn foo() {}").build();

    p.cargo("fix")
        .with_status(101)
        .with_stderr(
            "\
             error: no VCS found for this package and `cargo fix` can potentially perform \
             destructive changes; if you'd like to suppress this error pass `--allow-no-vcs`\
             ",
        )
        .run();
    p.cargo("fix --allow-no-vcs").run();
}

#[test]
fn warns_about_dirty_working_directory() {
    let p = project().file("src/lib.rs", "pub fn foo() {}").build();

    let repo = git2::Repository::init(&p.root()).unwrap();
    let mut cfg = t!(repo.config());
    t!(cfg.set_str("user.email", "foo@bar.com"));
    t!(cfg.set_str("user.name", "Foo Bar"));
    drop(cfg);
    git::add(&repo);
    git::commit(&repo);
    File::create(p.root().join("src/lib.rs")).unwrap();

    p.cargo("fix")
        .with_status(101)
        .with_stderr(
            "\
error: the working directory of this package has uncommitted changes, \
and `cargo fix` can potentially perform destructive changes; if you'd \
like to suppress this error pass `--allow-dirty`, `--allow-staged`, or \
commit the changes to these files:

  * src/lib.rs (dirty)


",
        )
        .run();
    p.cargo("fix --allow-dirty").run();
}

#[test]
fn warns_about_staged_working_directory() {
    let p = project().file("src/lib.rs", "pub fn foo() {}").build();

    let repo = git2::Repository::init(&p.root()).unwrap();
    let mut cfg = t!(repo.config());
    t!(cfg.set_str("user.email", "foo@bar.com"));
    t!(cfg.set_str("user.name", "Foo Bar"));
    drop(cfg);
    git::add(&repo);
    git::commit(&repo);
    File::create(&p.root().join("src/lib.rs"))
        .unwrap()
        .write_all("pub fn bar() {}".to_string().as_bytes())
        .unwrap();
    git::add(&repo);

    p.cargo("fix")
        .with_status(101)
        .with_stderr(
            "\
error: the working directory of this package has uncommitted changes, \
and `cargo fix` can potentially perform destructive changes; if you'd \
like to suppress this error pass `--allow-dirty`, `--allow-staged`, or \
commit the changes to these files:

  * src/lib.rs (staged)


",
        )
        .run();
    p.cargo("fix --allow-staged").run();
}

#[test]
fn does_not_warn_about_clean_working_directory() {
    let p = project().file("src/lib.rs", "pub fn foo() {}").build();

    let repo = git2::Repository::init(&p.root()).unwrap();
    let mut cfg = t!(repo.config());
    t!(cfg.set_str("user.email", "foo@bar.com"));
    t!(cfg.set_str("user.name", "Foo Bar"));
    drop(cfg);
    git::add(&repo);
    git::commit(&repo);

    p.cargo("fix").run();
}

#[test]
fn does_not_warn_about_dirty_ignored_files() {
    let p = project()
        .file("src/lib.rs", "pub fn foo() {}")
        .file(".gitignore", "bar\n")
        .build();

    let repo = git2::Repository::init(&p.root()).unwrap();
    let mut cfg = t!(repo.config());
    t!(cfg.set_str("user.email", "foo@bar.com"));
    t!(cfg.set_str("user.name", "Foo Bar"));
    drop(cfg);
    git::add(&repo);
    git::commit(&repo);
    File::create(p.root().join("bar")).unwrap();

    p.cargo("fix").run();
}

#[test]
fn fix_all_targets_by_default() {
    let p = project()
        .file("src/lib.rs", "pub fn foo() { let mut x = 3; drop(x); }")
        .file("tests/foo.rs", "pub fn foo() { let mut x = 3; drop(x); }")
        .build();
    p.cargo("fix --allow-no-vcs")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
    assert!(!p.read_file("src/lib.rs").contains("let mut x"));
    assert!(!p.read_file("tests/foo.rs").contains("let mut x"));
}

#[test] fn prepare_for_and_enable0() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable1() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable2() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable3() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable4() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable5() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable6() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable7() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable8() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable9() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable10() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable11() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable12() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable13() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable14() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable15() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable16() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable17() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable18() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable19() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable20() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable21() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable22() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable23() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable24() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable25() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable26() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable27() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable28() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable29() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable30() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable31() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable32() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable33() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable34() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable35() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable36() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable37() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable38() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable39() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable40() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable41() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable42() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable43() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable44() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable45() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable46() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable47() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable48() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable49() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable50() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable51() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable52() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable53() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable54() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable55() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable56() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable57() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable58() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable59() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable60() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable61() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable62() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable63() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable64() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable65() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable66() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable67() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable68() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable69() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable70() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable71() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable72() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable73() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable74() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable75() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable76() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable77() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable78() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable79() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable80() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable81() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable82() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable83() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable84() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable85() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable86() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable87() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable88() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable89() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable90() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable91() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable92() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable93() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable94() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable95() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable96() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable97() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable98() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable99() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable100() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable101() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable102() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable103() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable104() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable105() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable106() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable107() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable108() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable109() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable110() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable111() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable112() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable113() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable114() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable115() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable116() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable117() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable118() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable119() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable120() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable121() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable122() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable123() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable124() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable125() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable126() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable127() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable128() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable129() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable130() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable131() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable132() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable133() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable134() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable135() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable136() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable137() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable138() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable139() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable140() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable141() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable142() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable143() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable144() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable145() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable146() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable147() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable148() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable149() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable150() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable151() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable152() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable153() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable154() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable155() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable156() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable157() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable158() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable159() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable160() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable161() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable162() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable163() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable164() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable165() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable166() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable167() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable168() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable169() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable170() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable171() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable172() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable173() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable174() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable175() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable176() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable177() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable178() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable179() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable180() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable181() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable182() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable183() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable184() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable185() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable186() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable187() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable188() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable189() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable190() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable191() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable192() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable193() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable194() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable195() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable196() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable197() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable198() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable199() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable200() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable201() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable202() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable203() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable204() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable205() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable206() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable207() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable208() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable209() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable210() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable211() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable212() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable213() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable214() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable215() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable216() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable217() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable218() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable219() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable220() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable221() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable222() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable223() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable224() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable225() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable226() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable227() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable228() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable229() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable230() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable231() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable232() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable233() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable234() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable235() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable236() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable237() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable238() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable239() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable240() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable241() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable242() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable243() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable244() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable245() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable246() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable247() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable248() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable249() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable250() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable251() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable252() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable253() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable254() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable255() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable256() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable257() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable258() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable259() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable260() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable261() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable262() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable263() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable264() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable265() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable266() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable267() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable268() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable269() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable270() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable271() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable272() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable273() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable274() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable275() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable276() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable277() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable278() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable279() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable280() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable281() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable282() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable283() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable284() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable285() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable286() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable287() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable288() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable289() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable290() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable291() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable292() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable293() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable294() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable295() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable296() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable297() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable298() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable299() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable300() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable301() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable302() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable303() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable304() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable305() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable306() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable307() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable308() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable309() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable310() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable311() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable312() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable313() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable314() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable315() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable316() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable317() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable318() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable319() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable320() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable321() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable322() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable323() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable324() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable325() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable326() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable327() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable328() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable329() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable330() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable331() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable332() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable333() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable334() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable335() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable336() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable337() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable338() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable339() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable340() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable341() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable342() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable343() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable344() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable345() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable346() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable347() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable348() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable349() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable350() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable351() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable352() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable353() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable354() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable355() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable356() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable357() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable358() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable359() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable360() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable361() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable362() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable363() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable364() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable365() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable366() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable367() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable368() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable369() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable370() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable371() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable372() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable373() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable374() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable375() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable376() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable377() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable378() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable379() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable380() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable381() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable382() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable383() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable384() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable385() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable386() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable387() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable388() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable389() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable390() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable391() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable392() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable393() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable394() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable395() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable396() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable397() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable398() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable399() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable400() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable401() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable402() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable403() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable404() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable405() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable406() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable407() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable408() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable409() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable410() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable411() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable412() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable413() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable414() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable415() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable416() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable417() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable418() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable419() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable420() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable421() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable422() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable423() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable424() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable425() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable426() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable427() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable428() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable429() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable430() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable431() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable432() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable433() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable434() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable435() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable436() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable437() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable438() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable439() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable440() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable441() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable442() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable443() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable444() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable445() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable446() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable447() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable448() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable449() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable450() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable451() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable452() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable453() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable454() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable455() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable456() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable457() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable458() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable459() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable460() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable461() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable462() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable463() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable464() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable465() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable466() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable467() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable468() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable469() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable470() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable471() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable472() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable473() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable474() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable475() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable476() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable477() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable478() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable479() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable480() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable481() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable482() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable483() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable484() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable485() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable486() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable487() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable488() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable489() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable490() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable491() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable492() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable493() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable494() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable495() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable496() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable497() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable498() { prepare_for_and_enable(); }
#[test] fn prepare_for_and_enable499() { prepare_for_and_enable(); }

#[test]
fn prepare_for_and_enable() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = 'foo'
                version = '0.1.0'
                edition = '2018'
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    let stderr = "\
error: cannot prepare for the 2018 edition when it is enabled, so cargo cannot
automatically fix errors in `src/lib.rs`

To prepare for the 2018 edition you should first remove `edition = '2018'` from
your `Cargo.toml` and then rerun this command. Once all warnings have been fixed
then you can re-enable the `edition` key in `Cargo.toml`. For some more
information about transitioning to the 2018 edition see:

  https://[..]

";
    p.cargo("fix --edition --allow-no-vcs")
        .with_stderr_contains(stderr)
        .with_status(101)
        .run();
}

#[test]
fn fix_overlapping() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![feature(rust_2018_preview)]

                pub fn foo<T>() {}
                pub struct A;

                pub mod bar {
                    pub fn baz() {
                        ::foo::<::A>();
                    }
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo [..]
[FIXING] src/lib.rs (2 fixes)
[FINISHED] dev [..]
";

    p.cargo("fix --allow-no-vcs --prepare-for 2018 --lib")
        .with_stderr(stderr)
        .run();

    let contents = p.read_file("src/lib.rs");
    println!("{}", contents);
    assert!(contents.contains("crate::foo::<crate::A>()"));
}

#[test]
fn fix_idioms() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = 'foo'
                version = '0.1.0'
                edition = '2018'
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                use std::any::Any;
                pub fn foo() {
                    let _x: Box<Any> = Box::new(3);
                }
            "#,
        )
        .build();

    let stderr = "\
[CHECKING] foo [..]
[FIXING] src/lib.rs (1 fix)
[FINISHED] [..]
";
    p.cargo("fix --edition-idioms --allow-no-vcs")
        .with_stderr(stderr)
        .with_status(0)
        .run();

    assert!(p.read_file("src/lib.rs").contains("Box<dyn Any>"));
}

#[test]
fn idioms_2015_ok() {
    let p = project().file("src/lib.rs", "").build();

    p.cargo("fix --edition-idioms --allow-no-vcs")
        .masquerade_as_nightly_cargo()
        .with_status(0)
        .run();
}

#[test]
fn both_edition_migrate_flags() {
    let p = project().file("src/lib.rs", "").build();

    let stderr = "\
error: The argument '--edition' cannot be used with '--prepare-for <prepare-for>'

USAGE:
    cargo[..] fix --edition --message-format <FMT>

For more information try --help
";

    p.cargo("fix --prepare-for 2018 --edition")
        .with_status(1)
        .with_stderr(stderr)
        .run();
}

#[test]
fn shows_warnings_on_second_run_without_changes() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                use std::default::Default;

                pub fn foo() {
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs")
        .with_stderr_contains("[..]warning: unused import[..]")
        .run();

    p.cargo("fix --allow-no-vcs")
        .with_stderr_contains("[..]warning: unused import[..]")
        .run();
}

#[test]
fn shows_warnings_on_second_run_without_changes_on_multiple_targets() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                use std::default::Default;

                pub fn a() -> u32 { 3 }
            "#,
        )
        .file(
            "src/main.rs",
            r#"
                use std::default::Default;
                fn main() { println!("3"); }
            "#,
        )
        .file(
            "tests/foo.rs",
            r#"
                use std::default::Default;
                #[test]
                fn foo_test() {
                    println!("3");
                }
            "#,
        )
        .file(
            "tests/bar.rs",
            r#"
                use std::default::Default;

                #[test]
                fn foo_test() {
                    println!("3");
                }
            "#,
        )
        .file(
            "examples/fooxample.rs",
            r#"
                use std::default::Default;

                fn main() {
                    println!("3");
                }
            "#,
        )
        .build();

    p.cargo("fix --allow-no-vcs --all-targets")
        .with_stderr_contains(" --> examples/fooxample.rs:2:21")
        .with_stderr_contains(" --> src/lib.rs:2:21")
        .with_stderr_contains(" --> src/main.rs:2:21")
        .with_stderr_contains(" --> tests/bar.rs:2:21")
        .with_stderr_contains(" --> tests/foo.rs:2:21")
        .run();

    p.cargo("fix --allow-no-vcs --all-targets")
        .with_stderr_contains(" --> examples/fooxample.rs:2:21")
        .with_stderr_contains(" --> src/lib.rs:2:21")
        .with_stderr_contains(" --> src/main.rs:2:21")
        .with_stderr_contains(" --> tests/bar.rs:2:21")
        .with_stderr_contains(" --> tests/foo.rs:2:21")
        .run();
}

#[test]
fn doesnt_rebuild_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                bar = { path = 'bar' }

                [workspace]
            "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("fix --allow-no-vcs -p foo")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([..])
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("fix --allow-no-vcs -p foo")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr(
            "\
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn does_not_crash_with_rustc_wrapper() {
    // We don't have /usr/bin/env on Windows.
    if cfg!(windows) {
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("fix --allow-no-vcs")
        .env("RUSTC_WRAPPER", "/usr/bin/env")
        .run();
}

#[test]
fn only_warn_for_relevant_crates() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                a = { path = 'a' }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "0.1.0"
            "#,
        )
        .file(
            "a/src/lib.rs",
            "
                pub fn foo() {}
                pub mod bar {
                    use foo;
                    pub fn baz() { foo() }
                }
            ",
        )
        .build();

    p.cargo("fix --allow-no-vcs --edition")
        .with_stderr(
            "\
[CHECKING] a v0.1.0 ([..])
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn fix_to_broken_code() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = 'foo'
                version = '0.1.0'
                [workspace]
            "#,
        )
        .file(
            "foo/src/main.rs",
            r##"
                use std::env;
                use std::fs;
                use std::io::Write;
                use std::path::{Path, PathBuf};
                use std::process::{self, Command};

                fn main() {
                    let is_lib_rs = env::args_os()
                        .map(PathBuf::from)
                        .any(|l| l == Path::new("src/lib.rs"));
                    if is_lib_rs {
                        let path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
                        let path = path.join("foo");
                        if path.exists() {
                            panic!()
                        } else {
                            fs::File::create(&path).unwrap();
                        }
                    }

                    let status = Command::new("rustc")
                        .args(env::args().skip(1))
                        .status()
                        .expect("failed to run rustc");
                    process::exit(status.code().unwrap_or(2));
                }
            "##,
        )
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = 'bar'
                version = '0.1.0'
                [workspace]
            "#,
        )
        .file("bar/build.rs", "fn main() {}")
        .file("bar/src/lib.rs", "pub fn foo() { let mut x = 3; drop(x); }")
        .build();

    // Build our rustc shim
    p.cargo("build").cwd(p.root().join("foo")).run();

    // Attempt to fix code, but our shim will always fail the second compile
    p.cargo("fix --allow-no-vcs --broken-code")
        .cwd(p.root().join("bar"))
        .env("RUSTC", p.root().join("foo/target/debug/foo"))
        .with_status(101)
        .run();

    assert_eq!(
        p.read_file("bar/src/lib.rs"),
        "pub fn foo() { let x = 3; drop(x); }"
    );
}

#[test]
fn fix_with_common() {
    let p = project()
        .file("src/lib.rs", "")
        .file("tests/t1.rs", "mod common; #[test] fn t1() { common::try(); }")
        .file("tests/t2.rs", "mod common; #[test] fn t2() { common::try(); }")
        .file("tests/common/mod.rs", "pub fn try() {}")
        .build();

    p.cargo("fix --edition --allow-no-vcs").run();

    assert_eq!(p.read_file("tests/common/mod.rs"), "pub fn r#try() {}");
}
