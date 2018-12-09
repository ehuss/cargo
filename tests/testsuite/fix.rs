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

#[test] fn broken_fixes_backed_out0() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out1() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out2() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out3() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out4() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out5() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out6() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out7() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out8() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out9() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out10() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out11() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out12() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out13() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out14() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out15() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out16() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out17() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out18() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out19() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out20() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out21() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out22() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out23() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out24() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out25() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out26() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out27() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out28() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out29() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out30() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out31() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out32() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out33() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out34() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out35() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out36() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out37() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out38() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out39() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out40() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out41() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out42() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out43() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out44() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out45() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out46() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out47() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out48() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out49() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out50() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out51() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out52() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out53() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out54() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out55() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out56() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out57() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out58() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out59() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out60() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out61() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out62() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out63() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out64() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out65() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out66() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out67() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out68() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out69() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out70() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out71() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out72() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out73() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out74() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out75() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out76() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out77() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out78() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out79() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out80() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out81() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out82() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out83() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out84() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out85() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out86() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out87() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out88() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out89() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out90() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out91() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out92() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out93() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out94() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out95() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out96() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out97() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out98() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out99() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out100() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out101() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out102() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out103() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out104() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out105() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out106() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out107() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out108() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out109() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out110() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out111() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out112() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out113() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out114() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out115() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out116() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out117() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out118() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out119() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out120() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out121() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out122() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out123() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out124() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out125() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out126() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out127() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out128() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out129() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out130() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out131() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out132() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out133() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out134() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out135() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out136() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out137() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out138() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out139() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out140() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out141() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out142() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out143() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out144() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out145() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out146() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out147() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out148() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out149() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out150() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out151() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out152() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out153() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out154() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out155() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out156() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out157() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out158() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out159() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out160() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out161() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out162() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out163() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out164() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out165() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out166() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out167() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out168() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out169() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out170() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out171() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out172() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out173() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out174() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out175() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out176() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out177() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out178() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out179() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out180() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out181() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out182() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out183() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out184() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out185() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out186() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out187() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out188() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out189() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out190() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out191() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out192() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out193() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out194() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out195() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out196() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out197() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out198() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out199() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out200() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out201() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out202() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out203() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out204() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out205() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out206() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out207() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out208() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out209() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out210() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out211() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out212() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out213() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out214() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out215() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out216() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out217() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out218() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out219() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out220() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out221() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out222() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out223() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out224() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out225() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out226() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out227() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out228() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out229() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out230() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out231() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out232() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out233() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out234() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out235() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out236() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out237() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out238() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out239() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out240() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out241() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out242() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out243() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out244() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out245() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out246() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out247() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out248() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out249() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out250() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out251() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out252() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out253() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out254() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out255() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out256() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out257() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out258() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out259() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out260() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out261() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out262() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out263() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out264() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out265() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out266() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out267() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out268() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out269() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out270() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out271() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out272() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out273() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out274() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out275() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out276() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out277() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out278() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out279() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out280() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out281() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out282() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out283() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out284() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out285() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out286() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out287() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out288() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out289() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out290() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out291() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out292() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out293() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out294() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out295() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out296() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out297() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out298() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out299() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out300() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out301() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out302() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out303() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out304() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out305() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out306() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out307() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out308() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out309() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out310() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out311() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out312() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out313() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out314() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out315() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out316() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out317() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out318() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out319() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out320() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out321() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out322() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out323() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out324() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out325() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out326() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out327() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out328() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out329() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out330() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out331() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out332() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out333() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out334() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out335() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out336() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out337() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out338() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out339() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out340() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out341() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out342() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out343() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out344() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out345() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out346() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out347() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out348() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out349() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out350() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out351() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out352() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out353() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out354() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out355() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out356() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out357() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out358() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out359() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out360() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out361() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out362() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out363() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out364() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out365() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out366() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out367() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out368() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out369() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out370() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out371() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out372() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out373() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out374() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out375() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out376() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out377() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out378() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out379() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out380() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out381() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out382() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out383() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out384() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out385() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out386() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out387() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out388() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out389() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out390() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out391() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out392() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out393() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out394() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out395() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out396() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out397() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out398() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out399() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out400() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out401() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out402() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out403() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out404() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out405() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out406() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out407() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out408() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out409() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out410() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out411() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out412() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out413() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out414() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out415() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out416() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out417() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out418() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out419() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out420() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out421() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out422() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out423() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out424() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out425() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out426() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out427() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out428() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out429() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out430() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out431() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out432() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out433() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out434() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out435() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out436() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out437() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out438() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out439() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out440() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out441() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out442() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out443() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out444() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out445() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out446() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out447() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out448() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out449() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out450() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out451() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out452() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out453() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out454() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out455() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out456() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out457() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out458() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out459() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out460() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out461() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out462() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out463() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out464() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out465() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out466() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out467() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out468() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out469() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out470() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out471() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out472() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out473() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out474() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out475() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out476() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out477() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out478() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out479() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out480() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out481() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out482() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out483() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out484() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out485() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out486() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out487() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out488() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out489() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out490() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out491() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out492() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out493() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out494() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out495() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out496() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out497() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out498() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out499() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out500() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out501() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out502() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out503() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out504() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out505() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out506() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out507() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out508() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out509() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out510() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out511() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out512() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out513() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out514() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out515() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out516() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out517() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out518() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out519() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out520() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out521() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out522() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out523() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out524() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out525() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out526() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out527() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out528() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out529() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out530() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out531() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out532() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out533() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out534() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out535() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out536() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out537() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out538() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out539() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out540() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out541() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out542() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out543() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out544() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out545() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out546() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out547() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out548() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out549() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out550() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out551() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out552() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out553() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out554() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out555() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out556() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out557() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out558() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out559() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out560() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out561() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out562() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out563() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out564() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out565() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out566() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out567() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out568() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out569() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out570() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out571() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out572() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out573() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out574() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out575() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out576() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out577() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out578() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out579() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out580() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out581() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out582() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out583() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out584() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out585() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out586() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out587() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out588() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out589() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out590() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out591() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out592() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out593() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out594() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out595() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out596() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out597() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out598() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out599() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out600() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out601() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out602() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out603() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out604() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out605() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out606() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out607() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out608() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out609() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out610() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out611() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out612() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out613() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out614() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out615() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out616() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out617() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out618() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out619() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out620() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out621() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out622() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out623() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out624() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out625() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out626() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out627() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out628() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out629() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out630() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out631() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out632() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out633() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out634() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out635() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out636() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out637() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out638() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out639() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out640() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out641() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out642() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out643() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out644() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out645() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out646() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out647() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out648() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out649() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out650() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out651() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out652() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out653() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out654() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out655() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out656() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out657() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out658() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out659() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out660() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out661() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out662() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out663() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out664() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out665() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out666() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out667() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out668() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out669() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out670() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out671() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out672() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out673() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out674() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out675() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out676() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out677() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out678() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out679() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out680() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out681() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out682() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out683() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out684() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out685() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out686() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out687() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out688() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out689() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out690() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out691() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out692() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out693() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out694() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out695() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out696() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out697() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out698() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out699() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out700() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out701() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out702() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out703() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out704() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out705() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out706() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out707() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out708() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out709() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out710() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out711() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out712() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out713() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out714() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out715() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out716() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out717() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out718() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out719() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out720() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out721() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out722() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out723() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out724() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out725() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out726() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out727() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out728() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out729() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out730() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out731() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out732() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out733() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out734() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out735() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out736() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out737() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out738() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out739() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out740() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out741() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out742() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out743() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out744() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out745() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out746() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out747() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out748() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out749() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out750() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out751() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out752() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out753() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out754() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out755() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out756() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out757() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out758() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out759() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out760() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out761() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out762() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out763() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out764() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out765() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out766() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out767() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out768() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out769() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out770() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out771() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out772() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out773() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out774() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out775() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out776() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out777() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out778() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out779() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out780() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out781() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out782() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out783() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out784() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out785() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out786() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out787() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out788() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out789() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out790() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out791() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out792() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out793() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out794() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out795() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out796() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out797() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out798() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out799() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out800() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out801() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out802() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out803() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out804() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out805() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out806() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out807() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out808() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out809() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out810() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out811() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out812() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out813() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out814() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out815() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out816() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out817() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out818() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out819() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out820() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out821() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out822() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out823() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out824() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out825() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out826() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out827() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out828() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out829() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out830() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out831() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out832() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out833() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out834() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out835() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out836() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out837() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out838() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out839() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out840() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out841() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out842() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out843() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out844() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out845() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out846() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out847() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out848() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out849() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out850() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out851() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out852() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out853() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out854() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out855() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out856() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out857() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out858() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out859() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out860() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out861() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out862() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out863() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out864() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out865() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out866() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out867() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out868() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out869() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out870() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out871() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out872() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out873() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out874() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out875() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out876() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out877() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out878() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out879() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out880() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out881() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out882() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out883() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out884() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out885() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out886() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out887() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out888() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out889() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out890() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out891() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out892() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out893() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out894() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out895() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out896() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out897() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out898() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out899() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out900() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out901() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out902() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out903() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out904() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out905() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out906() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out907() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out908() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out909() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out910() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out911() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out912() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out913() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out914() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out915() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out916() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out917() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out918() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out919() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out920() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out921() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out922() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out923() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out924() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out925() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out926() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out927() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out928() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out929() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out930() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out931() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out932() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out933() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out934() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out935() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out936() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out937() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out938() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out939() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out940() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out941() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out942() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out943() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out944() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out945() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out946() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out947() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out948() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out949() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out950() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out951() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out952() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out953() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out954() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out955() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out956() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out957() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out958() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out959() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out960() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out961() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out962() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out963() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out964() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out965() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out966() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out967() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out968() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out969() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out970() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out971() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out972() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out973() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out974() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out975() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out976() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out977() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out978() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out979() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out980() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out981() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out982() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out983() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out984() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out985() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out986() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out987() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out988() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out989() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out990() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out991() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out992() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out993() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out994() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out995() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out996() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out997() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out998() { broken_fixes_backed_out(); }
#[test] fn broken_fixes_backed_out999() { broken_fixes_backed_out(); }


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
        )
        .with_stderr_does_not_contain("[..][FIXING][..]")
        .run();
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
