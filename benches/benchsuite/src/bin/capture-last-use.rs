use cargo::core::last_use::{self, GlobalLastUse};
use cargo::Config;
use std::fs;
use std::path::Path;

fn main() {
    let shell = cargo::core::Shell::new();
    let homedir = Path::new(env!("CARGO_MANIFEST_DIR")).join("last-use");
    let last_use_db = homedir.join(".last-use"); // TODO
    if last_use_db.exists() {
        fs::remove_file(&last_use_db).unwrap();
    }
    let cwd = homedir.clone();
    let mut config = Config::new(shell, cwd, homedir.clone());
    config
        .configure(
            0,
            false,
            None,
            false,
            false,
            false,
            &None,
            &["gc".to_string()],
            &[],
        )
        .unwrap();
    let _lock = config.acquire_package_cache_lock().unwrap();
    let mut last_use = GlobalLastUse::new(&config).unwrap();

    // ~/.cargo/registry/cache/github.com-1ecc6299db9ec823
    let real_home = cargo::util::homedir(&std::env::current_dir().unwrap()).unwrap();

    // let index_dir = real_home.join("registry/index");
    // for dir_ent in fs::read_dir(index_dir).unwrap() {
    //     let registry = dir_ent.unwrap();
    //     last_use.mark_used(LastUseKind::RegistryIndex {
    //         encoded_registry_name: registry.file_name().to_string_lossy().into_owned(),
    //     });
    // }

    let cache_dir = real_home.join("registry/cache");
    for dir_ent in fs::read_dir(cache_dir).unwrap() {
        let registry = dir_ent.unwrap();
        let encoded_registry_name = registry.file_name().to_string_lossy().into_owned();
        for krate in fs::read_dir(registry.path()).unwrap() {
            let krate = krate.unwrap();
            let meta = krate.metadata().unwrap();
            last_use.mark_registry_crate_used_stamp(
                last_use::RegistryCrate {
                    encoded_registry_name: encoded_registry_name.clone(),
                    crate_filename: krate.file_name().to_string_lossy().into_owned(),
                    size: meta.len(),
                },
                Some(&meta.modified().unwrap()),
            );
        }
    }

    let cache_dir = real_home.join("registry/src");
    for dir_ent in fs::read_dir(cache_dir).unwrap() {
        let registry = dir_ent.unwrap();
        let encoded_registry_name = registry.file_name().to_string_lossy().into_owned();
        for krate in fs::read_dir(registry.path()).unwrap() {
            let krate = krate.unwrap();
            let meta = krate.metadata().unwrap();
            last_use.mark_registry_src_used_stamp(
                last_use::RegistrySrc {
                    encoded_registry_name: encoded_registry_name.clone(),
                    package_dir: krate.file_name().to_string_lossy().into_owned(),
                    size: Some(cargo_util::paths::du(&krate.path()).unwrap()),
                },
                Some(&meta.modified().unwrap()),
            );
        }
    }

    // let git_db_dir = real_home.join("git/db");
    // for dir_ent in fs::read_dir(git_db_dir).unwrap() {
    //     let git_source = dir_ent.unwrap();
    //     last_use.mark_used(LastUseKind::GitDb {
    //         encoded_git_name: git_source.file_name().to_string_lossy().into_owned(),
    //     });
    // }

    let git_co_dir = real_home.join("git/checkouts");
    for dir_ent in fs::read_dir(git_co_dir).unwrap() {
        let git_source = dir_ent.unwrap();
        let encoded_git_name = git_source.file_name().to_string_lossy().into_owned();
        for co in fs::read_dir(git_source.path()).unwrap() {
            let co = co.unwrap();
            let meta = co.metadata().unwrap();
            last_use.mark_git_checkout_used_stamp(
                last_use::GitCheckout {
                    encoded_git_name: encoded_git_name.clone(),
                    short_name: co.file_name().to_string_lossy().into_owned(),
                },
                Some(&meta.modified().unwrap()),
            );
        }
    }

    last_use.save().unwrap();
    fs::rename(&last_use_db, homedir.join("last-use-sample")).unwrap();
    fs::remove_file(homedir.join(".package-cache")).unwrap();
}
