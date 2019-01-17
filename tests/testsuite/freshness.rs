use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::SystemTime;

use crate::support::paths::CargoPathExt;
use crate::support::registry::Package;
use crate::support::sleep_ms;
use crate::support::{basic_manifest, is_coarse_mtime, project};

#[test]
fn modifying_and_moving() {
    let p = project()
        .file("src/main.rs", "mod a; fn main() {}")
        .file("src/a.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("build").with_stdout("").run();
    p.root().move_into_the_past();
    p.root().join("target").move_into_the_past();

    File::create(&p.root().join("src/a.rs"))
        .unwrap()
        .write_all(b"#[allow(unused)]fn main() {}")
        .unwrap();
    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    fs::rename(&p.root().join("src/a.rs"), &p.root().join("src/b.rs")).unwrap();
    p.cargo("build")
        .with_status(101)
        .with_stderr_contains("[..]file not found[..]")
        .run();
}

#[test]
fn modify_only_some_files() {
    let p = project()
        .file("src/lib.rs", "mod a;")
        .file("src/a.rs", "")
        .file("src/main.rs", "mod b; fn main() {}")
        .file("src/b.rs", "")
        .file("tests/test.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.cargo("test").run();
    sleep_ms(1000);

    assert!(p.bin("foo").is_file());

    let lib = p.root().join("src/lib.rs");
    let bin = p.root().join("src/b.rs");

    File::create(&lib)
        .unwrap()
        .write_all(b"invalid rust code")
        .unwrap();
    File::create(&bin)
        .unwrap()
        .write_all(b"#[allow(unused)]fn foo() {}")
        .unwrap();
    lib.move_into_the_past();

    // Make sure the binary is rebuilt, not the lib
    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    assert!(p.bin("foo").is_file());
}

#[test]
fn rebuild_sub_package_then_while_package() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"

            [dependencies.a]
            path = "a"
            [dependencies.b]
            path = "b"
        "#,
        )
        .file("src/lib.rs", "extern crate a; extern crate b;")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            authors = []
            version = "0.0.1"
            [dependencies.b]
            path = "../b"
        "#,
        )
        .file("a/src/lib.rs", "extern crate b;")
        .file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("build").run();

    File::create(&p.root().join("b/src/lib.rs"))
        .unwrap()
        .write_all(br#"pub fn b() {}"#)
        .unwrap();

    p.cargo("build -pb").run();

    File::create(&p.root().join("src/lib.rs"))
        .unwrap()
        .write_all(br#"extern crate a; extern crate b; pub fn toplevel() {}"#)
        .unwrap();

    p.cargo("build").run();
}

#[test]
fn changing_lib_features_caches_targets() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"

            [features]
            foo = []
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("build --features foo")
        .with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    /* Targets should be cached from the first build */

    p.cargo("build")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();

    p.cargo("build").with_stdout("").run();

    p.cargo("build --features foo")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
}

#[test]
fn changing_profiles_caches_targets() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"

            [profile.dev]
            panic = "abort"
        "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("test")
        .with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target[..]debug[..]deps[..]foo-[..][EXE]
[DOCTEST] foo
",
        )
        .run();

    /* Targets should be cached from the first build */

    p.cargo("build")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();

    p.cargo("test foo")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target[..]debug[..]deps[..]foo-[..][EXE]
[DOCTEST] foo
",
        )
        .run();
}

