use std::fs::File;

use git2;

use support::git;
use support::is_nightly;
use support::{basic_manifest, project};

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
        ).build();

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
        ).build();

    p.cargo("fix --allow-no-vcs --broken-code")
        .env("__CARGO_FIX_YOLO", "1")
        .run();
}

#[test]
fn broken_fixes_backed_out() {
    let p = project()
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = 'foo'
                version = '0.1.0'
                [workspace]
            "#,
        ).file(
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
                            fs::File::create("src/lib.rs")
                                .unwrap()
                                .write_all(b"not rust code")
                                .unwrap();
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
        ).file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = 'bar'
                version = '0.1.0'
                [workspace]
            "#,
        ).file("bar/build.rs", "fn main() {}")
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() {
                    let mut x = 3;
                    drop(x);
                }
            "#,
        ).build();

    // Build our rustc shim
    p.cargo("build").cwd(p.root().join("foo")).run();

    // Attempt to fix code, but our shim will always fail the second compile
    p.cargo("fix --allow-no-vcs")
        .cwd(p.root().join("bar"))
        .env("__CARGO_FIX_YOLO", "1")
        .env("RUSTC", p.root().join("foo/target/debug/foo"))
        .with_status(101)
        .with_stderr_contains("[..]not rust code[..]")
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
             quoting the full output of this command we'd be very appreciative!\
             ",
        ).with_stderr_does_not_contain("[..][FIXING][..]")
        .run();
}

