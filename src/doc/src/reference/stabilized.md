## Stabilized Features

The following is a list of [unstable features](unstable.md) which have graduated to be stabilized and generally available.

### Compile progress

The compile-progress feature has been stabilized in the 1.30 release.
Progress bars are now enabled by default.
See [`term.progress`](config.md#termprogresswhen) for more information about
controlling this feature.

### Edition

Specifying the `edition` in `Cargo.toml` has been stabilized in the 1.31 release.
See [the edition field](manifest.md#the-edition-field) for more information
about specifying this field.

### rename-dependency

Specifying renamed dependencies in `Cargo.toml` has been stabilized in the 1.31 release.
See [renaming dependencies](specifying-dependencies.md#renaming-dependencies-in-cargotoml)
for more information about renaming dependencies.

### Alternate Registries

Support for alternate registries has been stabilized in the 1.34 release.
See the [Registries chapter](registries.md) for more information about alternate registries.

### Offline Mode

The offline feature has been stabilized in the 1.36 release.
See the [`--offline` flag](../commands/cargo.md#option-cargo---offline) for
more information on using the offline mode.

### publish-lockfile

The `publish-lockfile` feature has been removed in the 1.37 release.
The `Cargo.lock` file is always included when a package is published if the
package contains a binary target. `cargo install` requires the `--locked` flag
to use the `Cargo.lock` file.
See [`cargo package`](../commands/cargo-package.md) and
[`cargo install`](../commands/cargo-install.md) for more information.

### default-run

The `default-run` feature has been stabilized in the 1.37 release.
See [the `default-run` field](manifest.md#the-default-run-field) for more
information about specifying the default target to run.

### cache-messages

Compiler message caching has been stabilized in the 1.40 release.
Compiler warnings are now cached by default and will be replayed automatically
when re-running Cargo.

### install-upgrade

The `install-upgrade` feature has been stabilized in the 1.41 release.
[`cargo install`] will now automatically upgrade packages if they appear to be
out-of-date. See the [`cargo install`] documentation for more information.

[`cargo install`]: ../commands/cargo-install.md

### Profile Overrides

Profile overrides have been stabilized in the 1.41 release.
See [Profile Overrides](profiles.md#overrides) for more information on using
overrides.

### Config Profiles

Specifying profiles in Cargo config files and environment variables has been
stabilized in the 1.43 release.
See the [config `[profile]` table](config.md#profile) for more information
about specifying [profiles](profiles.md) in config files.

### crate-versions

The `-Z crate-versions` flag has been stabilized in the 1.47 release.
The crate version is now automatically included in the
[`cargo doc`](../commands/cargo-doc.md) documentation sidebar.

### Features

The `-Z features` flag has been stabilized in the 1.51 release.
See [feature resolver version 2](features.md#feature-resolver-version-2)
for more information on using the new feature resolver.

### package-features

The `-Z package-features` flag has been stabilized in the 1.51 release.
See the [resolver version 2 command-line flags](features.md#resolver-version-2-command-line-flags)
for more information on using the features CLI options.

### Resolver

The `resolver` feature in `Cargo.toml` has been stabilized in the 1.51 release.
See the [resolver versions](resolver.md#resolver-versions) for more
information about specifying resolvers.

### extra-link-arg

The `extra-link-arg` feature to specify additional linker arguments in build
scripts has been stabilized in the 1.56 release. See the [build script
documentation](build-scripts.md#outputs-of-the-build-script) for more
information on specifying extra linker arguments.

### configurable-env

The `configurable-env` feature to specify environment variables in Cargo
configuration has been stabilized in the 1.56 release. See the [config
documentation](config.html#env) for more information about configuring
environment variables.

### rust-version

The `rust-version` field in `Cargo.toml` has been stabilized in the 1.56 release.
See the [rust-version field](manifest.html#the-rust-version-field) for more
information on using the `rust-version` field and the `--ignore-rust-version` option.

### patch-in-config

The `-Z patch-in-config` flag, and the corresponding support for
`[patch]` section in Cargo configuration files has been stabilized in
the 1.56 release. See the [patch field](config.html#patch) for more
information.

### edition 2021

The 2021 edition has been stabilized in the 1.56 release.
See the [`edition` field](manifest.md#the-edition-field) for more information on setting the edition.
See [`cargo fix --edition`](../commands/cargo-fix.md) and [The Edition Guide](../../edition-guide/index.html) for more information on migrating existing projects.


### Custom named profiles

Custom named profiles have been stabilized in the 1.57 release. See the
[profiles chapter](profiles.md#custom-profiles) for more information.

### Profile `strip` option

The profile `strip` option has been stabilized in the 1.59 release. See the
[profiles chapter](profiles.md#strip) for more information.

### Future incompat report

Support for generating a future-incompat report has been stabilized
in the 1.59 release. See the [future incompat report chapter](future-incompat-report.md)
for more information.

### Namespaced features

Namespaced features has been stabilized in the 1.60 release.
See the [Features chapter](features.md#optional-dependencies) for more information.

### Weak dependency features

Weak dependency features has been stabilized in the 1.60 release.
See the [Features chapter](features.md#dependency-features) for more information.

### timings

The `-Ztimings` option has been stabilized as `--timings` in the 1.60 release.
(`--timings=html` and the machine-readable `--timings=json` output remain
unstable and require `-Zunstable-options`.)

### config-cli

The `--config` CLI option has been stabilized in the 1.63 release. See
the [config documentation](config.html#command-line-overrides) for more
information.

### multitarget

The `-Z multitarget` option has been stabilized in the 1.64 release.
See [`build.target`](config.md#buildtarget) for more information about
setting the default [target platform triples][target triple].

### crate-type

The `--crate-type` flag for `cargo rustc` has been stabilized in the 1.64
release. See the [`cargo rustc` documentation](../commands/cargo-rustc.md)
for more information.

### Workspace Inheritance

Workspace Inheritance has been stabilized in the 1.64 release.
See [workspace.package](workspaces.md#the-package-table),
[workspace.dependencies](workspaces.md#the-dependencies-table),
and [inheriting-a-dependency-from-a-workspace](specifying-dependencies.md#inheriting-a-dependency-from-a-workspace)
for more information.

### terminal-width

The `-Z terminal-width` option has been stabilized in the 1.68 release.
The terminal width is always passed to the compiler when running from a
terminal where Cargo can automatically detect the width.

### sparse-registry

Sparse registry support has been stabilized in the 1.68 release.
See [Registry Protocols](registries.md#registry-protocols) for more information.
