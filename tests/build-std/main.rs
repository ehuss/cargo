//! A test suite for `-Zbuild-std` which is much more expensive than the
//! standard test suite.
//!
//! This test suite attempts to perform a full integration test where we
//! actually compile the standard library from source (like the real one) and
//! the various tests associated with that.
//!
//! YOU SHOULD IDEALLY NOT WRITE TESTS HERE.
//!
//! If possible, use `tests/testsuite/standard_lib.rs` instead. That uses a
//! 'mock' sysroot which is much faster to compile. The tests here are
//! extremely intensive and are only intended to run on CI and are theoretically
//! not catching any regressions that `tests/testsuite/standard_lib.rs` isn't
//! already catching.
//!
//! All tests here should use `#[cargo_test(build_std)]` to indicate that
//! boilerplate should be generated to require the nightly toolchain and the
//! `CARGO_RUN_BUILD_STD_TESTS` env var to be set to actually run these tests.
//! Otherwise the tests are skipped.

use cargo_test_support::*;
use std::env;
use std::path::Path;

// Helper methods used in the tests below
trait BuildStd: Sized {
    fn build_std(&mut self) -> &mut Self;
    fn target_host(&mut self) -> &mut Self;
}

impl BuildStd for Execs {
    fn build_std(&mut self) -> &mut Self {
        self.env_remove("CARGO_HOME")
            .env_remove("HOME")
            .arg("-Zno-index-update")
            .arg("-Zbuild-std")
            .masquerade_as_nightly_cargo()
    }

    fn target_host(&mut self) -> &mut Self {
        self.arg("--target").arg(rustc_host());
        self
    }
}

#[cargo_test(build_std)]
fn basic() {
    let p = project()
        .file(
            "src/main.rs",
            "
                fn main() {
                    foo::f();
                }

                #[test]
                fn smoke_bin_unit() {
                    foo::f();
                }
            ",
        )
        .file(
            "src/lib.rs",
            "
                extern crate alloc;
                extern crate proc_macro;

                /// ```
                /// foo::f();
                /// ```
                pub fn f() {
                }

                #[test]
                fn smoke_lib_unit() {
                    f();
                }
            ",
        )
        .file(
            "tests/smoke.rs",
            "
                #[test]
                fn smoke_integration() {
                    foo::f();
                }
            ",
        )
        .build();

    p.cargo("check").build_std().target_host().run();
    p.cargo("build").build_std().target_host().run();
    p.cargo("run").build_std().target_host().run();
    p.cargo("test").build_std().target_host().run();
}

#[cargo_test(build_std)]
fn cross_custom() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                core = { stdlib = true }
            "#,
        )
        .file("src/lib.rs", "#![no_std] pub fn f() {}")
        .file(
            "custom-target.json",
            r#"
            {
                "llvm-target": "x86_64-unknown-none-gnu",
                "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
                "arch": "x86_64",
                "target-endian": "little",
                "target-pointer-width": "64",
                "target-c-int-width": "32",
                "os": "none",
                "linker-flavor": "ld.lld"
            }
            "#,
        )
        .build();

    p.cargo("build --target custom-target.json -v")
        .build_std()
        .run();
}

#[cargo_test(build_std)]
fn custom_test_framework() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                core = { stdlib = true }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #![no_std]
            #![cfg_attr(test, no_main)]
            #![feature(custom_test_frameworks)]
            #![test_runner(crate::test_runner)]

            pub fn test_runner(_tests: &[&dyn Fn()]) {}

            #[panic_handler]
            fn panic(_info: &core::panic::PanicInfo) -> ! {
                loop {}
            }
            "#,
        )
        .file(
            "target.json",
            r#"
            {
                "llvm-target": "x86_64-unknown-none-gnu",
                "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
                "arch": "x86_64",
                "target-endian": "little",
                "target-pointer-width": "64",
                "target-c-int-width": "32",
                "os": "none",
                "linker-flavor": "ld.lld",
                "linker": "rust-lld",
                "executables": true,
                "panic-strategy": "abort"
            }
            "#,
        )
        .build();

    // This is a bit of a hack to use the rust-lld that ships with most toolchains.
    let sysroot = paths::sysroot();
    let sysroot = Path::new(&sysroot);
    let sysroot_bin = sysroot
        .join("lib")
        .join("rustlib")
        .join(rustc_host())
        .join("bin");
    let path = env::var_os("PATH").unwrap_or_default();
    let mut paths = env::split_paths(&path).collect::<Vec<_>>();
    paths.insert(0, sysroot_bin);
    let new_path = env::join_paths(paths).unwrap();

    p.cargo("test --target target.json --no-run -v")
        .env("PATH", new_path)
        .build_std()
        .run();
}
