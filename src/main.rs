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
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};
use clap::Parser;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use toml_edit::{
    DocumentMut,
    Item,
};
use walkdir::WalkDir;

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

    /// Path to workspace root
    #[arg(long, default_value = ".")]
    workspace_path: PathBuf,

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

/// Logger for handling output with quiet mode and cargo-style ephemeral
/// messages
struct Logger {
    quiet: bool,
    progress: Option<ProgressBar>,
}

impl Logger {
    fn new(quiet: bool) -> Self {
        Self {
            quiet,
            progress: None,
        }
    }

    /// Set a status message with a progress bar (ephemeral, like cargo's
    /// "Compiling")
    fn set_progress(&mut self, total: u64) {
        if !self.quiet {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            self.progress = Some(pb);
        }
    }

    /// Update progress status message
    fn set_message(&self, msg: &str) {
        if let Some(pb) = &self.progress {
            pb.set_message(msg.to_string());
        }
    }

    /// Increment progress
    fn inc(&self) {
        if let Some(pb) = &self.progress {
            pb.inc(1);
        }
    }

    /// Print a permanent message (will be kept in output)
    fn println(&mut self, msg: &str) {
        if !self.quiet {
            // If we have an active progress bar, suspend it while printing
            if let Some(pb) = &self.progress {
                pb.suspend(|| {
                    println!("{}", msg);
                });
            } else {
                println!("{}", msg);
            }
        }
    }

    /// Clear/finish the progress bar
    fn finish(&mut self) {
        if let Some(pb) = self.progress.take() {
            pb.finish_and_clear();
        }
    }
}

fn propagate_features(args: PropagateArgs) -> Result<()> {
    let target_features: HashSet<String> = args
        .features
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut logger = Logger::new(args.quiet);

    let crates = discover_crates(&args.workspace_path)?;

    // Build a map of crate name -> features
    let crate_features: HashMap<String, HashSet<String>> = crates
        .iter()
        .map(|c| (c.name.clone(), c.features.clone()))
        .collect();

    let mut total_changes = 0;

    // First pass: Remove hardcoded runtime features from dependencies
    logger.set_progress(crates.len() as u64);
    logger.set_message("🧹 Checking for hardcoded features");

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
            &mut logger,
        )?;

        // Check dev-dependencies
        changed |= remove_hardcoded_features(
            &mut doc,
            "dev-dependencies",
            &target_features,
            &crate_info.name,
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
                        if !dep_name.starts_with("ekg-") {
                            continue;
                        }

                        let Some(dep_table) = dep_value.as_inline_table_mut() else {
                            continue;
                        };

                        if !dep_table.contains_key("workspace")
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
    logger.set_message("🔗 Propagating features");

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
                        logger.println(&format!(
                            "   ✓ {}: Adding {}/{} to {}",
                            crate_info.name, dep, feature_name, feature_name
                        ));
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
    logger: &mut Logger,
) -> Result<bool> {
    let mut changed = false;

    let Some(deps) = doc.get_mut(section).and_then(|d| d.as_table_mut()) else {
        return Ok(false);
    };

    for (dep_name, dep_value) in deps.iter_mut() {
        // Only check workspace dependencies that start with "ekg-"
        if !dep_name.starts_with("ekg-") {
            continue;
        }

        let Some(dep_table) = dep_value.as_inline_table_mut() else {
            continue;
        };

        if !dep_table.contains_key("workspace") || !dep_table.contains_key("features") {
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

fn discover_crates(workspace_path: &Path) -> Result<Vec<CrateInfo>> {
    let mut crates = Vec::new();
    let crates_dir = workspace_path.join("crates");

    for entry in WalkDir::new(&crates_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some("Cargo.toml".as_ref())
            && let Some(crate_info) = parse_crate_info(path)?
        {
            crates.push(crate_info);
        }
    }

    Ok(crates)
}

fn parse_crate_info(cargo_toml_path: &Path) -> Result<Option<CrateInfo>> {
    let content = std::fs::read_to_string(cargo_toml_path)?;
    let doc: toml::Value = toml::from_str(&content)?;

    let package = match doc.get("package") {
        Some(p) => p,
        None => return Ok(None),
    };

    let name = match package.get("name").and_then(|name_val| name_val.as_str()) {
        Some(name_str) => name_str.to_string(),
        None => return Ok(None),
    };

    // Parse features
    let mut features = HashSet::new();
    if let Some(features_table) = doc.get("features").and_then(|f| f.as_table()) {
        for (feature_name, _) in features_table {
            features.insert(feature_name.clone());
        }
    }

    // Parse dependencies (both regular and target-specific)
    let mut dependencies = HashSet::new();

    // Regular dependencies
    if let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table()) {
        for (dep_name, dep_val) in deps {
            // Only include workspace dependencies that start with "ekg-"
            if !dep_name.starts_with("ekg-") {
                continue;
            }

            if let Some(table) = dep_val.as_table()
                && table.contains_key("workspace")
            {
                dependencies.insert(dep_name.clone());
            }
        }
    }

    // Target-specific dependencies
    // Note: This section has deep nesting due to TOML structure traversal
    #[allow(clippy::excessive_nesting)]
    if let Some(target) = doc.get("target").and_then(|t| t.as_table()) {
        for (_, target_config) in target {
            if let Some(deps) = target_config.get("dependencies").and_then(|d| d.as_table()) {
                for (dep_name, dep_val) in deps {
                    if !dep_name.starts_with("ekg-") {
                        continue;
                    }

                    if let Some(table) = dep_val.as_table()
                        && table.contains_key("workspace")
                    {
                        dependencies.insert(dep_name.clone());
                    }
                }
            }
        }
    }

    let crate_dir = cargo_toml_path.parent().unwrap().to_path_buf();

    Ok(Some(CrateInfo {
        name,
        path: crate_dir,
        features,
        dependencies,
    }))
}
