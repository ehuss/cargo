//! TODO

use crate::core::last_use::GlobalLastUse;
use crate::ops::CleanContext;
use crate::{CargoResult, Config};
use anyhow::format_err;
use anyhow::{bail, Context};
use serde::Deserialize;
use std::time::Duration;

pub struct Gc<'a, 'config> {
    config: &'config Config,
    global_last_use: &'a mut GlobalLastUse,
}

// NOTE: Not all of these options may get stabilized. Some of them are
// very low-level details, and may not be something typical users need.
#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct AutoConfig {
    frequency: Option<String>,
    max_src_age: Option<String>,
    max_crate_age: Option<String>,
    max_index_age: Option<String>,
    max_git_co_age: Option<String>,
    max_git_db_age: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct GcOpts {
    pub dry_run: bool,
    pub max_src_age: Option<Duration>,
    pub max_crate_age: Option<Duration>,
    pub max_index_age: Option<Duration>,
    pub max_git_co_age: Option<Duration>,
    pub max_git_db_age: Option<Duration>,
    pub max_src_size: Option<u64>,
    pub max_crate_size: Option<u64>,
    pub max_download_size: Option<u64>,

    pub max_target_age: Option<Duration>,
    pub max_shared_target_age: Option<Duration>,
    pub max_target_size: Option<u64>,
    pub max_shared_target_size: Option<u64>,
}

impl GcOpts {
    pub fn is_cache_opt_set(&self) -> bool {
        self.max_src_age.is_some()
            || self.max_crate_age.is_some()
            || self.max_index_age.is_some()
            || self.max_git_co_age.is_some()
            || self.max_git_db_age.is_some()
            || self.max_src_size.is_some()
            || self.max_crate_size.is_some()
            || self.max_download_size.is_some()
    }

    pub fn is_target_opt_set(&self) -> bool {
        self.max_target_size.is_some()
            || self.max_target_age.is_some()
            || self.max_shared_target_age.is_some()
            || self.max_shared_target_size.is_some()
    }

    pub fn update_for_auto_gc(
        &mut self,
        config: &Config,
        kinds: &[AutoGcKind],
        max_download_age: Option<Duration>,
    ) -> CargoResult<()> {
        let auto_config = config
            .get::<Option<AutoConfig>>("gc.auto")?
            .unwrap_or_default();
        self.update_for_auto_gc_config(&auto_config, kinds, max_download_age)
    }

    fn update_for_auto_gc_config(
        &mut self,
        auto_config: &AutoConfig,
        kinds: &[AutoGcKind],
        max_download_age: Option<Duration>,
    ) -> CargoResult<()> {
        for kind in kinds {
            if matches!(kind, AutoGcKind::All | AutoGcKind::Download) {
                self.max_src_age = newer_time_span_for_config(
                    self.max_src_age,
                    "gc.auto.max-src-age",
                    auto_config.max_src_age.as_deref().unwrap_or("1 month"),
                )?;
                self.max_crate_age = newer_time_span_for_config(
                    self.max_crate_age,
                    "gc.auto.max-crate-age",
                    auto_config.max_crate_age.as_deref().unwrap_or("3 months"),
                )?;
                self.max_index_age = newer_time_span_for_config(
                    self.max_index_age,
                    "gc.auto.max-index-age",
                    auto_config.max_index_age.as_deref().unwrap_or("3 months"),
                )?;
                self.max_git_co_age = newer_time_span_for_config(
                    self.max_git_co_age,
                    "gc.auto.max-git-co-age",
                    auto_config.max_git_co_age.as_deref().unwrap_or("1 month"),
                )?;
                self.max_git_db_age = newer_time_span_for_config(
                    self.max_git_db_age,
                    "gc.auto.max-git-db-age",
                    auto_config.max_git_db_age.as_deref().unwrap_or("3 months"),
                )?;
            }
            if matches!(kind, AutoGcKind::Target | AutoGcKind::SharedTarget) {
                bail!("target is unimplemented");
            }
        }
        if let Some(max_download_age) = max_download_age {
            self.max_src_age = Some(maybe_newer_span(max_download_age, self.max_src_age));
            self.max_crate_age = Some(maybe_newer_span(max_download_age, self.max_crate_age));
            self.max_index_age = Some(maybe_newer_span(max_download_age, self.max_index_age));
            self.max_git_co_age = Some(maybe_newer_span(max_download_age, self.max_git_co_age));
            self.max_git_db_age = Some(maybe_newer_span(max_download_age, self.max_git_db_age));
        }
        Ok(())
    }
}

