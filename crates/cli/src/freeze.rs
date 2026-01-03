//! OpenType feature freezing.

use std::path::Path;

use anyhow::{Context, Result};
use font_feature_freezer::freeze_features_with_stats;
use log::{info, warn};
use rayon::prelude::*;

use crate::io::{read_font, write_font};

/// Whether to auto-prepend 'rvrn' feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoRvrn {
    Enabled,
    #[default]
    Disabled,
}

/// Feature freezer with configurable options.
#[derive(Default)]
pub struct Freezer {
    features: Vec<String>,
    auto_rvrn: AutoRvrn,
}

impl Freezer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_features(mut self, features: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.features.extend(features.into_iter().map(|f| f.as_ref().to_string()));
        self
    }

    pub fn auto_rvrn(mut self, auto: AutoRvrn) -> Self {
        self.auto_rvrn = auto;
        self
    }

    fn resolved_features(&self) -> Vec<String> {
        let needs_rvrn = self.auto_rvrn == AutoRvrn::Enabled
            && !self.features.iter().any(|f| f == "rvrn");

        if needs_rvrn {
            std::iter::once("rvrn".to_string())
                .chain(self.features.iter().cloned())
                .collect()
        } else {
            self.features.clone()
        }
    }

    /// Freeze features in font data, returning the frozen data.
    pub fn freeze(&self, data: &[u8]) -> Result<(Vec<u8>, usize)> {
        let features = self.resolved_features();
        let (frozen_data, stats) = freeze_features_with_stats(data, features.iter().map(|f| f.as_str()))?;
        Ok((frozen_data, stats.substitutions_applied))
    }

    /// Freeze features in a font file in-place.
    pub fn freeze_file(&self, path: &Path) -> Result<usize> {
        let data = read_font(path)?;
        let (frozen_data, subs) = self.freeze(&data)
            .with_context(|| format!("Failed to freeze features in {}", path.display()))?;
        write_font(path, frozen_data)?;

        info!(
            "{}: {} substitutions applied",
            path.file_name().unwrap_or_default().to_string_lossy(),
            subs
        );

        Ok(subs)
    }

    /// Freeze features in multiple files in parallel.
    pub fn freeze_files(&self, files: &[impl AsRef<Path> + Sync]) -> Result<FreezeStats> {
        let features = self.resolved_features();
        if features.is_empty() {
            info!("No features specified");
            return Ok(FreezeStats::default());
        }

        info!("Freezing features: {}", features.join(","));

        let results: Vec<_> = files
            .par_iter()
            .map(|path| self.freeze_file(path.as_ref()))
            .collect();

        let mut stats = FreezeStats::default();
        for result in results {
            match result {
                Ok(subs) => {
                    stats.succeeded += 1;
                    stats.total_substitutions += subs;
                }
                Err(e) => {
                    warn!("{e:?}");
                    stats.failed += 1;
                }
            }
        }

        info!(
            "Freeze complete: {} succeeded, {} failed, {} substitutions applied",
            stats.succeeded, stats.failed, stats.total_substitutions
        );

        Ok(stats)
    }
}

/// Statistics from a freeze operation.
#[derive(Debug, Default)]
pub struct FreezeStats {
    pub succeeded: usize,
    pub failed: usize,
    pub total_substitutions: usize,
}

/// Freeze features in multiple files (convenience function).
pub fn freeze_features(
    files: &[impl AsRef<Path> + Sync],
    features: &[impl AsRef<str> + Sync],
    auto_rvrn: AutoRvrn,
) -> Result<()> {
    Freezer::new()
        .with_features(features.iter().map(|f| f.as_ref()))
        .auto_rvrn(auto_rvrn)
        .freeze_files(files)?;
    Ok(())
}
