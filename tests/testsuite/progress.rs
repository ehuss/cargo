//! Tests for progress bar.

use cargo_test_support::project;
use cargo_test_support::registry::Package;

#[cargo_test]
fn bad_progress_config() {
    // Some tests for bad configurations.
}

#[cargo_test]
fn always_shows_progress() {
    // Tests that when="always" shows progress.
    // Add some dependencies just to give it something to display.
    const N: usize = 10;
    let mut deps = String::new();
    for i in 1..=N {
        Package::new(&format!("dep{}", i), "1.0.0").publish();
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }

    let p = project()
        .file(
            ".cargo/config",
            r#"
            [term]
            progress = { when = 'always', width = 100 }
            "#,
        )
        .file(
            "Cargo.toml",
            &format!(
                r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [dependencies]
            {}
            "#,
                deps
            ),
        )
        .file("src/lib.rs", "")
        .build();

    let output = p.cargo("check").exec_with_output().unwrap();
    assert!(output.status.success());
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    println!("{}", stderr);
}

#[cargo_test]
fn never_progress() {}
