use std::{
    fs::{read, write},
    path::Path,
};

use anyhow::{Context, Result};
use font_feature_freezer::freeze_features_with_stats;
use log::{info, warn};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoRvrn {
    Enabled,
    Disabled,
}

pub fn freeze_features(
    files: &[impl AsRef<Path> + Sync],
    features: &[impl AsRef<str> + Sync],
    auto_rvrn: AutoRvrn,
) -> Result<()> {
    if features.is_empty() {
        info!("No features specified");
        return Ok(());
    }

    let needs_rvrn =
        auto_rvrn == AutoRvrn::Enabled && !features.iter().any(|f| f.as_ref() == "rvrn");

    let feature_tags: Vec<String> = if needs_rvrn {
        std::iter::once("rvrn".to_string())
            .chain(features.iter().map(|f| f.as_ref().to_string()))
            .collect()
    } else {
        features.iter().map(|f| f.as_ref().to_string()).collect()
    };
    info!("Freezing features: {}", feature_tags.join(","));

    let results: Vec<_> = files
        .par_iter()
        .map(|path| freeze_single(path.as_ref(), &feature_tags))
        .collect();

    let mut success = 0;
    let mut failed = 0;
    let mut total_substitutions = 0;

    for result in results {
        match result {
            Ok(subs) => {
                success += 1;
                total_substitutions += subs;
            }
            Err(e) => {
                warn!("{e:?}");
                failed += 1;
            }
        }
    }

    info!(
        "Freeze complete: {success} succeeded, {failed} failed, {total_substitutions} substitutions applied"
    );
    Ok(())
}

fn freeze_single(path: &Path, features: &[String]) -> Result<usize> {
    let data = read(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let (frozen_data, stats) =
        freeze_features_with_stats(&data, features.iter().map(|f| f.as_str()))
            .with_context(|| format!("Failed to freeze features in {}", path.display()))?;

    write(path, frozen_data).with_context(|| format!("Failed to write {}", path.display()))?;

    info!(
        "{}: {} substitutions applied",
        path.file_name().unwrap_or_default().to_string_lossy(),
        stats.substitutions_applied
    );

    Ok(stats.substitutions_applied)
}
