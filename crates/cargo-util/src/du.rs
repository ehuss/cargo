//! A simple disk usage estimator.

use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkBuilder, WalkState};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

/// Determines the disk usage of all files in the given directory.
///
/// The given patterns are gitignore style patterns relative to the given
/// path. If there are patterns, it will only count things matching that
/// pattern. `!` can be used to exclude things. See [`OverrideBuilder::add`]
/// for more info.
///
/// This is a primitive implementation that doesn't handle hard links, and
/// isn't particularly fast (for example, not using `getattrlistbulk` on
/// macOS). It also only uses actual byte sizes instead of block counts (and
/// thus vastly undercounts directories with lots of small files). It would be
/// nice to improve this or replace it with something better.
pub fn du(path: &Path, patterns: &[&str]) -> Result<u64> {
    du_inner(path, patterns).with_context(|| format!("failed to walk `{}`", path.display()))
}

/// Builds directory visitors, where there is one visitor per thread.
///
/// When this is dropped, then all visiting has been completed.
struct DuBuilder {
    /// A condition variable triggered (via the drop impl) with the walking is finished.
    finish_cv: Arc<(Mutex<bool>, Condvar)>,
    /// The total number of bytes.
    total: Arc<AtomicU64>,
    /// A slot used to indicate there was an error while walking.
    ///
    /// It is possible that more than one error happens (such as in different
    /// threads). The error returned is arbitrary in that case.
    err: Arc<Mutex<Option<anyhow::Error>>>,
}

impl<'s> ParallelVisitorBuilder<'s> for DuBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(DuVisitor {
            total: self.total.clone(),
            err: self.err.clone(),
        })
    }
}

/// A visitor, created by [`DuBuilder`], which is responsible for collecting
/// file sizes.
///
/// There is one visitor per thread.
struct DuVisitor {
    total: Arc<AtomicU64>,
    err: Arc<Mutex<Option<anyhow::Error>>>,
}

impl ParallelVisitor for DuVisitor {
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        match entry {
            Ok(entry) => match entry.metadata() {
                Ok(meta) => {
                    if meta.is_file() {
                        self.total.fetch_add(meta.len(), Ordering::SeqCst);
                    }
                }
                Err(e) => {
                    *self.err.lock().unwrap() = Some(e.into());
                    return WalkState::Quit;
                }
            },
            Err(e) => {
                *self.err.lock().unwrap() = Some(e.into());
                return WalkState::Quit;
            }
        }
        WalkState::Continue
    }
}

impl Drop for DuBuilder {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.finish_cv;
        let mut finished = lock.lock().unwrap();
        *finished = true;
        cvar.notify_one();
    }
}

fn du_inner(path: &Path, patterns: &[&str]) -> Result<u64> {
    let mut builder = OverrideBuilder::new(path);
    for pattern in patterns {
        builder.add(pattern)?;
    }
    let overrides = builder.build()?;

    let mut builder = WalkBuilder::new(path);
    builder
        .overrides(overrides)
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false);
    let walker = builder.build_parallel();
    let total = Arc::new(AtomicU64::new(0));

    let finish_cv = Arc::new((Mutex::new(false), Condvar::new()));

    let err = Arc::new(Mutex::new(None));
    {
        let mut visit_builder = DuBuilder {
            finish_cv: finish_cv.clone(),
            total: total.clone(),
            err: err.clone(),
        };
        walker.visit(&mut visit_builder);
    }

    // Wait for the walker to finish.
    let (lock, cvar) = &*finish_cv;
    let mut finished = lock.lock().unwrap();
    while !*finished {
        finished = cvar.wait(finished).unwrap();
    }
    if let Some(e) = err.lock().unwrap().take() {
        return Err(e);
    }

    Ok(total.load(Ordering::SeqCst))
}
