use cargo_test_support::registry::{Dependency, Package};
use cargo_test_support::ProjectBuilder;
use cargo_test_support::{basic_manifest, is_nightly, paths, project, rustc_host, Execs};
use std::path::PathBuf;

struct Setup {
    rustc_wrapper: PathBuf,
    real_sysroot: String,
}

fn setup() -> Option<Setup> {
    if !is_nightly() {
        // -Zbuild-std is nightly
        // We don't want these tests to run on rust-lang/rust.
        return None;
    }

    // Our mock sysroot requires a few packages from crates.io, so make sure
    // they're "published" to crates.io. Also edit their code a bit to make sure
    // that they have access to our custom crates with custom apis.
    Package::new("registry-dep-using-core", "1.0.0")
        .file(
            "src/lib.rs",
            "
                #![no_std]

                #[cfg(feature = \"mockbuild\")]
                pub fn custom_api() {
                }

                #[cfg(not(feature = \"mockbuild\"))]
                pub fn non_sysroot_api() {
                    core::custom_api();
                }
            ",
        )
        .add_dep(Dependency::new("rustc-std-workspace-core", "*").optional(true))
        .feature("mockbuild", &["rustc-std-workspace-core"])
        .publish();
    Package::new("registry-dep-using-alloc", "1.0.0")
        .file(
            "src/lib.rs",
            "
                #![no_std]

                extern crate alloc;

                #[cfg(feature = \"mockbuild\")]
                pub fn custom_api() {
                }

                #[cfg(not(feature = \"mockbuild\"))]
                pub fn non_sysroot_api() {
                    core::custom_api();
                    alloc::custom_api();
                }
            ",
        )
        .add_dep(Dependency::new("rustc-std-workspace-core", "*").optional(true))
        .add_dep(Dependency::new("rustc-std-workspace-alloc", "*").optional(true))
        .feature(
            "mockbuild",
            &["rustc-std-workspace-core", "rustc-std-workspace-alloc"],
        )
        .publish();
    Package::new("registry-dep-using-std", "1.0.0")
        .file(
            "src/lib.rs",
            "
                #[cfg(feature = \"mockbuild\")]
                pub fn custom_api() {
                }

                #[cfg(not(feature = \"mockbuild\"))]
                pub fn non_sysroot_api() {
                    std::custom_api();
                }
            ",
        )
        .add_dep(Dependency::new("rustc-std-workspace-std", "*").optional(true))
        .feature("mockbuild", &["rustc-std-workspace-std"])
        .publish();

    let p = ProjectBuilder::new(paths::root().join("rustc-wrapper"))
        .file(
            "src/main.rs",
            r#"
                use std::process::Command;
                use std::env;
                fn main() {
                    let mut args = env::args().skip(1).collect::<Vec<_>>();

                    let is_sysroot_crate = env::var_os("RUSTC_BOOTSTRAP").is_some();
                    if is_sysroot_crate {
                        args.push("--sysroot".to_string());
                        args.push(env::var("REAL_SYSROOT").unwrap());
                    } else if args.iter().any(|arg| arg == "--target") {
                        // build-std target unit
                        args.push("--sysroot".to_string());
                        args.push("/path/to/nowhere".to_string());
                    } else {
                        // host unit, do not use sysroot
                    }

                    let ret = Command::new(&args[0]).args(&args[1..]).status().unwrap();
                    std::process::exit(ret.code().unwrap_or(1));
                }
            "#,
        )
        .build();
    p.cargo("build").run();

    Some(Setup {
        rustc_wrapper: p.bin("foo"),
        real_sysroot: paths::sysroot(),
    })
}

