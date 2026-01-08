# Contributing to cargo-propagate-features

Thank you for your interest in contributing! This project uses
[Angular Conventional Commits](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
(also known as the
[Conventional Commits](https://www.conventionalcommits.org/)
specification) and [Cocogitto](https://github.com/cocogitto/cocogitto)
for automated changelog generation.

## Commit Message Format

We follow the **Angular Conventional Commits** specification. Each
commit message should be structured as follows:

```text
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: A new feature (appears in changelog)
- **fix**: A bug fix (appears in changelog)
- **docs**: Documentation changes (appears in changelog)
- **refactor**: Code refactoring (appears in changelog)
- **perf**: Performance improvements (appears in changelog)
- **build**: Changes to build system (appears in changelog)
- **revert**: Reverts a previous commit (appears in changelog)
- **style**: Code style changes (omitted from changelog)
- **test**: Adding or updating tests (omitted from changelog)
- **ci**: CI/CD changes (omitted from changelog)
- **chore**: Other changes (omitted from changelog)

### Scope (Optional)

The scope provides additional context:

- `features`: Changes to feature propagation logic
- `workspace`: Changes to workspace detection/parsing
- `deps`: Dependency updates
- `docs`: Documentation

### Examples

```bash
# Feature commits
feat(features): add support for optional feature propagation
feat: add dry-run mode for previewing changes

# Bug fix commits
fix(workspace): correct workspace root detection
fix: handle missing features section gracefully

# Documentation commits
docs: update README with usage examples
docs(api): improve rustdoc for feature propagation

# Refactoring commits
refactor(features): simplify feature dependency logic
refactor: extract common workspace parsing logic

# Chore commits (won't appear in changelog)
chore: update dependencies
test: add tests for feature propagation
ci: update GitHub Actions workflow
```

## Breaking Changes

For breaking changes, add `!` after the type/scope and include a
`BREAKING CHANGE:` section in the footer:

```bash
feat!: change default feature list

BREAKING CHANGE: Default features changed from
backend,cli,desktop,web to backend,cli
```

## Development Workflow

### 1. Fork and Clone

```bash
git clone git@github.com:YOUR_USERNAME/cargo-propagate-features.git
cd cargo-propagate-features
```

### 2. Create a Branch

```bash
git checkout -b feat/my-new-feature
# or
git checkout -b fix/bug-description
```

### 3. Make Changes

- Write code
- Add tests
- Update documentation
- Ensure tests pass: `cargo test`
- Ensure linting passes: `cargo clippy`
- Format code: `cargo fmt`

### 4. Commit Changes

Use Angular Conventional Commits format:

```bash
git add .
git commit -m "feat(features): add new feature"
```

**Tip**: Install Cocogitto locally to validate commits:

```bash
cargo install cocogitto
cog check  # Validate commits
```

### 5. Push and Create Pull Request

```bash
git push origin feat/my-new-feature
```

Then create a Pull Request on GitHub.

## Releasing a New Version

Only maintainers can release new versions. The process is automated:

### 1. Update Version in Cargo.toml

```bash
# Edit Cargo.toml and bump the version
vim Cargo.toml  # Change version = "0.1.0" to "0.1.1"
```

### 2. Commit the Version Bump

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"
git push origin main
```

### 3. Automatic Release Process

When the version in `Cargo.toml` changes, the CI workflow will
automatically:

1. ✅ Run all checks (format, clippy, build, test)
2. 📝 Generate changelog using Cocogitto (from conventional commits)
3. 📌 Create a git tag (e.g., `v0.1.1`)
4. 🎉 Create a GitHub Release with the changelog
5. 📦 Publish to crates.io
6. 📦 Build and upload binaries for all platforms

**No manual tagging required!** The workflow detects version changes
and handles everything.

## Changelog Generation

The changelog is generated automatically from commit messages:

- **Included**: `feat`, `fix`, `docs`, `refactor`, `perf`, `build`,
  `revert`
- **Excluded**: `style`, `test`, `ci`, `chore`

This encourages meaningful commit messages and creates a clean,
user-focused changelog.

## Code Review

All contributions go through code review:

- Ensure CI passes (all checks must be green)
- Follow Rust best practices
- Add tests for new features
- Update documentation
- Use Angular Conventional Commits format

## Questions?

Feel free to open an issue for questions or discussions!
