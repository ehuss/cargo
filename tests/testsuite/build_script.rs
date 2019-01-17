use std::env;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

use crate::support::paths::CargoPathExt;
use crate::support::registry::Package;
use crate::support::{basic_manifest, cross_compile, project};
use crate::support::{rustc_host, sleep_ms};
use cargo::util::paths::remove_dir_all;

#[test]
fn custom_build_script_failed() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]
            build = "build.rs"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("build.rs", "fn main() { std::process::exit(101); }")
        .build();
    p.cargo("build -v")
        .with_status(101)
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc --crate-name build_script_build build.rs --color never --crate-type bin [..]`
[RUNNING] `[..]/build-script-build`
[ERROR] failed to run custom build command for `foo v0.5.0 ([CWD])`
process didn't exit successfully: `[..]/build-script-build` (exit code: 101)",
        )
        .run();
}

#[test]
fn custom_build_env_vars() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [features]
            bar_feat = ["bar/foo"]

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]
            build = "build.rs"

            [features]
            foo = []
        "#,
        )
        .file("bar/src/lib.rs", "pub fn hello() {}");

    let file_content = format!(
        r#"
            use std::env;
            use std::io::prelude::*;
            use std::path::Path;
            use std::fs;

            fn main() {{
                let _target = env::var("TARGET").unwrap();
                let _ncpus = env::var("NUM_JOBS").unwrap();
                let _dir = env::var("CARGO_MANIFEST_DIR").unwrap();

                let opt = env::var("OPT_LEVEL").unwrap();
                assert_eq!(opt, "0");

                let opt = env::var("PROFILE").unwrap();
                assert_eq!(opt, "debug");

                let debug = env::var("DEBUG").unwrap();
                assert_eq!(debug, "true");

                let out = env::var("OUT_DIR").unwrap();
                assert!(out.starts_with(r"{0}"));
                assert!(fs::metadata(&out).map(|m| m.is_dir()).unwrap_or(false));

                let _host = env::var("HOST").unwrap();

                let _feat = env::var("CARGO_FEATURE_FOO").unwrap();

                let _cargo = env::var("CARGO").unwrap();

                let rustc = env::var("RUSTC").unwrap();
                assert_eq!(rustc, "rustc");

                let rustdoc = env::var("RUSTDOC").unwrap();
                assert_eq!(rustdoc, "rustdoc");

                assert!(env::var("RUSTC_LINKER").is_err());
            }}
        "#,
        p.root()
            .join("target")
            .join("debug")
            .join("build")
            .display()
    );

    let p = p.file("bar/build.rs", &file_content).build();

    p.cargo("build --features bar_feat").run();
}

#[test]
fn custom_build_env_var_rustc_linker() {
    if cross_compile::disabled() {
        return;
    }
    let target = cross_compile::alternate();
    let p = project()
        .file(
            ".cargo/config",
            &format!(
                r#"
                [target.{}]
                linker = "/path/to/linker"
                "#,
                target
            ),
        )
        .file(
            "build.rs",
            r#"
            use std::env;

            fn main() {
                assert!(env::var("RUSTC_LINKER").unwrap().ends_with("/path/to/linker"));
            }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    // no crate type set => linker never called => build succeeds if and
    // only if build.rs succeeds, despite linker binary not existing.
    p.cargo("build --target").arg(&target).run();
}

#[test]
fn custom_build_script_wrong_rustc_flags() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]
            build = "build.rs"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-flags=-aaa -bbb"); }"#,
        )
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr_contains(
            "\
             [ERROR] Only `-l` and `-L` flags are allowed in build script of `foo v0.5.0 ([CWD])`: \
             `-aaa -bbb`",
        )
        .run();
}

/*
#[test]
fn custom_build_script_rustc_flags() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]

            name = "bar"
            version = "0.5.0"
            authors = ["wycats@example.com"]

            [dependencies.foo]
            path = "foo"
        "#,
        ).file("src/main.rs", "fn main() {}")
        .file(
            "foo/Cargo.toml",
            r#"
            [project]

            name = "foo"
            version = "0.5.0"
            authors = ["wycats@example.com"]
            build = "build.rs"
        "#,
        ).file("foo/src/lib.rs", "")
        .file(
            "foo/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-flags=-l nonexistinglib -L /dummy/path1 -L /dummy/path2");
            }
        "#,
        ).build();

    // TODO: TEST FAILS BECAUSE OF WRONG STDOUT (but otherwise, the build works)
    p.cargo("build --verbose")
        .with_status(101)
        .with_stderr(
            "\
[COMPILING] bar v0.5.0 ([CWD])
[RUNNING] `rustc --crate-name test [CWD]/src/lib.rs --crate-type lib -C debuginfo=2 \
        -C metadata=[..] \
        -C extra-filename=-[..] \
        --out-dir [CWD]/target \
        --emit=dep-info,link \
        -L [CWD]/target \
        -L [CWD]/target/deps`
",
        ).run();
}
*/

#[test]
fn links_no_build_cmd() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] package `foo v0.5.0 ([CWD])` specifies that it links to `a` but does \
not have a custom build script
",
        )
        .run();
}

#[test]
fn links_duplicates() {
    // this tests that the links_duplicates are caught at resolver time
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"

            [dependencies.a-sys]
            path = "a-sys"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "")
        .file(
            "a-sys/Cargo.toml",
            r#"
            [project]
            name = "a-sys"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"
        "#,
        )
        .file("a-sys/src/lib.rs", "")
        .file("a-sys/build.rs", "")
        .build();

    p.cargo("build").with_status(101)
                       .with_stderr("\
error: failed to select a version for `a-sys`.
    ... required by package `foo v0.5.0 ([..])`
versions that meet the requirements `*` are: 0.5.0

the package `a-sys` links to the native library `a`, but it conflicts with a previous package which links to `a` as well:
package `foo v0.5.0 ([..])`

failed to select a version for `a-sys` which could resolve this conflict
").run();
}

#[test]
fn links_duplicates_deep_dependency() {
    // this tests that the links_duplicates are caught at resolver time
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"

            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies.a-sys]
            path = "a-sys"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file("a/build.rs", "")
        .file(
            "a/a-sys/Cargo.toml",
            r#"
            [project]
            name = "a-sys"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"
        "#,
        )
        .file("a/a-sys/src/lib.rs", "")
        .file("a/a-sys/build.rs", "")
        .build();

    p.cargo("build").with_status(101)
                       .with_stderr("\
error: failed to select a version for `a-sys`.
    ... required by package `a v0.5.0 ([..])`
    ... which is depended on by `foo v0.5.0 ([..])`
versions that meet the requirements `*` are: 0.5.0

the package `a-sys` links to the native library `a`, but it conflicts with a previous package which links to `a` as well:
package `foo v0.5.0 ([..])`

failed to select a version for `a-sys` which could resolve this conflict
").run();
}

#[test]
fn overrides_and_links() {
    let target = rustc_host();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;
            fn main() {
                assert_eq!(env::var("DEP_FOO_FOO").ok().expect("FOO missing"),
                           "bar");
                assert_eq!(env::var("DEP_FOO_BAR").ok().expect("BAR missing"),
                           "baz");
            }
        "#,
        )
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.foo]
            rustc-flags = "-L foo -L bar"
            foo = "bar"
            bar = "baz"
        "#,
                target
            ),
        )
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file("a/build.rs", "not valid rust code")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[..]
[..]
[..]
[..]
[..]
[RUNNING] `rustc --crate-name foo [..] -L foo -L bar`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn unused_overrides() {
    let target = rustc_host();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.foo]
            rustc-flags = "-L foo -L bar"
            foo = "bar"
            bar = "baz"
        "#,
                target
            ),
        )
        .build();

    p.cargo("build -v").run();
}

#[test]
fn links_passes_env_vars() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;
            fn main() {
                assert_eq!(env::var("DEP_FOO_FOO").unwrap(), "bar");
                assert_eq!(env::var("DEP_FOO_BAR").unwrap(), "baz");
            }
        "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"
            use std::env;
            fn main() {
                let lib = env::var("CARGO_MANIFEST_LINKS").unwrap();
                assert_eq!(lib, "foo");

                println!("cargo:foo=bar");
                println!("cargo:bar=baz");
            }
        "#,
        )
        .build();

    p.cargo("build -v").run();
}

#[test]
fn only_rerun_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    p.cargo("build -v").run();
    p.root().move_into_the_past();

    File::create(&p.root().join("some-new-file")).unwrap();
    p.root().move_into_the_past();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn rebuild_continues_to_pass_env_vars() {
    let a = project()
        .at("a")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::time::Duration;
            fn main() {
                println!("cargo:foo=bar");
                println!("cargo:bar=baz");
                std::thread::sleep(Duration::from_millis(500));
            }
        "#,
        )
        .build();
    a.root().move_into_the_past();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies.a]
            path = '{}'
        "#,
                a.root().display()
            ),
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;
            fn main() {
                assert_eq!(env::var("DEP_FOO_FOO").unwrap(), "bar");
                assert_eq!(env::var("DEP_FOO_BAR").unwrap(), "baz");
            }
        "#,
        )
        .build();

    p.cargo("build -v").run();
    p.root().move_into_the_past();

    File::create(&p.root().join("some-new-file")).unwrap();
    p.root().move_into_the_past();

    p.cargo("build -v").run();
}

#[test]
fn testing_and_such() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();

    println!("build");
    p.cargo("build -v").run();
    p.root().move_into_the_past();

    File::create(&p.root().join("src/lib.rs")).unwrap();
    p.root().move_into_the_past();

    println!("test");
    p.cargo("test -vj1")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..]`
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/foo-[..][EXE]`
[DOCTEST] foo
[RUNNING] `rustdoc --test [..]`",
        )
        .with_stdout_contains_n("running 0 tests", 2)
        .run();

    println!("doc");
    p.cargo("doc -v")
        .with_stderr(
            "\
[DOCUMENTING] foo v0.5.0 ([CWD])
[RUNNING] `rustdoc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    File::create(&p.root().join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    println!("run");
    p.cargo("run")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target/debug/foo[EXE]`
",
        )
        .run();
}

#[test]
fn propagation_of_l_flags() {
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "bar"
            build = "build.rs"

            [dependencies.b]
            path = "../b"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"fn main() { println!("cargo:rustc-flags=-L bar"); }"#,
        )
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("b/src/lib.rs", "")
        .file("b/build.rs", "bad file")
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.foo]
            rustc-flags = "-L foo"
        "#,
                target
            ),
        )
        .build();

    p.cargo("build -v -j1")
        .with_stderr_contains(
            "\
[RUNNING] `rustc --crate-name a [..] -L bar[..]-L foo[..]`
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc --crate-name foo [..] -L bar -L foo`
",
        )
        .run();
}