#[test] fn fix_path_deps1() { fix_path_deps(); }
#[test] fn fix_path_deps2() { fix_path_deps(); }
#[test] fn fix_path_deps3() { fix_path_deps(); }
#[test] fn fix_path_deps4() { fix_path_deps(); }
#[test] fn fix_path_deps5() { fix_path_deps(); }
#[test] fn fix_path_deps6() { fix_path_deps(); }
#[test] fn fix_path_deps7() { fix_path_deps(); }
#[test] fn fix_path_deps8() { fix_path_deps(); }
#[test] fn fix_path_deps9() { fix_path_deps(); }
#[test] fn fix_path_deps10() { fix_path_deps(); }
#[test] fn fix_path_deps11() { fix_path_deps(); }
#[test] fn fix_path_deps12() { fix_path_deps(); }
#[test] fn fix_path_deps13() { fix_path_deps(); }
#[test] fn fix_path_deps14() { fix_path_deps(); }
#[test] fn fix_path_deps15() { fix_path_deps(); }
#[test] fn fix_path_deps16() { fix_path_deps(); }
#[test] fn fix_path_deps17() { fix_path_deps(); }
#[test] fn fix_path_deps18() { fix_path_deps(); }
#[test] fn fix_path_deps19() { fix_path_deps(); }
#[test] fn fix_path_deps20() { fix_path_deps(); }
#[test] fn fix_path_deps21() { fix_path_deps(); }
#[test] fn fix_path_deps22() { fix_path_deps(); }
#[test] fn fix_path_deps23() { fix_path_deps(); }
#[test] fn fix_path_deps24() { fix_path_deps(); }
#[test] fn fix_path_deps25() { fix_path_deps(); }
#[test] fn fix_path_deps26() { fix_path_deps(); }
#[test] fn fix_path_deps27() { fix_path_deps(); }
#[test] fn fix_path_deps28() { fix_path_deps(); }
#[test] fn fix_path_deps29() { fix_path_deps(); }
#[test] fn fix_path_deps30() { fix_path_deps(); }
#[test] fn fix_path_deps31() { fix_path_deps(); }
#[test] fn fix_path_deps32() { fix_path_deps(); }
#[test] fn fix_path_deps33() { fix_path_deps(); }
#[test] fn fix_path_deps34() { fix_path_deps(); }
#[test] fn fix_path_deps35() { fix_path_deps(); }
#[test] fn fix_path_deps36() { fix_path_deps(); }
#[test] fn fix_path_deps37() { fix_path_deps(); }
#[test] fn fix_path_deps38() { fix_path_deps(); }
#[test] fn fix_path_deps39() { fix_path_deps(); }
#[test] fn fix_path_deps40() { fix_path_deps(); }
#[test] fn fix_path_deps41() { fix_path_deps(); }
#[test] fn fix_path_deps42() { fix_path_deps(); }
#[test] fn fix_path_deps43() { fix_path_deps(); }
#[test] fn fix_path_deps44() { fix_path_deps(); }
#[test] fn fix_path_deps45() { fix_path_deps(); }
#[test] fn fix_path_deps46() { fix_path_deps(); }
#[test] fn fix_path_deps47() { fix_path_deps(); }
#[test] fn fix_path_deps48() { fix_path_deps(); }
#[test] fn fix_path_deps49() { fix_path_deps(); }
#[test] fn fix_path_deps50() { fix_path_deps(); }
#[test] fn fix_path_deps51() { fix_path_deps(); }
#[test] fn fix_path_deps52() { fix_path_deps(); }
#[test] fn fix_path_deps53() { fix_path_deps(); }
#[test] fn fix_path_deps54() { fix_path_deps(); }
#[test] fn fix_path_deps55() { fix_path_deps(); }
#[test] fn fix_path_deps56() { fix_path_deps(); }
#[test] fn fix_path_deps57() { fix_path_deps(); }
#[test] fn fix_path_deps58() { fix_path_deps(); }
#[test] fn fix_path_deps59() { fix_path_deps(); }
#[test] fn fix_path_deps60() { fix_path_deps(); }
#[test] fn fix_path_deps61() { fix_path_deps(); }
#[test] fn fix_path_deps62() { fix_path_deps(); }
#[test] fn fix_path_deps63() { fix_path_deps(); }
#[test] fn fix_path_deps64() { fix_path_deps(); }
#[test] fn fix_path_deps65() { fix_path_deps(); }
#[test] fn fix_path_deps66() { fix_path_deps(); }
#[test] fn fix_path_deps67() { fix_path_deps(); }
#[test] fn fix_path_deps68() { fix_path_deps(); }
#[test] fn fix_path_deps69() { fix_path_deps(); }
#[test] fn fix_path_deps70() { fix_path_deps(); }
#[test] fn fix_path_deps71() { fix_path_deps(); }
#[test] fn fix_path_deps72() { fix_path_deps(); }
#[test] fn fix_path_deps73() { fix_path_deps(); }
#[test] fn fix_path_deps74() { fix_path_deps(); }
#[test] fn fix_path_deps75() { fix_path_deps(); }
#[test] fn fix_path_deps76() { fix_path_deps(); }
#[test] fn fix_path_deps77() { fix_path_deps(); }
#[test] fn fix_path_deps78() { fix_path_deps(); }
#[test] fn fix_path_deps79() { fix_path_deps(); }
#[test] fn fix_path_deps80() { fix_path_deps(); }
#[test] fn fix_path_deps81() { fix_path_deps(); }
#[test] fn fix_path_deps82() { fix_path_deps(); }
#[test] fn fix_path_deps83() { fix_path_deps(); }
#[test] fn fix_path_deps84() { fix_path_deps(); }
#[test] fn fix_path_deps85() { fix_path_deps(); }
#[test] fn fix_path_deps86() { fix_path_deps(); }
#[test] fn fix_path_deps87() { fix_path_deps(); }
#[test] fn fix_path_deps88() { fix_path_deps(); }
#[test] fn fix_path_deps89() { fix_path_deps(); }
#[test] fn fix_path_deps90() { fix_path_deps(); }
#[test] fn fix_path_deps91() { fix_path_deps(); }
#[test] fn fix_path_deps92() { fix_path_deps(); }
#[test] fn fix_path_deps93() { fix_path_deps(); }
#[test] fn fix_path_deps94() { fix_path_deps(); }
#[test] fn fix_path_deps95() { fix_path_deps(); }
#[test] fn fix_path_deps96() { fix_path_deps(); }
#[test] fn fix_path_deps97() { fix_path_deps(); }
#[test] fn fix_path_deps98() { fix_path_deps(); }
#[test] fn fix_path_deps99() { fix_path_deps(); }
#[test] fn fix_path_deps100() { fix_path_deps(); }
#[test] fn fix_path_deps101() { fix_path_deps(); }
#[test] fn fix_path_deps102() { fix_path_deps(); }
#[test] fn fix_path_deps103() { fix_path_deps(); }
#[test] fn fix_path_deps104() { fix_path_deps(); }
#[test] fn fix_path_deps105() { fix_path_deps(); }
#[test] fn fix_path_deps106() { fix_path_deps(); }
#[test] fn fix_path_deps107() { fix_path_deps(); }
#[test] fn fix_path_deps108() { fix_path_deps(); }
#[test] fn fix_path_deps109() { fix_path_deps(); }
#[test] fn fix_path_deps110() { fix_path_deps(); }
#[test] fn fix_path_deps111() { fix_path_deps(); }
#[test] fn fix_path_deps112() { fix_path_deps(); }
#[test] fn fix_path_deps113() { fix_path_deps(); }
#[test] fn fix_path_deps114() { fix_path_deps(); }
#[test] fn fix_path_deps115() { fix_path_deps(); }
#[test] fn fix_path_deps116() { fix_path_deps(); }
#[test] fn fix_path_deps117() { fix_path_deps(); }
#[test] fn fix_path_deps118() { fix_path_deps(); }
#[test] fn fix_path_deps119() { fix_path_deps(); }
#[test] fn fix_path_deps120() { fix_path_deps(); }
#[test] fn fix_path_deps121() { fix_path_deps(); }
#[test] fn fix_path_deps122() { fix_path_deps(); }
#[test] fn fix_path_deps123() { fix_path_deps(); }
#[test] fn fix_path_deps124() { fix_path_deps(); }
#[test] fn fix_path_deps125() { fix_path_deps(); }
#[test] fn fix_path_deps126() { fix_path_deps(); }
#[test] fn fix_path_deps127() { fix_path_deps(); }
#[test] fn fix_path_deps128() { fix_path_deps(); }
#[test] fn fix_path_deps129() { fix_path_deps(); }
#[test] fn fix_path_deps130() { fix_path_deps(); }
#[test] fn fix_path_deps131() { fix_path_deps(); }
#[test] fn fix_path_deps132() { fix_path_deps(); }
#[test] fn fix_path_deps133() { fix_path_deps(); }
#[test] fn fix_path_deps134() { fix_path_deps(); }
#[test] fn fix_path_deps135() { fix_path_deps(); }
#[test] fn fix_path_deps136() { fix_path_deps(); }
#[test] fn fix_path_deps137() { fix_path_deps(); }
#[test] fn fix_path_deps138() { fix_path_deps(); }
#[test] fn fix_path_deps139() { fix_path_deps(); }
#[test] fn fix_path_deps140() { fix_path_deps(); }
#[test] fn fix_path_deps141() { fix_path_deps(); }
#[test] fn fix_path_deps142() { fix_path_deps(); }
#[test] fn fix_path_deps143() { fix_path_deps(); }
#[test] fn fix_path_deps144() { fix_path_deps(); }
#[test] fn fix_path_deps145() { fix_path_deps(); }
#[test] fn fix_path_deps146() { fix_path_deps(); }
#[test] fn fix_path_deps147() { fix_path_deps(); }
#[test] fn fix_path_deps148() { fix_path_deps(); }
#[test] fn fix_path_deps149() { fix_path_deps(); }
#[test] fn fix_path_deps150() { fix_path_deps(); }
#[test] fn fix_path_deps151() { fix_path_deps(); }
#[test] fn fix_path_deps152() { fix_path_deps(); }
#[test] fn fix_path_deps153() { fix_path_deps(); }
#[test] fn fix_path_deps154() { fix_path_deps(); }
#[test] fn fix_path_deps155() { fix_path_deps(); }
#[test] fn fix_path_deps156() { fix_path_deps(); }
#[test] fn fix_path_deps157() { fix_path_deps(); }
#[test] fn fix_path_deps158() { fix_path_deps(); }
#[test] fn fix_path_deps159() { fix_path_deps(); }
#[test] fn fix_path_deps160() { fix_path_deps(); }
#[test] fn fix_path_deps161() { fix_path_deps(); }
#[test] fn fix_path_deps162() { fix_path_deps(); }
#[test] fn fix_path_deps163() { fix_path_deps(); }
#[test] fn fix_path_deps164() { fix_path_deps(); }
#[test] fn fix_path_deps165() { fix_path_deps(); }
#[test] fn fix_path_deps166() { fix_path_deps(); }
#[test] fn fix_path_deps167() { fix_path_deps(); }
#[test] fn fix_path_deps168() { fix_path_deps(); }
#[test] fn fix_path_deps169() { fix_path_deps(); }
#[test] fn fix_path_deps170() { fix_path_deps(); }
#[test] fn fix_path_deps171() { fix_path_deps(); }
#[test] fn fix_path_deps172() { fix_path_deps(); }
#[test] fn fix_path_deps173() { fix_path_deps(); }
#[test] fn fix_path_deps174() { fix_path_deps(); }
#[test] fn fix_path_deps175() { fix_path_deps(); }
#[test] fn fix_path_deps176() { fix_path_deps(); }
#[test] fn fix_path_deps177() { fix_path_deps(); }
#[test] fn fix_path_deps178() { fix_path_deps(); }
#[test] fn fix_path_deps179() { fix_path_deps(); }
#[test] fn fix_path_deps180() { fix_path_deps(); }
#[test] fn fix_path_deps181() { fix_path_deps(); }
#[test] fn fix_path_deps182() { fix_path_deps(); }
#[test] fn fix_path_deps183() { fix_path_deps(); }
#[test] fn fix_path_deps184() { fix_path_deps(); }
#[test] fn fix_path_deps185() { fix_path_deps(); }
#[test] fn fix_path_deps186() { fix_path_deps(); }
#[test] fn fix_path_deps187() { fix_path_deps(); }
#[test] fn fix_path_deps188() { fix_path_deps(); }
#[test] fn fix_path_deps189() { fix_path_deps(); }
#[test] fn fix_path_deps190() { fix_path_deps(); }
#[test] fn fix_path_deps191() { fix_path_deps(); }
#[test] fn fix_path_deps192() { fix_path_deps(); }
#[test] fn fix_path_deps193() { fix_path_deps(); }
#[test] fn fix_path_deps194() { fix_path_deps(); }
#[test] fn fix_path_deps195() { fix_path_deps(); }
#[test] fn fix_path_deps196() { fix_path_deps(); }
#[test] fn fix_path_deps197() { fix_path_deps(); }
#[test] fn fix_path_deps198() { fix_path_deps(); }
#[test] fn fix_path_deps199() { fix_path_deps(); }
#[test] fn fix_path_deps200() { fix_path_deps(); }
#[test] fn fix_path_deps201() { fix_path_deps(); }
#[test] fn fix_path_deps202() { fix_path_deps(); }
#[test] fn fix_path_deps203() { fix_path_deps(); }
#[test] fn fix_path_deps204() { fix_path_deps(); }
#[test] fn fix_path_deps205() { fix_path_deps(); }
#[test] fn fix_path_deps206() { fix_path_deps(); }
#[test] fn fix_path_deps207() { fix_path_deps(); }
#[test] fn fix_path_deps208() { fix_path_deps(); }
#[test] fn fix_path_deps209() { fix_path_deps(); }
#[test] fn fix_path_deps210() { fix_path_deps(); }
#[test] fn fix_path_deps211() { fix_path_deps(); }
#[test] fn fix_path_deps212() { fix_path_deps(); }
#[test] fn fix_path_deps213() { fix_path_deps(); }
#[test] fn fix_path_deps214() { fix_path_deps(); }
#[test] fn fix_path_deps215() { fix_path_deps(); }
#[test] fn fix_path_deps216() { fix_path_deps(); }
#[test] fn fix_path_deps217() { fix_path_deps(); }
#[test] fn fix_path_deps218() { fix_path_deps(); }
#[test] fn fix_path_deps219() { fix_path_deps(); }
#[test] fn fix_path_deps220() { fix_path_deps(); }
#[test] fn fix_path_deps221() { fix_path_deps(); }
#[test] fn fix_path_deps222() { fix_path_deps(); }
#[test] fn fix_path_deps223() { fix_path_deps(); }
#[test] fn fix_path_deps224() { fix_path_deps(); }
#[test] fn fix_path_deps225() { fix_path_deps(); }
#[test] fn fix_path_deps226() { fix_path_deps(); }
#[test] fn fix_path_deps227() { fix_path_deps(); }
#[test] fn fix_path_deps228() { fix_path_deps(); }
#[test] fn fix_path_deps229() { fix_path_deps(); }
#[test] fn fix_path_deps230() { fix_path_deps(); }
#[test] fn fix_path_deps231() { fix_path_deps(); }
#[test] fn fix_path_deps232() { fix_path_deps(); }
#[test] fn fix_path_deps233() { fix_path_deps(); }
#[test] fn fix_path_deps234() { fix_path_deps(); }
#[test] fn fix_path_deps235() { fix_path_deps(); }
#[test] fn fix_path_deps236() { fix_path_deps(); }
#[test] fn fix_path_deps237() { fix_path_deps(); }
#[test] fn fix_path_deps238() { fix_path_deps(); }
#[test] fn fix_path_deps239() { fix_path_deps(); }
#[test] fn fix_path_deps240() { fix_path_deps(); }
#[test] fn fix_path_deps241() { fix_path_deps(); }
#[test] fn fix_path_deps242() { fix_path_deps(); }
#[test] fn fix_path_deps243() { fix_path_deps(); }
#[test] fn fix_path_deps244() { fix_path_deps(); }
#[test] fn fix_path_deps245() { fix_path_deps(); }
#[test] fn fix_path_deps246() { fix_path_deps(); }
#[test] fn fix_path_deps247() { fix_path_deps(); }
#[test] fn fix_path_deps248() { fix_path_deps(); }
#[test] fn fix_path_deps249() { fix_path_deps(); }
#[test] fn fix_path_deps250() { fix_path_deps(); }
#[test] fn fix_path_deps251() { fix_path_deps(); }
#[test] fn fix_path_deps252() { fix_path_deps(); }
#[test] fn fix_path_deps253() { fix_path_deps(); }
#[test] fn fix_path_deps254() { fix_path_deps(); }
#[test] fn fix_path_deps255() { fix_path_deps(); }
#[test] fn fix_path_deps256() { fix_path_deps(); }
#[test] fn fix_path_deps257() { fix_path_deps(); }
#[test] fn fix_path_deps258() { fix_path_deps(); }
#[test] fn fix_path_deps259() { fix_path_deps(); }
#[test] fn fix_path_deps260() { fix_path_deps(); }
#[test] fn fix_path_deps261() { fix_path_deps(); }
#[test] fn fix_path_deps262() { fix_path_deps(); }
#[test] fn fix_path_deps263() { fix_path_deps(); }
#[test] fn fix_path_deps264() { fix_path_deps(); }
#[test] fn fix_path_deps265() { fix_path_deps(); }
#[test] fn fix_path_deps266() { fix_path_deps(); }
#[test] fn fix_path_deps267() { fix_path_deps(); }
#[test] fn fix_path_deps268() { fix_path_deps(); }
#[test] fn fix_path_deps269() { fix_path_deps(); }
#[test] fn fix_path_deps270() { fix_path_deps(); }
#[test] fn fix_path_deps271() { fix_path_deps(); }
#[test] fn fix_path_deps272() { fix_path_deps(); }
#[test] fn fix_path_deps273() { fix_path_deps(); }
#[test] fn fix_path_deps274() { fix_path_deps(); }
#[test] fn fix_path_deps275() { fix_path_deps(); }
#[test] fn fix_path_deps276() { fix_path_deps(); }
#[test] fn fix_path_deps277() { fix_path_deps(); }
#[test] fn fix_path_deps278() { fix_path_deps(); }
#[test] fn fix_path_deps279() { fix_path_deps(); }
#[test] fn fix_path_deps280() { fix_path_deps(); }
#[test] fn fix_path_deps281() { fix_path_deps(); }
#[test] fn fix_path_deps282() { fix_path_deps(); }
#[test] fn fix_path_deps283() { fix_path_deps(); }
#[test] fn fix_path_deps284() { fix_path_deps(); }
#[test] fn fix_path_deps285() { fix_path_deps(); }
#[test] fn fix_path_deps286() { fix_path_deps(); }
#[test] fn fix_path_deps287() { fix_path_deps(); }
#[test] fn fix_path_deps288() { fix_path_deps(); }
#[test] fn fix_path_deps289() { fix_path_deps(); }
#[test] fn fix_path_deps290() { fix_path_deps(); }
#[test] fn fix_path_deps291() { fix_path_deps(); }
#[test] fn fix_path_deps292() { fix_path_deps(); }
#[test] fn fix_path_deps293() { fix_path_deps(); }
#[test] fn fix_path_deps294() { fix_path_deps(); }
#[test] fn fix_path_deps295() { fix_path_deps(); }
#[test] fn fix_path_deps296() { fix_path_deps(); }
#[test] fn fix_path_deps297() { fix_path_deps(); }
#[test] fn fix_path_deps298() { fix_path_deps(); }
#[test] fn fix_path_deps299() { fix_path_deps(); }
#[test] fn fix_path_deps300() { fix_path_deps(); }
#[test] fn fix_path_deps301() { fix_path_deps(); }
#[test] fn fix_path_deps302() { fix_path_deps(); }
#[test] fn fix_path_deps303() { fix_path_deps(); }
#[test] fn fix_path_deps304() { fix_path_deps(); }
#[test] fn fix_path_deps305() { fix_path_deps(); }
#[test] fn fix_path_deps306() { fix_path_deps(); }
#[test] fn fix_path_deps307() { fix_path_deps(); }
#[test] fn fix_path_deps308() { fix_path_deps(); }
#[test] fn fix_path_deps309() { fix_path_deps(); }
#[test] fn fix_path_deps310() { fix_path_deps(); }
#[test] fn fix_path_deps311() { fix_path_deps(); }
#[test] fn fix_path_deps312() { fix_path_deps(); }
#[test] fn fix_path_deps313() { fix_path_deps(); }
#[test] fn fix_path_deps314() { fix_path_deps(); }
#[test] fn fix_path_deps315() { fix_path_deps(); }
#[test] fn fix_path_deps316() { fix_path_deps(); }
#[test] fn fix_path_deps317() { fix_path_deps(); }
#[test] fn fix_path_deps318() { fix_path_deps(); }
#[test] fn fix_path_deps319() { fix_path_deps(); }
#[test] fn fix_path_deps320() { fix_path_deps(); }
#[test] fn fix_path_deps321() { fix_path_deps(); }
#[test] fn fix_path_deps322() { fix_path_deps(); }
#[test] fn fix_path_deps323() { fix_path_deps(); }
#[test] fn fix_path_deps324() { fix_path_deps(); }
#[test] fn fix_path_deps325() { fix_path_deps(); }
#[test] fn fix_path_deps326() { fix_path_deps(); }
#[test] fn fix_path_deps327() { fix_path_deps(); }
#[test] fn fix_path_deps328() { fix_path_deps(); }
#[test] fn fix_path_deps329() { fix_path_deps(); }
#[test] fn fix_path_deps330() { fix_path_deps(); }
#[test] fn fix_path_deps331() { fix_path_deps(); }
#[test] fn fix_path_deps332() { fix_path_deps(); }
#[test] fn fix_path_deps333() { fix_path_deps(); }
#[test] fn fix_path_deps334() { fix_path_deps(); }
#[test] fn fix_path_deps335() { fix_path_deps(); }
#[test] fn fix_path_deps336() { fix_path_deps(); }
#[test] fn fix_path_deps337() { fix_path_deps(); }
#[test] fn fix_path_deps338() { fix_path_deps(); }
#[test] fn fix_path_deps339() { fix_path_deps(); }
#[test] fn fix_path_deps340() { fix_path_deps(); }
#[test] fn fix_path_deps341() { fix_path_deps(); }
#[test] fn fix_path_deps342() { fix_path_deps(); }
#[test] fn fix_path_deps343() { fix_path_deps(); }
#[test] fn fix_path_deps344() { fix_path_deps(); }
#[test] fn fix_path_deps345() { fix_path_deps(); }
#[test] fn fix_path_deps346() { fix_path_deps(); }
#[test] fn fix_path_deps347() { fix_path_deps(); }
#[test] fn fix_path_deps348() { fix_path_deps(); }
#[test] fn fix_path_deps349() { fix_path_deps(); }
#[test] fn fix_path_deps350() { fix_path_deps(); }
#[test] fn fix_path_deps351() { fix_path_deps(); }
#[test] fn fix_path_deps352() { fix_path_deps(); }
#[test] fn fix_path_deps353() { fix_path_deps(); }
#[test] fn fix_path_deps354() { fix_path_deps(); }
#[test] fn fix_path_deps355() { fix_path_deps(); }
#[test] fn fix_path_deps356() { fix_path_deps(); }
#[test] fn fix_path_deps357() { fix_path_deps(); }
#[test] fn fix_path_deps358() { fix_path_deps(); }
#[test] fn fix_path_deps359() { fix_path_deps(); }
#[test] fn fix_path_deps360() { fix_path_deps(); }
#[test] fn fix_path_deps361() { fix_path_deps(); }
#[test] fn fix_path_deps362() { fix_path_deps(); }
#[test] fn fix_path_deps363() { fix_path_deps(); }
#[test] fn fix_path_deps364() { fix_path_deps(); }
#[test] fn fix_path_deps365() { fix_path_deps(); }
#[test] fn fix_path_deps366() { fix_path_deps(); }
#[test] fn fix_path_deps367() { fix_path_deps(); }
#[test] fn fix_path_deps368() { fix_path_deps(); }
#[test] fn fix_path_deps369() { fix_path_deps(); }
#[test] fn fix_path_deps370() { fix_path_deps(); }
#[test] fn fix_path_deps371() { fix_path_deps(); }
#[test] fn fix_path_deps372() { fix_path_deps(); }
#[test] fn fix_path_deps373() { fix_path_deps(); }
#[test] fn fix_path_deps374() { fix_path_deps(); }
#[test] fn fix_path_deps375() { fix_path_deps(); }
#[test] fn fix_path_deps376() { fix_path_deps(); }
#[test] fn fix_path_deps377() { fix_path_deps(); }
#[test] fn fix_path_deps378() { fix_path_deps(); }
#[test] fn fix_path_deps379() { fix_path_deps(); }
#[test] fn fix_path_deps380() { fix_path_deps(); }
#[test] fn fix_path_deps381() { fix_path_deps(); }
#[test] fn fix_path_deps382() { fix_path_deps(); }
#[test] fn fix_path_deps383() { fix_path_deps(); }
#[test] fn fix_path_deps384() { fix_path_deps(); }
#[test] fn fix_path_deps385() { fix_path_deps(); }
#[test] fn fix_path_deps386() { fix_path_deps(); }
#[test] fn fix_path_deps387() { fix_path_deps(); }
#[test] fn fix_path_deps388() { fix_path_deps(); }
#[test] fn fix_path_deps389() { fix_path_deps(); }
#[test] fn fix_path_deps390() { fix_path_deps(); }
#[test] fn fix_path_deps391() { fix_path_deps(); }
#[test] fn fix_path_deps392() { fix_path_deps(); }
#[test] fn fix_path_deps393() { fix_path_deps(); }
#[test] fn fix_path_deps394() { fix_path_deps(); }
#[test] fn fix_path_deps395() { fix_path_deps(); }
#[test] fn fix_path_deps396() { fix_path_deps(); }
#[test] fn fix_path_deps397() { fix_path_deps(); }
#[test] fn fix_path_deps398() { fix_path_deps(); }
#[test] fn fix_path_deps399() { fix_path_deps(); }
#[test] fn fix_path_deps400() { fix_path_deps(); }


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
        ).file(
            "src/lib.rs",
            r#"
                extern crate bar;

                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        ).file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        ).build();

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
        ).run();
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
        ).file("foo/src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file(
            "bar/src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }
            "#,
        ).build();

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
        ).build();

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
    assert!(
        p.read_file("src/lib.rs")
            .contains("let x = crate::foo::FOO;")
    );
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
        ).build();

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
        ).file(
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
        ).file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
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
        ).build();

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
        ).build();

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
        ).build();

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
        ).build();

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
        ).build();

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
        ).build();

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
        ).build();

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
        ).file(
            "src/bar.rs",
            "
                pub fn foo() -> u32 {
                    let mut x = 3;
                    x
                }

            ",
        ).build();

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
        ).file(
            "tests/a.rs",
            r#"
                #[test]
                pub fn foo() { let mut x = 3; drop(x); }
            "#,
        ).file("examples/foo.rs", "fn main() { let mut x = 3; drop(x); }")
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
            .with_stderr_contains("[FINISHED] [..]").run();
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
        ).file(
            "src/lib.rs",
            r#"
            #[cfg(feature = "bar")]
            pub fn foo() -> u32 { let mut x = 3; x }
        "#,
        ).build();

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
        ).run();
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
        ).run();
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
        ).run();
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
        ).file("src/lib.rs", "")
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
        ).build();

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
        ).file(
            "src/lib.rs",
            r#"
                use std::any::Any;
                pub fn foo() {
                    let _x: Box<Any> = Box::new(3);
                }
            "#,
        ).build();

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
        ).file("src/lib.rs", "extern crate bar;")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "")
        .build();

    p.cargo("fix --allow-no-vcs -p foo")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr("\
[CHECKING] bar v0.1.0 ([..])
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
")
        .run();

    p.cargo("fix --allow-no-vcs -p foo")
        .env("__CARGO_FIX_YOLO", "1")
        .with_stdout("")
        .with_stderr("\
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
")
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
        .with_stderr("\
[CHECKING] a v0.1.0 ([..])
[CHECKING] foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
")
        .run();
}
