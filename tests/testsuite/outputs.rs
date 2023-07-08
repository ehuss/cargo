//! Test for Cargo tracking the outputs from rustc.

use super::config::ConfigBuilder;
use cargo::core::compiler::{CompileMode, Context, FileFlavor, Unit};
use cargo::core::compiler::{DefaultExecutor, Executor, UnitInterner};
use cargo::core::{TargetKind, Workspace};
use cargo::ops::{self, CompileFilter, CompileOptions};
use cargo_test_support::paths::CargoPathExt;
use cargo_test_support::{project, t};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

#[cargo_test(nightly, reason = "high risk of breaking in rust-lang/rust")]
fn knows_all_the_outputs() {
    // Check that Cargo knows the list of output files for lots of targets.
    let output = match Command::new("rustup")
        .arg("target")
        .arg("list")
        .arg("--installed")
        .output()
    {
        Err(e) => {
            eprintln!("rustup failed: {:?}", e);
            return;
        }
        Ok(output) => output,
    };
    let stdout = String::from_utf8(output.stdout).unwrap();
    if !output.status.success() {
        eprintln!(
            "rustup failed:\n{}\n{}",
            stdout,
            String::from_utf8(output.stderr).unwrap()
        );
    }
    let targets: Vec<&str> = stdout.lines().collect();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo-pkg"
            version = "0.1.0"
            edition = "2018"

            [lib]
            name = "foo_lib"
            path = "src/lib.rs"
            crate-type = ["rlib", "dylib", "staticlib"]

            [[example]]
            name = "foo-ex-lib"
            crate-type = ["rlib", "dylib", "staticlib"]

            [[example]]
            name = "foo-ex-cdylib"
            crate-type = ["cdylib"]
            "#,
        )
        .file("build.rs", "fn main() {}")
        .file("src/lib.rs", "")
        .file("src/bin/foo-bin.rs", "fn main() {}")
        .file("examples/foo-ex-bin.rs", "fn main() {}")
        .file("examples/foo-ex-lib.rs", "")
        .file("examples/foo-ex-cdylib.rs", "")
        .file("benches/foo-bench.rs", "")
        .file("tests/foo-test.rs", "")
        .build();

    let config = ConfigBuilder::new().build();
    let ws = t!(Workspace::new(&p.root().join("Cargo.toml"), &config));
    for mode in &[CompileMode::Build] {
        let mut options = t!(CompileOptions::new(&config, *mode));
        options.filter = CompileFilter::new_all_targets();
        let interner = UnitInterner::new();
        let bcx = t!(ops::create_bcx(&ws, &options, &interner));

        let mut cx = t!(Context::new(&bcx));
        t!(cx.prepare_units());
        let mut outputs = Vec::new();
        for unit in bcx.unit_graph.keys() {
            let unit_outputs = t!(cx.outputs(unit));
            println!(
                "{} {:?} {:?}",
                unit.target.name(),
                unit.mode,
                unit.target.kind()
            );
            for output in unit_outputs.iter() {
                let path = output.path.strip_prefix(p.target_debug_dir()).unwrap();
                let hlink = output
                    .hardlink
                    .as_ref()
                    .map(|hl| hl.strip_prefix(p.target_debug_dir()).unwrap());
                println!("  {:?} {} {:?}", output.flavor, path.display(), hlink);
                outputs.push((
                    unit.target.name(),
                    unit.mode,
                    output.path.clone(),
                    output.hardlink.clone(),
                ));
            }
            // let expected = get_expected("x86_64-apple-darwin", unit);
        }
        drop(cx);
        let cx = t!(Context::new(&bcx));
        let exec: Arc<dyn Executor> = Arc::new(DefaultExecutor);
        t!(cx.compile(&exec));
        for output in outputs {
            clean(&output.2);
            if let Some(hlink) = output.3 {
                clean(&hlink)
            }
        }

        fn clean(path: &Path) {
            eprintln!("remove {:?}", path);
            use cargo::util::paths;
            if path.is_symlink() {
                t!(paths::remove_file(path));
            } else if path.is_dir() {
                t!(paths::remove_dir_all(path));
            } else {
                t!(paths::remove_file(path));
            }
        }

        for entry in walkdir::WalkDir::new(p.build_dir()) {
            let entry = t!(entry);
            println!("{:?}", entry.path());
        }
    }
}

type EFile = (&'static str, FileFlavor, Option<&'static str>);
type EEntry = (&'static str, CompileMode, TargetKind, &'static [EFile]);
use CompileMode::*;
use FileFlavor::*;
use TargetKind::*;
#[rustfmt::skip]
const EXPECTED: &[EEntry] = &[
    ("x86_64-apple_darwin", Build, Bin, &[
        ("deps/foo-bin-*", Normal, Some("foo-bin")),
        ("deps/foo-bin-*.dSYM", DebugInfo, Some("foo-bin.dSYM")),
        ("deps/foo-bin-*.d", DepInfo, None),
    ]),
    ("x86_64-apple_darwin", Build, ExampleBin, &[
        ("examples/foo-ex1-*", Normal, Some("examples/foo-ex1")),
        ("examples/foo-ex1-*.dSYM", DebugInfo, Some("examples/foo-ex1.dSYM")),
        ("examples/foo-ex1-*.d", DepInfo, None),
    ])
];

fn get_expected(triple: &str, unit: &Unit) -> &'static [EFile] {
    let mut i = EXPECTED
        .iter()
        .filter(|ent| ent.0 == triple && ent.1 == unit.mode && &ent.2 == unit.target.kind());
    let result = i.next().unwrap();
    // Check that there is only one match.
    assert!(i.next().is_none());
    result.3
}
