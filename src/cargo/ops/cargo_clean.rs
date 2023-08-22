use crate::core::compiler::{CompileKind, CompileMode, Layout, RustcTargetData};
use crate::core::gc::{AutoGcKind, Gc, GcOpts};
use crate::core::last_use::GlobalLastUse;
use crate::core::profiles::Profiles;
use crate::core::{PackageIdSpec, TargetKind, Workspace};
use crate::ops;
use crate::util::cache_lock::CacheLockMode;
use crate::util::edit_distance;
use crate::util::errors::CargoResult;
use crate::util::interning::InternedString;
use crate::util::{human_readable_bytes, Config, Progress, ProgressStyle};

use cargo_util::paths;
use std::fs;
use std::path::Path;

pub struct CleanOptions<'cfg> {
    pub config: &'cfg Config,
    /// A list of packages to clean. If empty, everything is cleaned.
    pub spec: Vec<String>,
    /// The target arch triple to clean, or None for the host arch
    pub targets: Vec<String>,
    /// Whether to clean the release directory
    pub profile_specified: bool,
    /// Whether to clean the directory of a certain build profile
    pub requested_profile: InternedString,
    /// Whether to just clean the doc directory
    pub doc: bool,
    pub dry_run: bool,
    pub gc_opts: GcOpts,
}

pub struct CleanContext<'cfg> {
    pub config: &'cfg Config,
    progress: Box<dyn CleaningProgressBar + 'cfg>,
    pub dry_run: bool,
    num_files_folders_cleaned: u64,
    total_bytes_removed: u64,
}

/// Cleans various caches.
pub fn clean(ws: CargoResult<Workspace<'_>>, opts: &CleanOptions<'_>) -> CargoResult<()> {
    let config = opts.config;
    let mut ctx = CleanContext::new(config);
    ctx.dry_run = opts.dry_run;

    let any_download_cache_opts = opts.gc_opts.is_download_cache_opt_set();

    // The following options need a workspace.
    let any_ws_opts = !opts.spec.is_empty()
        || !opts.targets.is_empty()
        || opts.profile_specified
        || opts.doc
        || opts.gc_opts.is_target_opt_set();

    // When no options are specified, do the default action.
    let no_opts_specified = !any_download_cache_opts && !any_ws_opts;

    if any_ws_opts || no_opts_specified {
        let ws = ws?;
        let mut target_dir = ws.target_dir();

        if opts.doc {
            // If the doc option is set, we just want to delete the doc directory.
            //
            // FIXME: This ignores other flags, which it probably shouldn't.
            // See https://github.com/rust-lang/cargo/issues/8790
            target_dir = target_dir.join("doc");
            return ctx.clean_entire_folder(&target_dir.into_path_unlocked());
        }

        let profiles = Profiles::new(&ws, opts.requested_profile)?;

        if opts.profile_specified {
            // After parsing profiles we know the dir-name of the profile, if a profile
            // was passed from the command line. If so, delete only the directory of
            // that profile.
            let dir_name = profiles.get_dir_name();
            target_dir = target_dir.join(dir_name);
        }

        // If we have a spec, then we need to delete some packages, otherwise, just
        // remove the whole target directory and be done with it!
        //
        // Note that we don't bother grabbing a lock here as we're just going to
        // blow it all away anyway.
        if opts.spec.is_empty() {
            ctx.clean_entire_folder(&target_dir.into_path_unlocked())?;
        } else {
            clean_specs(&mut ctx, &ws, &profiles, &opts.targets, &opts.spec)?;
        }
    }

    if config.cli_unstable().gc {
        // TODO: Think about trying to consolidate these 4 lines somehow.
        let _lock = config.acquire_package_cache_lock(CacheLockMode::MutateExclusive)?;
        let mut last_use = GlobalLastUse::new(&config)?;
        let mut gc = Gc::new(config, &mut last_use)?;
        if no_opts_specified {
            // This is the behavior for `cargo clean` without *any* options.
            // It uses the defaults from config to determine what is cleaned.
            let mut gc_opts = opts.gc_opts.clone();
            gc_opts.update_for_auto_gc(config, &[AutoGcKind::All], None)?;
            gc.gc(&mut ctx, &gc_opts)?;
        } else {
            gc.gc(&mut ctx, &opts.gc_opts)?;
        }
    }

    ctx.display_summary()?;
    Ok(())
}

