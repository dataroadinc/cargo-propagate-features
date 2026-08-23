//! Integration tests for workspace.package.edition inheritance support.
//!
//! This test ensures that cargo-propagate-features correctly handles workspaces
//! where crates inherit their edition from the workspace manifest via
//! `edition.workspace = true`.
//!
//! Regression test for: https://github.com/dataroadinc/cargo-propagate-features/issues/1
//! The original error was: `workspace.package.edition` was not defined

use std::process::Command;

async fn write_file(path: impl AsRef<std::path::Path>, content: &str) {
    async_fs_io::write_bytes(path, content.as_bytes())
        .await
        .expect("write test file");
}

/// Test that the tool works with a workspace that uses inherited edition.
///
/// This creates a temporary workspace with:
/// - A root Cargo.toml with `[workspace.package]` containing `edition = "2024"`
/// - A member crate that inherits the edition via `edition.workspace = true`
/// - A second member crate with a feature that should be propagated
#[tokio::test]
async fn test_workspace_inherited_edition() {
    // Create a temporary directory for our test workspace
    let temp_dir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();

    // Create the workspace root Cargo.toml with [workspace.package]
    let workspace_toml = r#"[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2024"
version = "0.1.0"
"#;
    write_file(workspace_root.join("Cargo.toml"), workspace_toml).await;

    // Create the crates directory
    async_fs_io::ensure_dir(workspace_root.join("crates/crate-a/src"))
        .await
        .expect("Failed to create crate-a directory");
    async_fs_io::ensure_dir(workspace_root.join("crates/crate-b/src"))
        .await
        .expect("Failed to create crate-b directory");

    // Create crate-a that inherits edition and has a feature that depends on
    // crate-b
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
    write_file(
        workspace_root.join("crates/crate-a/Cargo.toml"),
        crate_a_toml,
    )
    .await;
    write_file(
        workspace_root.join("crates/crate-a/src/lib.rs"),
        "// crate-a\n",
    )
    .await;

    // Create crate-b that inherits edition and has the same feature
    let crate_b_toml = r#"[package]
name = "crate-b"
edition.workspace = true
version.workspace = true

[features]
default = []
web = []
"#;
    write_file(
        workspace_root.join("crates/crate-b/Cargo.toml"),
        crate_b_toml,
    )
    .await;
    write_file(
        workspace_root.join("crates/crate-b/src/lib.rs"),
        "// crate-b\n",
    )
    .await;

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
#[tokio::test]
async fn test_workspace_direct_edition_2024() {
    let temp_dir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();

    // Create a simple workspace without [workspace.package]
    let workspace_toml = r#"[workspace]
resolver = "2"
members = ["crates/*"]
"#;
    write_file(workspace_root.join("Cargo.toml"), workspace_toml).await;

    // Create the crates directory
    async_fs_io::ensure_dir(workspace_root.join("crates/crate-a/src"))
        .await
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
    write_file(
        workspace_root.join("crates/crate-a/Cargo.toml"),
        crate_a_toml,
    )
    .await;
    write_file(
        workspace_root.join("crates/crate-a/src/lib.rs"),
        "// crate-a\n",
    )
    .await;

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