#[test]
fn propagation_of_l_flags_new() {
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "bar"
            build = "build.rs"

            [dependencies.b]
            path = "../b"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=bar");
            }
        "#,
        )
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("b/src/lib.rs", "")
        .file("b/build.rs", "bad file")
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.foo]
            rustc-link-search = ["foo"]
        "#,
                target
            ),
        )
        .build();

    p.cargo("build -v -j1")
        .with_stderr_contains(
            "\
[RUNNING] `rustc --crate-name a [..] -L bar[..]-L foo[..]`
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc --crate-name foo [..] -L bar -L foo`
",
        )
        .run();
}

#[test]
fn build_deps_simple() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
            [build-dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            "
            #[allow(unused_extern_crates)]
            extern crate a;
            fn main() {}
        ",
        )
        .file("a/Cargo.toml", &basic_manifest("a", "0.5.0"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] a v0.5.0 ([CWD]/a)
[RUNNING] `rustc --crate-name a [..]`
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc [..] build.rs [..] --extern a=[..]`
[RUNNING] `[..]/foo-[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn build_deps_not_for_normal() {
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
            [build-dependencies.aaaaa]
            path = "a"
        "#,
        )
        .file(
            "src/lib.rs",
            "#[allow(unused_extern_crates)] extern crate aaaaa;",
        )
        .file(
            "build.rs",
            "
            #[allow(unused_extern_crates)]
            extern crate aaaaa;
            fn main() {}
        ",
        )
        .file("a/Cargo.toml", &basic_manifest("aaaaa", "0.5.0"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build -v --target")
        .arg(&target)
        .with_status(101)
        .with_stderr_contains("[..]can't find crate for `aaaaa`[..]")
        .with_stderr_contains(
            "\
[ERROR] Could not compile `foo`.

Caused by:
  process didn't exit successfully: [..]
",
        )
        .run();
}

#[test]
fn build_cmd_with_a_build_cmd() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [build-dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            "
            #[allow(unused_extern_crates)]
            extern crate a;
            fn main() {}
        ",
        )
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [build-dependencies.b]
            path = "../b"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            "#[allow(unused_extern_crates)] extern crate b; fn main() {}",
        )
        .file("b/Cargo.toml", &basic_manifest("b", "0.5.0"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] b v0.5.0 ([CWD]/b)
[RUNNING] `rustc --crate-name b [..]`
[COMPILING] a v0.5.0 ([CWD]/a)
[RUNNING] `rustc [..] a/build.rs [..] --extern b=[..]`
[RUNNING] `[..]/a-[..]/build-script-build`
[RUNNING] `rustc --crate-name a [..]lib.rs --color never --crate-type lib \
    --emit=dep-info,link -C debuginfo=2 \
    -C metadata=[..] \
    --out-dir [..]target/debug/deps \
    -L [..]target/debug/deps`
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc --crate-name build_script_build build.rs --color never --crate-type bin \
    --emit=dep-info,link \
    -C debuginfo=2 -C metadata=[..] --out-dir [..] \
    -L [..]target/debug/deps \
    --extern a=[..]liba[..].rlib`
[RUNNING] `[..]/foo-[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..]lib.rs --color never --crate-type lib \
    --emit=dep-info,link -C debuginfo=2 \
    -C metadata=[..] \
    --out-dir [..] \
    -L [..]target/debug/deps`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn out_dir_is_preserved() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::fs::File;
            use std::path::Path;
            fn main() {
                let out = env::var("OUT_DIR").unwrap();
                File::create(Path::new(&out).join("foo")).unwrap();
            }
        "#,
        )
        .build();

    // Make the file
    p.cargo("build -v").run();
    p.root().move_into_the_past();

    // Change to asserting that it's there
    File::create(&p.root().join("build.rs"))
        .unwrap()
        .write_all(
            br#"
        use std::env;
        use std::old_io::File;
        fn main() {
            let out = env::var("OUT_DIR").unwrap();
            File::open(&Path::new(&out).join("foo")).unwrap();
        }
    "#,
        )
        .unwrap();
    p.root().move_into_the_past();
    p.cargo("build -v").run();

    // Run a fresh build where file should be preserved
    p.cargo("build -v").run();

    // One last time to make sure it's still there.
    File::create(&p.root().join("foo")).unwrap();
    p.cargo("build -v").run();
}

#[test]
fn output_separate_lines() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-flags=-L foo");
                println!("cargo:rustc-flags=-l static=foo");
            }
        "#,
        )
        .build();
    p.cargo("build -v")
        .with_status(101)
        .with_stderr_contains(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc [..] build.rs [..]`
[RUNNING] `[..]/foo-[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..] -L foo -l static=foo`
[ERROR] could not find native static library [..]
",
        )
        .run();
}

#[test]
fn output_separate_lines_new() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=foo");
                println!("cargo:rustc-link-lib=static=foo");
            }
        "#,
        )
        .build();
    p.cargo("build -v")
        .with_status(101)
        .with_stderr_contains(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[RUNNING] `rustc [..] build.rs [..]`
[RUNNING] `[..]/foo-[..]/build-script-build`
[RUNNING] `rustc --crate-name foo [..] -L foo -l static=foo`
[ERROR] could not find native static library [..]
",
        )
        .run();
}

#[cfg(not(windows))] // FIXME(#867)
#[test]
fn code_generation() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            include!(concat!(env!("OUT_DIR"), "/hello.rs"));

            fn main() {
                println!("{}", message());
            }
        "#,
        )
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::fs::File;
            use std::io::prelude::*;
            use std::path::PathBuf;

            fn main() {
                let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
                let mut f = File::create(&dst.join("hello.rs")).unwrap();
                f.write_all(b"
                    pub fn message() -> &'static str {
                        \"Hello, World!\"
                    }
                ").unwrap();
            }
        "#,
        )
        .build();

    p.cargo("run")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target/debug/foo`",
        )
        .with_stdout("Hello, World!")
        .run();

    p.cargo("test").run();
}

#[test]
fn release_with_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {}
        "#,
        )
        .build();

    p.cargo("build -v --release").run();
}

#[test]
fn build_script_only() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
              [project]
              name = "foo"
              version = "0.0.0"
              authors = []
              build = "build.rs"
        "#,
        )
        .file("build.rs", r#"fn main() {}"#)
        .build();
    p.cargo("build -v")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]`

Caused by:
  no targets specified in the manifest
  either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present",
        )
        .run();
}

#[test]
fn shared_dep_with_a_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies.a]
            path = "a"

            [build-dependencies.b]
            path = "b"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("a/build.rs", "fn main() {}")
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.5.0"
            authors = []

            [dependencies.a]
            path = "../a"
        "#,
        )
        .file("b/src/lib.rs", "")
        .build();
    p.cargo("build -v").run();
}

#[test]
fn transitive_dep_host() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [build-dependencies.b]
            path = "b"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("a/build.rs", "fn main() {}")
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.5.0"
            authors = []

            [lib]
            name = "b"
            plugin = true

            [dependencies.a]
            path = "../a"
        "#,
        )
        .file("b/src/lib.rs", "")
        .build();
    p.cargo("build").run();
}

#[test]
fn test_a_lib_with_a_build_command() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            include!(concat!(env!("OUT_DIR"), "/foo.rs"));

            /// ```
            /// foo::bar();
            /// ```
            pub fn bar() {
                assert_eq!(foo(), 1);
            }
        "#,
        )
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::io::prelude::*;
            use std::fs::File;
            use std::path::PathBuf;

            fn main() {
                let out = PathBuf::from(env::var("OUT_DIR").unwrap());
                File::create(out.join("foo.rs")).unwrap().write_all(b"
                    fn foo() -> i32 { 1 }
                ").unwrap();
            }
        "#,
        )
        .build();
    p.cargo("test").run();
}

#[test]
fn test_dev_dep_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [dev-dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("a/build.rs", "fn main() {}")
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("test").run();
}

