use std::fs::{self, File};
use std::io::prelude::*;

use cargotest::sleep_ms;
use cargotest::support::paths::CargoPathExt;
use cargotest::support::registry::Package;
use cargotest::support::{execs, path2url, project};
use hamcrest::{assert_that, existing_file};

#[test]
fn modifying_and_moving() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            mod a; fn main() {}
        "#,
        )
        .file("src/a.rs", "")
        .build();

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(format!(
            "\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = path2url(p.root())
        )),
    );

    assert_that(p.cargo("build"), execs().with_status(0).with_stdout(""));
    p.root().move_into_the_past();
    p.root().join("target").move_into_the_past();

    File::create(&p.root().join("src/a.rs"))
        .unwrap()
        .write_all(b"#[allow(unused)]fn main() {}")
        .unwrap();
    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(format!(
            "\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = path2url(p.root())
        )),
    );

    fs::rename(&p.root().join("src/a.rs"), &p.root().join("src/b.rs")).unwrap();
    assert_that(p.cargo("build"), execs().with_status(101));
}

#[test]
fn modify_only_some_files() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.1"
        "#,
        )
        .file("src/lib.rs", "mod a;")
        .file("src/a.rs", "")
        .file(
            "src/main.rs",
            r#"
            mod b;
            fn main() {}
        "#,
        )
        .file("src/b.rs", "")
        .file("tests/test.rs", "")
        .build();

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(format!(
            "\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = path2url(p.root())
        )),
    );
    assert_that(p.cargo("test"), execs().with_status(0));
    sleep_ms(1000);

    assert_that(&p.bin("foo"), existing_file());

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
    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(format!(
            "\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = path2url(p.root())
        )),
    );
    assert_that(&p.bin("foo"), existing_file());
}

#[test]
fn rebuild_sub_package_then_while_package() {
    let p = project("foo")
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
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            authors = []
            version = "0.0.1"
        "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));

    File::create(&p.root().join("b/src/lib.rs"))
        .unwrap()
        .write_all(
            br#"
        pub fn b() {}
    "#,
        )
        .unwrap();

    assert_that(p.cargo("build").arg("-pb"), execs().with_status(0));

    File::create(&p.root().join("src/lib.rs"))
        .unwrap()
        .write_all(
            br#"
        extern crate a;
        extern crate b;
        pub fn toplevel() {}
    "#,
        )
        .unwrap();

    assert_that(p.cargo("build"), execs().with_status(0));
}

#[test]
fn changing_lib_features_caches_targets() {
    let p = project("foo")
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

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );

    assert_that(
        p.cargo("build").arg("--features").arg("foo"),
        execs().with_status(0).with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );

    /* Targets should be cached from the first build */

    assert_that(
        p.cargo("build"),
        execs()
            .with_status(0)
            .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]"),
    );

    assert_that(p.cargo("build"), execs().with_status(0).with_stdout(""));

    assert_that(
        p.cargo("build").arg("--features").arg("foo"),
        execs()
            .with_status(0)
            .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]"),
    );
}

#[test]
fn changing_profiles_caches_targets() {
    let p = project("foo")
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

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );

    assert_that(
        p.cargo("test"),
        execs().with_status(0).with_stderr(
            "\
[..]Compiling foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target[..]debug[..]deps[..]foo-[..][EXE]
[DOCTEST] foo
",
        ),
    );

    /* Targets should be cached from the first build */

    assert_that(
        p.cargo("build"),
        execs()
            .with_status(0)
            .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]"),
    );

    assert_that(
        p.cargo("test").arg("foo"),
        execs().with_status(0).with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] target[..]debug[..]deps[..]foo-[..][EXE]
[DOCTEST] foo
",
        ),
    );
}

