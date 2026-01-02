use anyhow::{Context, Result};
use font_feature_freezer::freeze_features_with_stats;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

pub fn freeze_features(files: &[PathBuf], features: &[String], auto_rvrn: bool) -> Result<()> {
    if features.is_empty() {
        println!("No features specified");
        return Ok(());
    }

    let features: Vec<String> = if auto_rvrn && !features.iter().any(|f| f == "rvrn") {
        let mut with_rvrn = vec!["rvrn".to_string()];
        with_rvrn.extend(features.iter().cloned());
        with_rvrn
    } else {
        features.to_vec()
    };

    let feature_list = features.join(",");
    println!("Freezing features: {feature_list}");

    let results: Vec<_> = files
        .par_iter()
        .map(|path| freeze_single(path, &features))
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
                eprintln!("{e:?}");
                failed += 1;
            }
        }
    }

    println!(
        "Freeze complete: {success} succeeded, {failed} failed, {total_substitutions} substitutions applied"
    );
    Ok(())
}

fn freeze_single(path: &PathBuf, features: &[String]) -> Result<usize> {
    let data = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let (frozen_data, stats) =
        freeze_features_with_stats(&data, features.iter().map(|s| s.as_str()))
            .with_context(|| format!("Failed to freeze features in {}", path.display()))?;

    fs::write(path, frozen_data).with_context(|| format!("Failed to write {}", path.display()))?;

    println!(
        "{}: {} substitutions applied",
        path.file_name().unwrap_or_default().to_string_lossy(),
        stats.substitutions_applied
    );

    Ok(stats.substitutions_applied)
}