#[test]
fn build_script_with_dynamic_native_dependency() {
    let build = project()
        .at("builder")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "builder"
            version = "0.0.1"
            authors = []

            [lib]
            name = "builder"
            crate-type = ["dylib"]
        "#,
        )
        .file("src/lib.rs", "#[no_mangle] pub extern fn foo() {}")
        .build();

    let foo = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"

            [build-dependencies.bar]
            path = "bar"
        "#,
        )
        .file("build.rs", "extern crate bar; fn main() { bar::bar() }")
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "bar/build.rs",
            r#"
            use std::env;
            use std::fs;
            use std::path::PathBuf;

            fn main() {
                let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
                let root = PathBuf::from(env::var("BUILDER_ROOT").unwrap());
                let file = format!("{}builder{}",
                    env::consts::DLL_PREFIX,
                    env::consts::DLL_SUFFIX);
                let src = root.join(&file);
                let dst = out_dir.join(&file);
                fs::copy(src, dst).unwrap();
                if cfg!(windows) {
                    fs::copy(root.join("builder.dll.lib"),
                             out_dir.join("builder.dll.lib")).unwrap();
                }
                println!("cargo:rustc-link-search=native={}", out_dir.display());
            }
        "#,
        )
        .file(
            "bar/src/lib.rs",
            r#"
            pub fn bar() {
                #[cfg_attr(not(target_env = "msvc"), link(name = "builder"))]
                #[cfg_attr(target_env = "msvc", link(name = "builder.dll"))]
                extern { fn foo(); }
                unsafe { foo() }
            }
        "#,
        )
        .build();

    build
        .cargo("build -v")
        .env("RUST_LOG", "cargo::ops::cargo_rustc")
        .run();

    let root = build.root().join("target").join("debug");
    foo.cargo("build -v")
        .env("BUILDER_ROOT", root)
        .env("RUST_LOG", "cargo::ops::cargo_rustc")
        .run();
}

#[test]
fn profile_and_opt_level_set_correctly() {
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
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
              use std::env;

              fn main() {
                  assert_eq!(env::var("OPT_LEVEL").unwrap(), "3");
                  assert_eq!(env::var("PROFILE").unwrap(), "release");
                  assert_eq!(env::var("DEBUG").unwrap(), "false");
              }
        "#,
        )
        .build();
    p.cargo("bench").run();
}

#[test]
fn profile_debug_0() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [profile.dev]
            debug = 0
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
              use std::env;

              fn main() {
                  assert_eq!(env::var("OPT_LEVEL").unwrap(), "0");
                  assert_eq!(env::var("PROFILE").unwrap(), "debug");
                  assert_eq!(env::var("DEBUG").unwrap(), "false");
              }
        "#,
        )
        .build();
    p.cargo("build").run();
}

#[test]
fn build_script_with_lto() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"

            [profile.dev]
            lto = true
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .build();
    p.cargo("build").run();
}

#[test]
fn test_duplicate_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            build = "build.rs"

            [dependencies.bar]
            path = "bar"

            [build-dependencies.bar]
            path = "bar"
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            extern crate bar;
            fn main() { bar::do_nothing() }
        "#,
        )
        .file(
            "build.rs",
            r#"
            extern crate bar;
            fn main() { bar::do_nothing() }
        "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn do_nothing() {}")
        .build();

    p.cargo("build").run();
}

#[test]
fn cfg_feedback() {
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
        )
        .file("src/main.rs", "#[cfg(foo)] fn main() {}")
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-cfg=foo"); }"#,
        )
        .build();
    p.cargo("build -v").run();
}

#[test]
fn cfg_override() {
    let target = rustc_host();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"
        "#,
        )
        .file("src/main.rs", "#[cfg(foo)] fn main() {}")
        .file("build.rs", "")
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.a]
            rustc-cfg = ["foo"]
        "#,
                target
            ),
        )
        .build();

    p.cargo("build -v").run();
}

#[test]
fn cfg_test() {
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
        )
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-cfg=foo"); }"#,
        )
        .file(
            "src/lib.rs",
            r#"
            ///
            /// ```
            /// extern crate foo;
            ///
            /// fn main() {
            ///     foo::foo()
            /// }
            /// ```
            ///
            #[cfg(foo)]
            pub fn foo() {}

            #[cfg(foo)]
            #[test]
            fn test_foo() {
                foo()
            }
        "#,
        )
        .file("tests/test.rs", "#[cfg(foo)] #[test] fn test_bar() {}")
        .build();
    p.cargo("test -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] [..] build.rs [..]
[RUNNING] `[..]/build-script-build`
[RUNNING] [..] --cfg foo[..]
[RUNNING] [..] --cfg foo[..]
[RUNNING] [..] --cfg foo[..]
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/foo-[..][EXE]`
[RUNNING] `[..]/test-[..][EXE]`
[DOCTEST] foo
[RUNNING] [..] --cfg foo[..]",
        )
        .with_stdout_contains("test test_foo ... ok")
        .with_stdout_contains("test test_bar ... ok")
        .with_stdout_contains_n("test [..] ... ok", 3)
        .run();
}

#[test]
fn cfg_doc() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-cfg=foo"); }"#,
        )
        .file("src/lib.rs", "#[cfg(foo)] pub fn foo() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "bar/build.rs",
            r#"fn main() { println!("cargo:rustc-cfg=bar"); }"#,
        )
        .file("bar/src/lib.rs", "#[cfg(bar)] pub fn bar() {}")
        .build();
    p.cargo("doc").run();
    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/fn.foo.html").is_file());
    assert!(p.root().join("target/doc/bar/fn.bar.html").is_file());
}

#[test]
fn cfg_override_test() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"
            links = "a"
        "#,
        )
        .file("build.rs", "")
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.a]
            rustc-cfg = ["foo"]
        "#,
                rustc_host()
            ),
        )
        .file(
            "src/lib.rs",
            r#"
            ///
            /// ```
            /// extern crate foo;
            ///
            /// fn main() {
            ///     foo::foo()
            /// }
            /// ```
            ///
            #[cfg(foo)]
            pub fn foo() {}

            #[cfg(foo)]
            #[test]
            fn test_foo() {
                foo()
            }
        "#,
        )
        .file("tests/test.rs", "#[cfg(foo)] #[test] fn test_bar() {}")
        .build();
    p.cargo("test -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] `[..]`
[RUNNING] `[..]`
[RUNNING] `[..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/foo-[..][EXE]`
[RUNNING] `[..]/test-[..][EXE]`
[DOCTEST] foo
[RUNNING] [..] --cfg foo[..]",
        )
        .with_stdout_contains("test test_foo ... ok")
        .with_stdout_contains("test test_bar ... ok")
        .with_stdout_contains_n("test [..] ... ok", 3)
        .run();
}

#[test]
fn cfg_override_doc() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"
            links = "a"

            [dependencies.bar]
            path = "bar"
        "#,
        )
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{target}.a]
            rustc-cfg = ["foo"]
            [target.{target}.b]
            rustc-cfg = ["bar"]
        "#,
                target = rustc_host()
            ),
        )
        .file("build.rs", "")
        .file("src/lib.rs", "#[cfg(foo)] pub fn foo() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []
            build = "build.rs"
            links = "b"
        "#,
        )
        .file("bar/build.rs", "")
        .file("bar/src/lib.rs", "#[cfg(bar)] pub fn bar() {}")
        .build();
    p.cargo("doc").run();
    assert!(p.root().join("target/doc").is_dir());
    assert!(p.root().join("target/doc/foo/fn.foo.html").is_file());
    assert!(p.root().join("target/doc/bar/fn.bar.html").is_file());
}

#[test]
fn env_build() {
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
        )
        .file(
            "src/main.rs",
            r#"
            const FOO: &'static str = env!("FOO");
            fn main() {
                println!("{}", FOO);
            }
        "#,
        )
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-env=FOO=foo"); }"#,
        )
        .build();
    p.cargo("build -v").run();
    p.cargo("run -v").with_stdout("foo\n").run();
}

#[test]
fn env_test() {
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
        )
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-env=FOO=foo"); }"#,
        )
        .file(
            "src/lib.rs",
            r#"pub const FOO: &'static str = env!("FOO"); "#,
        )
        .file(
            "tests/test.rs",
            r#"
            extern crate foo;

            #[test]
            fn test_foo() {
                assert_eq!("foo", foo::FOO);
            }
        "#,
        )
        .build();
    p.cargo("test -v")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[RUNNING] [..] build.rs [..]