#[test]
fn changing_bin_paths_common_target_features_caches_targets() {
    // Make sure dep_cache crate is built once per feature
    let p = project()
        .no_manifest()
        .file(
            ".cargo/config",
            r#"
            [build]
            target-dir = "./target"
        "#,
        )
        .file(
            "dep_crate/Cargo.toml",
            r#"
            [package]
            name    = "dep_crate"
            version = "0.0.1"
            authors = []

            [features]
            ftest  = []
        "#,
        )
        .file(
            "dep_crate/src/lib.rs",
            r#"
            #[cfg(feature = "ftest")]
            pub fn yo() {
                println!("ftest on")
            }
            #[cfg(not(feature = "ftest"))]
            pub fn yo() {
                println!("ftest off")
            }
        "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name    = "a"
            version = "0.0.1"
            authors = []

            [dependencies]
            dep_crate = {path = "../dep_crate", features = []}
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "a/src/main.rs",
            r#"
            extern crate dep_crate;
            use dep_crate::yo;
            fn main() {
                yo();
            }
        "#,
        )
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name    = "b"
            version = "0.0.1"
            authors = []

            [dependencies]
            dep_crate = {path = "../dep_crate", features = ["ftest"]}
        "#,
        )
        .file("b/src/lib.rs", "")
        .file(
            "b/src/main.rs",
            r#"
            extern crate dep_crate;
            use dep_crate::yo;
            fn main() {
                yo();
            }
        "#,
        )
        .build();

    /* Build and rebuild a/. Ensure dep_crate only builds once */
    p.cargo("run")
        .cwd(p.root().join("a"))
        .with_stdout("ftest off")
        .with_stderr(
            "\
[..]Compiling dep_crate v0.0.1 ([..])
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/a[EXE]`
",
        )
        .run();
    p.cargo("clean -p a").cwd(p.root().join("a")).run();
    p.cargo("run")
        .cwd(p.root().join("a"))
        .with_stdout("ftest off")
        .with_stderr(
            "\
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/a[EXE]`
",
        )
        .run();

    /* Build and rebuild b/. Ensure dep_crate only builds once */
    p.cargo("run")
        .cwd(p.root().join("b"))
        .with_stdout("ftest on")
        .with_stderr(
            "\
[..]Compiling dep_crate v0.0.1 ([..])
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/b[EXE]`
",
        )
        .run();
    p.cargo("clean -p b").cwd(p.root().join("b")).run();
    p.cargo("run")
        .cwd(p.root().join("b"))
        .with_stdout("ftest on")
        .with_stderr(
            "\
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/b[EXE]`
",
        )
        .run();

    /* Build a/ package again. If we cache different feature dep builds correctly,
     * this should not cause a rebuild of dep_crate */
    p.cargo("clean -p a").cwd(p.root().join("a")).run();
    p.cargo("run")
        .cwd(p.root().join("a"))
        .with_stdout("ftest off")
        .with_stderr(
            "\
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/a[EXE]`
",
        )
        .run();

    /* Build b/ package again. If we cache different feature dep builds correctly,
     * this should not cause a rebuild */
    p.cargo("clean -p b").cwd(p.root().join("b")).run();
    p.cargo("run")
        .cwd(p.root().join("b"))
        .with_stdout("ftest on")
        .with_stderr(
            "\
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target/debug/b[EXE]`
",
        )
        .run();
}