fn enable_build_std(e: &mut Execs, setup: &Setup) {
    // First up, force Cargo to use our "mock sysroot" which mimics what
    // libstd looks like upstream.
    let root = paths::root();
    let root = root
        .parent() // chop off test name
        .unwrap()
        .parent() // chop off `citN`
        .unwrap()
        .parent() // chop off `target`
        .unwrap()
        .join("tests/testsuite/mock-std");
    e.env("__CARGO_TESTS_ONLY_SRC_ROOT", &root);

    e.arg("-Zbuild-std");
    e.masquerade_as_nightly_cargo();

    // We do various shenanigans to ensure our "mock sysroot" actually links
    // with the real sysroot, so we don't have to actually recompile std for
    // each test. Perform all that logic here, namely:
    //
    // * RUSTC_WRAPPER - uses our shim executable built above to control rustc
    // * REAL_SYSROOT - used by the shim executable to swap out to the real
    //   sysroot temporarily for some compilations
    // * RUST{,DOC}FLAGS - an extra `-L` argument to ensure we can always load
    //   crates from the sysroot, but only indirectly through other crates.
    e.env("RUSTC_WRAPPER", &setup.rustc_wrapper);
    e.env("REAL_SYSROOT", &setup.real_sysroot);
    let libdir = format!("/lib/rustlib/{}/lib", rustc_host());
    e.env(
        "RUSTFLAGS",
        format!("-Ldependency={}{}", setup.real_sysroot, libdir),
    );
    e.env(
        "RUSTDOCFLAGS",
        format!("-Ldependency={}{}", setup.real_sysroot, libdir),
    );
}

// Helper methods used in the tests below
trait BuildStd: Sized {
    fn build_std(&mut self, setup: &Setup) -> &mut Self;
    fn build_std_error(&mut self) -> &mut Self;
    fn target_host(&mut self) -> &mut Self;
}

impl BuildStd for Execs {
    fn build_std(&mut self, setup: &Setup) -> &mut Self {
        enable_build_std(self, setup);
        self
    }

    /// This is a variant of `build_std` that doesn't set up a mock
    /// environment, and should only be used for error testing that doesn't
    /// trigger a build. Be careful checking the output, since concurrent
    /// tests may still cause "Blocking waiting for file lock on package
    /// cache" message.
    fn build_std_error(&mut self) -> &mut Self {
        self.arg("-Zno-index-update")
            .arg("-Zbuild-std")
            .masquerade_as_nightly_cargo()
            .env_remove("CARGO_HOME")
            .env_remove("HOME")
    }

    fn target_host(&mut self) -> &mut Self {
        self.arg("--target").arg(rustc_host())
    }
}

#[cargo_test]
fn basic() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };

    let p = project()
        .file(
            "src/main.rs",
            "
                fn main() {
                    std::custom_api();
                    foo::f();
                }

                #[test]
                fn smoke_bin_unit() {
                    std::custom_api();
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
                    core::custom_api();
                    std::custom_api();
                    alloc::custom_api();
                    proc_macro::custom_api();
                }

                #[test]
                fn smoke_lib_unit() {
                    std::custom_api();
                    f();
                }
            ",
        )
        .file(
            "tests/smoke.rs",
            "
                #[test]
                fn smoke_integration() {
                    std::custom_api();
                    foo::f();
                }
            ",
        )
        .build();

    p.cargo("check -v").build_std(&setup).target_host().run();
    p.cargo("build").build_std(&setup).target_host().run();
    p.cargo("run").build_std(&setup).target_host().run();
    p.cargo("test").build_std(&setup).target_host().run();
}

#[cargo_test]
fn simple_lib_std() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project().file("src/lib.rs", "").build();
    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[RUNNING] `[..]--crate-name std [..]`")
        .run();
    // Check freshness.
    p.change_file("src/lib.rs", " ");
    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[FRESH] std[..]")
        .run();
}

#[cargo_test]
fn simple_bin_std() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project().file("src/main.rs", "fn main() {}").build();
    p.cargo("run -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn lib_nostd() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
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
                pub fn foo() {
                    assert_eq!(core::u8::MIN, 0);
                }
            "#,
        )
        .build();
    p.cargo("build -v --lib")
        .build_std(&setup)
        .target_host()
        .with_stderr_does_not_contain("[..]libstd[..]")
        // Panic runtimes should not be built or used.
        .with_stderr_does_not_contain("[..]panic[..]")
        .run();
}

#[cargo_test]
fn check_core() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
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
        .file("src/lib.rs", "#![no_std] fn unused_fn() {}")
        .build();

    p.cargo("check -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[WARNING] [..]unused_fn[..]`")
        .run();
}