#[test]
fn changing_bin_paths_common_target_features_caches_targets() {
    // Make sure dep_cache crate is built once per feature
    let p = project("foo")
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
    assert_that(
        p.cargo("run").cwd(p.root().join("a")),
        execs().with_status(0).with_stdout("ftest off").with_stderr(
            "\
[..]Compiling dep_crate v0.0.1 ([..])
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]a[EXE]`
",
        ),
    );
    assert_that(
        p.cargo("clean").arg("-p").arg("a").cwd(p.root().join("a")),
        execs().with_status(0),
    );
    assert_that(
        p.cargo("run").cwd(p.root().join("a")),
        execs().with_status(0).with_stdout("ftest off").with_stderr(
            "\
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]a[EXE]`
",
        ),
    );

    /* Build and rebuild b/. Ensure dep_crate only builds once */
    assert_that(
        p.cargo("run").cwd(p.root().join("b")),
        execs().with_status(0).with_stdout("ftest on").with_stderr(
            "\
[..]Compiling dep_crate v0.0.1 ([..])
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]b[EXE]`
",
        ),
    );
    assert_that(
        p.cargo("clean").arg("-p").arg("b").cwd(p.root().join("b")),
        execs().with_status(0),
    );
    assert_that(
        p.cargo("run").cwd(p.root().join("b")),
        execs().with_status(0).with_stdout("ftest on").with_stderr(
            "\
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]b[EXE]`
",
        ),
    );

    /* Build a/ package again. If we cache different feature dep builds correctly,
     * this should not cause a rebuild of dep_crate */
    assert_that(
        p.cargo("clean").arg("-p").arg("a").cwd(p.root().join("a")),
        execs().with_status(0),
    );
    assert_that(
        p.cargo("run").cwd(p.root().join("a")),
        execs().with_status(0).with_stdout("ftest off").with_stderr(
            "\
[..]Compiling a v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]a[EXE]`
",
        ),
    );

    /* Build b/ package again. If we cache different feature dep builds correctly,
     * this should not cause a rebuild */
    assert_that(
        p.cargo("clean").arg("-p").arg("b").cwd(p.root().join("b")),
        execs().with_status(0),
    );
    assert_that(
        p.cargo("run").cwd(p.root().join("b")),
        execs().with_status(0).with_stdout("ftest on").with_stderr(
            "\
[..]Compiling b v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `[..]target[/]debug[/]b[EXE]`
",
        ),
    );
}

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
#[test] fn changing_bin_features_caches_targets400() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets401() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets402() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets403() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets404() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets405() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets406() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets407() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets408() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets409() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets410() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets411() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets412() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets413() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets414() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets415() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets416() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets417() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets418() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets419() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets420() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets421() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets422() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets423() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets424() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets425() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets426() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets427() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets428() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets429() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets430() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets431() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets432() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets433() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets434() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets435() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets436() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets437() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets438() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets439() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets440() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets441() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets442() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets443() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets444() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets445() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets446() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets447() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets448() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets449() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets450() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets451() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets452() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets453() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets454() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets455() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets456() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets457() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets458() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets459() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets460() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets461() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets462() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets463() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets464() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets465() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets466() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets467() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets468() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets469() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets470() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets471() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets472() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets473() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets474() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets475() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets476() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets477() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets478() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets479() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets480() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets481() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets482() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets483() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets484() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets485() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets486() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets487() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets488() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets489() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets490() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets491() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets492() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets493() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets494() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets495() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets496() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets497() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets498() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets499() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets500() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets501() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets502() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets503() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets504() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets505() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets506() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets507() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets508() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets509() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets510() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets511() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets512() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets513() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets514() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets515() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets516() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets517() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets518() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets519() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets520() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets521() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets522() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets523() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets524() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets525() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets526() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets527() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets528() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets529() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets530() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets531() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets532() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets533() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets534() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets535() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets536() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets537() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets538() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets539() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets540() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets541() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets542() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets543() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets544() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets545() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets546() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets547() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets548() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets549() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets550() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets551() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets552() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets553() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets554() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets555() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets556() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets557() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets558() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets559() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets560() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets561() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets562() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets563() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets564() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets565() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets566() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets567() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets568() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets569() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets570() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets571() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets572() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets573() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets574() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets575() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets576() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets577() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets578() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets579() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets580() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets581() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets582() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets583() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets584() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets585() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets586() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets587() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets588() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets589() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets590() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets591() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets592() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets593() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets594() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets595() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets596() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets597() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets598() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets599() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets600() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets601() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets602() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets603() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets604() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets605() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets606() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets607() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets608() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets609() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets610() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets611() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets612() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets613() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets614() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets615() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets616() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets617() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets618() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets619() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets620() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets621() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets622() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets623() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets624() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets625() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets626() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets627() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets628() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets629() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets630() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets631() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets632() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets633() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets634() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets635() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets636() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets637() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets638() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets639() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets640() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets641() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets642() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets643() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets644() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets645() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets646() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets647() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets648() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets649() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets650() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets651() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets652() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets653() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets654() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets655() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets656() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets657() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets658() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets659() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets660() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets661() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets662() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets663() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets664() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets665() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets666() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets667() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets668() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets669() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets670() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets671() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets672() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets673() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets674() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets675() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets676() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets677() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets678() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets679() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets680() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets681() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets682() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets683() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets684() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets685() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets686() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets687() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets688() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets689() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets690() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets691() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets692() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets693() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets694() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets695() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets696() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets697() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets698() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets699() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets700() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets701() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets702() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets703() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets704() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets705() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets706() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets707() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets708() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets709() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets710() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets711() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets712() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets713() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets714() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets715() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets716() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets717() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets718() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets719() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets720() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets721() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets722() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets723() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets724() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets725() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets726() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets727() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets728() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets729() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets730() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets731() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets732() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets733() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets734() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets735() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets736() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets737() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets738() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets739() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets740() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets741() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets742() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets743() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets744() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets745() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets746() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets747() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets748() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets749() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets750() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets751() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets752() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets753() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets754() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets755() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets756() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets757() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets758() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets759() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets760() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets761() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets762() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets763() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets764() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets765() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets766() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets767() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets768() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets769() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets770() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets771() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets772() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets773() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets774() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets775() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets776() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets777() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets778() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets779() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets780() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets781() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets782() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets783() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets784() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets785() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets786() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets787() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets788() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets789() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets790() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets791() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets792() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets793() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets794() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets795() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets796() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets797() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets798() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets799() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets800() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets801() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets802() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets803() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets804() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets805() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets806() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets807() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets808() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets809() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets810() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets811() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets812() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets813() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets814() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets815() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets816() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets817() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets818() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets819() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets820() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets821() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets822() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets823() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets824() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets825() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets826() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets827() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets828() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets829() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets830() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets831() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets832() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets833() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets834() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets835() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets836() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets837() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets838() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets839() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets840() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets841() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets842() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets843() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets844() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets845() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets846() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets847() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets848() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets849() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets850() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets851() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets852() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets853() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets854() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets855() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets856() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets857() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets858() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets859() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets860() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets861() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets862() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets863() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets864() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets865() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets866() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets867() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets868() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets869() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets870() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets871() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets872() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets873() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets874() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets875() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets876() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets877() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets878() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets879() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets880() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets881() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets882() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets883() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets884() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets885() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets886() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets887() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets888() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets889() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets890() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets891() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets892() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets893() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets894() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets895() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets896() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets897() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets898() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets899() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets900() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets901() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets902() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets903() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets904() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets905() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets906() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets907() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets908() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets909() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets910() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets911() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets912() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets913() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets914() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets915() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets916() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets917() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets918() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets919() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets920() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets921() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets922() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets923() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets924() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets925() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets926() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets927() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets928() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets929() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets930() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets931() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets932() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets933() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets934() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets935() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets936() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets937() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets938() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets939() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets940() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets941() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets942() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets943() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets944() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets945() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets946() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets947() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets948() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets949() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets950() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets951() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets952() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets953() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets954() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets955() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets956() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets957() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets958() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets959() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets960() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets961() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets962() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets963() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets964() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets965() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets966() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets967() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets968() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets969() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets970() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets971() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets972() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets973() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets974() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets975() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets976() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets977() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets978() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets979() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets980() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets981() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets982() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets983() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets984() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets985() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets986() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets987() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets988() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets989() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets990() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets991() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets992() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets993() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets994() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets995() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets996() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets997() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets998() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets999() { changing_bin_features_caches_targets(); }
#[test] fn changing_bin_features_caches_targets1000() { changing_bin_features_caches_targets(); }

