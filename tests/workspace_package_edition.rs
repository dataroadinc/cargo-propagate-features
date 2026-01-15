//! Integration tests for workspace.package.edition inheritance support.
//!
//! This test ensures that cargo-propagate-features correctly handles workspaces
//! where crates inherit their edition from the workspace manifest via
//! `edition.workspace = true`.
//!
//! Regression test for: https://github.com/dataroadinc/cargo-propagate-features/issues/1
//! The original error was: `workspace.package.edition` was not defined

use std::fs;
use std::process::Command;

/// Test that the tool works with a workspace that uses inherited edition.
///
/// This creates a temporary workspace with:
/// - A root Cargo.toml with `[workspace.package]` containing `edition = "2024"`
/// - A member crate that inherits the edition via `edition.workspace = true`
/// - A second member crate with a feature that should be propagated
#[test]
fn test_workspace_inherited_edition() {
    // Create a temporary directory for our test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();

    // Create the workspace root Cargo.toml with [workspace.package]
    let workspace_toml = r#"[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2024"
version = "0.1.0"
"#;
    fs::write(workspace_root.join("Cargo.toml"), workspace_toml)
        .expect("Failed to write workspace Cargo.toml");

    // Create the crates directory
    fs::create_dir_all(workspace_root.join("crates/crate-a/src"))
        .expect("Failed to create crate-a directory");
    fs::create_dir_all(workspace_root.join("crates/crate-b/src"))
        .expect("Failed to create crate-b directory");

    // Create crate-a that inherits edition and has a feature that depends on crate-b
    let crate_a_toml = r#"[package]
name = "crate-a"
edition.workspace = true
version.workspace = true

[dependencies]
crate-b = { path = "../crate-b" }

[features]
default = []
web = []
"#;
    fs::write(workspace_root.join("crates/crate-a/Cargo.toml"), crate_a_toml)
        .expect("Failed to write crate-a Cargo.toml");
    fs::write(
        workspace_root.join("crates/crate-a/src/lib.rs"),
        "// crate-a\n",
    )
    .expect("Failed to write crate-a lib.rs");

    // Create crate-b that inherits edition and has the same feature
    let crate_b_toml = r#"[package]
name = "crate-b"
edition.workspace = true
version.workspace = true

[features]
default = []
web = []
"#;
    fs::write(workspace_root.join("crates/crate-b/Cargo.toml"), crate_b_toml)
        .expect("Failed to write crate-b Cargo.toml");
    fs::write(
        workspace_root.join("crates/crate-b/src/lib.rs"),
        "// crate-b\n",
    )
    .expect("Failed to write crate-b lib.rs");

    // Run cargo-propagate-features with --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-propagate-features"))
        .args(["propagate-features", "--dry-run", "--features", "web"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to execute cargo-propagate-features");

    // Check that it succeeded (no error about workspace.package.edition)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "cargo-propagate-features failed with status: {:?}\nstderr: {}\nstdout: {}",
        output.status,
        stderr,
        stdout
    );

    // Ensure we didn't get the specific error about edition not being defined
    assert!(
        !stderr.contains("workspace.package.edition"),
        "Got workspace.package.edition error: {}",
        stderr
    );
}

/// Test that the tool also works with a workspace that uses Rust 2024 edition
/// directly in crates (not inherited).
#[test]
fn test_workspace_direct_edition_2024() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();

    // Create a simple workspace without [workspace.package]
    let workspace_toml = r#"[workspace]
resolver = "2"
members = ["crates/*"]
"#;
    fs::write(workspace_root.join("Cargo.toml"), workspace_toml)
        .expect("Failed to write workspace Cargo.toml");

    // Create the crates directory
    fs::create_dir_all(workspace_root.join("crates/crate-a/src"))
        .expect("Failed to create crate-a directory");

    // Create a crate with edition = "2024" directly
    let crate_a_toml = r#"[package]
name = "crate-a"
edition = "2024"
version = "0.1.0"

[features]
default = []
web = []
"#;
    fs::write(workspace_root.join("crates/crate-a/Cargo.toml"), crate_a_toml)
        .expect("Failed to write crate-a Cargo.toml");
    fs::write(
        workspace_root.join("crates/crate-a/src/lib.rs"),
        "// crate-a\n",
    )
    .expect("Failed to write crate-a lib.rs");

    // Run cargo-propagate-features with --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-propagate-features"))
        .args(["propagate-features", "--dry-run", "--features", "web"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to execute cargo-propagate-features");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "cargo-propagate-features failed with status: {:?}\nstderr: {}\nstdout: {}",
        output.status,
        stderr,
        stdout
    );
}