#[test] fn changing_bin_features_caches_targets0() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets1() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets2() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets3() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets4() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets5() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets6() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets7() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets8() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets9() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets10() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets11() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets12() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets13() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets14() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets15() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets16() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets17() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets18() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets19() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets20() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets21() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets22() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets23() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets24() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets25() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets26() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets27() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets28() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets29() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets30() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets31() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets32() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets33() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets34() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets35() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets36() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets37() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets38() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets39() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets40() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets41() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets42() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets43() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets44() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets45() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets46() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets47() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets48() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets49() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets50() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets51() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets52() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets53() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets54() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets55() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets56() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets57() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets58() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets59() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets60() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets61() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets62() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets63() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets64() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets65() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets66() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets67() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets68() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets69() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets70() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets71() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets72() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets73() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets74() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets75() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets76() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets77() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets78() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets79() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets80() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets81() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets82() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets83() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets84() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets85() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets86() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets87() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets88() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets89() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets90() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets91() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets92() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets93() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets94() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets95() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets96() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets97() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets98() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets99() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets100() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets101() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets102() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets103() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets104() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets105() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets106() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets107() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets108() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets109() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets110() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets111() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets112() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets113() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets114() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets115() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets116() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets117() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets118() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets119() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets120() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets121() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets122() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets123() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets124() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets125() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets126() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets127() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets128() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets129() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets130() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets131() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets132() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets133() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets134() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets135() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets136() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets137() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets138() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets139() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets140() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets141() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets142() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets143() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets144() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets145() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets146() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets147() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets148() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets149() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets150() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets151() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets152() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets153() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets154() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets155() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets156() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets157() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets158() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets159() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets160() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets161() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets162() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets163() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets164() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets165() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets166() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets167() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets168() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets169() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets170() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets171() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets172() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets173() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets174() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets175() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets176() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets177() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets178() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets179() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets180() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets181() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets182() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets183() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets184() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets185() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets186() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets187() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets188() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets189() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets190() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets191() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets192() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets193() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets194() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets195() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets196() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets197() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets198() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets199() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets200() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets201() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets202() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets203() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets204() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets205() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets206() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets207() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets208() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets209() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets210() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets211() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets212() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets213() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets214() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets215() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets216() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets217() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets218() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets219() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets220() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets221() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets222() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets223() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets224() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets225() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets226() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets227() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets228() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets229() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets230() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets231() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets232() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets233() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets234() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets235() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets236() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets237() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets238() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets239() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets240() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets241() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets242() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets243() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets244() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets245() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets246() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets247() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets248() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets249() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets250() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets251() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets252() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets253() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets254() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets255() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets256() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets257() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets258() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets259() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets260() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets261() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets262() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets263() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets264() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets265() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets266() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets267() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets268() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets269() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets270() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets271() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets272() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets273() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets274() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets275() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets276() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets277() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets278() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets279() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets280() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets281() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets282() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets283() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets284() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets285() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets286() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets287() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets288() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets289() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets290() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets291() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets292() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets293() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets294() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets295() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets296() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets297() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets298() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets299() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets300() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets301() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets302() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets303() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets304() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets305() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets306() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets307() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets308() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets309() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets310() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets311() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets312() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets313() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets314() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets315() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets316() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets317() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets318() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets319() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets320() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets321() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets322() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets323() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets324() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets325() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets326() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets327() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets328() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets329() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets330() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets331() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets332() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets333() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets334() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets335() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets336() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets337() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets338() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets339() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets340() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets341() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets342() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets343() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets344() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets345() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets346() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets347() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets348() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets349() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets350() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets351() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets352() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets353() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets354() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets355() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets356() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets357() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets358() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets359() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets360() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets361() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets362() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets363() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets364() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets365() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets366() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets367() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets368() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets369() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets370() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets371() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets372() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets373() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets374() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets375() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets376() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets377() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets378() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets379() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets380() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets381() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets382() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets383() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets384() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets385() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets386() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets387() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets388() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets389() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets390() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets391() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets392() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets393() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets394() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets395() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets396() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets397() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets398() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets399() { changing_bin_features_caches_targets(); }

#[test]
fn changing_bin_features_caches_targets() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"

            [features]
            foo = []
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                let msg = if cfg!(feature = "foo") { "feature on" } else { "feature off" };
                println!("{}", msg);
            }
        "#,
        )
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.safely_rename_run("foo", "off1").with_stdout("feature off").run();

    p.cargo("build --features foo")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.safely_rename_run("foo", "on1").with_stdout("feature on").run();

    /* Targets should be cached from the first build */

    p.cargo("build")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.safely_rename_run("foo", "off2").with_stdout("feature off").run();

    p.cargo("build --features foo")
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
    p.safely_rename_run("foo", "on2").with_stdout("feature on").run();
}