[RUNNING] `[..]/build-script-build`
[RUNNING] [..] --crate-name foo[..]
[RUNNING] [..] --crate-name foo[..]
[RUNNING] [..] --crate-name test[..]
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/foo-[..][EXE]`
[RUNNING] `[..]/test-[..][EXE]`
[DOCTEST] foo
[RUNNING] [..] --crate-name foo[..]",
        )
        .with_stdout_contains_n("running 0 tests", 2)
        .with_stdout_contains("test test_foo ... ok")
        .run();
}

#[test]
fn env_doc() {
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
        )
        .file(
            "src/main.rs",
            r#"
            const FOO: &'static str = env!("FOO");
            fn main() {}
        "#,
        )
        .file(
            "build.rs",
            r#"fn main() { println!("cargo:rustc-env=FOO=foo"); }"#,
        )
        .build();
    p.cargo("doc -v").run();
}

#[test]
fn flags_go_into_tests() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [dependencies]
            b = { path = "b" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("tests/foo.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []
            [dependencies]
            a = { path = "../a" }
        "#,
        )
        .file("b/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=test");
            }
        "#,
        )
        .build();

    p.cargo("test -v --test=foo")
        .with_stderr(
            "\
[COMPILING] a v0.5.0 ([..]
[RUNNING] `rustc [..] a/build.rs [..]`
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc [..] a/src/lib.rs [..] -L test[..]`
[COMPILING] b v0.5.0 ([..]
[RUNNING] `rustc [..] b/src/lib.rs [..] -L test[..]`
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..] src/lib.rs [..] -L test[..]`
[RUNNING] `rustc [..] tests/foo.rs [..] -L test[..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/foo-[..][EXE]`",
        )
        .with_stdout_contains("running 0 tests")
        .run();

    p.cargo("test -v -pb --lib")
        .with_stderr(
            "\
[FRESH] a v0.5.0 ([..]
[COMPILING] b v0.5.0 ([..]
[RUNNING] `rustc [..] b/src/lib.rs [..] -L test[..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]/b-[..][EXE]`",
        )
        .with_stdout_contains("running 0 tests")
        .run();
}

#[test]
fn diamond_passes_args_only_once() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [dependencies]
            a = { path = "a" }
            b = { path = "b" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("tests/foo.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            [dependencies]
            b = { path = "../b" }
            c = { path = "../c" }
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []
            [dependencies]
            c = { path = "../c" }
        "#,
        )
        .file("b/src/lib.rs", "")
        .file(
            "c/Cargo.toml",
            r#"
            [project]
            name = "c"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "c/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=native=test");
            }
        "#,
        )
        .file("c/src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] c v0.5.0 ([..]
[RUNNING] `rustc [..]`
[RUNNING] `[..]`
[RUNNING] `rustc [..]`
[COMPILING] b v0.5.0 ([..]
[RUNNING] `rustc [..]`
[COMPILING] a v0.5.0 ([..]
[RUNNING] `rustc [..]`
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `[..]rlib -L native=test`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn adding_an_override_invalidates() {
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(".cargo/config", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=native=foo");
            }
        "#,
        )
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..]`
[RUNNING] `[..]`
[RUNNING] `rustc [..] -L native=foo`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    File::create(p.root().join(".cargo/config"))
        .unwrap()
        .write_all(
            format!(
                "
        [target.{}.foo]
        rustc-link-search = [\"native=bar\"]
    ",
                target
            )
            .as_bytes(),
        )
        .unwrap();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..] -L native=bar`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn changing_an_override_invalidates() {
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            &format!(
                "
            [target.{}.foo]
            rustc-link-search = [\"native=foo\"]
        ",
                target
            ),
        )
        .file("build.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..] -L native=foo`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    File::create(p.root().join(".cargo/config"))
        .unwrap()
        .write_all(
            format!(
                "
        [target.{}.foo]
        rustc-link-search = [\"native=bar\"]
    ",
                target
            )
            .as_bytes(),
        )
        .unwrap();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..] -L native=bar`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn fresh_builds_possible_with_link_libs() {
    // The bug is non-deterministic. Sometimes you can get a fresh build
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "nativefoo"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            &format!(
                "
            [target.{}.nativefoo]
            rustc-link-lib = [\"a\"]
            rustc-link-search = [\"./b\"]
            rustc-flags = \"-l z -L ./\"
        ",
                target
            ),
        )
        .file("build.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("build -v")
        .env("RUST_LOG", "cargo::ops::cargo_rustc::fingerprint=info")
        .with_stderr(
            "\
[FRESH] foo v0.5.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn fresh_builds_possible_with_multiple_metadata_overrides() {
    // The bug is non-deterministic. Sometimes you can get a fresh build
    let target = rustc_host();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "foo"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            &format!(
                "
            [target.{}.foo]
            a = \"\"
            b = \"\"
            c = \"\"
            d = \"\"
            e = \"\"
        ",
                target
            ),
        )
        .file("build.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..]
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("build -v")
        .env("RUST_LOG", "cargo::ops::cargo_rustc::fingerprint=info")
        .with_stderr(
            "\
[FRESH] foo v0.5.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn rebuild_only_on_explicit_paths() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rerun-if-changed=foo");
                println!("cargo:rerun-if-changed=bar");
            }
        "#,
        )
        .build();

    p.cargo("build -v").run();

    // files don't exist, so should always rerun if they don't exist
    println!("run without");
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc [..] src/lib.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    sleep_ms(1000);
    File::create(p.root().join("foo")).unwrap();
    File::create(p.root().join("bar")).unwrap();
    sleep_ms(1000); // make sure the to-be-created outfile has a timestamp distinct from the infiles

    // now the exist, so run once, catch the mtime, then shouldn't run again
    println!("run with");
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc [..] src/lib.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    println!("run with2");
    p.cargo("build -v")
        .with_stderr(
            "\
[FRESH] foo v0.5.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    sleep_ms(1000);

    // random other files do not affect freshness
    println!("run baz");
    File::create(p.root().join("baz")).unwrap();
    p.cargo("build -v")
        .with_stderr(
            "\
[FRESH] foo v0.5.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    // but changing dependent files does
    println!("run foo change");
    File::create(p.root().join("foo")).unwrap();
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc [..] src/lib.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    // .. as does deleting a file
    println!("run foo delete");
    fs::remove_file(p.root().join("bar")).unwrap();
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..]/build-script-build`
[RUNNING] `rustc [..] src/lib.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn doctest_receives_build_link_args() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            [dependencies.a]
            path = "a"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "bar"
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=native=bar");
            }
        "#,
        )
        .build();

    p.cargo("test -v")
        .with_stderr_contains(
            "[RUNNING] `rustdoc --test [..] --crate-name foo [..]-L native=bar[..]`",
        )
        .run();
}

#[test]
fn please_respect_the_dag() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [dependencies]
            a = { path = 'a' }
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=native=foo");
            }
        "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "bar"
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=native=bar");
            }
        "#,
        )
        .build();

    p.cargo("build -v")
        .with_stderr_contains("[RUNNING] `rustc [..] -L native=foo -L native=bar[..]`")
        .run();
}

#[test]
fn non_utf8_output() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file(
            "build.rs",
            r#"
            use std::io::prelude::*;

            fn main() {
                let mut out = std::io::stdout();
                // print something that's not utf8
                out.write_all(b"\xff\xff\n").unwrap();

                // now print some cargo metadata that's utf8
                println!("cargo:rustc-cfg=foo");

                // now print more non-utf8
                out.write_all(b"\xff\xff\n").unwrap();
            }
        "#,
        )
        .file("src/main.rs", "#[cfg(foo)] fn main() {}")
        .build();

    p.cargo("build -v").run();
}

#[test]
fn custom_target_dir() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [dependencies]
            a = { path = "a" }
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            ".cargo/config",
            r#"
            [build]
            target-dir = 'test'
        "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("a/build.rs", "fn main() {}")
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build -v").run();
}

#[test]
fn panic_abort_with_build_scripts() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [profile.release]
            panic = 'abort'

            [dependencies]
            a = { path = "a" }
        "#,
        )
        .file(
            "src/lib.rs",
            "#[allow(unused_extern_crates)] extern crate a;",
        )
        .file("build.rs", "fn main() {}")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            build = "build.rs"

            [build-dependencies]
            b = { path = "../b" }
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/build.rs",
            "#[allow(unused_extern_crates)] extern crate b; fn main() {}",
        )
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []
        "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build -v --release").run();

    p.root().join("target").rm_rf();

    p.cargo("test --release -v")
        .with_stderr_does_not_contain("[..]panic[..]")
        .run();
}

#[test]
fn warnings_emitted() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:warning=foo");
                println!("cargo:warning=bar");
            }
        "#,
        )
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `rustc [..]`
[RUNNING] `[..]`
warning: foo
warning: bar
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn warnings_hidden_for_upstream() {
    Package::new("bar", "0.1.0")
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:warning=foo");
                    println!("cargo:warning=bar");
                }
            "#,
        )
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "bar"
                version = "0.1.0"
                authors = []
                build = "build.rs"
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
            version = "0.5.0"
            authors = []

            [dependencies]
            bar = "*"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build -v")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 ([..])
[COMPILING] bar v0.1.0
[RUNNING] `rustc [..]`
[RUNNING] `[..]`
[RUNNING] `rustc [..]`
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn warnings_printed_on_vv() {
    Package::new("bar", "0.1.0")
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:warning=foo");
                    println!("cargo:warning=bar");
                }
            "#,
        )
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "bar"
                version = "0.1.0"
                authors = []
                build = "build.rs"
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
            version = "0.5.0"
            authors = []

            [dependencies]
            bar = "*"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build -vv")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v0.1.0 ([..])
[COMPILING] bar v0.1.0
[RUNNING] `[..] rustc [..]`
[RUNNING] `[..]`
warning: foo
warning: bar
[RUNNING] `[..] rustc [..]`
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..] rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn output_shows_on_vv() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::io::prelude::*;

            fn main() {
                std::io::stderr().write_all(b"stderr\n").unwrap();
                std::io::stdout().write_all(b"stdout\n").unwrap();
            }
        "#,
        )
        .build();

    p.cargo("build -vv")
        .with_stdout("[foo 0.5.0] stdout")
        .with_stderr(
            "\
[COMPILING] foo v0.5.0 ([..])
[RUNNING] `[..] rustc [..]`
[RUNNING] `[..]`
[foo 0.5.0] stderr
[RUNNING] `[..] rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn links_with_dots() {
    let target = rustc_host();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            build = "build.rs"
            links = "a.b"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-link-search=bar")
            }
        "#,
        )
        .file(
            ".cargo/config",
            &format!(
                r#"
            [target.{}.'a.b']
            rustc-link-search = ["foo"]
        "#,
                target
            ),
        )
        .build();

    p.cargo("build -v")
        .with_stderr_contains("[RUNNING] `rustc --crate-name foo [..] [..] -L foo[..]`")
        .run();
}

#[test]
fn rustc_and_rustdoc_set_correctly() {
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
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
              use std::env;

              fn main() {
                  assert_eq!(env::var("RUSTC").unwrap(), "rustc");
                  assert_eq!(env::var("RUSTDOC").unwrap(), "rustdoc");
              }
        "#,
        )
        .build();
    p.cargo("bench").run();
}

#[test]
fn cfg_env_vars_available() {
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
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;

            fn main() {
                let fam = env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
                if cfg!(unix) {
                    assert_eq!(fam, "unix");
                } else {
                    assert_eq!(fam, "windows");
                }
            }
        "#,
        )
        .build();
    p.cargo("bench").run();
}