#[test]
fn changing_bin_features_caches_targets() {
    let p = project("foo")
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

    // Windows has a problem with replacing a binary that was just executed.
    // Unlinking it will succeed, but then attempting to immediately replace
    // it will sometimes fail with "Already Exists".
    // See https://github.com/rust-lang/cargo/issues/5481
    let foo_proc = |name: &str| {
        let src = p.bin("foo");
        let dst = p.bin(name);
        fs::rename(&src, &dst).expect("Failed to link foo");
        p.process(dst)
    };

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );
    assert_that(
        foo_proc("off1"),
        execs().with_status(0).with_stdout("feature off"),
    );

    assert_that(
        p.cargo("build").arg("--features").arg("foo"),
        execs().with_status(0).with_stderr(
            "\
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );
    assert_that(
        foo_proc("on1"),
        execs().with_status(0).with_stdout("feature on"),
    );

    /* Targets should be cached from the first build */

    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );
    assert_that(
        foo_proc("off2"),
        execs().with_status(0).with_stdout("feature off"),
    );

    assert_that(
        p.cargo("build").arg("--features").arg("foo"),
        execs().with_status(0).with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );
    assert_that(
        foo_proc("on2"),
        execs().with_status(0).with_stdout("feature on"),
    );
}

#[test]
fn rebuild_tests_if_lib_changes() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#,
        )
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

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(p.cargo("test"), execs().with_status(0));

    sleep_ms(1000);
    File::create(&p.root().join("src/lib.rs")).unwrap();

    assert_that(p.cargo("build").arg("-v"), execs().with_status(0));
    assert_that(p.cargo("test").arg("-v"), execs().with_status(101));
}

