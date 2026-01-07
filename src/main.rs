//! Cargo subcommand to automatically propagate workspace crate features to
//! dependencies.
//!
//! For any crate A with feature X depending on crate B that also has feature X,
//! this tool ensures that crate A's feature X includes "B/X" in its feature
//! dependencies.

use std::collections::{
    HashMap,
    HashSet,
};
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use cargo_plugin_utils::ProgressLogger;
use clap::Parser;
use toml_edit::{
    DocumentMut,
    Item,
};

#[derive(Parser, Debug)]
#[command(
    name = "cargo-propagate-features",
    about = "Automatically propagate workspace crate features to dependencies",
    bin_name = "cargo"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    #[command(name = "propagate-features")]
    PropagateFeatures(PropagateArgs),
}

#[derive(Parser, Debug)]
struct PropagateArgs {
    /// Show what would be changed without modifying files
    #[arg(long)]
    dry_run: bool,

    /// Comma-separated list of features to propagate
    #[arg(long, default_value = "backend,cli,desktop,web")]
    features: String,

    /// Path to Cargo.toml manifest (idiomatic cargo flag)
    ///
    /// Note: When using `cargo run`, place this flag BEFORE the `--`:
    /// `cargo run --manifest-path <path> -- propagate-features`
    #[arg(long)]
    manifest_path: Option<PathBuf>,

    /// Suppress output when there are no changes
    #[arg(long)]
    quiet: bool,
}

#[derive(Debug, Clone)]
struct CrateInfo {
    name: String,
    path: PathBuf,
    features: HashSet<String>,
    dependencies: HashSet<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::PropagateFeatures(args) => propagate_features(args),
    }
}