#[test] fn switch_features_rerun0() { switch_features_rerun(); }
#[test] fn switch_features_rerun1() { switch_features_rerun(); }
#[test] fn switch_features_rerun2() { switch_features_rerun(); }
#[test] fn switch_features_rerun3() { switch_features_rerun(); }
#[test] fn switch_features_rerun4() { switch_features_rerun(); }
#[test] fn switch_features_rerun5() { switch_features_rerun(); }
#[test] fn switch_features_rerun6() { switch_features_rerun(); }
#[test] fn switch_features_rerun7() { switch_features_rerun(); }
#[test] fn switch_features_rerun8() { switch_features_rerun(); }
#[test] fn switch_features_rerun9() { switch_features_rerun(); }
#[test] fn switch_features_rerun10() { switch_features_rerun(); }
#[test] fn switch_features_rerun11() { switch_features_rerun(); }
#[test] fn switch_features_rerun12() { switch_features_rerun(); }
#[test] fn switch_features_rerun13() { switch_features_rerun(); }
#[test] fn switch_features_rerun14() { switch_features_rerun(); }
#[test] fn switch_features_rerun15() { switch_features_rerun(); }
#[test] fn switch_features_rerun16() { switch_features_rerun(); }
#[test] fn switch_features_rerun17() { switch_features_rerun(); }
#[test] fn switch_features_rerun18() { switch_features_rerun(); }
#[test] fn switch_features_rerun19() { switch_features_rerun(); }
#[test] fn switch_features_rerun20() { switch_features_rerun(); }
#[test] fn switch_features_rerun21() { switch_features_rerun(); }
#[test] fn switch_features_rerun22() { switch_features_rerun(); }
#[test] fn switch_features_rerun23() { switch_features_rerun(); }
#[test] fn switch_features_rerun24() { switch_features_rerun(); }
#[test] fn switch_features_rerun25() { switch_features_rerun(); }
#[test] fn switch_features_rerun26() { switch_features_rerun(); }
#[test] fn switch_features_rerun27() { switch_features_rerun(); }
#[test] fn switch_features_rerun28() { switch_features_rerun(); }
#[test] fn switch_features_rerun29() { switch_features_rerun(); }
#[test] fn switch_features_rerun30() { switch_features_rerun(); }
#[test] fn switch_features_rerun31() { switch_features_rerun(); }
#[test] fn switch_features_rerun32() { switch_features_rerun(); }
#[test] fn switch_features_rerun33() { switch_features_rerun(); }
#[test] fn switch_features_rerun34() { switch_features_rerun(); }
#[test] fn switch_features_rerun35() { switch_features_rerun(); }
#[test] fn switch_features_rerun36() { switch_features_rerun(); }
#[test] fn switch_features_rerun37() { switch_features_rerun(); }
#[test] fn switch_features_rerun38() { switch_features_rerun(); }
#[test] fn switch_features_rerun39() { switch_features_rerun(); }
#[test] fn switch_features_rerun40() { switch_features_rerun(); }
#[test] fn switch_features_rerun41() { switch_features_rerun(); }
#[test] fn switch_features_rerun42() { switch_features_rerun(); }
#[test] fn switch_features_rerun43() { switch_features_rerun(); }
#[test] fn switch_features_rerun44() { switch_features_rerun(); }
#[test] fn switch_features_rerun45() { switch_features_rerun(); }
#[test] fn switch_features_rerun46() { switch_features_rerun(); }
#[test] fn switch_features_rerun47() { switch_features_rerun(); }
#[test] fn switch_features_rerun48() { switch_features_rerun(); }
#[test] fn switch_features_rerun49() { switch_features_rerun(); }
#[test] fn switch_features_rerun50() { switch_features_rerun(); }
#[test] fn switch_features_rerun51() { switch_features_rerun(); }
#[test] fn switch_features_rerun52() { switch_features_rerun(); }
#[test] fn switch_features_rerun53() { switch_features_rerun(); }
#[test] fn switch_features_rerun54() { switch_features_rerun(); }
#[test] fn switch_features_rerun55() { switch_features_rerun(); }
#[test] fn switch_features_rerun56() { switch_features_rerun(); }
#[test] fn switch_features_rerun57() { switch_features_rerun(); }
#[test] fn switch_features_rerun58() { switch_features_rerun(); }
#[test] fn switch_features_rerun59() { switch_features_rerun(); }
#[test] fn switch_features_rerun60() { switch_features_rerun(); }
#[test] fn switch_features_rerun61() { switch_features_rerun(); }
#[test] fn switch_features_rerun62() { switch_features_rerun(); }
#[test] fn switch_features_rerun63() { switch_features_rerun(); }
#[test] fn switch_features_rerun64() { switch_features_rerun(); }
#[test] fn switch_features_rerun65() { switch_features_rerun(); }
#[test] fn switch_features_rerun66() { switch_features_rerun(); }
#[test] fn switch_features_rerun67() { switch_features_rerun(); }
#[test] fn switch_features_rerun68() { switch_features_rerun(); }
#[test] fn switch_features_rerun69() { switch_features_rerun(); }
#[test] fn switch_features_rerun70() { switch_features_rerun(); }
#[test] fn switch_features_rerun71() { switch_features_rerun(); }
#[test] fn switch_features_rerun72() { switch_features_rerun(); }
#[test] fn switch_features_rerun73() { switch_features_rerun(); }
#[test] fn switch_features_rerun74() { switch_features_rerun(); }
#[test] fn switch_features_rerun75() { switch_features_rerun(); }
#[test] fn switch_features_rerun76() { switch_features_rerun(); }
#[test] fn switch_features_rerun77() { switch_features_rerun(); }
#[test] fn switch_features_rerun78() { switch_features_rerun(); }
#[test] fn switch_features_rerun79() { switch_features_rerun(); }
#[test] fn switch_features_rerun80() { switch_features_rerun(); }
#[test] fn switch_features_rerun81() { switch_features_rerun(); }
#[test] fn switch_features_rerun82() { switch_features_rerun(); }
#[test] fn switch_features_rerun83() { switch_features_rerun(); }
#[test] fn switch_features_rerun84() { switch_features_rerun(); }
#[test] fn switch_features_rerun85() { switch_features_rerun(); }
#[test] fn switch_features_rerun86() { switch_features_rerun(); }
#[test] fn switch_features_rerun87() { switch_features_rerun(); }
#[test] fn switch_features_rerun88() { switch_features_rerun(); }
#[test] fn switch_features_rerun89() { switch_features_rerun(); }
#[test] fn switch_features_rerun90() { switch_features_rerun(); }
#[test] fn switch_features_rerun91() { switch_features_rerun(); }
#[test] fn switch_features_rerun92() { switch_features_rerun(); }
#[test] fn switch_features_rerun93() { switch_features_rerun(); }
#[test] fn switch_features_rerun94() { switch_features_rerun(); }
#[test] fn switch_features_rerun95() { switch_features_rerun(); }
#[test] fn switch_features_rerun96() { switch_features_rerun(); }
#[test] fn switch_features_rerun97() { switch_features_rerun(); }
#[test] fn switch_features_rerun98() { switch_features_rerun(); }
#[test] fn switch_features_rerun99() { switch_features_rerun(); }
#[test] fn switch_features_rerun100() { switch_features_rerun(); }
#[test] fn switch_features_rerun101() { switch_features_rerun(); }
#[test] fn switch_features_rerun102() { switch_features_rerun(); }
#[test] fn switch_features_rerun103() { switch_features_rerun(); }
#[test] fn switch_features_rerun104() { switch_features_rerun(); }
#[test] fn switch_features_rerun105() { switch_features_rerun(); }
#[test] fn switch_features_rerun106() { switch_features_rerun(); }
#[test] fn switch_features_rerun107() { switch_features_rerun(); }
#[test] fn switch_features_rerun108() { switch_features_rerun(); }
#[test] fn switch_features_rerun109() { switch_features_rerun(); }
#[test] fn switch_features_rerun110() { switch_features_rerun(); }
#[test] fn switch_features_rerun111() { switch_features_rerun(); }
#[test] fn switch_features_rerun112() { switch_features_rerun(); }
#[test] fn switch_features_rerun113() { switch_features_rerun(); }
#[test] fn switch_features_rerun114() { switch_features_rerun(); }
#[test] fn switch_features_rerun115() { switch_features_rerun(); }
#[test] fn switch_features_rerun116() { switch_features_rerun(); }
#[test] fn switch_features_rerun117() { switch_features_rerun(); }
#[test] fn switch_features_rerun118() { switch_features_rerun(); }
#[test] fn switch_features_rerun119() { switch_features_rerun(); }
#[test] fn switch_features_rerun120() { switch_features_rerun(); }
#[test] fn switch_features_rerun121() { switch_features_rerun(); }
#[test] fn switch_features_rerun122() { switch_features_rerun(); }
#[test] fn switch_features_rerun123() { switch_features_rerun(); }
#[test] fn switch_features_rerun124() { switch_features_rerun(); }
#[test] fn switch_features_rerun125() { switch_features_rerun(); }
#[test] fn switch_features_rerun126() { switch_features_rerun(); }
#[test] fn switch_features_rerun127() { switch_features_rerun(); }
#[test] fn switch_features_rerun128() { switch_features_rerun(); }
#[test] fn switch_features_rerun129() { switch_features_rerun(); }
#[test] fn switch_features_rerun130() { switch_features_rerun(); }
#[test] fn switch_features_rerun131() { switch_features_rerun(); }
#[test] fn switch_features_rerun132() { switch_features_rerun(); }
#[test] fn switch_features_rerun133() { switch_features_rerun(); }
#[test] fn switch_features_rerun134() { switch_features_rerun(); }
#[test] fn switch_features_rerun135() { switch_features_rerun(); }
#[test] fn switch_features_rerun136() { switch_features_rerun(); }
#[test] fn switch_features_rerun137() { switch_features_rerun(); }
#[test] fn switch_features_rerun138() { switch_features_rerun(); }
#[test] fn switch_features_rerun139() { switch_features_rerun(); }
#[test] fn switch_features_rerun140() { switch_features_rerun(); }
#[test] fn switch_features_rerun141() { switch_features_rerun(); }
#[test] fn switch_features_rerun142() { switch_features_rerun(); }
#[test] fn switch_features_rerun143() { switch_features_rerun(); }
#[test] fn switch_features_rerun144() { switch_features_rerun(); }
#[test] fn switch_features_rerun145() { switch_features_rerun(); }
#[test] fn switch_features_rerun146() { switch_features_rerun(); }
#[test] fn switch_features_rerun147() { switch_features_rerun(); }
#[test] fn switch_features_rerun148() { switch_features_rerun(); }
#[test] fn switch_features_rerun149() { switch_features_rerun(); }
#[test] fn switch_features_rerun150() { switch_features_rerun(); }
#[test] fn switch_features_rerun151() { switch_features_rerun(); }
#[test] fn switch_features_rerun152() { switch_features_rerun(); }
#[test] fn switch_features_rerun153() { switch_features_rerun(); }
#[test] fn switch_features_rerun154() { switch_features_rerun(); }
#[test] fn switch_features_rerun155() { switch_features_rerun(); }
#[test] fn switch_features_rerun156() { switch_features_rerun(); }
#[test] fn switch_features_rerun157() { switch_features_rerun(); }
#[test] fn switch_features_rerun158() { switch_features_rerun(); }
#[test] fn switch_features_rerun159() { switch_features_rerun(); }
#[test] fn switch_features_rerun160() { switch_features_rerun(); }
#[test] fn switch_features_rerun161() { switch_features_rerun(); }
#[test] fn switch_features_rerun162() { switch_features_rerun(); }
#[test] fn switch_features_rerun163() { switch_features_rerun(); }
#[test] fn switch_features_rerun164() { switch_features_rerun(); }
#[test] fn switch_features_rerun165() { switch_features_rerun(); }
#[test] fn switch_features_rerun166() { switch_features_rerun(); }
#[test] fn switch_features_rerun167() { switch_features_rerun(); }
#[test] fn switch_features_rerun168() { switch_features_rerun(); }
#[test] fn switch_features_rerun169() { switch_features_rerun(); }
#[test] fn switch_features_rerun170() { switch_features_rerun(); }
#[test] fn switch_features_rerun171() { switch_features_rerun(); }
#[test] fn switch_features_rerun172() { switch_features_rerun(); }
#[test] fn switch_features_rerun173() { switch_features_rerun(); }
#[test] fn switch_features_rerun174() { switch_features_rerun(); }
#[test] fn switch_features_rerun175() { switch_features_rerun(); }
#[test] fn switch_features_rerun176() { switch_features_rerun(); }
#[test] fn switch_features_rerun177() { switch_features_rerun(); }
#[test] fn switch_features_rerun178() { switch_features_rerun(); }
#[test] fn switch_features_rerun179() { switch_features_rerun(); }
#[test] fn switch_features_rerun180() { switch_features_rerun(); }
#[test] fn switch_features_rerun181() { switch_features_rerun(); }
#[test] fn switch_features_rerun182() { switch_features_rerun(); }
#[test] fn switch_features_rerun183() { switch_features_rerun(); }
#[test] fn switch_features_rerun184() { switch_features_rerun(); }
#[test] fn switch_features_rerun185() { switch_features_rerun(); }
#[test] fn switch_features_rerun186() { switch_features_rerun(); }
#[test] fn switch_features_rerun187() { switch_features_rerun(); }
#[test] fn switch_features_rerun188() { switch_features_rerun(); }
#[test] fn switch_features_rerun189() { switch_features_rerun(); }
#[test] fn switch_features_rerun190() { switch_features_rerun(); }
#[test] fn switch_features_rerun191() { switch_features_rerun(); }
#[test] fn switch_features_rerun192() { switch_features_rerun(); }
#[test] fn switch_features_rerun193() { switch_features_rerun(); }
#[test] fn switch_features_rerun194() { switch_features_rerun(); }
#[test] fn switch_features_rerun195() { switch_features_rerun(); }
#[test] fn switch_features_rerun196() { switch_features_rerun(); }
#[test] fn switch_features_rerun197() { switch_features_rerun(); }
#[test] fn switch_features_rerun198() { switch_features_rerun(); }
#[test] fn switch_features_rerun199() { switch_features_rerun(); }
#[test] fn switch_features_rerun200() { switch_features_rerun(); }
#[test] fn switch_features_rerun201() { switch_features_rerun(); }
#[test] fn switch_features_rerun202() { switch_features_rerun(); }
#[test] fn switch_features_rerun203() { switch_features_rerun(); }
#[test] fn switch_features_rerun204() { switch_features_rerun(); }
#[test] fn switch_features_rerun205() { switch_features_rerun(); }
#[test] fn switch_features_rerun206() { switch_features_rerun(); }
#[test] fn switch_features_rerun207() { switch_features_rerun(); }
#[test] fn switch_features_rerun208() { switch_features_rerun(); }
#[test] fn switch_features_rerun209() { switch_features_rerun(); }
#[test] fn switch_features_rerun210() { switch_features_rerun(); }
#[test] fn switch_features_rerun211() { switch_features_rerun(); }
#[test] fn switch_features_rerun212() { switch_features_rerun(); }
#[test] fn switch_features_rerun213() { switch_features_rerun(); }
#[test] fn switch_features_rerun214() { switch_features_rerun(); }
#[test] fn switch_features_rerun215() { switch_features_rerun(); }
#[test] fn switch_features_rerun216() { switch_features_rerun(); }
#[test] fn switch_features_rerun217() { switch_features_rerun(); }
#[test] fn switch_features_rerun218() { switch_features_rerun(); }
#[test] fn switch_features_rerun219() { switch_features_rerun(); }
#[test] fn switch_features_rerun220() { switch_features_rerun(); }
#[test] fn switch_features_rerun221() { switch_features_rerun(); }
#[test] fn switch_features_rerun222() { switch_features_rerun(); }
#[test] fn switch_features_rerun223() { switch_features_rerun(); }
#[test] fn switch_features_rerun224() { switch_features_rerun(); }
#[test] fn switch_features_rerun225() { switch_features_rerun(); }
#[test] fn switch_features_rerun226() { switch_features_rerun(); }
#[test] fn switch_features_rerun227() { switch_features_rerun(); }
#[test] fn switch_features_rerun228() { switch_features_rerun(); }
#[test] fn switch_features_rerun229() { switch_features_rerun(); }
#[test] fn switch_features_rerun230() { switch_features_rerun(); }
#[test] fn switch_features_rerun231() { switch_features_rerun(); }
#[test] fn switch_features_rerun232() { switch_features_rerun(); }
#[test] fn switch_features_rerun233() { switch_features_rerun(); }
#[test] fn switch_features_rerun234() { switch_features_rerun(); }
#[test] fn switch_features_rerun235() { switch_features_rerun(); }
#[test] fn switch_features_rerun236() { switch_features_rerun(); }
#[test] fn switch_features_rerun237() { switch_features_rerun(); }
#[test] fn switch_features_rerun238() { switch_features_rerun(); }
#[test] fn switch_features_rerun239() { switch_features_rerun(); }
#[test] fn switch_features_rerun240() { switch_features_rerun(); }
#[test] fn switch_features_rerun241() { switch_features_rerun(); }
#[test] fn switch_features_rerun242() { switch_features_rerun(); }
#[test] fn switch_features_rerun243() { switch_features_rerun(); }
#[test] fn switch_features_rerun244() { switch_features_rerun(); }
#[test] fn switch_features_rerun245() { switch_features_rerun(); }
#[test] fn switch_features_rerun246() { switch_features_rerun(); }
#[test] fn switch_features_rerun247() { switch_features_rerun(); }
#[test] fn switch_features_rerun248() { switch_features_rerun(); }
#[test] fn switch_features_rerun249() { switch_features_rerun(); }
#[test] fn switch_features_rerun250() { switch_features_rerun(); }
#[test] fn switch_features_rerun251() { switch_features_rerun(); }
#[test] fn switch_features_rerun252() { switch_features_rerun(); }
#[test] fn switch_features_rerun253() { switch_features_rerun(); }
#[test] fn switch_features_rerun254() { switch_features_rerun(); }
#[test] fn switch_features_rerun255() { switch_features_rerun(); }
#[test] fn switch_features_rerun256() { switch_features_rerun(); }
#[test] fn switch_features_rerun257() { switch_features_rerun(); }
#[test] fn switch_features_rerun258() { switch_features_rerun(); }
#[test] fn switch_features_rerun259() { switch_features_rerun(); }
#[test] fn switch_features_rerun260() { switch_features_rerun(); }
#[test] fn switch_features_rerun261() { switch_features_rerun(); }
#[test] fn switch_features_rerun262() { switch_features_rerun(); }
#[test] fn switch_features_rerun263() { switch_features_rerun(); }
#[test] fn switch_features_rerun264() { switch_features_rerun(); }
#[test] fn switch_features_rerun265() { switch_features_rerun(); }
#[test] fn switch_features_rerun266() { switch_features_rerun(); }
#[test] fn switch_features_rerun267() { switch_features_rerun(); }
#[test] fn switch_features_rerun268() { switch_features_rerun(); }
#[test] fn switch_features_rerun269() { switch_features_rerun(); }
#[test] fn switch_features_rerun270() { switch_features_rerun(); }
#[test] fn switch_features_rerun271() { switch_features_rerun(); }
#[test] fn switch_features_rerun272() { switch_features_rerun(); }
#[test] fn switch_features_rerun273() { switch_features_rerun(); }
#[test] fn switch_features_rerun274() { switch_features_rerun(); }
#[test] fn switch_features_rerun275() { switch_features_rerun(); }
#[test] fn switch_features_rerun276() { switch_features_rerun(); }
#[test] fn switch_features_rerun277() { switch_features_rerun(); }
#[test] fn switch_features_rerun278() { switch_features_rerun(); }
#[test] fn switch_features_rerun279() { switch_features_rerun(); }
#[test] fn switch_features_rerun280() { switch_features_rerun(); }
#[test] fn switch_features_rerun281() { switch_features_rerun(); }
#[test] fn switch_features_rerun282() { switch_features_rerun(); }
#[test] fn switch_features_rerun283() { switch_features_rerun(); }
#[test] fn switch_features_rerun284() { switch_features_rerun(); }
#[test] fn switch_features_rerun285() { switch_features_rerun(); }
#[test] fn switch_features_rerun286() { switch_features_rerun(); }
#[test] fn switch_features_rerun287() { switch_features_rerun(); }
#[test] fn switch_features_rerun288() { switch_features_rerun(); }
#[test] fn switch_features_rerun289() { switch_features_rerun(); }
#[test] fn switch_features_rerun290() { switch_features_rerun(); }
#[test] fn switch_features_rerun291() { switch_features_rerun(); }
#[test] fn switch_features_rerun292() { switch_features_rerun(); }
#[test] fn switch_features_rerun293() { switch_features_rerun(); }
#[test] fn switch_features_rerun294() { switch_features_rerun(); }
#[test] fn switch_features_rerun295() { switch_features_rerun(); }
#[test] fn switch_features_rerun296() { switch_features_rerun(); }
#[test] fn switch_features_rerun297() { switch_features_rerun(); }
#[test] fn switch_features_rerun298() { switch_features_rerun(); }
#[test] fn switch_features_rerun299() { switch_features_rerun(); }
#[test] fn switch_features_rerun300() { switch_features_rerun(); }
#[test] fn switch_features_rerun301() { switch_features_rerun(); }
#[test] fn switch_features_rerun302() { switch_features_rerun(); }
#[test] fn switch_features_rerun303() { switch_features_rerun(); }
#[test] fn switch_features_rerun304() { switch_features_rerun(); }
#[test] fn switch_features_rerun305() { switch_features_rerun(); }
#[test] fn switch_features_rerun306() { switch_features_rerun(); }
#[test] fn switch_features_rerun307() { switch_features_rerun(); }
#[test] fn switch_features_rerun308() { switch_features_rerun(); }
#[test] fn switch_features_rerun309() { switch_features_rerun(); }
#[test] fn switch_features_rerun310() { switch_features_rerun(); }
#[test] fn switch_features_rerun311() { switch_features_rerun(); }
#[test] fn switch_features_rerun312() { switch_features_rerun(); }
#[test] fn switch_features_rerun313() { switch_features_rerun(); }
#[test] fn switch_features_rerun314() { switch_features_rerun(); }
#[test] fn switch_features_rerun315() { switch_features_rerun(); }
#[test] fn switch_features_rerun316() { switch_features_rerun(); }
#[test] fn switch_features_rerun317() { switch_features_rerun(); }
#[test] fn switch_features_rerun318() { switch_features_rerun(); }
#[test] fn switch_features_rerun319() { switch_features_rerun(); }
#[test] fn switch_features_rerun320() { switch_features_rerun(); }
#[test] fn switch_features_rerun321() { switch_features_rerun(); }
#[test] fn switch_features_rerun322() { switch_features_rerun(); }
#[test] fn switch_features_rerun323() { switch_features_rerun(); }
#[test] fn switch_features_rerun324() { switch_features_rerun(); }
#[test] fn switch_features_rerun325() { switch_features_rerun(); }
#[test] fn switch_features_rerun326() { switch_features_rerun(); }
#[test] fn switch_features_rerun327() { switch_features_rerun(); }
#[test] fn switch_features_rerun328() { switch_features_rerun(); }
#[test] fn switch_features_rerun329() { switch_features_rerun(); }
#[test] fn switch_features_rerun330() { switch_features_rerun(); }
#[test] fn switch_features_rerun331() { switch_features_rerun(); }
#[test] fn switch_features_rerun332() { switch_features_rerun(); }
#[test] fn switch_features_rerun333() { switch_features_rerun(); }
#[test] fn switch_features_rerun334() { switch_features_rerun(); }
#[test] fn switch_features_rerun335() { switch_features_rerun(); }
#[test] fn switch_features_rerun336() { switch_features_rerun(); }
#[test] fn switch_features_rerun337() { switch_features_rerun(); }
#[test] fn switch_features_rerun338() { switch_features_rerun(); }
#[test] fn switch_features_rerun339() { switch_features_rerun(); }
#[test] fn switch_features_rerun340() { switch_features_rerun(); }
#[test] fn switch_features_rerun341() { switch_features_rerun(); }
#[test] fn switch_features_rerun342() { switch_features_rerun(); }
#[test] fn switch_features_rerun343() { switch_features_rerun(); }
#[test] fn switch_features_rerun344() { switch_features_rerun(); }
#[test] fn switch_features_rerun345() { switch_features_rerun(); }
#[test] fn switch_features_rerun346() { switch_features_rerun(); }
#[test] fn switch_features_rerun347() { switch_features_rerun(); }
#[test] fn switch_features_rerun348() { switch_features_rerun(); }
#[test] fn switch_features_rerun349() { switch_features_rerun(); }
#[test] fn switch_features_rerun350() { switch_features_rerun(); }
#[test] fn switch_features_rerun351() { switch_features_rerun(); }
#[test] fn switch_features_rerun352() { switch_features_rerun(); }
#[test] fn switch_features_rerun353() { switch_features_rerun(); }
#[test] fn switch_features_rerun354() { switch_features_rerun(); }
#[test] fn switch_features_rerun355() { switch_features_rerun(); }
#[test] fn switch_features_rerun356() { switch_features_rerun(); }
#[test] fn switch_features_rerun357() { switch_features_rerun(); }
#[test] fn switch_features_rerun358() { switch_features_rerun(); }
#[test] fn switch_features_rerun359() { switch_features_rerun(); }
#[test] fn switch_features_rerun360() { switch_features_rerun(); }
#[test] fn switch_features_rerun361() { switch_features_rerun(); }
#[test] fn switch_features_rerun362() { switch_features_rerun(); }
#[test] fn switch_features_rerun363() { switch_features_rerun(); }
#[test] fn switch_features_rerun364() { switch_features_rerun(); }
#[test] fn switch_features_rerun365() { switch_features_rerun(); }
#[test] fn switch_features_rerun366() { switch_features_rerun(); }
#[test] fn switch_features_rerun367() { switch_features_rerun(); }
#[test] fn switch_features_rerun368() { switch_features_rerun(); }
#[test] fn switch_features_rerun369() { switch_features_rerun(); }
#[test] fn switch_features_rerun370() { switch_features_rerun(); }
#[test] fn switch_features_rerun371() { switch_features_rerun(); }
#[test] fn switch_features_rerun372() { switch_features_rerun(); }
#[test] fn switch_features_rerun373() { switch_features_rerun(); }
#[test] fn switch_features_rerun374() { switch_features_rerun(); }
#[test] fn switch_features_rerun375() { switch_features_rerun(); }
#[test] fn switch_features_rerun376() { switch_features_rerun(); }
#[test] fn switch_features_rerun377() { switch_features_rerun(); }
#[test] fn switch_features_rerun378() { switch_features_rerun(); }
#[test] fn switch_features_rerun379() { switch_features_rerun(); }
#[test] fn switch_features_rerun380() { switch_features_rerun(); }
#[test] fn switch_features_rerun381() { switch_features_rerun(); }
#[test] fn switch_features_rerun382() { switch_features_rerun(); }
#[test] fn switch_features_rerun383() { switch_features_rerun(); }
#[test] fn switch_features_rerun384() { switch_features_rerun(); }
#[test] fn switch_features_rerun385() { switch_features_rerun(); }
#[test] fn switch_features_rerun386() { switch_features_rerun(); }
#[test] fn switch_features_rerun387() { switch_features_rerun(); }
#[test] fn switch_features_rerun388() { switch_features_rerun(); }
#[test] fn switch_features_rerun389() { switch_features_rerun(); }
#[test] fn switch_features_rerun390() { switch_features_rerun(); }
#[test] fn switch_features_rerun391() { switch_features_rerun(); }
#[test] fn switch_features_rerun392() { switch_features_rerun(); }
#[test] fn switch_features_rerun393() { switch_features_rerun(); }
#[test] fn switch_features_rerun394() { switch_features_rerun(); }
#[test] fn switch_features_rerun395() { switch_features_rerun(); }
#[test] fn switch_features_rerun396() { switch_features_rerun(); }
#[test] fn switch_features_rerun397() { switch_features_rerun(); }
#[test] fn switch_features_rerun398() { switch_features_rerun(); }
#[test] fn switch_features_rerun399() { switch_features_rerun(); }

