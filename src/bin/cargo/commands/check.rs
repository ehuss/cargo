use crate::command_prelude::*;

use cargo::ops;

pub fn cli() -> App {
    subcommand("check")
        // subcommand aliases are handled in aliased_command()
        // .alias("c")
        .about("Check a local package and all of its dependencies for errors")
        .arg(opt("quiet", "No output printed to stdout").short("q"))
        .arg_package_spec(
            "Package(s) to check",
            "Check all packages in the workspace",
            "Exclude packages from the check",
        )
        .arg_jobs()
        .arg_targets_all(
            "Check only this package's library",
            "Check only the specified binary",
            "Check all binaries",
            "Check only the specified example",
            "Check all examples",
            "Check only the specified test target",
            "Check all tests",
            "Check only the specified bench target",
            "Check all benches",
            "Check all targets",
        )
        .arg_release("Check artifacts in release mode, with optimizations")
        .arg_profile("Check artifacts with the specified profile")
        .arg_features()
        .arg_target_triple("Check for the target triple")
        .arg_target_dir()
        .arg_manifest_path()
        .arg_ignore_rust_version()
        .arg_message_format()
        .arg_unit_graph()
        .arg_future_incompat_report()
        .after_help("Run `cargo help check` for more detailed information.\n")
}

pub fn exec(config: &mut Config, args: &ArgMatches<'_>) -> CliResult {
    if std::env::var("CARGO_REAL_CHECK").is_err() {
        fixit()?;
        return Ok(());
    }
    let ws = args.workspace(config)?;
    let test = match args.value_of("profile") {
        Some("test") => true,
        None => false,
        Some(profile) => {
            let err = anyhow::format_err!(
                "unknown profile: `{}`, only `test` is \
                 currently supported",
                profile
            );
            return Err(CliError::new(err, 101));
        }
    };
    let mode = CompileMode::Check { test };
    let compile_opts = args.compile_options(config, mode, Some(&ws), ProfileChecking::Unchecked)?;

    ops::compile(&ws, &compile_opts)?;
    Ok(())
}

fn fixit() -> CliResult {
    use anyhow::Context;
    use cargo_util::{paths, ProcessBuilder};
    use std::path::Path;

    eprintln!("Copying to /tmp/fixit");
    ProcessBuilder::new("cp")
        .args(&["-a", ".", "/tmp/fixit"])
        .exec()?;
    std::env::set_current_dir("/tmp/fixit").map_err(|e| anyhow::format_err!("cd failed {}", e))?;

    let mut manifest = paths::read(Path::new("Cargo.toml"))?;
    let ed_re = regex::Regex::new(r#"(?m)^ *edition *= *['"]([^'"]+)['"]"#).unwrap();
    let man_clone = manifest.clone();
    let ed_cap = match ed_re.captures(&man_clone) {
        None => {
            eprintln!("no edition found in manifest, probably 2015, skipping");
            return Ok(());
        }
        Some(caps) => caps.get(1).unwrap(),
    };
    if ed_cap.as_str() != "2018" {
        eprintln!("skipping non-2018 edition `{}`", ed_cap.as_str());
        return Ok(());
    }
    eprintln!("Running `cargo fix --edition`");
    // Skip "cargo check"
    let args: Vec<_> = std::env::args().skip(2).collect();
    ProcessBuilder::new("cargo")
        .args(&["fix", "--edition", "--allow-no-vcs", "--allow-dirty"])
        .args(&args)
        .exec()
        .with_context(|| "failed to migrate to next edition")?;
    manifest.replace_range(ed_cap.range(), "2021");
    paths::write("Cargo.toml", manifest)?;
    eprintln!("Running `cargo check` to verify 2021");
    ProcessBuilder::new("cargo")
        .args(&["check"])
        .args(&args)
        .env("CARGO_REAL_CHECK", "1")
        .exec()
        .with_context(|| "failed to check after updating to 2021")?;
    Ok(())
}