#[cargo_test]
fn depend_same_as_std() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };

    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    registry_dep_using_core::non_sysroot_api();
                    registry_dep_using_alloc::non_sysroot_api();
                    registry_dep_using_std::non_sysroot_api();
                }
            "#,
        )
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                registry-dep-using-core = "1.0"
                registry-dep-using-alloc = "1.0"
                registry-dep-using-std = "1.0"
            "#,
        )
        .build();

    p.cargo("build -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn test() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn it_works() {
                        assert_eq!(2 + 2, 4);
                    }
                }
            "#,
        )
        .build();

    p.cargo("test -v")
        .build_std(&setup)
        .target_host()
        .with_stdout_contains("test tests::it_works ... ok")
        .run();
}

#[cargo_test]
fn target_proc_macro() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                extern crate proc_macro;
                pub fn f() {
                    let _ts = proc_macro::TokenStream::new();
                }
            "#,
        )
        .build();

    p.cargo("build -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn bench() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                #![feature(test)]
                extern crate test;

                #[bench]
                fn b1(b: &mut test::Bencher) {
                    b.iter(|| ())
                }
            "#,
        )
        .build();

    p.cargo("bench -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn doc() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                /// Doc
                pub fn f() -> Result<(), ()> {Ok(())}
            "#,
        )
        .build();

    p.cargo("doc -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn check_std() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            "
                extern crate core;
                extern crate alloc;
                extern crate proc_macro;
                pub fn f() {}
            ",
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "tests/t1.rs",
            r#"
                #[test]
                fn t1() {
                    assert_eq!(1, 2);
                }
            "#,
        )
        .build();

    p.cargo("check -v --all-targets")
        .build_std(&setup)
        .target_host()
        .run();
    p.cargo("check -v --all-targets --profile=test")
        .build_std(&setup)
        .target_host()
        .run();
}

#[cargo_test]
fn doctest() {
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                /// Doc
                /// ```
                /// std::custom_api();
                /// ```
                pub fn f() {}
            "#,
        )
        .build();

    p.cargo("test --doc -v")
        .build_std(&setup)
        .with_stdout_contains("test src/lib.rs - f [..] ... ok")
        .target_host()
        .run();
}

#[cargo_test]
fn no_implicit_alloc() {
    // Demonstrate that alloc is not implicitly in scope.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    let _: Vec<i32> = alloc::vec::Vec::new();
                }
            "#,
        )
        .build();

    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[..]use of undeclared [..]`alloc`")
        .with_status(101)
        .run();
}

#[cargo_test]
fn macro_expanded_shadow() {
    // This tests a bug caused by the previous use of `--extern` to directly
    // load sysroot crates. This necessitated the switch to `--sysroot` to
    // retain existing behavior. See
    // https://github.com/rust-lang/wg-cargo-std-aware/issues/40 for more
    // detail.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                macro_rules! a {
                    () => (extern crate std as alloc;)
                }
                a!();
            "#,
        )
        .build();

    p.cargo("build -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn no_explicit_default_in_dep() {
    // no_std with a dependency with no explicit stdlib dependencies, does not build std.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    Package::new("implicit_dep", "0.1.0")
        .file(
            "src/lib.rs",
            r#"
                #![no_std]
                pub fn f() {}
            "#,
        )
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                core = { stdlib = true }
                implicit_dep = "0.1"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![no_std]
                pub fn f() { implicit_dep::f(); }
            "#,
        )
        .build();

    p.cargo("build -v --lib")
        .build_std(&setup)
        .target_host()
        .with_stderr_does_not_contain("[..]libstd[..]")
        .run();

    // Can't access std at all.
    Package::new("implicit_dep", "0.1.1")
        .file("src/lib.rs", "pub fn f() { std::custom_api(); }")
        .publish();

    std::fs::remove_file(p.root().join("Cargo.lock")).unwrap();

    p.cargo("build -v --lib")
        .build_std(&setup)
        .target_host()
        // std not found
        .with_stderr_contains("error[E0463][..]")
        .with_status(101)
        .run();
}