#[test]
fn switch_features_rerun() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"

            [features]
            foo = []
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                println!(include_str!(concat!(env!("OUT_DIR"), "/output")));
            }
        "#,
        )
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::fs::File;
            use std::io::Write;
            use std::path::Path;

            fn main() {
                let out_dir = env::var_os("OUT_DIR").unwrap();
                let out_dir = Path::new(&out_dir).join("output");
                let mut f = File::create(&out_dir).unwrap();

                if env::var_os("CARGO_FEATURE_FOO").is_some() {
                    f.write_all(b"foo").unwrap();
                } else {
                    f.write_all(b"bar").unwrap();
                }
            }
        "#,
        )
        .build();

    p.cargo("build -v --features=foo").run();
    p.safely_rename_run("foo", "with_foo").with_stdout("foo\n").run();
    p.cargo("build -v").run();
    p.safely_rename_run("foo", "without_foo").with_stdout("bar\n").run();
    p.cargo("build -v --features=foo").run();
    p.safely_rename_run("foo", "with_foo").with_stdout("foo\n").run();
}

#[test]
fn assume_build_script_when_build_rs_present() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
            fn main() {
                if ! cfg!(foo) {
                    panic!("the build script was not run");
                }
            }
        "#,
        )
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-cfg=foo");
            }
        "#,
        )
        .build();

    p.cargo("run -v").run();
}

