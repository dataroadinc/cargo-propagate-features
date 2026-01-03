# cargo-propagate-features

[![Crates.io](https://img.shields.io/crates/v/cargo-propagate-features.svg)](https://crates.io/crates/cargo-propagate-features)
[![Documentation](https://docs.rs/cargo-propagate-features/badge.svg)](https://docs.rs/cargo-propagate-features)
[![CI](https://github.com/agnos-ai/cargo-propagate-features/workflows/CI%2FCD/badge.svg)](https://github.com/agnos-ai/cargo-propagate-features/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/cargo-propagate-features.svg)](https://crates.io/crates/cargo-propagate-features)

Cargo subcommand to automatically propagate workspace crate features
to their dependencies.

## What it does

For any crate A with feature X depending on crate B that also has
feature X, this tool ensures that crate A's feature X includes "B/X"
in its feature dependencies.

## Example

If `ekg-deployment-config` has features `[backend, cli, desktop, web]`
and depends on `ekg-types` which also has those features, this tool
will update:

```toml
[features]
backend = []
cli = []
desktop = []
web = []
```

To:

```toml
[features]
backend = ["ekg-types/backend", "ekg-constants/backend", "ekg-util-env/backend"]
cli = ["ekg-types/cli", "ekg-constants/cli", "ekg-util-env/cli"]
desktop = ["ekg-types/desktop", "ekg-constants/desktop", "ekg-util-env/desktop"]
web = ["ekg-types/web", "ekg-constants/web", "ekg-util-env/web"]
```

## Installation

### Using cargo-binstall (Recommended)

First install cargo-binstall if you haven't already:

```bash
cargo install cargo-binstall
```

Then install cargo-propagate-features:

```bash
cargo binstall cargo-propagate-features
```

### Using cargo install

```bash
cargo install cargo-propagate-features
```

## Usage

From the workspace root:

```bash
cargo propagate-features [OPTIONS]
```

### Options

- `--dry-run`: Show what would be changed without modifying files
- `--features <FEATURES>`: Comma-separated list of features to
  propagate (default: backend,cli,desktop,web)
- `--workspace-path <PATH>`: Path to workspace root or Cargo.toml
  (optional, defaults to workspace containing the manifest). When
  using `cargo run`, you can point to a Cargo.toml file directly.
- `--quiet`: Suppress output when there are no changes

The command automatically respects Cargo's standard options when
installed and invoked via `cargo`:

- `--manifest-path <PATH>`: Path to Cargo.toml (automatically handled
  via cargo_metadata)
- `--package <SPEC>`: Work on a specific package (if supported)

## License

MIT License - see [LICENSE](LICENSE) file for details.