#[test]
fn no_rebuild_transitive_target_deps() {
    let p = project("foo")
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
        .file(
            "c/Cargo.toml",
            r#"
            [package]
            name = "c"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("c/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(
        p.cargo("test").arg("--no-run"),
        execs().with_status(0).with_stderr(
            "\
[COMPILING] c v0.0.1 ([..])
[COMPILING] b v0.0.1 ([..])
[COMPILING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        ),
    );
}

#[test]
fn rerun_if_changed_in_dep() {
    let p = project("foo")
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

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(p.cargo("build"), execs().with_status(0).with_stdout(""));
}

#[test]
fn same_build_dir_cached_packages() {
    let p = project("foo")
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
        .file(
            "d/Cargo.toml",
            r#"
            [package]
            name = "d"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("d/src/lib.rs", "")
        .file(
            ".cargo/config",
            r#"
            [build]
            target-dir = "./target"
        "#,
        )
        .build();

    assert_that(
        p.cargo("build").cwd(p.root().join("a1")),
        execs().with_status(0).with_stderr(&format!(
            "\
[COMPILING] d v0.0.1 ({dir}/d)
[COMPILING] c v0.0.1 ({dir}/c)
[COMPILING] b v0.0.1 ({dir}/b)
[COMPILING] a1 v0.0.1 ({dir}/a1)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = p.url()
        )),
    );
    assert_that(
        p.cargo("build").cwd(p.root().join("a2")),
        execs().with_status(0).with_stderr(&format!(
            "\
[COMPILING] a2 v0.0.1 ({dir}/a2)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
            dir = p.url()
        )),
    );
}

#[test]
fn no_rebuild_if_build_artifacts_move_backwards_in_time() {
    let p = project("backwards_in_time")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "backwards_in_time"
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
        "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));

    p.root().move_into_the_past();

    assert_that(
        p.cargo("build"),
        execs()
            .with_status(0)
            .with_stdout("")
            .with_stderr("[FINISHED] [..]"),
    );
}

#[test]
fn rebuild_if_build_artifacts_move_forward_in_time() {
    let p = project("forwards_in_time")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "forwards_in_time"
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
        "#,
        )
        .file("a/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));

    p.root().move_into_the_future();

    assert_that(
        p.cargo("build").env("RUST_LOG", ""),
        execs().with_status(0).with_stdout("").with_stderr(
            "\
[COMPILING] a v0.0.1 ([..])
[COMPILING] forwards_in_time v0.0.1 ([..])
[FINISHED] [..]
",
        ),
    );
}

#[test]
fn rebuild_if_environment_changes() {
    let p = project("env_change")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "env_change"
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

    assert_that(
        p.cargo("run"),
        execs()
            .with_status(0)
            .with_stdout("old desc")
            .with_stderr(&format!(
                "\
[COMPILING] env_change v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]env_change[EXE]`
",
                dir = p.url()
            )),
    );

    File::create(&p.root().join("Cargo.toml"))
        .unwrap()
        .write_all(
            br#"
        [package]
        name = "env_change"
        description = "new desc"
        version = "0.0.1"
        authors = []
    "#,
        )
        .unwrap();

    assert_that(
        p.cargo("run"),
        execs()
            .with_status(0)
            .with_stdout("new desc")
            .with_stderr(&format!(
                "\
[COMPILING] env_change v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]env_change[EXE]`
",
                dir = p.url()
            )),
    );
}

#[test]
fn no_rebuild_when_rename_dir() {
    let p = project("foo")
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
        .file(
            "foo/Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("foo/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    let mut new = p.root();
    new.pop();
    new.push("bar");
    fs::rename(p.root(), &new).unwrap();

    assert_that(
        p.cargo("build").cwd(&new),
        execs()
            .with_status(0)
            .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]"),
    );
}

#[test]
fn unused_optional_dep() {
    Package::new("registry1", "0.1.0").publish();
    Package::new("registry2", "0.1.0").publish();
    Package::new("registry3", "0.1.0").publish();

    let p = project("p")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "p"
                authors = []
                version = "0.1.0"

                [dependencies]
                foo = { path = "foo" }
                bar = { path = "bar" }
                registry1 = "*"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.1"
                authors = []

                [dev-dependencies]
                registry2 = "*"
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"
                authors = []

                [dependencies]
                registry3 = { version = "*", optional = true }
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr("[FINISHED] [..]"),
    );
}

#[test]
fn path_dev_dep_registry_updates() {
    Package::new("registry1", "0.1.0").publish();
    Package::new("registry2", "0.1.0").publish();

    let p = project("p")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "p"
                authors = []
                version = "0.1.0"

                [dependencies]
                foo = { path = "foo" }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.1"
                authors = []

                [dependencies]
                registry1 = "*"

                [dev-dependencies]
                bar = { path = "../bar"}
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"
                authors = []

                [dependencies]
                registry2 = "*"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr("[FINISHED] [..]"),
    );
}

#[test]
fn change_panic_mode() {
    let p = project("p")
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ['foo', 'bar']
                [profile.dev]
                panic = 'abort'
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.1"
                authors = []
            "#,
        )
        .file("foo/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"
                authors = []

                [lib]
                proc-macro = true

                [dependencies]
                foo = { path = '../foo' }
            "#,
        )
        .file("bar/src/lib.rs", "extern crate foo;")
        .build();

    assert_that(p.cargo("build -p foo"), execs().with_status(0));
    assert_that(p.cargo("build -p bar"), execs().with_status(0));
}

#[test]
fn dont_rebuild_based_on_plugins() {
    let p = project("p")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.1"

                [workspace]
                members = ['bar']

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
                baz = { path = '../baz' }
            "#,
        )
        .file("proc-macro-thing/src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"

                [dependencies]
                baz = { path = '../baz' }
            "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .file(
            "baz/Cargo.toml",
            r#"
                [package]
                name = "baz"
                version = "0.1.1"
            "#,
        )
        .file("baz/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(p.cargo("build -p bar"), execs().with_status(0));
    assert_that(
        p.cargo("build"),
        execs().with_status(0).with_stderr("[FINISHED] [..]\n"),
    );
    assert_that(
        p.cargo("build -p bar"),
        execs().with_status(0).with_stderr("[FINISHED] [..]\n"),
    );
}

#[test]
fn reuse_workspace_lib() {
    let p = project("p")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.1"

                [workspace]

                [dependencies]
                bar = { path = 'bar' }
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.1"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(
        p.cargo("test -p bar -v --no-run"),
        execs().with_status(0).with_stderr("\
[COMPILING] bar v0.1.1 ([..])
[RUNNING] `rustc[..] --test [..]`
[FINISHED] [..]
"));
}
