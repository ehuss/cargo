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
        .with_stderr_contains("[ERROR] Could not compile `foo`.")
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

#[test] fn fixes_missing_ampersand0() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand1() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand2() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand3() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand4() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand5() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand6() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand7() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand8() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand9() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand10() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand11() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand12() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand13() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand14() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand15() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand16() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand17() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand18() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand19() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand20() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand21() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand22() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand23() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand24() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand25() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand26() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand27() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand28() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand29() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand30() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand31() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand32() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand33() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand34() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand35() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand36() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand37() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand38() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand39() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand40() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand41() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand42() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand43() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand44() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand45() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand46() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand47() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand48() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand49() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand50() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand51() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand52() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand53() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand54() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand55() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand56() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand57() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand58() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand59() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand60() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand61() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand62() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand63() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand64() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand65() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand66() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand67() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand68() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand69() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand70() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand71() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand72() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand73() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand74() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand75() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand76() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand77() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand78() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand79() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand80() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand81() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand82() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand83() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand84() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand85() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand86() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand87() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand88() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand89() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand90() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand91() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand92() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand93() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand94() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand95() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand96() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand97() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand98() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand99() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand100() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand101() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand102() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand103() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand104() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand105() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand106() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand107() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand108() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand109() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand110() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand111() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand112() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand113() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand114() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand115() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand116() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand117() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand118() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand119() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand120() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand121() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand122() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand123() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand124() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand125() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand126() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand127() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand128() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand129() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand130() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand131() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand132() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand133() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand134() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand135() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand136() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand137() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand138() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand139() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand140() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand141() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand142() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand143() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand144() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand145() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand146() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand147() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand148() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand149() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand150() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand151() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand152() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand153() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand154() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand155() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand156() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand157() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand158() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand159() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand160() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand161() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand162() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand163() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand164() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand165() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand166() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand167() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand168() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand169() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand170() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand171() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand172() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand173() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand174() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand175() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand176() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand177() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand178() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand179() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand180() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand181() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand182() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand183() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand184() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand185() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand186() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand187() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand188() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand189() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand190() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand191() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand192() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand193() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand194() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand195() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand196() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand197() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand198() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand199() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand200() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand201() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand202() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand203() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand204() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand205() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand206() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand207() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand208() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand209() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand210() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand211() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand212() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand213() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand214() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand215() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand216() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand217() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand218() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand219() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand220() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand221() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand222() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand223() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand224() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand225() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand226() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand227() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand228() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand229() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand230() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand231() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand232() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand233() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand234() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand235() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand236() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand237() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand238() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand239() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand240() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand241() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand242() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand243() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand244() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand245() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand246() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand247() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand248() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand249() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand250() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand251() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand252() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand253() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand254() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand255() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand256() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand257() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand258() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand259() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand260() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand261() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand262() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand263() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand264() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand265() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand266() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand267() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand268() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand269() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand270() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand271() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand272() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand273() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand274() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand275() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand276() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand277() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand278() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand279() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand280() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand281() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand282() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand283() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand284() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand285() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand286() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand287() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand288() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand289() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand290() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand291() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand292() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand293() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand294() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand295() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand296() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand297() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand298() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand299() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand300() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand301() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand302() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand303() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand304() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand305() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand306() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand307() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand308() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand309() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand310() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand311() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand312() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand313() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand314() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand315() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand316() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand317() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand318() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand319() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand320() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand321() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand322() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand323() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand324() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand325() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand326() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand327() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand328() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand329() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand330() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand331() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand332() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand333() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand334() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand335() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand336() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand337() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand338() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand339() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand340() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand341() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand342() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand343() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand344() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand345() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand346() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand347() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand348() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand349() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand350() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand351() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand352() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand353() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand354() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand355() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand356() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand357() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand358() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand359() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand360() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand361() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand362() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand363() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand364() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand365() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand366() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand367() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand368() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand369() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand370() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand371() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand372() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand373() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand374() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand375() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand376() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand377() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand378() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand379() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand380() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand381() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand382() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand383() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand384() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand385() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand386() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand387() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand388() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand389() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand390() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand391() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand392() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand393() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand394() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand395() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand396() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand397() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand398() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand399() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand400() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand401() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand402() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand403() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand404() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand405() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand406() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand407() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand408() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand409() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand410() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand411() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand412() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand413() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand414() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand415() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand416() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand417() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand418() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand419() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand420() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand421() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand422() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand423() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand424() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand425() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand426() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand427() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand428() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand429() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand430() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand431() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand432() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand433() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand434() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand435() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand436() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand437() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand438() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand439() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand440() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand441() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand442() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand443() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand444() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand445() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand446() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand447() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand448() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand449() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand450() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand451() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand452() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand453() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand454() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand455() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand456() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand457() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand458() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand459() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand460() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand461() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand462() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand463() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand464() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand465() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand466() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand467() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand468() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand469() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand470() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand471() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand472() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand473() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand474() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand475() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand476() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand477() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand478() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand479() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand480() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand481() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand482() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand483() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand484() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand485() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand486() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand487() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand488() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand489() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand490() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand491() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand492() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand493() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand494() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand495() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand496() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand497() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand498() { fixes_missing_ampersand(); }
#[test] fn fixes_missing_ampersand499() { fixes_missing_ampersand(); }

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
        .with_stderr_contains("[WARNING] failed to automatically apply fixes [..]")
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