fn propagate_features(args: PropagateArgs) -> Result<()> {
    let target_features: HashSet<String> = args
        .features
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut logger = ProgressLogger::new(args.quiet);

    // Use cargo_metadata to get all workspace packages - no need to manually
    // discover! Process all packages in the workspace (supports both single-package
    // projects and workspace projects with packages in crates/ or elsewhere).
    let packages = cargo_plugin_utils::get_workspace_packages(args.manifest_path.as_deref())
        .context("Failed to get cargo metadata. Make sure you're in a Cargo project or use --manifest-path.")?;

    let mut crates: Vec<CrateInfo> = packages
        .iter()
        .filter_map(|pkg| {
            let manifest_dir = pkg.manifest_path.as_std_path().parent()?;

            // Extract features from cargo_metadata (already parsed!)
            let features: HashSet<String> = pkg.features.keys().cloned().collect();

            // Extract workspace dependencies (any dependency that exists in the workspace)
            // We'll filter to workspace packages later when we have the full set
            let dependencies: HashSet<String> = pkg
                .dependencies
                .iter()
                .map(|dep| dep.name.as_str().to_string())
                .collect();

            Some(CrateInfo {
                name: pkg.name.as_str().to_string(),
                path: manifest_dir.to_path_buf(),
                features,
                dependencies,
            })
        })
        .collect();

    // Build a set of workspace package names for efficient lookup
    let workspace_package_names: HashSet<String> = packages
        .iter()
        .map(|p| p.name.as_str().to_string())
        .collect();

    // Filter dependencies to only include those actually in the workspace
    for crate_info in &mut crates {
        crate_info
            .dependencies
            .retain(|dep_name| workspace_package_names.contains(dep_name));
    }

    // Build a map of crate name -> features
    let crate_features: HashMap<String, HashSet<String>> = crates
        .iter()
        .map(|c| (c.name.clone(), c.features.clone()))
        .collect();

    let mut total_changes = 0;

    // First pass: Remove hardcoded runtime features from dependencies
    logger.set_progress(crates.len() as u64);
    logger.set_message("Checking");

    for crate_info in &crates {
        logger.inc();

        let cargo_toml_path = crate_info.path.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml_path)
            .context(format!("Failed to read {:?}", cargo_toml_path))?;

        let mut doc = content
            .parse::<DocumentMut>()
            .context(format!("Failed to parse {:?}", cargo_toml_path))?;

        let mut changed = false;

        // Check regular dependencies
        changed |= remove_hardcoded_features(
            &mut doc,
            "dependencies",
            &target_features,
            &crate_info.name,
            &workspace_package_names,
            &mut logger,
        )?;

        // Check dev-dependencies
        changed |= remove_hardcoded_features(
            &mut doc,
            "dev-dependencies",
            &target_features,
            &crate_info.name,
            &workspace_package_names,
            &mut logger,
        )?;

        // Check target-specific dependencies
        // Note: This section has deep nesting due to TOML structure traversal
        #[allow(clippy::excessive_nesting)]
        if let Some(target_table) = doc.get_mut("target").and_then(|t| t.as_table_mut()) {
            for (target_name, target_config) in target_table.iter_mut() {
                let section_name = format!("target.{}.dependencies", target_name);
                if let Some(deps) = target_config
                    .get_mut("dependencies")
                    .and_then(|d| d.as_table_mut())
                {
                    for (dep_name, dep_value) in deps.iter_mut() {
                        let Some(dep_table) = dep_value.as_inline_table_mut() else {
                            continue;
                        };

                        // Only process workspace dependencies that are actually in the workspace
                        let dep_name_str = dep_name.to_string();
                        if !dep_table.contains_key("workspace")
                            || !workspace_package_names.contains(&dep_name_str)
                            || !dep_table.contains_key("features")
                        {
                            continue;
                        }

                        let Some(features_array) =
                            dep_table.get_mut("features").and_then(|f| f.as_array_mut())
                        else {
                            continue;
                        };

                        let original_len = features_array.len();
                        features_array.retain(|v| {
                            v.as_str()
                                .map(|s| !target_features.contains(s))
                                .unwrap_or(true)
                        });

                        if features_array.len() < original_len {
                            logger.println(&format!(
                                "   ⚠️  {}: Removed hardcoded runtime features from {} in {}",
                                crate_info.name, dep_name, section_name
                            ));
                            changed = true;
                            total_changes += 1;

                            // If features array is now empty, remove it
                            if features_array.is_empty() {
                                dep_table.remove("features");
                            }
                        }
                    }
                }
            }
        }

        if changed {
            if args.dry_run {
                logger.println(&format!("   [DRY RUN] Would update {:?}", cargo_toml_path));
            } else {
                std::fs::write(&cargo_toml_path, doc.to_string())
                    .context(format!("Failed to write {:?}", cargo_toml_path))?;
                logger.println(&format!("   💾 Updated {:?}", cargo_toml_path));
            }
        }
    }
    logger.finish();

    // Second pass: Propagate features
    logger.set_progress(crates.len() as u64);
    logger.set_message("Propagating");

    for crate_info in &crates {
        logger.inc();

        // Check if this crate has any of our target features
        let has_target_features: Vec<_> = target_features
            .iter()
            .filter(|f| crate_info.features.contains(*f))
            .collect();

        if has_target_features.is_empty() {
            continue;
        }

        let cargo_toml_path = crate_info.path.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml_path)
            .context(format!("Failed to read {:?}", cargo_toml_path))?;

        let mut doc = content
            .parse::<DocumentMut>()
            .context(format!("Failed to parse {:?}", cargo_toml_path))?;

        let mut changed = false;

        // Note: This section has deep nesting due to TOML structure traversal
        #[allow(clippy::excessive_nesting)]
        for feature_name in &has_target_features {
            // Find dependencies that have this feature
            let deps_with_feature: Vec<_> = crate_info
                .dependencies
                .iter()
                .filter(|dep| {
                    crate_features
                        .get(*dep)
                        .map(|f| f.contains(*feature_name))
                        .unwrap_or(false)
                })
                .collect();

            if deps_with_feature.is_empty() {
                continue;
            }

            // Update the feature in the TOML
            if let Some(feature_item) = doc
                .get_mut("features")
                .and_then(|f| f.as_table_like_mut())
                .and_then(|table| table.get_mut(feature_name.as_str()))
            {
                let feature_array = match feature_item {
                    Item::Value(toml_edit::Value::Array(arr)) => arr,
                    _ => continue,
                };

                // Get existing feature dependencies
                let mut existing: HashSet<String> = feature_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                // Add missing feature dependencies
                for dep in &deps_with_feature {
                    let dep_feature = format!("{}/{}", dep, feature_name);
                    if !existing.contains(&dep_feature) {
                        logger.status(
                            "Adding",
                            &format!("{}/{} to {}", dep, feature_name, feature_name),
                        );
                        feature_array.push(dep_feature.clone());
                        existing.insert(dep_feature);
                        changed = true;
                        total_changes += 1;
                    }
                }
            }
        }

        if changed {
            if args.dry_run {
                logger.println(&format!("   [DRY RUN] Would update {:?}", cargo_toml_path));
            } else {
                std::fs::write(&cargo_toml_path, doc.to_string())
                    .context(format!("Failed to write {:?}", cargo_toml_path))?;
                logger.println(&format!("   💾 Updated {:?}", cargo_toml_path));
            }
        }
    }
    logger.finish();

    // In quiet mode, show nothing. Otherwise show summary.
    if !args.quiet {
        if total_changes > 0 {
            logger.println("✨ Complete!");
            if args.dry_run {
                logger.println(&format!("   Would make {} changes", total_changes));
                logger.println("   Run without --dry-run to apply changes");
            } else {
                logger.println(&format!("   Made {} changes", total_changes));
            }
        } else {
            logger.println("✨ No changes needed");
        }
    }

    Ok(())
}

fn remove_hardcoded_features(
    doc: &mut DocumentMut,
    section: &str,
    target_features: &HashSet<String>,
    crate_name: &str,
    workspace_package_names: &HashSet<String>,
    logger: &mut ProgressLogger,
) -> Result<bool> {
    let mut changed = false;

    let Some(deps) = doc.get_mut(section).and_then(|d| d.as_table_mut()) else {
        return Ok(false);
    };

    for (dep_name, dep_value) in deps.iter_mut() {
        let Some(dep_table) = dep_value.as_inline_table_mut() else {
            continue;
        };

        // Only process workspace dependencies that are actually in the workspace
        let dep_name_str = dep_name.to_string();
        if !dep_table.contains_key("workspace")
            || !workspace_package_names.contains(&dep_name_str)
            || !dep_table.contains_key("features")
        {
            continue;
        }

        let Some(features_array) = dep_table.get_mut("features").and_then(|f| f.as_array_mut())
        else {
            continue;
        };

        let original_len = features_array.len();
        features_array.retain(|v| {
            v.as_str()
                .map(|s| !target_features.contains(s))
                .unwrap_or(true)
        });

        if features_array.len() < original_len {
            logger.println(&format!(
                "   ⚠️  {}: Removed hardcoded runtime features from {} in [{}]",
                crate_name, dep_name, section
            ));
            changed = true;

            // If features array is now empty, remove it
            if features_array.is_empty() {
                dep_table.remove("features");
            }
        }
    }

    Ok(changed)
}
