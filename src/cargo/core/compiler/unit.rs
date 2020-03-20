use crate::core::compiler::{CompileKind, CompileMode};
use crate::core::{profiles::Profile, InternedString, Package, Target};
use crate::util::hex::short_hash;
use std::hash::Hash;
use std::rc::Rc;

/// All information needed to define a unit.
///
/// A unit is an object that has enough information so that cargo knows how to build it.
/// For example, if your package has dependencies, then every dependency will be built as a library
/// unit. If your package is a library, then it will be built as a library unit as well, or if it
/// is a binary with `main.rs`, then a binary will be output. There are also separate unit types
/// for `test`ing and `check`ing, amongst others.
///
/// The unit also holds information about all possible metadata about the package in `pkg`.
///
/// A unit needs to know extra information in addition to the type and root source file. For
/// example, it needs to know the target architecture (OS, chip arch etc.) and it needs to know
/// whether you want a debug or release build. There is enough information in this struct to figure
/// all that out.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unit {
    /// Information about available targets, which files to include/exclude, etc. Basically stuff in
    /// `Cargo.toml`.
    pub pkg: Rc<Package>,
    /// Information about the specific target to build, out of the possible targets in `pkg`. Not
    /// to be confused with *target-triple* (or *target architecture* ...), the target arch for a
    /// build.
    pub target: Rc<Target>,
    /// The profile contains information about *how* the build should be run, including debug
    /// level, etc.
    pub profile: Profile,
    /// Whether this compilation unit is for the host or target architecture.
    ///
    /// For example, when
    /// cross compiling and using a custom build script, the build script needs to be compiled for
    /// the host architecture so the host rustc can use it (when compiling to the target
    /// architecture).
    pub kind: CompileKind,
    /// The "mode" this unit is being compiled for. See [`CompileMode`] for more details.
    pub mode: CompileMode,
    /// The `cfg` features to enable for this unit.
    /// This must be sorted.
    pub features: Vec<InternedString>,
    /// Whether this is a standard library unit.
    pub is_std: bool,
}

impl Unit {
    /// Returns whether compilation of this unit requires all upstream artifacts
    /// to be available.
    ///
    /// This effectively means that this unit is a synchronization point (if the
    /// return value is `true`) that all previously pipelined units need to
    /// finish in their entirety before this one is started.
    pub fn requires_upstream_objects(&self) -> bool {
        self.mode.is_any_test() || self.target.kind().requires_upstream_objects()
    }

    pub fn buildkey(&self) -> String {
        format!("{}-{}", self.pkg.name(), short_hash(self))
    }
}
