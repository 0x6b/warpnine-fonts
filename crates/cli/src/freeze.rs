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

    let feature_list: String = if needs_rvrn {
        std::iter::once("rvrn")
            .chain(features.iter().map(|f| f.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    } else {
        features.iter().map(|f| f.as_ref()).collect::<Vec<_>>().join(",")
    };
    info!("Freezing features: {feature_list}");

    let results: Vec<_> = files
        .par_iter()
        .map(|path| freeze_single(path.as_ref(), features, needs_rvrn))
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

fn freeze_single(path: &Path, features: &[impl AsRef<str>], prepend_rvrn: bool) -> Result<usize> {
    let data = read(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let feature_iter: Box<dyn Iterator<Item = &str>> = if prepend_rvrn {
        Box::new(std::iter::once("rvrn").chain(features.iter().map(|f| f.as_ref())))
    } else {
        Box::new(features.iter().map(|f| f.as_ref()))
    };

    let (frozen_data, stats) = freeze_features_with_stats(&data, feature_iter)
        .with_context(|| format!("Failed to freeze features in {}", path.display()))?;

    write(path, frozen_data).with_context(|| format!("Failed to write {}", path.display()))?;

    info!(
        "{}: {} substitutions applied",
        path.file_name().unwrap_or_default().to_string_lossy(),
        stats.substitutions_applied
    );

    Ok(stats.substitutions_applied)
}