pub enum AutoGcKind {
    All,
    Download,
    Target,
    SharedTarget,
}

impl AutoGcKind {
    pub fn from_str(s: &str) -> CargoResult<AutoGcKind> {
        match s {
            "all" => Ok(AutoGcKind::All),
            "download" => Ok(AutoGcKind::Download),
            "target" => bail!("target is unimplemented"),
            "shared-target" => bail!("shared-target is unimplemented"),
            _ => bail!("unexpected value `s`, expected all, download, target, or shared-target"),
        }
    }
}

impl<'a, 'config> Gc<'a, 'config> {
    pub fn new(config: &'config Config, global_last_use: &'a mut GlobalLastUse) -> Gc<'a, 'config> {
        Gc {
            config,
            global_last_use,
        }
    }

    pub fn auto(&mut self, clean_ctx: &mut CleanContext<'config>) -> CargoResult<()> {
        if !self.config.cli_unstable().gc {
            return Ok(());
        }
        // TODO: Consider how errors should be handled in config (particularly
        // for forwards compat, like if we add new forms of frequency).
        let auto_config = self
            .config
            .get::<Option<AutoConfig>>("gc.auto")?
            .unwrap_or_default();
        let Some(freq) = parse_frequency(auto_config.frequency.as_deref().unwrap_or("1 day"))?
        else {
            tracing::trace!("auto gc disabled");
            return Ok(());
        };
        if !self.global_last_use.should_run_auto_gc(freq)? {
            return Ok(());
        }
        let mut gc_opts = GcOpts::default();
        gc_opts.update_for_auto_gc_config(&auto_config, &[AutoGcKind::All], None)?;
        self.gc(clean_ctx, &gc_opts)?;
        if !clean_ctx.dry_run {
            self.global_last_use.set_last_auto_gc()?;
        }
        Ok(())
    }

    pub fn gc(
        &mut self,
        clean_ctx: &mut CleanContext<'config>,
        gc_opts: &GcOpts,
    ) -> CargoResult<()> {
        self.global_last_use.clean(clean_ctx, gc_opts)?;
        // In the future, other gc operations go here, such as target cleaning.
        Ok(())
    }
}

fn newer_time_span_for_config(
    cur_span: Option<Duration>,
    config_name: &str,
    config_span: &str,
) -> CargoResult<Option<Duration>> {
    let config_span = parse_time_span_for_config(config_name, config_span)?;
    Ok(Some(maybe_newer_span(config_span, cur_span)))
}

fn maybe_newer_span(a: Duration, b: Option<Duration>) -> Duration {
    match b {
        Some(b) => {
            if b < a {
                b
            } else {
                a
            }
        }
        None => a,
    }
}

fn parse_frequency(frequency: &str) -> CargoResult<Option<Duration>> {
    if frequency == "always" {
        return Ok(Some(Duration::new(0, 0)));
    } else if frequency == "never" {
        return Ok(None);
    }
    let duration = maybe_parse_time_span(frequency).ok_or_else(|| {
        format_err!(
            "config option `gc.auto.frequency` expected a value of \"always\", \"never\", \
             or \"N seconds/minutes/days/weeks/months\", got: {frequency:?}"
        )
    })?;
    Ok(Some(duration))
}

fn parse_time_span_for_config(config_name: &str, span: &str) -> CargoResult<Duration> {
    maybe_parse_time_span(span).ok_or_else(|| {
        format_err!(
            "config option `{config_name}` expected a value of the form \
             \"N seconds/minutes/days/weeks/months\", got: {span:?}"
        )
    })
}

fn maybe_parse_time_span(span: &str) -> Option<Duration> {
    let Some((left, right)) = span.split_once(' ') else {
        return None;
    };
    let count: u64 = left.parse().ok()?;
    let factor = match right {
        "second" | "seconds" => 1,
        "minute" | "minutes" => 60,
        "hour" | "hours" => 60 * 60,
        "day" | "days" => 24 * 60 * 60,
        "week" | "weeks" => 7 * 24 * 60 * 60,
        "month" | "months" => 30 * 24 * 60 * 60,
        _ => return None,
    };
    Some(Duration::from_secs(factor * count))
}

