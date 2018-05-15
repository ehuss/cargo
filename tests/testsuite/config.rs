use cargo::core::Shell;
use cargo::util::config::Config;
use cargo::util::toml;
use cargotest::support::{execs, paths, project};
use hamcrest::assert_that;
use std;
use std::collections;
use std::fs;

#[test]
fn read_env_vars_for_config() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            authors = []
            version = "0.0.0"
            build = "build.rs"
        "#,
        )
        .file("src/lib.rs", "")
        .file(
            "build.rs",
            r#"
            use std::env;
            fn main() {
                assert_eq!(env::var("NUM_JOBS").unwrap(), "100");
            }
        "#,
        )
        .build();

    assert_that(
        p.cargo("build").env("CARGO_BUILD_JOBS", "100"),
        execs().with_status(0),
    );
}

fn new_config() -> Config {
    let shell = Shell::new();
    let cwd = paths::root();
    let homedir = paths::home();
    Config::new(shell, cwd, homedir)
}

#[test]
fn load_type() {
    let path = paths::root().join(".cargo/config");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        "\
[S]
f1 = 123
",
    ).unwrap();

    let config = new_config();

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct S {
        f1: Option<i64>,
    }
    let s: S = config.load_type("S").unwrap();
    assert_eq!(s, S { f1: Some(123) });
    std::env::set_var("CARGO_S_F1", "456");
    let s: S = config.load_type("S").unwrap();
    assert_eq!(s, S { f1: Some(456) });
}

#[test]
fn load_toml_profile() {
    let path = paths::root().join(".cargo/config");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        "\
[profile.dev]
opt-level = 3
lto = true
codegen-units=4
debug = true
debug-assertions = true
rpath = true
panic = 'abort'
overflow-checks = true
incremental = true

[profile.dev.build-override]
opt-level = 1

[profile.dev.overrides.bar]
codegen-units = 9
",
    ).unwrap();

    let config = new_config();

    // TODO: Unset
    std::env::set_var("CARGO_PROFILE_DEV_CODEGEN_UNITS", "5");
    std::env::set_var("CARGO_PROFILE_DEV_BUILD_OVERRIDE_CODEGEN_UNITS", "11");
    std::env::set_var("CARGO_PROFILE_DEV_OVERRIDES_env_CODEGEN_UNITS", "13");

    // TODO: don't use actual tomlprofile
    let p: toml::TomlProfile = config.load_type("profile.dev").unwrap();
    let mut overrides = collections::BTreeMap::new();
    let key = toml::ProfilePackageSpec::Spec(::cargo::core::PackageIdSpec::parse("bar").unwrap());
    let o_profile = toml::TomlProfile {
        opt_level: None,
        lto: None,
        codegen_units: Some(9),
        debug: None,
        debug_assertions: None,
        rpath: None,
        panic: None,
        overflow_checks: None,
        incremental: None,
        overrides: None,
        build_override: None,
    };
    overrides.insert(key, o_profile);
    let key = toml::ProfilePackageSpec::Spec(::cargo::core::PackageIdSpec::parse("env").unwrap());
    let o_profile = toml::TomlProfile {
        opt_level: None,
        lto: None,
        codegen_units: Some(13),
        debug: None,
        debug_assertions: None,
        rpath: None,
        panic: None,
        overflow_checks: None,
        incremental: None,
        overrides: None,
        build_override: None,
    };
    overrides.insert(key, o_profile);

    assert_eq!(
        p,
        toml::TomlProfile {
            opt_level: Some(toml::TomlOptLevel("3".to_string())),
            lto: Some(toml::StringOrBool::Bool(true)),
            codegen_units: Some(5),
            debug: Some(toml::U32OrBool::Bool(true)),
            debug_assertions: Some(true),
            rpath: Some(true),
            panic: Some("abort".to_string()),
            overflow_checks: Some(true),
            incremental: Some(true),
            overrides: Some(overrides),
            build_override: Some(Box::new(toml::TomlProfile {
                opt_level: Some(toml::TomlOptLevel("1".to_string())),
                lto: None,
                codegen_units: Some(11),
                debug: None,
                debug_assertions: None,
                rpath: None,
                panic: None,
                overflow_checks: None,
                incremental: None,
                overrides: None,
                build_override: None
            }))
        }
    );
}

#[test]
fn load_nested() {
    let path = paths::root().join(".cargo/config");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        "\
[nest.foo]
f1 = 1
f2 = 2
[nest.bar]
asdf = 3
",
    ).unwrap();

    let config = new_config();

    type Nested = collections::HashMap<String, collections::HashMap<String, u8>>;

    let n: Nested = config.load_type("nest").unwrap();
    let mut expected = collections::HashMap::new();
    let mut foo = collections::HashMap::new();
    foo.insert("f1".to_string(), 1);
    foo.insert("f2".to_string(), 2);
    expected.insert("foo".to_string(), foo);
    let mut bar = collections::HashMap::new();
    bar.insert("asdf".to_string(), 3);
    expected.insert("bar".to_string(), bar);
    assert_eq!(n, expected);

    // TODO: unset
    std::env::set_var("CARGO_NESTE_foo_f1", "1");
    std::env::set_var("CARGO_NESTE_foo_f2", "2");
    std::env::set_var("CARGO_NESTE_bar_asdf", "3");

    let n: Nested = config.load_type("neste").unwrap();
    assert_eq!(n, expected);
}