#[test]
fn if_build_set_to_false_dont_treat_build_rs_as_build_script() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = false
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                if cfg!(foo) {
                    panic!("the build script was run");
                }
            }
        "#,
        )
        .file(
            "build.rs",
            r#"
            fn main() {
                println!("cargo:rustc-cfg=foo");
            }
        "#,
        )
        .build();

    p.cargo("run -v").run();
}

#[test]
fn deterministic_rustc_dependency_flags() {
    // This bug is non-deterministic hence the large number of dependencies
    // in the hopes it will have a much higher chance of triggering it.

    Package::new("dep1", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "dep1"
                version = "0.1.0"
                authors = []
                build = "build.rs"
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:rustc-flags=-L native=test1");
                }
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    Package::new("dep2", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "dep2"
                version = "0.1.0"
                authors = []
                build = "build.rs"
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:rustc-flags=-L native=test2");
                }
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    Package::new("dep3", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "dep3"
                version = "0.1.0"
                authors = []
                build = "build.rs"
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:rustc-flags=-L native=test3");
                }
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    Package::new("dep4", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "dep4"
                version = "0.1.0"
                authors = []
                build = "build.rs"
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    println!("cargo:rustc-flags=-L native=test4");
                }
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
            authors = []

            [dependencies]
            dep1 = "*"
            dep2 = "*"
            dep3 = "*"
            dep4 = "*"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build -v")
        .with_stderr_contains(
            "\
[RUNNING] `rustc --crate-name foo [..] -L native=test1 -L native=test2 \
-L native=test3 -L native=test4`
",
        )
        .run();
}

