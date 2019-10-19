use crate::util::errors::CargoResult;
use std::hash::{self, Hash};
use url::Url;

/// A newtype wrapper around `Url` which represents a "canonical" version of an
/// original URL.
///
/// A "canonical" url is only intended for internal comparison purposes in
/// Cargo. It's to help paper over mistakes such as depending on
/// `github.com/foo/bar` vs `github.com/foo/bar.git`. This is **only** for
/// internal purposes within Cargo and provides no means to actually read the
/// underlying string value of the `Url` it contains. This is intentional,
/// because all fetching should still happen within the context of the original
/// URL.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CanonicalUrl(Url);

impl CanonicalUrl {
    pub fn new(url: &Url) -> CargoResult<CanonicalUrl> {
        let mut url = url.clone();

        // Strip a trailing slash.
        if url.path().ends_with('/') {
            url.path_segments_mut().unwrap().pop_if_empty();
        }

        // For GitHub URLs specifically, just lower-case everything. GitHub
        // treats both the same, but they hash differently, and we're gonna be
        // hashing them. This wants a more general solution, and also we're
        // almost certainly not using the same case conversion rules that GitHub
        // does. (See issue #84)
        if url.host_str() == Some("github.com") {
            url.set_scheme("https").unwrap();
            let path = url.path().to_lowercase();
            url.set_path(&path);
        }

        // Repos can generally be accessed with or without `.git` extension.
        let needs_chopping = url.path().ends_with(".git");
        if needs_chopping {
            if let Some(mut segments) = url.path_segments() {
                let last = {
                    let last = segments.next_back().unwrap();
                    last[..last.len() - 4].to_owned()
                };
                url.path_segments_mut().unwrap().pop().push(&last);
            }
        }

        Ok(CanonicalUrl(url))
    }

    /// Returns the raw canonicalized URL, although beware that this should
    /// never be used/displayed/etc, it should only be used for internal data
    /// structures and hashes and such.
    pub fn raw_canonicalized_url(&self) -> &Url {
        &self.0
    }
}

// See comment in `source_id.rs` for why we explicitly use `as_str()` here.
impl Hash for CanonicalUrl {
    fn hash<S: hash::Hasher>(&self, into: &mut S) {
        self.0.as_str().hash(into);
    }
}
