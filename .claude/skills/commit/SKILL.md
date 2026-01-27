---
name: commit
description:
  Create commits following Angular Conventional Commits format with
  proper scope naming for consistent changelog generation
---

# Commit Skill

Use this skill when creating git commits. All commits must follow
Angular Conventional Commits format because:

1. Git hooks enforce it (commit will be rejected otherwise)
2. Cocogitto parses commits to generate release changelogs

## Commit Message Format

```text
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Requirements:**

- Type: lowercase, from allowed list
- Scope: mandatory, lowercase, describes what changed
- Subject: lowercase start, imperative mood, no period
- Breaking changes: add exclamation mark before colon (e.g. feat(api)!:)

## Allowed Types

### Angular standard types

| Type       | Purpose                          | In Changelog |
| ---------- | -------------------------------- | ------------ |
| `feat`     | New feature                      | Yes          |
| `fix`      | Bug fix                          | Yes          |
| `docs`     | Documentation only               | Yes          |
| `refactor` | Code change (no feat/fix)        | Yes          |
| `perf`     | Performance improvement          | Yes          |
| `build`    | Build system changes             | Yes          |
| `style`    | Formatting, whitespace           | No           |
| `test`     | Adding/fixing tests              | No           |
| `ci`       | CI/CD configuration              | No           |

### Extended types (not in original Angular spec)

These are widely adopted extensions configured in `cog.toml`:

| Type       | Purpose                          | In Changelog |
| ---------- | -------------------------------- | ------------ |
| `chore`    | Maintenance, deps, tooling       | No           |
| `revert`   | Revert previous commit           | Yes          |

## Scope Naming Guidelines

Scopes should be consistent to group related changes in changelogs.

### Component scopes

- `cli` - command-line interface
- `features` - feature propagation logic
- `toml` - Cargo.toml parsing and manipulation
- `workspace` - workspace handling

### Infrastructure scopes

- `version` - version bumps
- `deps` - dependency updates
- `ci` - CI/CD workflows (use with `ci:` or `chore:` type)
- `config` - configuration files
- `msrv` - minimum supported Rust version

### Documentation scopes

- `readme` - README.md changes
- `claude` - CLAUDE.md, Claude Code skills

### Testing scopes

- `tests` - general test changes

## Examples

### Good commit messages

```text
feat(features): add transitive feature propagation
fix(toml): handle missing features section
docs(readme): update installation instructions
refactor(cli): simplify argument parsing
test(features): add tests for circular dependencies
chore(deps): update toml_edit to 0.22
ci(workflows): add release binary uploads
feat(cli)!: change default propagation behavior
```

### Bad commit messages

```text
feat: add feature              # missing scope
Fix(cli): Fix bug              # uppercase type and subject
feat(cli): Add feature.        # uppercase subject, has period
updated the readme             # wrong format entirely
feat(cli) add feature          # missing colon
feat(Cli): add feature         # uppercase scope
```

## Breaking Changes

For breaking changes, add an exclamation mark after the scope:

```text
feat(features)!: change default propagation depth

BREAKING CHANGE: Features now propagate to all transitive dependencies.
Use --depth=1 for the old behavior.
```

This will appear prominently in the changelog.

## Commit Process

1. Stage your changes: `git add <files>`
2. Commit with message: `git commit -m "type(scope): subject"`
3. Hooks automatically run:
   - pre-commit: fmt and clippy checks
   - commit-msg: validates conventional commit format
   - post-commit: verifies signature

If the commit is rejected, fix the issue and try again.

## Multi-line Commits

For complex changes, use a body:

```bash
git commit -m "feat(features): add selective propagation

Allows specifying which features to propagate.
Use --include and --exclude flags for fine control.

Closes #42"
```

## Checking Recent Commits

To see the commit style used in this repo:

```bash
git log --oneline -20
```
