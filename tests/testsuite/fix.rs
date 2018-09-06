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
#[test] fn fix_path_deps401() { fix_path_deps(); }
#[test] fn fix_path_deps402() { fix_path_deps(); }
#[test] fn fix_path_deps403() { fix_path_deps(); }
#[test] fn fix_path_deps404() { fix_path_deps(); }
#[test] fn fix_path_deps405() { fix_path_deps(); }
#[test] fn fix_path_deps406() { fix_path_deps(); }
#[test] fn fix_path_deps407() { fix_path_deps(); }
#[test] fn fix_path_deps408() { fix_path_deps(); }
#[test] fn fix_path_deps409() { fix_path_deps(); }
#[test] fn fix_path_deps410() { fix_path_deps(); }
#[test] fn fix_path_deps411() { fix_path_deps(); }
#[test] fn fix_path_deps412() { fix_path_deps(); }
#[test] fn fix_path_deps413() { fix_path_deps(); }
#[test] fn fix_path_deps414() { fix_path_deps(); }
#[test] fn fix_path_deps415() { fix_path_deps(); }
#[test] fn fix_path_deps416() { fix_path_deps(); }
#[test] fn fix_path_deps417() { fix_path_deps(); }
#[test] fn fix_path_deps418() { fix_path_deps(); }
#[test] fn fix_path_deps419() { fix_path_deps(); }
#[test] fn fix_path_deps420() { fix_path_deps(); }
#[test] fn fix_path_deps421() { fix_path_deps(); }
#[test] fn fix_path_deps422() { fix_path_deps(); }
#[test] fn fix_path_deps423() { fix_path_deps(); }
#[test] fn fix_path_deps424() { fix_path_deps(); }
#[test] fn fix_path_deps425() { fix_path_deps(); }
#[test] fn fix_path_deps426() { fix_path_deps(); }
#[test] fn fix_path_deps427() { fix_path_deps(); }
#[test] fn fix_path_deps428() { fix_path_deps(); }
#[test] fn fix_path_deps429() { fix_path_deps(); }
#[test] fn fix_path_deps430() { fix_path_deps(); }
#[test] fn fix_path_deps431() { fix_path_deps(); }
#[test] fn fix_path_deps432() { fix_path_deps(); }
#[test] fn fix_path_deps433() { fix_path_deps(); }
#[test] fn fix_path_deps434() { fix_path_deps(); }
#[test] fn fix_path_deps435() { fix_path_deps(); }
#[test] fn fix_path_deps436() { fix_path_deps(); }
#[test] fn fix_path_deps437() { fix_path_deps(); }
#[test] fn fix_path_deps438() { fix_path_deps(); }
#[test] fn fix_path_deps439() { fix_path_deps(); }
#[test] fn fix_path_deps440() { fix_path_deps(); }
#[test] fn fix_path_deps441() { fix_path_deps(); }
#[test] fn fix_path_deps442() { fix_path_deps(); }
#[test] fn fix_path_deps443() { fix_path_deps(); }
#[test] fn fix_path_deps444() { fix_path_deps(); }
#[test] fn fix_path_deps445() { fix_path_deps(); }
#[test] fn fix_path_deps446() { fix_path_deps(); }
#[test] fn fix_path_deps447() { fix_path_deps(); }
#[test] fn fix_path_deps448() { fix_path_deps(); }
#[test] fn fix_path_deps449() { fix_path_deps(); }
#[test] fn fix_path_deps450() { fix_path_deps(); }
#[test] fn fix_path_deps451() { fix_path_deps(); }
#[test] fn fix_path_deps452() { fix_path_deps(); }
#[test] fn fix_path_deps453() { fix_path_deps(); }
#[test] fn fix_path_deps454() { fix_path_deps(); }
#[test] fn fix_path_deps455() { fix_path_deps(); }
#[test] fn fix_path_deps456() { fix_path_deps(); }
#[test] fn fix_path_deps457() { fix_path_deps(); }
#[test] fn fix_path_deps458() { fix_path_deps(); }
#[test] fn fix_path_deps459() { fix_path_deps(); }
#[test] fn fix_path_deps460() { fix_path_deps(); }
#[test] fn fix_path_deps461() { fix_path_deps(); }
#[test] fn fix_path_deps462() { fix_path_deps(); }
#[test] fn fix_path_deps463() { fix_path_deps(); }
#[test] fn fix_path_deps464() { fix_path_deps(); }
#[test] fn fix_path_deps465() { fix_path_deps(); }
#[test] fn fix_path_deps466() { fix_path_deps(); }
#[test] fn fix_path_deps467() { fix_path_deps(); }
#[test] fn fix_path_deps468() { fix_path_deps(); }
#[test] fn fix_path_deps469() { fix_path_deps(); }
#[test] fn fix_path_deps470() { fix_path_deps(); }
#[test] fn fix_path_deps471() { fix_path_deps(); }
#[test] fn fix_path_deps472() { fix_path_deps(); }
#[test] fn fix_path_deps473() { fix_path_deps(); }
#[test] fn fix_path_deps474() { fix_path_deps(); }
#[test] fn fix_path_deps475() { fix_path_deps(); }
#[test] fn fix_path_deps476() { fix_path_deps(); }
#[test] fn fix_path_deps477() { fix_path_deps(); }
#[test] fn fix_path_deps478() { fix_path_deps(); }
#[test] fn fix_path_deps479() { fix_path_deps(); }
#[test] fn fix_path_deps480() { fix_path_deps(); }
#[test] fn fix_path_deps481() { fix_path_deps(); }
#[test] fn fix_path_deps482() { fix_path_deps(); }
#[test] fn fix_path_deps483() { fix_path_deps(); }
#[test] fn fix_path_deps484() { fix_path_deps(); }
#[test] fn fix_path_deps485() { fix_path_deps(); }
#[test] fn fix_path_deps486() { fix_path_deps(); }
#[test] fn fix_path_deps487() { fix_path_deps(); }
#[test] fn fix_path_deps488() { fix_path_deps(); }
#[test] fn fix_path_deps489() { fix_path_deps(); }
#[test] fn fix_path_deps490() { fix_path_deps(); }
#[test] fn fix_path_deps491() { fix_path_deps(); }
#[test] fn fix_path_deps492() { fix_path_deps(); }
#[test] fn fix_path_deps493() { fix_path_deps(); }
#[test] fn fix_path_deps494() { fix_path_deps(); }
#[test] fn fix_path_deps495() { fix_path_deps(); }
#[test] fn fix_path_deps496() { fix_path_deps(); }
#[test] fn fix_path_deps497() { fix_path_deps(); }
#[test] fn fix_path_deps498() { fix_path_deps(); }
#[test] fn fix_path_deps499() { fix_path_deps(); }
#[test] fn fix_path_deps500() { fix_path_deps(); }
#[test] fn fix_path_deps501() { fix_path_deps(); }
#[test] fn fix_path_deps502() { fix_path_deps(); }
#[test] fn fix_path_deps503() { fix_path_deps(); }
#[test] fn fix_path_deps504() { fix_path_deps(); }
#[test] fn fix_path_deps505() { fix_path_deps(); }
#[test] fn fix_path_deps506() { fix_path_deps(); }
#[test] fn fix_path_deps507() { fix_path_deps(); }
#[test] fn fix_path_deps508() { fix_path_deps(); }
#[test] fn fix_path_deps509() { fix_path_deps(); }
#[test] fn fix_path_deps510() { fix_path_deps(); }
#[test] fn fix_path_deps511() { fix_path_deps(); }
#[test] fn fix_path_deps512() { fix_path_deps(); }
#[test] fn fix_path_deps513() { fix_path_deps(); }
#[test] fn fix_path_deps514() { fix_path_deps(); }
#[test] fn fix_path_deps515() { fix_path_deps(); }
#[test] fn fix_path_deps516() { fix_path_deps(); }
#[test] fn fix_path_deps517() { fix_path_deps(); }
#[test] fn fix_path_deps518() { fix_path_deps(); }
#[test] fn fix_path_deps519() { fix_path_deps(); }
#[test] fn fix_path_deps520() { fix_path_deps(); }
#[test] fn fix_path_deps521() { fix_path_deps(); }
#[test] fn fix_path_deps522() { fix_path_deps(); }
#[test] fn fix_path_deps523() { fix_path_deps(); }
#[test] fn fix_path_deps524() { fix_path_deps(); }
#[test] fn fix_path_deps525() { fix_path_deps(); }
#[test] fn fix_path_deps526() { fix_path_deps(); }
#[test] fn fix_path_deps527() { fix_path_deps(); }
#[test] fn fix_path_deps528() { fix_path_deps(); }
#[test] fn fix_path_deps529() { fix_path_deps(); }
#[test] fn fix_path_deps530() { fix_path_deps(); }
#[test] fn fix_path_deps531() { fix_path_deps(); }
#[test] fn fix_path_deps532() { fix_path_deps(); }
#[test] fn fix_path_deps533() { fix_path_deps(); }
#[test] fn fix_path_deps534() { fix_path_deps(); }
#[test] fn fix_path_deps535() { fix_path_deps(); }
#[test] fn fix_path_deps536() { fix_path_deps(); }
#[test] fn fix_path_deps537() { fix_path_deps(); }
#[test] fn fix_path_deps538() { fix_path_deps(); }
#[test] fn fix_path_deps539() { fix_path_deps(); }
#[test] fn fix_path_deps540() { fix_path_deps(); }
#[test] fn fix_path_deps541() { fix_path_deps(); }
#[test] fn fix_path_deps542() { fix_path_deps(); }
#[test] fn fix_path_deps543() { fix_path_deps(); }
#[test] fn fix_path_deps544() { fix_path_deps(); }
#[test] fn fix_path_deps545() { fix_path_deps(); }
#[test] fn fix_path_deps546() { fix_path_deps(); }
#[test] fn fix_path_deps547() { fix_path_deps(); }
#[test] fn fix_path_deps548() { fix_path_deps(); }
#[test] fn fix_path_deps549() { fix_path_deps(); }
#[test] fn fix_path_deps550() { fix_path_deps(); }
#[test] fn fix_path_deps551() { fix_path_deps(); }
#[test] fn fix_path_deps552() { fix_path_deps(); }
#[test] fn fix_path_deps553() { fix_path_deps(); }
#[test] fn fix_path_deps554() { fix_path_deps(); }
#[test] fn fix_path_deps555() { fix_path_deps(); }
#[test] fn fix_path_deps556() { fix_path_deps(); }
#[test] fn fix_path_deps557() { fix_path_deps(); }
#[test] fn fix_path_deps558() { fix_path_deps(); }
#[test] fn fix_path_deps559() { fix_path_deps(); }
#[test] fn fix_path_deps560() { fix_path_deps(); }
#[test] fn fix_path_deps561() { fix_path_deps(); }
#[test] fn fix_path_deps562() { fix_path_deps(); }
#[test] fn fix_path_deps563() { fix_path_deps(); }
#[test] fn fix_path_deps564() { fix_path_deps(); }
#[test] fn fix_path_deps565() { fix_path_deps(); }
#[test] fn fix_path_deps566() { fix_path_deps(); }
#[test] fn fix_path_deps567() { fix_path_deps(); }
#[test] fn fix_path_deps568() { fix_path_deps(); }
#[test] fn fix_path_deps569() { fix_path_deps(); }
#[test] fn fix_path_deps570() { fix_path_deps(); }
#[test] fn fix_path_deps571() { fix_path_deps(); }
#[test] fn fix_path_deps572() { fix_path_deps(); }
#[test] fn fix_path_deps573() { fix_path_deps(); }
#[test] fn fix_path_deps574() { fix_path_deps(); }
#[test] fn fix_path_deps575() { fix_path_deps(); }
#[test] fn fix_path_deps576() { fix_path_deps(); }
#[test] fn fix_path_deps577() { fix_path_deps(); }
#[test] fn fix_path_deps578() { fix_path_deps(); }
#[test] fn fix_path_deps579() { fix_path_deps(); }
#[test] fn fix_path_deps580() { fix_path_deps(); }
#[test] fn fix_path_deps581() { fix_path_deps(); }
#[test] fn fix_path_deps582() { fix_path_deps(); }
#[test] fn fix_path_deps583() { fix_path_deps(); }
#[test] fn fix_path_deps584() { fix_path_deps(); }
#[test] fn fix_path_deps585() { fix_path_deps(); }
#[test] fn fix_path_deps586() { fix_path_deps(); }
#[test] fn fix_path_deps587() { fix_path_deps(); }
#[test] fn fix_path_deps588() { fix_path_deps(); }
#[test] fn fix_path_deps589() { fix_path_deps(); }
#[test] fn fix_path_deps590() { fix_path_deps(); }
#[test] fn fix_path_deps591() { fix_path_deps(); }
#[test] fn fix_path_deps592() { fix_path_deps(); }
#[test] fn fix_path_deps593() { fix_path_deps(); }
#[test] fn fix_path_deps594() { fix_path_deps(); }
#[test] fn fix_path_deps595() { fix_path_deps(); }
#[test] fn fix_path_deps596() { fix_path_deps(); }
#[test] fn fix_path_deps597() { fix_path_deps(); }
#[test] fn fix_path_deps598() { fix_path_deps(); }
#[test] fn fix_path_deps599() { fix_path_deps(); }
#[test] fn fix_path_deps600() { fix_path_deps(); }
#[test] fn fix_path_deps601() { fix_path_deps(); }
#[test] fn fix_path_deps602() { fix_path_deps(); }
#[test] fn fix_path_deps603() { fix_path_deps(); }
#[test] fn fix_path_deps604() { fix_path_deps(); }
#[test] fn fix_path_deps605() { fix_path_deps(); }
#[test] fn fix_path_deps606() { fix_path_deps(); }
#[test] fn fix_path_deps607() { fix_path_deps(); }
#[test] fn fix_path_deps608() { fix_path_deps(); }
#[test] fn fix_path_deps609() { fix_path_deps(); }
#[test] fn fix_path_deps610() { fix_path_deps(); }
#[test] fn fix_path_deps611() { fix_path_deps(); }
#[test] fn fix_path_deps612() { fix_path_deps(); }
#[test] fn fix_path_deps613() { fix_path_deps(); }
#[test] fn fix_path_deps614() { fix_path_deps(); }
#[test] fn fix_path_deps615() { fix_path_deps(); }
#[test] fn fix_path_deps616() { fix_path_deps(); }
#[test] fn fix_path_deps617() { fix_path_deps(); }
#[test] fn fix_path_deps618() { fix_path_deps(); }
#[test] fn fix_path_deps619() { fix_path_deps(); }
#[test] fn fix_path_deps620() { fix_path_deps(); }
#[test] fn fix_path_deps621() { fix_path_deps(); }
#[test] fn fix_path_deps622() { fix_path_deps(); }
#[test] fn fix_path_deps623() { fix_path_deps(); }
#[test] fn fix_path_deps624() { fix_path_deps(); }
#[test] fn fix_path_deps625() { fix_path_deps(); }
#[test] fn fix_path_deps626() { fix_path_deps(); }
#[test] fn fix_path_deps627() { fix_path_deps(); }
#[test] fn fix_path_deps628() { fix_path_deps(); }
#[test] fn fix_path_deps629() { fix_path_deps(); }
#[test] fn fix_path_deps630() { fix_path_deps(); }
#[test] fn fix_path_deps631() { fix_path_deps(); }
#[test] fn fix_path_deps632() { fix_path_deps(); }
#[test] fn fix_path_deps633() { fix_path_deps(); }
#[test] fn fix_path_deps634() { fix_path_deps(); }
#[test] fn fix_path_deps635() { fix_path_deps(); }
#[test] fn fix_path_deps636() { fix_path_deps(); }
#[test] fn fix_path_deps637() { fix_path_deps(); }
#[test] fn fix_path_deps638() { fix_path_deps(); }
#[test] fn fix_path_deps639() { fix_path_deps(); }
#[test] fn fix_path_deps640() { fix_path_deps(); }
#[test] fn fix_path_deps641() { fix_path_deps(); }
#[test] fn fix_path_deps642() { fix_path_deps(); }
#[test] fn fix_path_deps643() { fix_path_deps(); }
#[test] fn fix_path_deps644() { fix_path_deps(); }
#[test] fn fix_path_deps645() { fix_path_deps(); }
#[test] fn fix_path_deps646() { fix_path_deps(); }
#[test] fn fix_path_deps647() { fix_path_deps(); }
#[test] fn fix_path_deps648() { fix_path_deps(); }
#[test] fn fix_path_deps649() { fix_path_deps(); }
#[test] fn fix_path_deps650() { fix_path_deps(); }
#[test] fn fix_path_deps651() { fix_path_deps(); }
#[test] fn fix_path_deps652() { fix_path_deps(); }
#[test] fn fix_path_deps653() { fix_path_deps(); }
#[test] fn fix_path_deps654() { fix_path_deps(); }
#[test] fn fix_path_deps655() { fix_path_deps(); }
#[test] fn fix_path_deps656() { fix_path_deps(); }
#[test] fn fix_path_deps657() { fix_path_deps(); }
#[test] fn fix_path_deps658() { fix_path_deps(); }
#[test] fn fix_path_deps659() { fix_path_deps(); }
#[test] fn fix_path_deps660() { fix_path_deps(); }
#[test] fn fix_path_deps661() { fix_path_deps(); }
#[test] fn fix_path_deps662() { fix_path_deps(); }
#[test] fn fix_path_deps663() { fix_path_deps(); }
#[test] fn fix_path_deps664() { fix_path_deps(); }
#[test] fn fix_path_deps665() { fix_path_deps(); }
#[test] fn fix_path_deps666() { fix_path_deps(); }
#[test] fn fix_path_deps667() { fix_path_deps(); }
#[test] fn fix_path_deps668() { fix_path_deps(); }
#[test] fn fix_path_deps669() { fix_path_deps(); }
#[test] fn fix_path_deps670() { fix_path_deps(); }
#[test] fn fix_path_deps671() { fix_path_deps(); }
#[test] fn fix_path_deps672() { fix_path_deps(); }
#[test] fn fix_path_deps673() { fix_path_deps(); }
#[test] fn fix_path_deps674() { fix_path_deps(); }
#[test] fn fix_path_deps675() { fix_path_deps(); }
#[test] fn fix_path_deps676() { fix_path_deps(); }
#[test] fn fix_path_deps677() { fix_path_deps(); }
#[test] fn fix_path_deps678() { fix_path_deps(); }
#[test] fn fix_path_deps679() { fix_path_deps(); }
#[test] fn fix_path_deps680() { fix_path_deps(); }
#[test] fn fix_path_deps681() { fix_path_deps(); }
#[test] fn fix_path_deps682() { fix_path_deps(); }
#[test] fn fix_path_deps683() { fix_path_deps(); }
#[test] fn fix_path_deps684() { fix_path_deps(); }
#[test] fn fix_path_deps685() { fix_path_deps(); }
#[test] fn fix_path_deps686() { fix_path_deps(); }
#[test] fn fix_path_deps687() { fix_path_deps(); }
#[test] fn fix_path_deps688() { fix_path_deps(); }
#[test] fn fix_path_deps689() { fix_path_deps(); }
#[test] fn fix_path_deps690() { fix_path_deps(); }
#[test] fn fix_path_deps691() { fix_path_deps(); }
#[test] fn fix_path_deps692() { fix_path_deps(); }
#[test] fn fix_path_deps693() { fix_path_deps(); }
#[test] fn fix_path_deps694() { fix_path_deps(); }
#[test] fn fix_path_deps695() { fix_path_deps(); }
#[test] fn fix_path_deps696() { fix_path_deps(); }
#[test] fn fix_path_deps697() { fix_path_deps(); }
#[test] fn fix_path_deps698() { fix_path_deps(); }
#[test] fn fix_path_deps699() { fix_path_deps(); }
#[test] fn fix_path_deps700() { fix_path_deps(); }
#[test] fn fix_path_deps701() { fix_path_deps(); }
#[test] fn fix_path_deps702() { fix_path_deps(); }
#[test] fn fix_path_deps703() { fix_path_deps(); }
#[test] fn fix_path_deps704() { fix_path_deps(); }
#[test] fn fix_path_deps705() { fix_path_deps(); }
#[test] fn fix_path_deps706() { fix_path_deps(); }
#[test] fn fix_path_deps707() { fix_path_deps(); }
#[test] fn fix_path_deps708() { fix_path_deps(); }
#[test] fn fix_path_deps709() { fix_path_deps(); }
#[test] fn fix_path_deps710() { fix_path_deps(); }
#[test] fn fix_path_deps711() { fix_path_deps(); }
#[test] fn fix_path_deps712() { fix_path_deps(); }
#[test] fn fix_path_deps713() { fix_path_deps(); }
#[test] fn fix_path_deps714() { fix_path_deps(); }
#[test] fn fix_path_deps715() { fix_path_deps(); }
#[test] fn fix_path_deps716() { fix_path_deps(); }
#[test] fn fix_path_deps717() { fix_path_deps(); }
#[test] fn fix_path_deps718() { fix_path_deps(); }
#[test] fn fix_path_deps719() { fix_path_deps(); }
#[test] fn fix_path_deps720() { fix_path_deps(); }
#[test] fn fix_path_deps721() { fix_path_deps(); }
#[test] fn fix_path_deps722() { fix_path_deps(); }
#[test] fn fix_path_deps723() { fix_path_deps(); }
#[test] fn fix_path_deps724() { fix_path_deps(); }
#[test] fn fix_path_deps725() { fix_path_deps(); }
#[test] fn fix_path_deps726() { fix_path_deps(); }
#[test] fn fix_path_deps727() { fix_path_deps(); }
#[test] fn fix_path_deps728() { fix_path_deps(); }
#[test] fn fix_path_deps729() { fix_path_deps(); }
#[test] fn fix_path_deps730() { fix_path_deps(); }
#[test] fn fix_path_deps731() { fix_path_deps(); }
#[test] fn fix_path_deps732() { fix_path_deps(); }
#[test] fn fix_path_deps733() { fix_path_deps(); }
#[test] fn fix_path_deps734() { fix_path_deps(); }
#[test] fn fix_path_deps735() { fix_path_deps(); }
#[test] fn fix_path_deps736() { fix_path_deps(); }
#[test] fn fix_path_deps737() { fix_path_deps(); }
#[test] fn fix_path_deps738() { fix_path_deps(); }
#[test] fn fix_path_deps739() { fix_path_deps(); }
#[test] fn fix_path_deps740() { fix_path_deps(); }
#[test] fn fix_path_deps741() { fix_path_deps(); }
#[test] fn fix_path_deps742() { fix_path_deps(); }
#[test] fn fix_path_deps743() { fix_path_deps(); }
#[test] fn fix_path_deps744() { fix_path_deps(); }
#[test] fn fix_path_deps745() { fix_path_deps(); }
#[test] fn fix_path_deps746() { fix_path_deps(); }
#[test] fn fix_path_deps747() { fix_path_deps(); }
#[test] fn fix_path_deps748() { fix_path_deps(); }
#[test] fn fix_path_deps749() { fix_path_deps(); }
#[test] fn fix_path_deps750() { fix_path_deps(); }
#[test] fn fix_path_deps751() { fix_path_deps(); }
#[test] fn fix_path_deps752() { fix_path_deps(); }
#[test] fn fix_path_deps753() { fix_path_deps(); }
#[test] fn fix_path_deps754() { fix_path_deps(); }
#[test] fn fix_path_deps755() { fix_path_deps(); }
#[test] fn fix_path_deps756() { fix_path_deps(); }
#[test] fn fix_path_deps757() { fix_path_deps(); }
#[test] fn fix_path_deps758() { fix_path_deps(); }
#[test] fn fix_path_deps759() { fix_path_deps(); }
#[test] fn fix_path_deps760() { fix_path_deps(); }
#[test] fn fix_path_deps761() { fix_path_deps(); }
#[test] fn fix_path_deps762() { fix_path_deps(); }
#[test] fn fix_path_deps763() { fix_path_deps(); }
#[test] fn fix_path_deps764() { fix_path_deps(); }
#[test] fn fix_path_deps765() { fix_path_deps(); }
#[test] fn fix_path_deps766() { fix_path_deps(); }
#[test] fn fix_path_deps767() { fix_path_deps(); }
#[test] fn fix_path_deps768() { fix_path_deps(); }
#[test] fn fix_path_deps769() { fix_path_deps(); }
#[test] fn fix_path_deps770() { fix_path_deps(); }
#[test] fn fix_path_deps771() { fix_path_deps(); }
#[test] fn fix_path_deps772() { fix_path_deps(); }
#[test] fn fix_path_deps773() { fix_path_deps(); }
#[test] fn fix_path_deps774() { fix_path_deps(); }
#[test] fn fix_path_deps775() { fix_path_deps(); }
#[test] fn fix_path_deps776() { fix_path_deps(); }
#[test] fn fix_path_deps777() { fix_path_deps(); }
#[test] fn fix_path_deps778() { fix_path_deps(); }
#[test] fn fix_path_deps779() { fix_path_deps(); }
#[test] fn fix_path_deps780() { fix_path_deps(); }
#[test] fn fix_path_deps781() { fix_path_deps(); }
#[test] fn fix_path_deps782() { fix_path_deps(); }
#[test] fn fix_path_deps783() { fix_path_deps(); }
#[test] fn fix_path_deps784() { fix_path_deps(); }
#[test] fn fix_path_deps785() { fix_path_deps(); }
#[test] fn fix_path_deps786() { fix_path_deps(); }
#[test] fn fix_path_deps787() { fix_path_deps(); }
#[test] fn fix_path_deps788() { fix_path_deps(); }
#[test] fn fix_path_deps789() { fix_path_deps(); }
#[test] fn fix_path_deps790() { fix_path_deps(); }
#[test] fn fix_path_deps791() { fix_path_deps(); }
#[test] fn fix_path_deps792() { fix_path_deps(); }
#[test] fn fix_path_deps793() { fix_path_deps(); }
#[test] fn fix_path_deps794() { fix_path_deps(); }
#[test] fn fix_path_deps795() { fix_path_deps(); }
#[test] fn fix_path_deps796() { fix_path_deps(); }
#[test] fn fix_path_deps797() { fix_path_deps(); }
#[test] fn fix_path_deps798() { fix_path_deps(); }
#[test] fn fix_path_deps799() { fix_path_deps(); }
#[test] fn fix_path_deps800() { fix_path_deps(); }


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
        .with_stderr(
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
fn local_paths_no_fix() {
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "src/lib.rs",
            r#"
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
warning: failed to find `#![feature(rust_2018_preview)]` in `src/lib.rs`
this may cause `cargo fix` to not be able to fix all
issues in preparation for the 2018 edition
[FINISHED] [..]
";
    p.cargo("fix --edition --allow-no-vcs")
        .with_stderr(stderr)
        .with_stdout("")
        .run();
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
                cargo-features = ["edition"]

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
        .masquerade_as_nightly_cargo()
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
             error: no VCS found for this project and `cargo fix` can potentially perform \
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
error: the working directory of this project has uncommitted changes, \
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
error: the working directory of this project has uncommitted changes, \
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
    if !is_nightly() {
        return;
    }
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ['edition']

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
        .masquerade_as_nightly_cargo()
        .with_stderr_contains(stderr)
        .with_status(101)
        .run();
}

#[test]
fn prepare_for_without_feature_issues_warning() {
    if !is_nightly() {
        return;
    }
    let p = project().file("src/lib.rs", "").build();

    let stderr = "\
[CHECKING] foo v0.0.1 ([..])
warning: failed to find `#![feature(rust_2018_preview)]` in `src/lib.rs`
this may cause `cargo fix` to not be able to fix all
issues in preparation for the 2018 edition
[FINISHED] [..]
";
    p.cargo("fix --edition --allow-no-vcs")
        .masquerade_as_nightly_cargo()
        .with_stderr(stderr)
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
                cargo-features = ['edition']
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
        .masquerade_as_nightly_cargo()
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
    if !is_nightly() {
        return;
    }
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