#[test]
fn rebuild_tests_if_lib_changes() {
    let p = project()
        .file("src/lib.rs", "pub fn foo() {}")
        .file(
            "tests/foo.rs",
            r#"
            extern crate foo;
            #[test]
            fn test() { foo::foo(); }
        "#,
        )
        .build();

    p.cargo("build").run();
    p.cargo("test").run();

    sleep_ms(1000);
    File::create(&p.root().join("src/lib.rs")).unwrap();

    p.cargo("build -v").run();
    p.cargo("test -v")
        .with_status(101)
        .with_stderr_contains("[..]cannot find function `foo`[..]")
        .run();
}

#[test]
fn no_rebuild_transitive_target_deps() {
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
            [dev-dependencies]
            b = { path = "b" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("tests/foo.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            authors = []

            [target.foo.dependencies]
            c = { path = "../c" }
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.0.1"
            authors = []

            [dependencies]
            c = { path = "../c" }
        "#,
        )
        .file("b/src/lib.rs", "")
        .file("c/Cargo.toml", &basic_manifest("c", "0.0.1"))
        .file("c/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("test --no-run")
        .with_stderr(
            "\
[COMPILING] c v0.0.1 ([..])
[COMPILING] b v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn rerun_if_changed_in_dep() {
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
            build = "build.rs"
        "#,
        )
        .file(
            "a/build.rs",
            r#"
            fn main() {
                println!("cargo:rerun-if-changed=build.rs");
            }
        "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build").with_stdout("").run();
}

#[test]
fn same_build_dir_cached_packages() {
    let p = project()
        .no_manifest()
        .file(
            "a1/Cargo.toml",
            r#"
            [package]
            name = "a1"
            version = "0.0.1"
            authors = []
            [dependencies]
            b = { path = "../b" }
        "#,
        )
        .file("a1/src/lib.rs", "")
        .file(
            "a2/Cargo.toml",
            r#"
            [package]
            name = "a2"
            version = "0.0.1"
            authors = []
            [dependencies]
            b = { path = "../b" }
        "#,
        )
        .file("a2/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "0.0.1"
            authors = []
            [dependencies]
            c = { path = "../c" }
        "#,
        )
        .file("b/src/lib.rs", "")
        .file(
            "c/Cargo.toml",
            r#"
            [package]
            name = "c"
            version = "0.0.1"
            authors = []
            [dependencies]
            d = { path = "../d" }
        "#,
        )
        .file("c/src/lib.rs", "")
        .file("d/Cargo.toml", &basic_manifest("d", "0.0.1"))
        .file("d/src/lib.rs", "")
        .file(
            ".cargo/config",
            r#"
            [build]
            target-dir = "./target"
        "#,
        )
        .build();

    p.cargo("build")
        .cwd(p.root().join("a1"))
        .with_stderr(&format!(
            "\
[COMPILING] d v0.0.1 ({dir}/d)
[COMPILING] c v0.0.1 ({dir}/c)
[COMPILING] b v0.0.1 ({dir}/b)
[COMPILING] a1 v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = p.url().to_file_path().unwrap().to_str().unwrap()
        ))
        .run();
    p.cargo("build")
        .cwd(p.root().join("a2"))
        .with_stderr(
            "\
[COMPILING] a2 v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[test]
fn no_rebuild_if_build_artifacts_move_backwards_in_time() {
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
        "#,
        )
        .file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build").run();

    p.root().move_into_the_past();

    p.cargo("build")
        .with_stdout("")
        .with_stderr("[FINISHED] [..]")
        .run();
}

#[test]
fn rebuild_if_build_artifacts_move_forward_in_time() {
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
        "#,
        )
        .file("src/lib.rs", "")
        .file("a/Cargo.toml", &basic_manifest("a", "0.0.1"))
        .file("a/src/lib.rs", "")
        .build();

    p.cargo("build").run();

    p.root().move_into_the_future();

    p.cargo("build")
        .env("RUST_LOG", "")
        .with_stdout("")
        .with_stderr(
            "\
[COMPILING] a v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn rebuild_if_environment_changes() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            description = "old desc"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() {
                println!("{}", env!("CARGO_PKG_DESCRIPTION"));
            }
        "#,
        )
        .build();

    p.cargo("run")
        .with_stdout("old desc")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target/debug/foo[EXE]`
",
        )
        .run();

    File::create(&p.root().join("Cargo.toml"))
        .unwrap()
        .write_all(
            br#"
        [package]
        name = "foo"
        description = "new desc"
        version = "0.0.1"
        authors = []
    "#,
        )
        .unwrap();

    p.cargo("run")
        .with_stdout("new desc")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target/debug/foo[EXE]`
",
        )
        .run();
}

#[test]
fn no_rebuild_when_rename_dir() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []

            [dependencies]
            foo = { path = "foo" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("foo/Cargo.toml", &basic_manifest("foo", "0.0.1"))
        .file("foo/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    let mut new = p.root();
    new.pop();
    new.push("bar");
    fs::rename(p.root(), &new).unwrap();

    p.cargo("build")
        .cwd(&new)
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
}

#[test]
fn unused_optional_dep() {
    Package::new("registry1", "0.1.0").publish();
    Package::new("registry2", "0.1.0").publish();
    Package::new("registry3", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "p"
                authors = []
                version = "0.1.0"

                [dependencies]
                bar = { path = "bar" }
                baz = { path = "baz" }
                registry1 = "*"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"
                authors = []

                [dev-dependencies]
                registry2 = "*"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"
                authors = []

                [dependencies]
                registry3 = { version = "*", optional = true }
            "#,
        )
        .file("baz/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build").with_stderr("[FINISHED] [..]").run();
}

#[test]
fn path_dev_dep_registry_updates() {
    Package::new("registry1", "0.1.0").publish();
    Package::new("registry2", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "p"
                authors = []
                version = "0.1.0"

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
                version = "0.1.1"
                authors = []

                [dependencies]
                registry1 = "*"

                [dev-dependencies]
                baz = { path = "../baz"}
            "#,
        )
        .file("bar/src/lib.rs", "")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"
                authors = []

                [dependencies]
                registry2 = "*"
            "#,
        )
        .file("baz/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build").with_stderr("[FINISHED] [..]").run();
}

#[test]
fn change_panic_mode() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ['bar', 'baz']
                [profile.dev]
                panic = 'abort'
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.1"))
        .file("bar/src/lib.rs", "")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"
                authors = []

                [lib]
                proc-macro = true

                [dependencies]
                bar = { path = '../bar' }
            "#,
        )
        .file("baz/src/lib.rs", "extern crate bar;")
        .build();

    p.cargo("build -p bar").run();
    p.cargo("build -p baz").run();
}

