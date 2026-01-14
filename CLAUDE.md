# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

## Project Overview

`cargo-propagate-features` is a Cargo subcommand that automatically
propagates workspace crate features to dependencies. When crate A has
feature X and depends on crate B that also has feature X, this tool
ensures A's feature X includes "B/X" in its feature dependencies.

## Build Commands

```bash
# Build
cargo build

# Run directly (during development)
cargo run -- propagate-features [OPTIONS]

# Run after installation
cargo propagate-features [OPTIONS]

# Options:
#   --dry-run              Show changes without modifying files
#   --features <LIST>      Comma-separated features (default: backend,cli,desktop,web)
#   --manifest-path <PATH> Path to Cargo.toml
#   --quiet                Suppress output when no changes
```

## Testing and Linting

```bash
# Run tests
cargo test

# Format check (requires nightly)
cargo +nightly fmt --all -- --check

# Format code
cargo +nightly fmt --all

# Clippy (requires nightly)
cargo +nightly clippy --all-targets --all-features -- -D warnings
```

## Code Style

- **Rust Edition**: 2024, MSRV 1.92.0
- **Formatting**: Uses nightly rustfmt with vertical imports grouped
  by std/external/crate
- **Clippy**: Nightly with strict settings (max 120 lines/function,
  nesting threshold 5)
- **Disallowed variable names**: foo, bar, baz, qux, i, n

## Architecture

Single-binary CLI tool (`src/main.rs`) with two-pass algorithm:

1. **Pass 1 - Cleanup**: Removes hardcoded runtime features from
   workspace dependencies
2. **Pass 2 - Propagation**: Adds `dep/feature` entries to feature
   definitions

Key dependencies:

- `clap`: CLI argument parsing with derive macros
- `toml_edit`: Preserves TOML formatting during edits
- `cargo_plugin_utils`: Workspace package discovery via cargo_metadata
- `cargo-version-info`: Dynamic version computation in build.rs

## Version Management

Version is computed dynamically via `build.rs` using:

1. `BUILD_VERSION` env var (CI)
2. GitHub API (in GitHub Actions)
3. Cargo.toml version + git SHA
4. Fallback: `0.0.0-dev-<short-sha>`

Releases are automated: bump version in Cargo.toml, merge to main, CI
handles tagging and publishing. To bump the version:

```bash
cog bump --patch   # 0.0.1 -> 0.0.2
cog bump --minor   # 0.1.0 -> 0.2.0
cog bump --major   # 1.0.0 -> 2.0.0
```

## Git workflow

- Commits follow Angular Conventional Commits:
  `<type>(<scope>): <subject>`
- Types: feat, fix, docs, refactor, test, style, perf, build, ci,
  chore, revert
- Use lowercase for type, scope, and subject start
- Never bypass git hooks with `--no-verify`
- Never execute `git push` - user must push manually
- Prefer `git rebase` over `git merge` for linear history

## Markdown formatting

- Maximum line length: 70 characters
- Use `-` for unordered lists (not `*` or `+`)
- Use sentence case for headers (not Title Case)
- Indent nested lists with 2 spaces
- Do not trim trailing whitespace

### YAML/frontmatter

- Use multiline syntax (`>-` or `|`) instead of long quoted strings
- Keep lines under 70 characters

### Markdown linting

Configuration is in `.markdownlint.json`:

- Line length: 70 characters (MD013)
- Code blocks: unlimited line length
- Front matter titles: disabled (MD025)

VS Code is configured to run markdownlint on save with auto-fix
enabled. For manual linting:

```bash
markdownlint '**/*.md' --ignore node_modules --ignore target
```