#[cargo_test]
fn gated_config() {
    // Ignore the config without `-Zbuild-std`
    let p = project()
        .file(
            ".cargo/config",
            r#"
                [build.std]
                enabled = true
            "#,
        )
        .file("src/lib.rs", "")
        .build();
    p.cargo("build -v")
        .with_stderr(
            "\
[COMPILING] foo [..]
[RUNNING] `rustc --crate-name foo src/lib.rs [..]
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn explicit_invalid() {
    // Attempt to depend on something that doesn't exist.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                calamari = { stdlib = true }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .target_host()
        .build_std_error()
        .with_status(101)
        .with_stderr_contains(
            "\
[ERROR] stdlib dependency `calamari` not found, required by package `foo v0.1.0 [..]`

Caused by:
  package ID specification `calamari` matched no packages
",
        )
        .run();
}

#[cargo_test]
fn explicit_private() {
    // Attempt to access something built with -Zforce-unstable-if-unmarked
    // without the corresponding #![feature(rustc_private)].
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                something-private = { stdlib = true }
                std = { stdlib = true }
            "#,
        )
        .file("src/lib.rs", "extern crate something_private;")
        .build();

    p.cargo("build")
        .build_std(&setup)
        .target_host()
        .with_status(101)
        // can't find crate
        .with_stderr_contains("error[E0463]: [..]")
        .run();
}

#[cargo_test]
fn explicit_test() {
    // Explicit dependency on libtest.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                std = {stdlib = true}

                [dev-dependencies]
                test = {stdlib = true}
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![feature(test)]
                #[bench]
                fn b1(b: &mut test::Bencher) {
                    b.iter(|| ())
                }
            "#,
        )
        .build();

    p.cargo("test -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn test_no_harness() {
    // Do not build libtest if not using a harness.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]

                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                std.stdlib = true

                [lib]
                harness = false
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #[cfg(test)]
                fn main() {}
            "#,
        )
        .build();

    p.cargo("test -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_does_not_contain("[COMPILING] test[..]")
        .run();
}

#[cargo_test]
fn implicit_default_build() {
    // What gets built and what is in scope if nothing is explicitly listed.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo", "0.1.0"))
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    let _ = std::custom_api();
                    let _ = core::custom_api();
                }
            "#,
        )
        .build();

    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[COMPILING] core[..]")
        .with_stderr_contains("[COMPILING] alloc[..]")
        .with_stderr_contains("[COMPILING] std[..]")
        .with_stderr_contains("[COMPILING] test[..]")
        .with_stderr_contains("[COMPILING] proc_macro[..]")
        .run();

    p.change_file(
        "src/lib.rs",
        "
            pub fn f() {
                let _ = alloc::custom_api();
                let _ = test::custom_api();
                let _ = proc_macro::custom_api();
            }
        ",
    );
    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module
        .with_stderr_contains("error[E0433][..]`alloc`[..]")
        .with_stderr_contains("error[E0433][..]`test`[..]")
        .with_stderr_contains("error[E0433][..]`proc_macro`[..]")
        .run();

    // Add `extern crate`, make sure it works.
    p.change_file(
        "src/lib.rs",
        "
            #![feature(test)]
            extern crate alloc;
            extern crate test;
            extern crate proc_macro;

            pub fn f() {
                let _ = alloc::custom_api();
                let _ = test::custom_api();
                let _ = proc_macro::custom_api();
            }
        ",
    );

    p.cargo("build -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn skips_target_std_if_not_enabled() {
    // Don't build an explicit dep if it is an unactivated [target].
    let manifest = |target| {
        format!(
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                core = {{ stdlib = true }}

                [target.{}.dependencies]
                alloc = {{ stdlib = true }}
            "#,
            target
        )
    };

    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file("Cargo.toml", &manifest("invalid-target"))
        .file(
            "src/lib.rs",
            r#"
                #![no_std]
                pub fn foo() {
                    let _ = alloc::custom_api();
                }
            "#,
        )
        .build();
    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .with_stderr_does_not_contain("[..]liballoc[..]")
        .run();
    p.change_file("Cargo.toml", &manifest(&rustc_host()));
    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[..]liballoc[..]")
        .run();
}

#[cargo_test]
fn optional_std() {
    // optional=true explicit dep
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                core = { stdlib=true }
                alloc = { stdlib=true, optional=true }
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![no_std]
                pub fn foo() {
                    let _ = alloc::custom_api();
                }
            "#,
        )
        .build();

    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .with_stderr_does_not_contain("[..]liballoc[..]")
        .run();

    p.cargo("build -v --features=alloc")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[..]liballoc[..]")
        .run();
}

#[cargo_test]
fn build_dep_doesnt_build() {
    // Explicit build dependencies do not influence what gets built.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"
                edition = "2018"

                [dependencies]
                core.stdlib = true

                [build-dependencies]
                alloc.stdlib = true
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![no_std]
                fn f() {
                    let _ = alloc::custom_api();
                }
            "#,
        )
        .file(
            "build.rs",
            r#"
                fn main() {
                    let _ = alloc::string::String::new();
                }
            "#,
        )
        .build();

    p.cargo("build -v")
        .build_std(&setup)
        .target_host()
        .with_status(101)
        // error[E0433]: failed to resolve: use of undeclared type or module `alloc`
        .with_stderr_contains("error[E0433][..]")
        .run();

    p.change_file("src/lib.rs", "#![no_std]");

    p.cargo("build -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn rename() {
    // a `package` alias
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["explicit-std"]
                [package]
                name = "foo"
                version = "0.1.0"

                [dependencies]
                corealias = {stdlib=true, package="core"}
                stdalias = {stdlib=true, package="std"}
                allocalias = {stdlib=true, package="alloc"}

                [dev-dependencies]
                testalias = {stdlib=true, package="test"}
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                #![cfg_attr(test, feature(test))]

                pub fn f() {
                    let _ = corealias::custom_api();
                    let _ = stdalias::custom_api();
                    let _ = allocalias::custom_api();
                }

                #[cfg(test)]
                #[bench]
                fn b1(b: &mut testalias::Bencher) {
                    b.iter(|| ())
                }
            "#,
        )
        .build();

    p.cargo("build -v").build_std(&setup).target_host().run();
    p.cargo("test -v").build_std(&setup).target_host().run();
}

#[cargo_test]
fn panic_strategy_abort() {
    // "abort" strategy shouldn't use "unwind".
    // NOTE: panic_unwind is still built because it is a dependency of libstd.
    // Cargo could in theory disable the `panic-unwind` feature, but
    // unfortunately it is not known whether or not the unwind strategy is
    // needed when the standard lib is resolved. If it could be done later,
    // then Cargo could completely skip the unwind crate, but there is a
    // mutability problem with PackageSet that prevents it. Alternatively,
    // if feature resolution is removed from the resolver, then it can control
    // the panic-unwind feature more easily.
    let setup = match setup() {
        Some(s) => s,
        None => return,
    };
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"

                [profile.dev]
                panic = "abort"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn f() {
                    panic!("smurf");
                }
            "#,
        )
        .file("src/main.rs", "fn main() { foo::f(); }")
        .build();
    p.cargo("run -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains("[..]panicked at 'smurf'[..]")
        .with_stderr_contains("[RUNNING] [..]--crate-name foo[..]src/lib.rs[..]-C panic=abort[..]")
        .with_stderr_contains("[RUNNING] [..]--crate-name foo[..]src/main.rs[..]-C panic=abort[..]")
        // Exits with signal.
        .without_status()
        .run();

    // Implied -Zpanic-abort-tests
    p.cargo("test -v")
        .build_std(&setup)
        .target_host()
        .with_stderr_contains(
            "[RUNNING] [..]--crate-name foo[..]src/lib.rs[..]-C panic=abort[..]--test[..]",
        )
        .with_stderr_contains(
            "[RUNNING] [..]--crate-name foo[..]src/main.rs[..]-C panic=abort[..]--test[..]",
        )
        .with_stderr_contains("[RUNNING] `rustdoc[..]--test[..]")
        .run();
}