#[test]
fn dont_rebuild_based_on_plugins() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"

                [workspace]
                members = ['baz']

                [dependencies]
                proc-macro-thing = { path = 'proc-macro-thing' }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "proc-macro-thing/Cargo.toml",
            r#"
                [package]
                name = "proc-macro-thing"
                version = "0.1.1"

                [lib]
                proc-macro = true

                [dependencies]
                qux = { path = '../qux' }
            "#,
        )
        .file("proc-macro-thing/src/lib.rs", "")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"

                [dependencies]
                qux = { path = '../qux' }
            "#,
        )
        .file("baz/src/main.rs", "fn main() {}")
        .file("qux/Cargo.toml", &basic_manifest("qux", "0.1.1"))
        .file("qux/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build -p baz").run();
    p.cargo("build").with_stderr("[FINISHED] [..]\n").run();
    p.cargo("build -p bar")
        .with_stderr("[FINISHED] [..]\n")
        .run();
}

#[test]
fn reuse_workspace_lib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"

                [workspace]

                [dependencies]
                baz = { path = 'baz' }
            "#,
        )
        .file("src/lib.rs", "")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.1"))
        .file("baz/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("test -p baz -v --no-run")
        .with_stderr(
            "\
[COMPILING] baz v0.1.1 ([..])
[RUNNING] `rustc[..] --test [..]`
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn reuse_shared_build_dep() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            shared = {path = "shared"}

            [workspace]
            members = ["shared", "bar"]
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("shared/Cargo.toml", &basic_manifest("shared", "0.0.1"))
        .file("shared/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"

            [build-dependencies]
            shared = { path = "../shared" }
        "#,
        )
        .file("bar/src/lib.rs", "")
        .file("bar/build.rs", "fn main() {}")
        .build();

    p.cargo("build --all").run();
    // This should not recompile!
    p.cargo("build -p foo -v")
        .with_stderr(
            "\
[FRESH] shared [..]
[FRESH] foo [..]
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn changing_rustflags_is_cached() {
    let p = project().file("src/lib.rs", "").build();

    p.cargo("build").run();
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
    // This should not recompile!
    p.cargo("build")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
}

fn simple_deps_cleaner(mut dir: PathBuf, timestamp: filetime::FileTime) {
    // Cargo is experimenting with letting outside projects develop some
    // limited forms of GC for target_dir. This is one of the forms.
    // Specifically, Cargo is updating the mtime of files in
    // target/profile/deps each time it uses the file.
    // So a cleaner can remove files older then a time stamp without
    // effecting any builds that happened since that time stamp.
    let mut cleand = false;
    dir.push("deps");
    for dep in fs::read_dir(&dir).unwrap() {
        let dep = dep.unwrap();
        if filetime::FileTime::from_last_modification_time(&dep.metadata().unwrap()) <= timestamp {
            fs::remove_file(dep.path()).unwrap();
            println!("remove: {:?}", dep.path());
            cleand = true;
        }
    }
    assert!(
        cleand,
        "called simple_deps_cleaner, but there was nothing to remove"
    );
}

#[test]
fn simple_deps_cleaner_dose_not_rebuild() {
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
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
    if is_coarse_mtime() {
        sleep_ms(1000);
    }
    let timestamp = filetime::FileTime::from_system_time(SystemTime::now());
    if is_coarse_mtime() {
        sleep_ms(1000);
    }
    // This dose not make new files, but it dose update the mtime.
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
    simple_deps_cleaner(p.target_debug_dir(), timestamp);
    // This should not recompile!
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
    // But this should be cleaned and so need a rebuild
    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

fn fingerprint_cleaner(mut dir: PathBuf, timestamp: filetime::FileTime) {
    // Cargo is experimenting with letting outside projects develop some
    // limited forms of GC for target_dir. This is one of the forms.
    // Specifically, Cargo is updating the mtime of a file in
    // target/profile/.fingerprint each time it uses the fingerprint.
    // So a cleaner can remove files associated with a fingerprint
    // if all the files in the fingerprint's folder are older then a time stamp without
    // effecting any builds that happened since that time stamp.
    let mut cleand = false;
    dir.push(".fingerprint");
    for fing in fs::read_dir(&dir).unwrap() {
        let fing = fing.unwrap();

        if fs::read_dir(fing.path()).unwrap().all(|f| {
            filetime::FileTime::from_last_modification_time(&f.unwrap().metadata().unwrap())
                <= timestamp
        }) {
            fs::remove_dir_all(fing.path()).unwrap();
            println!("remove: {:?}", fing.path());
            // a real cleaner would remove the big files in deps and build as well
            // but fingerprint is sufficient for our tests
            cleand = true;
        } else {
        }
    }
    assert!(
        cleand,
        "called fingerprint_cleaner, but there was nothing to remove"
    );
}

#[test]
fn fingerprint_cleaner_dose_not_rebuild() {
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
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
    if is_coarse_mtime() {
        sleep_ms(1000);
    }
    let timestamp = filetime::FileTime::from_system_time(SystemTime::now());
    if is_coarse_mtime() {
        sleep_ms(1000);
    }
    // This dose not make new files, but it dose update the mtime.
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
    fingerprint_cleaner(p.target_debug_dir(), timestamp);
    // This should not recompile!
    p.cargo("build")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
    // But this should be cleaned and so need a rebuild
    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

#[test]
fn reuse_panic_build_dep_test() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [build-dependencies]
            bar = { path = "bar" }

            [dev-dependencies]
            bar = { path = "bar" }

            [profile.dev]
            panic = "abort"
        "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() {}")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .build();

    // Check that `bar` is not built twice. It is only needed once (without `panic`).
    p.cargo("test --lib --no-run -v")
        .with_stderr(
            "\
[COMPILING] bar [..]
[RUNNING] `rustc --crate-name bar [..]
[COMPILING] foo [..]
[RUNNING] `rustc --crate-name build_script_build [..]
[RUNNING] [..]build-script-build`
[RUNNING] `rustc --crate-name foo src/lib.rs [..]--test[..]
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn reuse_panic_pm() {
    // foo(panic) -> bar(panic)
    // somepm(nopanic) -> bar(nopanic)
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            bar = { path = "bar" }
            somepm = { path = "somepm" }

            [profile.dev]
            panic = "abort"
        "#,
        )
        .file("src/lib.rs", "extern crate bar;")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.0.1"))
        .file("bar/src/lib.rs", "")
        .file(
            "somepm/Cargo.toml",
            r#"
            [package]
            name = "somepm"
            version = "0.0.1"

            [lib]
            proc-macro = true

            [dependencies]
            bar = { path = "../bar" }
        "#,
        )
        .file("somepm/src/lib.rs", "extern crate bar;")
        .build();

    // bar is built once without panic (for proc-macro) and once with (for the
    // normal dependency).
    p.cargo("build -v")
        .with_stderr_unordered(
            "\
[COMPILING] bar [..]
[RUNNING] `rustc --crate-name bar bar/src/lib.rs [..]--crate-type lib --emit=dep-info,link -C debuginfo=2 [..]
[RUNNING] `rustc --crate-name bar bar/src/lib.rs [..]--crate-type lib --emit=dep-info,link -C panic=abort -C debuginfo=2 [..]
[COMPILING] somepm [..]
[RUNNING] `rustc --crate-name somepm [..]
[COMPILING] foo [..]
[RUNNING] `rustc --crate-name foo src/lib.rs [..]-C panic=abort[..]
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn bust_patched_dep() {
    Package::new("registry1", "0.1.0").publish();
    Package::new("registry2", "0.1.0")
        .dep("registry1", "0.1.0")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            registry2 = "0.1.0"

            [patch.crates-io]
            registry1 = { path = "reg1new" }
        "#,
        )
        .file("src/lib.rs", "")
        .file("reg1new/Cargo.toml", &basic_manifest("registry1", "0.1.0"))
        .file("reg1new/src/lib.rs", "")
        .build();

    p.cargo("build").run();
    if is_coarse_mtime() {
        sleep_ms(1000);
    }

    File::create(&p.root().join("reg1new/src/lib.rs")).unwrap();
    if is_coarse_mtime() {
        sleep_ms(1000);
    }

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] registry1 v0.1.0 ([..])
[COMPILING] registry2 v0.1.0
[COMPILING] foo v0.0.1 ([..])
[FINISHED] [..]
",
        )
        .run();

    p.cargo("build -v")
        .with_stderr(
            "\
[FRESH] registry1 v0.1.0 ([..])
[FRESH] registry2 v0.1.0
[FRESH] foo v0.0.1 ([..])
[FINISHED] [..]
",
        )
        .run();
}

#[test]
fn rebuild_on_mid_build_file_modification() {
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["root", "proc_macro_dep"]
        "#,
        )
        .file(
            "root/Cargo.toml",
            r#"
            [project]
            name = "root"
            version = "0.1.0"
            authors = []

            [dependencies]
            proc_macro_dep = { path = "../proc_macro_dep" }
        "#,
        )
        .file(
            "root/src/lib.rs",
            r#"
            #[macro_use]
            extern crate proc_macro_dep;

            #[derive(Noop)]
            pub struct X;
        "#,
        )
        .file(
            "proc_macro_dep/Cargo.toml",
            r#"
            [project]
            name = "proc_macro_dep"
            version = "0.1.0"
            authors = []

            [lib]
            proc-macro = true
        "#,
        )
        .file(
            "proc_macro_dep/src/lib.rs",
            &format!(
                r#"
                extern crate proc_macro;

                use std::io::Read;
                use std::net::TcpStream;
                use proc_macro::TokenStream;

                #[proc_macro_derive(Noop)]
                pub fn noop(_input: TokenStream) -> TokenStream {{
                    let mut stream = TcpStream::connect("{}").unwrap();
                    let mut v = Vec::new();
                    stream.read_to_end(&mut v).unwrap();
                    "".parse().unwrap()
                }}
            "#,
                addr
            ),
        )
        .build();
    let root = p.root();

    let t = thread::spawn(move || {
        let socket = server.accept().unwrap().0;
        sleep_ms(1000);
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(root.join("root/src/lib.rs"))
            .unwrap();
        writeln!(file, "// modified").expect("Failed to append to root sources");
        drop(file);
        drop(socket);
        drop(server.accept().unwrap());
    });

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] proc_macro_dep v0.1.0 ([..]/proc_macro_dep)
[COMPILING] root v0.1.0 ([..]/root)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] root v0.1.0 ([..]/root)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    t.join().ok().unwrap();
}