fn clean_specs(
    ctx: &mut CleanContext<'_>,
    ws: &Workspace<'_>,
    profiles: &Profiles,
    targets: &[String],
    spec: &[String],
) -> CargoResult<()> {
    // Clean specific packages.
    let requested_kinds = CompileKind::from_requested_targets(ctx.config, targets)?;
    let target_data = RustcTargetData::new(ws, &requested_kinds)?;
    let (pkg_set, resolve) = ops::resolve_ws(ws)?;
    let prof_dir_name = profiles.get_dir_name();
    let host_layout = Layout::new(ws, None, &prof_dir_name)?;
    // Convert requested kinds to a Vec of layouts.
    let target_layouts: Vec<(CompileKind, Layout)> = requested_kinds
        .into_iter()
        .filter_map(|kind| match kind {
            CompileKind::Target(target) => match Layout::new(ws, Some(target), &prof_dir_name) {
                Ok(layout) => Some(Ok((kind, layout))),
                Err(e) => Some(Err(e)),
            },
            CompileKind::Host => None,
        })
        .collect::<CargoResult<_>>()?;
    // A Vec of layouts. This is a little convoluted because there can only be
    // one host_layout.
    let layouts = if targets.is_empty() {
        vec![(CompileKind::Host, &host_layout)]
    } else {
        target_layouts
            .iter()
            .map(|(kind, layout)| (*kind, layout))
            .collect()
    };
    // Create a Vec that also includes the host for things that need to clean both.
    let layouts_with_host: Vec<(CompileKind, &Layout)> =
        std::iter::once((CompileKind::Host, &host_layout))
            .chain(layouts.iter().map(|(k, l)| (*k, *l)))
            .collect();

    // Cleaning individual rustdoc crates is currently not supported.
    // For example, the search index would need to be rebuilt to fully
    // remove it (otherwise you're left with lots of broken links).
    // Doc tests produce no output.

    // Get Packages for the specified specs.
    let mut pkg_ids = Vec::new();
    for spec_str in spec.iter() {
        // Translate the spec to a Package.
        let spec = PackageIdSpec::parse(spec_str)?;
        if spec.version().is_some() {
            ctx.config.shell().warn(&format!(
                "version qualifier in `-p {}` is ignored, \
                cleaning all versions of `{}` found",
                spec_str,
                spec.name()
            ))?;
        }
        if spec.url().is_some() {
            ctx.config.shell().warn(&format!(
                "url qualifier in `-p {}` ignored, \
                cleaning all versions of `{}` found",
                spec_str,
                spec.name()
            ))?;
        }
        let matches: Vec<_> = resolve.iter().filter(|id| spec.matches(*id)).collect();
        if matches.is_empty() {
            let mut suggestion = String::new();
            suggestion.push_str(&edit_distance::closest_msg(
                &spec.name(),
                resolve.iter(),
                |id| id.name().as_str(),
            ));
            anyhow::bail!(
                "package ID specification `{}` did not match any packages{}",
                spec,
                suggestion
            );
        }
        pkg_ids.extend(matches);
    }
    let packages = pkg_set.get_many(pkg_ids)?;

    ctx.progress = Box::new(CleaningPackagesBar::new(ctx.config, packages.len()));

    for pkg in packages {
        let pkg_dir = format!("{}-*", pkg.name());
        ctx.progress.on_cleaning_package(&pkg.name())?;

        // Clean fingerprints.
        for (_, layout) in &layouts_with_host {
            let dir = escape_glob_path(layout.fingerprint())?;
            ctx.rm_rf_package_glob_containing_hash(&pkg.name(), &Path::new(&dir).join(&pkg_dir))?;
        }

        for target in pkg.targets() {
            if target.is_custom_build() {
                // Get both the build_script_build and the output directory.
                for (_, layout) in &layouts_with_host {
                    let dir = escape_glob_path(layout.build())?;
                    ctx.rm_rf_package_glob_containing_hash(
                        &pkg.name(),
                        &Path::new(&dir).join(&pkg_dir),
                    )?;
                }
                continue;
            }
            let crate_name = target.crate_name();
            for &mode in &[
                CompileMode::Build,
                CompileMode::Test,
                CompileMode::Check { test: false },
            ] {
                for (compile_kind, layout) in &layouts {
                    let triple = target_data.short_name(compile_kind);

                    let (file_types, _unsupported) = target_data
                        .info(*compile_kind)
                        .rustc_outputs(mode, target.kind(), triple)?;
                    let (dir, uplift_dir) = match target.kind() {
                        TargetKind::ExampleBin | TargetKind::ExampleLib(..) => {
                            (layout.examples(), Some(layout.examples()))
                        }
                        // Tests/benchmarks are never uplifted.
                        TargetKind::Test | TargetKind::Bench => (layout.deps(), None),
                        _ => (layout.deps(), Some(layout.dest())),
                    };
                    for file_type in file_types {
                        // Some files include a hash in the filename, some don't.
                        let hashed_name = file_type.output_filename(target, Some("*"));
                        let unhashed_name = file_type.output_filename(target, None);
                        let dir_glob = escape_glob_path(dir)?;
                        let dir_glob = Path::new(&dir_glob);

                        ctx.rm_rf_glob(&dir_glob.join(&hashed_name))?;
                        ctx.rm_rf(&dir.join(&unhashed_name))?;
                        // Remove dep-info file generated by rustc. It is not tracked in
                        // file_types. It does not have a prefix.
                        let hashed_dep_info = dir_glob.join(format!("{}-*.d", crate_name));
                        ctx.rm_rf_glob(&hashed_dep_info)?;
                        let unhashed_dep_info = dir.join(format!("{}.d", crate_name));
                        ctx.rm_rf(&unhashed_dep_info)?;
                        // Remove split-debuginfo files generated by rustc.
                        let split_debuginfo_obj = dir_glob.join(format!("{}.*.o", crate_name));
                        ctx.rm_rf_glob(&split_debuginfo_obj)?;
                        let split_debuginfo_dwo = dir_glob.join(format!("{}.*.dwo", crate_name));
                        ctx.rm_rf_glob(&split_debuginfo_dwo)?;
                        let split_debuginfo_dwp = dir_glob.join(format!("{}.*.dwp", crate_name));
                        ctx.rm_rf_glob(&split_debuginfo_dwp)?;

                        // Remove the uplifted copy.
                        if let Some(uplift_dir) = uplift_dir {
                            let uplifted_path = uplift_dir.join(file_type.uplift_filename(target));
                            ctx.rm_rf(&uplifted_path)?;
                            // Dep-info generated by Cargo itself.
                            let dep_info = uplifted_path.with_extension("d");
                            ctx.rm_rf(&dep_info)?;
                        }
                    }
                    // TODO: what to do about build_script_build?
                    let dir = escape_glob_path(layout.incremental())?;
                    let incremental = Path::new(&dir).join(format!("{}-*", crate_name));
                    ctx.rm_rf_glob(&incremental)?;
                }
            }
        }
    }

    Ok(())
}