pub fn parse_time_span(span: &str) -> CargoResult<Duration> {
    maybe_parse_time_span(span).ok_or_else(|| {
        format_err!(
            "expected a value of the form \
             \"N seconds/minutes/days/weeks/months\", got: {span:?}"
        )
    })
}

pub fn parse_human_size(size: &str) -> CargoResult<u64> {
    let size = size.replace(' ', "");
    match size.split_once(|c: char| !c.is_ascii_digit() && c != '.') {
        Some((left, right)) => {
            let factor = match right.to_lowercase().as_str() {
                "b" => 1.0,
                "kb" => 1000.0,
                "mb" => 1000000.0,
                "gb" => 1000000000.0,
                "kib" => 1024.0,
                "mib" => 1024.0 * 1024.0,
                "gib" => 1024.0 * 1024.0 * 1024.0,
                _ => {
                    bail!("unknown size suffix `{right}`, expected B, kB, MB, GB, kiB, MiB, or GiB")
                }
            };
            left.parse::<f64>()
                .with_context(|| "expected an integer or float")
                .map(|size| (size * factor) as u64)
        }
        None => size.parse().with_context(|| "expected an integer size"),
    }
}

pub fn auto_gc(config: &Config) {
    if !config.cli_unstable().gc {
        return;
    }

    if let Err(e) = auto_gc_inner(config) {
        crate::display_warning_with_error(
            "failed to auto-clean cache data",
            &e,
            &mut config.shell(),
        );
    }
}

fn auto_gc_inner(config: &Config) -> CargoResult<()> {
    let _lock = config.acquire_package_cache_lock()?;
    let mut last_use = config.global_last_use()?;
    let mut gc = Gc::new(config, &mut last_use);
    let mut clean_ctx = CleanContext::new(config);
    gc.auto(&mut clean_ctx)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_spans() {
        let d = |x| Some(Duration::from_secs(x));
        assert_eq!(maybe_parse_time_span("0 seconds"), d(0));
        assert_eq!(maybe_parse_time_span("23 seconds"), d(23));
        assert_eq!(maybe_parse_time_span("5 minutes"), d(60 * 5));
        assert_eq!(maybe_parse_time_span("2 hours"), d(60 * 60 * 2));
        assert_eq!(maybe_parse_time_span("1 day"), d(60 * 60 * 24));
        assert_eq!(maybe_parse_time_span("2 weeks"), d(60 * 60 * 24 * 14));
        assert_eq!(maybe_parse_time_span("6 months"), d(60 * 60 * 24 * 30 * 6));

        assert_eq!(parse_frequency("5 seconds").unwrap(), d(5));
        assert_eq!(parse_frequency("always").unwrap(), d(0));
        assert_eq!(parse_frequency("never").unwrap(), None);
    }

    #[test]
    fn time_span_errors() {
        assert_eq!(maybe_parse_time_span(""), None);
        assert_eq!(maybe_parse_time_span("1"), None);
        assert_eq!(maybe_parse_time_span("day"), None);
        assert_eq!(maybe_parse_time_span("-1 days"), None);
        assert_eq!(maybe_parse_time_span("1.5 days"), None);
        assert_eq!(maybe_parse_time_span("1 dayz"), None);
        assert_eq!(maybe_parse_time_span("always"), None);
        assert_eq!(maybe_parse_time_span("never"), None);
        assert_eq!(maybe_parse_time_span("1 day "), None);
        assert_eq!(maybe_parse_time_span(" 1 day"), None);

        let e = parse_time_span_for_config("gc.auto.max-src-age", "-1 days").unwrap_err();
        assert_eq!(
            e.to_string(),
            "config option `gc.auto.max-src-age` \
             expected a value of the form \"N seconds/minutes/days/weeks/months\", \
             got: \"-1 days\""
        );
        let e = parse_frequency("abc").unwrap_err();
        assert_eq!(
            e.to_string(),
            "config option `gc.auto.frequency` \
             expected a value of \"always\", \"never\", or \"N seconds/minutes/days/weeks/months\", \
             got: \"abc\""
        );
    }
}
