use cargo::core::{LastUse, LastUseKind};
use cargo::Config;
use std::fs;
use std::path::PathBuf;

fn cargo_home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn main() {
    let shell = cargo::core::Shell::new();
    let homedir = cargo_home();
    if !homedir.exists() {
        fs::create_dir_all(&homedir).unwrap();
    }
    let f = homedir.join(".last-use"); // TODO
    if f.exists() {
        fs::remove_file(f).unwrap();
    }
    let cwd = homedir.clone();
    let config = Config::new(shell, cwd, homedir);
    let _lock = config.acquire_package_cache_lock().unwrap();
    let mut last_use = LastUse::load(&config).unwrap();

    // ~/.cargo/registry/cache/github.com-1ecc6299db9ec823
    let real_home = cargo::util::homedir(&std::env::current_dir().unwrap()).unwrap();

    let index_dir = real_home.join("registry/index");
    for dir_ent in fs::read_dir(index_dir).unwrap() {
        let registry = dir_ent.unwrap();
        last_use.mark_used(
            LastUseKind::RegistryIndex,
            registry.file_name().to_string_lossy().into_owned(),
        );
    }

    let cache_dir = real_home.join("registry/cache");
    for dir_ent in fs::read_dir(cache_dir).unwrap() {
        let registry = dir_ent.unwrap();
        let rc = LastUseKind::RegistryCrate(registry.file_name().to_string_lossy().into_owned());
        for krate in fs::read_dir(registry.path()).unwrap() {
            let krate = krate.unwrap();
            let meta = krate.metadata().unwrap();
            last_use.mark_used_stamp(
                rc.clone(),
                krate.file_name().to_string_lossy().into_owned(),
                &meta.modified().unwrap(),
            );
        }
    }

    let cache_dir = real_home.join("registry/src");
    for dir_ent in fs::read_dir(cache_dir).unwrap() {
        let registry = dir_ent.unwrap();
        let rc = LastUseKind::RegistrySrc(registry.file_name().to_string_lossy().into_owned());
        for krate in fs::read_dir(registry.path()).unwrap() {
            let krate = krate.unwrap();
            let meta = krate.metadata().unwrap();
            last_use.mark_used_stamp(
                rc.clone(),
                krate.file_name().to_string_lossy().into_owned(),
                &meta.modified().unwrap(),
            );
        }
    }

    let git_db_dir = real_home.join("git/db");
    for dir_ent in fs::read_dir(git_db_dir).unwrap() {
        let git_source = dir_ent.unwrap();
        last_use.mark_used(
            LastUseKind::GitDb,
            git_source.file_name().to_string_lossy().into_owned(),
        );
    }

    let git_co_dir = real_home.join("git/checkouts");
    for dir_ent in fs::read_dir(git_co_dir).unwrap() {
        let git_source = dir_ent.unwrap();
        let rc = LastUseKind::GitCheckout(git_source.file_name().to_string_lossy().into_owned());
        for co in fs::read_dir(git_source.path()).unwrap() {
            let co = co.unwrap();
            let meta = co.metadata().unwrap();
            last_use.mark_used_stamp(
                rc.clone(),
                co.file_name().to_string_lossy().into_owned(),
                &meta.modified().unwrap(),
            );
        }
    }

    last_use.save(&config).unwrap();
}
