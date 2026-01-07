# cargo-propagate-features

[![Crates.io](https://img.shields.io/crates/v/cargo-propagate-features.svg)](https://crates.io/crates/cargo-propagate-features)
[![Documentation](https://docs.rs/cargo-propagate-features/badge.svg)](https://docs.rs/cargo-propagate-features)
[![CI](https://github.com/dataroadinc/cargo-propagate-features/workflows/CI%2FCD/badge.svg)](https://github.com/dataroadinc/cargo-propagate-features/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/cargo-propagate-features.svg)](https://crates.io/crates/cargo-propagate-features)

Cargo subcommand to automatically propagate workspace crate features
to their dependencies.

## What it does

For any crate A with feature X depending on crate B that also has
feature X, this tool ensures that crate A's feature X includes "B/X"
in its feature dependencies.

This ensures that when you enable a feature on a crate, all its
dependencies that have the same feature also get that feature enabled.
Without this, enabling a feature might not work correctly because
dependencies don't have their corresponding features enabled.

## Example

Consider a workspace with three crates:

- `my-app` depends on `my-library` and `my-utils`
- All three crates have the same features: `backend`, `cli`, `desktop`, `web`

When you enable the `backend` feature on `my-app`, you typically want
the `backend` features of `my-library` and `my-utils` to also be
enabled. This tool automatically adds those feature dependencies.

**Before running the tool:**

```toml
# my-app/Cargo.toml
[dependencies]
my-library = { path = "../my-library" }
my-utils = { path = "../my-utils" }

[features]
backend = []      # ❌ Doesn't enable backend features on dependencies
cli = []
desktop = []
web = []
```

**After running the tool:**

```toml
# my-app/Cargo.toml
[dependencies]
my-library = { path = "../my-library" }
my-utils = { path = "../my-utils" }

[features]
backend = ["my-library/backend", "my-utils/backend"]  # ✅ Now properly propagates
cli = ["my-library/cli", "my-utils/cli"]
desktop = ["my-library/desktop", "my-utils/desktop"]
web = ["my-library/web", "my-utils/web"]
```

Now when you build with `cargo build --features backend`, the `backend`
features on `my-library` and `my-utils` will also be enabled.

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