fn escape_glob_path(pattern: &Path) -> CargoResult<String> {
    let pattern = pattern
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("expected utf-8 path"))?;
    Ok(glob::Pattern::escape(pattern))
}

impl<'cfg> CleanContext<'cfg> {
    pub fn new(config: &'cfg Config) -> CleanContext<'cfg> {
        // This progress bar will get replaced, this is just here to avoid needing
        // an Option until the actual bar is created.
        let progress = CleaningFolderBar::new(config, 0);
        CleanContext {
            config,
            progress: Box::new(progress),
            dry_run: false,
            num_files_folders_cleaned: 0,
            total_bytes_removed: 0,
        }
    }

    pub fn set_progress(&mut self, progress: Box<dyn CleaningProgressBar + 'cfg>) {
        self.progress = progress;
    }

    /// Glob remove artifacts for the provided `package`
    ///
    /// Make sure the artifact is for `package` and not another crate that is prefixed by
    /// `package` by getting the original name stripped of the trailing hash and possible
    /// extension
    fn rm_rf_package_glob_containing_hash(
        &mut self,
        package: &str,
        pattern: &Path,
    ) -> CargoResult<()> {
        // TODO: Display utf8 warning to user?  Or switch to globset?
        let pattern = pattern
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("expected utf-8 path"))?;
        for path in glob::glob(pattern)? {
            let path = path?;

            let pkg_name = path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .and_then(|artifact| artifact.rsplit_once('-'))
                .ok_or_else(|| anyhow::anyhow!("expected utf-8 path"))?
                .0;

            if pkg_name != package {
                continue;
            }

            self.rm_rf(&path)?;
        }
        Ok(())
    }

