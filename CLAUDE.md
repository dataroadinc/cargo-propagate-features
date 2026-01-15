# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

## Related Projects

This crate is part of a family of Rust projects that share the same
coding standards, tooling, and workflows:

Cargo plugins:

- `cargo-fmt-toml` - Format and normalize Cargo.toml files
- `cargo-nightly` - Nightly toolchain management
- `cargo-plugin-utils` - Shared utilities for cargo plugins
- `cargo-propagate-features` - Propagate features to dependencies
- `cargo-version-info` - Dynamic version computation

Other Rust crates:

- `dotenvage` - Environment variable management

All projects use identical configurations for rustfmt, clippy,
markdownlint, cocogitto, and git hooks. When making changes to
tooling or workflow conventions, apply them consistently across
all repositories.

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

## GitHub Actions Integration

The `dataroadinc/github-actions` repository provides a reusable action
for installing this tool in CI workflows:

```yaml
- name: Setup cargo-propagate-features
  uses: dataroadinc/github-actions/.github/actions/setup-cargo-propagate-features@main
  with:
    version: "0.2.0"  # Optional, defaults to latest
```

The action:

- Installs via cargo-binstall for fast binary downloads
- Caches the binary by version, OS, and architecture
- Adds `~/.cargo/bin` to PATH
- Depends on `setup-cargo-binstall` (automatically invoked)

Source: `../github-actions/.github/actions/setup-cargo-propagate-features/`

## Version Management

Use `cargo version-info bump` for version management. This command
updates Cargo.toml and creates a commit, but does NOT create tags
(tags are created by CI after tests pass).

```bash
cargo version-info bump --patch   # 0.0.1 -> 0.0.2
cargo version-info bump --minor   # 0.1.0 -> 0.2.0
cargo version-info bump --major   # 1.0.0 -> 2.0.0
```

**Do NOT use `cog bump`** - it creates local tags which conflict
with CI's tag creation workflow.

**Workflow:**

1. Create PR with version bump commit
2. Merge PR to main
3. CI detects version change, creates tag, publishes release

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