#[test]
fn links_duplicates_with_cycle() {
    // this tests that the links_duplicates are caught at resolver time
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"

            [dependencies.a]
            path = "a"

            [dev-dependencies]
            b = { path = "b" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [project]
            name = "a"
            version = "0.5.0"
            authors = []
            links = "a"
            build = "build.rs"
        "#,
        )
        .file("a/src/lib.rs", "")
        .file("a/build.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [project]
            name = "b"
            version = "0.5.0"
            authors = []

            [dependencies]
            foo = { path = ".." }
        "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build").with_status(101)
                       .with_stderr("\
error: failed to select a version for `a`.
    ... required by package `foo v0.5.0 ([..])`
versions that meet the requirements `*` are: 0.5.0

the package `a` links to the native library `a`, but it conflicts with a previous package which links to `a` as well:
package `foo v0.5.0 ([..])`

failed to select a version for `a` which could resolve this conflict
").run();
}

#[test]
fn rename_with_link_search_path() {
    _rename_with_link_search_path(false);
}

#[test]
fn rename_with_link_search_path_cross() {
    if cross_compile::disabled() {
        return;
    }

    _rename_with_link_search_path(true);
}

fn _rename_with_link_search_path(cross: bool) {
    let target_arg = if cross {
        format!(" --target={}", cross_compile::alternate())
    } else {
        "".to_string()
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.5.0"
            authors = []

            [lib]
            crate-type = ["cdylib"]
        "#,
        )
        .file(
            "src/lib.rs",
            "#[no_mangle] pub extern fn cargo_test_foo() {}",
        );
    let p = p.build();

    p.cargo(&format!("build{}", target_arg)).run();

    let p2 = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file(
            "build.rs",
            r#"
            use std::env;
            use std::fs;
            use std::path::PathBuf;

            fn main() {
                // Move the `libfoo.so` from the root of our project into the
                // build directory. This way Cargo should automatically manage
                // `LD_LIBRARY_PATH` and such.
                let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
                let file = format!("{}foo{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
                let src = root.join(&file);

                let dst_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
                let dst = dst_dir.join(&file);

                fs::copy(&src, &dst).unwrap();
                // handle windows, like below
                drop(fs::copy(root.join("foo.dll.lib"), dst_dir.join("foo.dll.lib")));

                println!("cargo:rerun-if-changed=build.rs");
                if cfg!(target_env = "msvc") {
                    println!("cargo:rustc-link-lib=foo.dll");
                } else {
                    println!("cargo:rustc-link-lib=foo");
                }
                println!("cargo:rustc-link-search=all={}",
                         dst.parent().unwrap().display());
            }
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            extern {
                #[link_name = "cargo_test_foo"]
                fn foo();
            }

            fn main() {
                unsafe { foo(); }
            }
        "#,
        );
    let p2 = p2.build();

    // Move the output `libfoo.so` into the directory of `p2`, and then delete
    // the `p` project. On OSX the `libfoo.dylib` artifact references the
    // original path in `p` so we want to make sure that it can't find it (hence
    // the deletion).
    let root = if cross {
        p.root()
            .join("target")
            .join(cross_compile::alternate())
            .join("debug")
            .join("deps")
    } else {
        p.root().join("target").join("debug").join("deps")
    };
    let file = format!("{}foo{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
    let src = root.join(&file);

    let dst = p2.root().join(&file);

    fs::copy(&src, &dst).unwrap();
    // copy the import library for windows, if it exists
    drop(fs::copy(
        &root.join("foo.dll.lib"),
        p2.root().join("foo.dll.lib"),
    ));
    remove_dir_all(p.root()).unwrap();

    // Everything should work the first time
    p2.cargo(&format!("run{}", target_arg)).run();

    // Now rename the root directory and rerun `cargo run`. Not only should we
    // not build anything but we also shouldn't crash.
    let mut new = p2.root();
    new.pop();
    new.push("bar2");

    // For whatever reason on Windows right after we execute a binary it's very
    // unlikely that we're able to successfully delete or rename that binary.
    // It's not really clear why this is the case or if it's a bug in Cargo
    // holding a handle open too long. In an effort to reduce the flakiness of
    // this test though we throw this in a loop
    //
    // For some more information see #5481 and rust-lang/rust#48775
    let mut i = 0;
    loop {
        let error = match fs::rename(p2.root(), &new) {
            Ok(()) => break,
            Err(e) => e,
        };
        i += 1;
        if !cfg!(windows) || error.kind() != io::ErrorKind::PermissionDenied || i > 10 {
            panic!("failed to rename: {}", error);
        }
        println!("assuming {} is spurious, waiting to try again", error);
        thread::sleep(Duration::from_millis(100));
    }

    p2.cargo(&format!("run{}", target_arg))
        .cwd(&new)
        .with_stderr(
            "\
[FINISHED] [..]
[RUNNING] [..]
",
        )
        .run();
}

#[test]
fn optional_build_script_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "foo"
                version = "0.5.0"
                authors = []

                [dependencies]
                bar = { path = "bar", optional = true }

                [build-dependencies]
                bar = { path = "bar", optional = true }
            "#,
        )
        .file(
            "build.rs",
            r#"
            #[cfg(feature = "bar")]
            extern crate bar;

            fn main() {
                #[cfg(feature = "bar")] {
                    println!("cargo:rustc-env=FOO={}", bar::bar());
                    return
                }
                println!("cargo:rustc-env=FOO=0");
            }
        "#,
        )
        .file(
            "src/main.rs",
            r#"
                #[cfg(feature = "bar")]
                extern crate bar;

                fn main() {
                    println!("{}", env!("FOO"));
                }
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file("bar/src/lib.rs", "pub fn bar() -> u32 { 1 }");
    let p = p.build();

    p.cargo("run").with_stdout("0\n").run();
    p.cargo("run --features bar").with_stdout("1\n").run();
}

#[test]
fn optional_build_dep_and_required_normal_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            bar = { path = "./bar", optional = true }

            [build-dependencies]
            bar = { path = "./bar" }
            "#,
        )
        .file("build.rs", "extern crate bar; fn main() { bar::bar(); }")
        .file(
            "src/main.rs",
            r#"
                #[cfg(feature = "bar")]
                extern crate bar;

                fn main() {
                    #[cfg(feature = "bar")] {
                        println!("{}", bar::bar());
                    }
                    #[cfg(not(feature = "bar"))] {
                        println!("0");
                    }
                }
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.5.0"))
        .file("bar/src/lib.rs", "pub fn bar() -> u32 { 1 }");
    let p = p.build();

    p.cargo("run")
        .with_stdout("0")
        .with_stderr(
            "\
[COMPILING] bar v0.5.0 ([..])
[COMPILING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]foo[EXE]`",
        )
        .run();

    p.cargo("run --all-features")
        .with_stdout("1")
        .with_stderr(
            "\
[COMPILING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]foo[EXE]`",
        )
        .run();
}