    fn rm_rf_glob(&mut self, pattern: &Path) -> CargoResult<()> {
        // TODO: Display utf8 warning to user?  Or switch to globset?
        let pattern = pattern
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("expected utf-8 path"))?;
        for path in glob::glob(pattern)? {
            self.rm_rf(&path?)?;
        }
        Ok(())
    }

    pub fn rm_rf(&mut self, path: &Path) -> CargoResult<()> {
        let Ok(meta) = fs::symlink_metadata(path) else {
            return Ok(());
        };

        if self.dry_run {
            // Concise because if in verbose mode, the path will be written in
            // the loop below.
            self.config
                .shell()
                .concise(|shell| Ok(writeln!(shell.out(), "{}", path.display())?))?;
        } else {
            self.config
                .shell()
                .verbose(|shell| shell.status("Removing", path.display()))?;
        }
        self.progress.display_now()?;

        let mut rm_file = |path: &Path, meta: Result<std::fs::Metadata, _>| {
            if let Ok(meta) = meta {
                self.total_bytes_removed += meta.len();
            }
            if !self.dry_run {
                paths::remove_file(path)?;
            }
            Ok(())
        };

        if !meta.is_dir() {
            self.num_files_folders_cleaned += 1;
            return rm_file(path, Ok(meta));
        }

        for entry in walkdir::WalkDir::new(path).contents_first(true) {
            let entry = entry?;
            self.progress.on_clean()?;
            self.num_files_folders_cleaned += 1;
            if self.dry_run {
                self.config
                    .shell()
                    .verbose(|shell| Ok(writeln!(shell.out(), "{}", entry.path().display())?))?;
            }
            if entry.file_type().is_dir() {
                // The contents should have been removed by now, but sometimes a race condition is hit
                // where other files have been added by the OS. `paths::remove_dir_all` also falls back
                // to `std::fs::remove_dir_all`, which may be more reliable than a simple walk in
                // platform-specific edge cases.
                if !self.dry_run {
                    paths::remove_dir_all(entry.path())?;
                }
            } else {
                rm_file(entry.path(), entry.metadata())?;
            }
        }

        Ok(())
    }

    fn display_summary(&self) -> CargoResult<()> {
        let status = if self.dry_run { "Summary" } else { "Removed" };
        let byte_count = if self.total_bytes_removed == 0 {
            String::new()
        } else {
            let (bytes, unit) = human_readable_bytes(self.total_bytes_removed);
            if bytes < 1024.0 {
                format!(", {bytes}{unit} total")
            } else {
                format!(", {bytes:.1}{unit} total")
            }
        };
        self.config.shell().status(
            status,
            format!(
                "{} files/directories{byte_count}",
                self.num_files_folders_cleaned
            ),
        )
    }

    fn clean_entire_folder(&mut self, path: &Path) -> CargoResult<()> {
        let num_paths = walkdir::WalkDir::new(path).into_iter().count();
        self.progress = Box::new(CleaningFolderBar::new(self.config, num_paths));
        self.rm_rf(path)?;
        Ok(())
    }
}

pub trait CleaningProgressBar {
    fn display_now(&mut self) -> CargoResult<()>;
    fn on_clean(&mut self) -> CargoResult<()>;
    fn on_cleaning_package(&mut self, _package: &str) -> CargoResult<()> {
        Ok(())
    }
}

pub struct CleaningFolderBar<'cfg> {
    bar: Progress<'cfg>,
    max: usize,
    cur: usize,
}

impl<'cfg> CleaningFolderBar<'cfg> {
    pub fn new(cfg: &'cfg Config, max: usize) -> Self {
        Self {
            bar: Progress::with_style("Cleaning", ProgressStyle::Percentage, cfg),
            max,
            cur: 0,
        }
    }

    fn cur_progress(&self) -> usize {
        std::cmp::min(self.cur, self.max)
    }
}

impl<'cfg> CleaningProgressBar for CleaningFolderBar<'cfg> {
    fn display_now(&mut self) -> CargoResult<()> {
        self.bar.tick_now(self.cur_progress(), self.max, "")
    }

    fn on_clean(&mut self) -> CargoResult<()> {
        self.cur += 1;
        self.bar.tick(self.cur_progress(), self.max, "")
    }
}

struct CleaningPackagesBar<'cfg> {
    bar: Progress<'cfg>,
    max: usize,
    cur: usize,
    num_files_folders_cleaned: usize,
    package_being_cleaned: String,
}

impl<'cfg> CleaningPackagesBar<'cfg> {
    fn new(cfg: &'cfg Config, max: usize) -> Self {
        Self {
            bar: Progress::with_style("Cleaning", ProgressStyle::Ratio, cfg),
            max,
            cur: 0,
            num_files_folders_cleaned: 0,
            package_being_cleaned: String::new(),
        }
    }

    fn cur_progress(&self) -> usize {
        std::cmp::min(self.cur, self.max)
    }

    fn format_message(&self) -> String {
        format!(
            ": {}, {} files/folders cleaned",
            self.package_being_cleaned, self.num_files_folders_cleaned
        )
    }
}

impl<'cfg> CleaningProgressBar for CleaningPackagesBar<'cfg> {
    fn display_now(&mut self) -> CargoResult<()> {
        self.bar
            .tick_now(self.cur_progress(), self.max, &self.format_message())
    }

    fn on_clean(&mut self) -> CargoResult<()> {
        self.bar
            .tick(self.cur_progress(), self.max, &self.format_message())?;
        self.num_files_folders_cleaned += 1;
        Ok(())
    }

    fn on_cleaning_package(&mut self, package: &str) -> CargoResult<()> {
        self.cur += 1;
        self.package_being_cleaned = String::from(package);
        self.bar
            .tick(self.cur_progress(), self.max, &self.format_message())
    }
}
