use std::{
    fs::{create_dir_all, read, write},
    path::Path,
};

use anyhow::{Context, Result};
use rayon::prelude::*;
use warpnine_font_merger::Merger;

pub fn merge_fonts(inputs: &[impl AsRef<Path>], output: &Path) -> Result<()> {
    println!("Merging {} fonts:", inputs.len());
    for input in inputs {
        println!("  - {}", input.as_ref().display());
    }

    let font_data: Vec<Vec<u8>> = inputs
        .iter()
        .map(|path| {
            read(path.as_ref())
                .with_context(|| format!("Failed to read {}", path.as_ref().display()))
        })
        .collect::<Result<Vec<_>>>()?;

    let font_refs: Vec<&[u8]> = font_data.iter().map(|v| v.as_slice()).collect();

    let merger = Merger::default();
    let merged_data = merger.merge(&font_refs).with_context(|| "Failed to merge fonts")?;

    if let Some(parent) = output.parent() {
        create_dir_all(parent)?;
    }

    write(output, &merged_data).with_context(|| format!("Failed to write {}", output.display()))?;

    let output_size = merged_data.len() as f64 / 1024.0 / 1024.0;
    println!("Merged font: {} ({output_size:.2} MB)", output.display());

    Ok(())
}

pub fn merge_batch(
    base_fonts: &[impl AsRef<Path> + Sync],
    fallback: &Path,
    output_dir: &Path,
) -> Result<()> {
    println!("Merging {} fonts with {}", base_fonts.len(), fallback.display());

    let fallback_data =
        read(fallback).with_context(|| format!("Failed to read {}", fallback.display()))?;

    create_dir_all(output_dir)?;

    base_fonts.par_iter().try_for_each(|base_path| -> Result<()> {
        let base_path = base_path.as_ref();
        let base_data =
            read(base_path).with_context(|| format!("Failed to read {}", base_path.display()))?;

        let merger = Merger::default();
        let merged_data = merger
            .merge(&[base_data.as_slice(), fallback_data.as_slice()])
            .with_context(|| format!("Failed to merge {}", base_path.display()))?;

        let output = output_dir.join(base_path.file_name().unwrap());
        write(&output, &merged_data)
            .with_context(|| format!("Failed to write {}", output.display()))?;

        println!("  Merged: {}", output.display());
        Ok(())
    })?;

    println!("Merged {} fonts", base_fonts.len());
    Ok(())
}
