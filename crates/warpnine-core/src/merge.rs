//! Font merging operations.

use std::{fs::create_dir_all, path::Path};

use anyhow::{Context, Result};
use log::info;
use rayon::prelude::*;
use warpnine_font_merger::Merger;

use crate::io::{read_font, write_font};

#[derive(Default)]
pub struct FontMerger {
    fonts: Vec<Vec<u8>>,
}

impl FontMerger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_font(&mut self, data: Vec<u8>) -> &mut Self {
        self.fonts.push(data);
        self
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>) -> Result<&mut Self> {
        let data = read_font(path.as_ref())?;
        self.fonts.push(data);
        Ok(self)
    }

    pub fn merge(&self) -> Result<Vec<u8>> {
        let font_refs: Vec<&[u8]> = self.fonts.iter().map(|v| v.as_slice()).collect();
        let merger = Merger::default();
        merger.merge(&font_refs).context("Failed to merge fonts")
    }

    pub fn merge_to_file(&self, output: &Path) -> Result<()> {
        let merged_data = self.merge()?;

        if let Some(parent) = output.parent() {
            create_dir_all(parent)?;
        }

        write_font(output, &merged_data)?;

        let output_size = merged_data.len() as f64 / 1024.0 / 1024.0;
        info!("Merged font: {} ({output_size:.2} MB)", output.display());

        Ok(())
    }
}

pub struct BatchMerger<'a> {
    fallback_data: &'a [u8],
}

impl<'a> BatchMerger<'a> {
    pub fn new(fallback_data: &'a [u8]) -> Self {
        Self { fallback_data }
    }

    pub fn from_file(path: &Path) -> Result<OwnedBatchMerger> {
        let data = read_font(path)?;
        Ok(OwnedBatchMerger { fallback_data: data })
    }

    pub fn merge_with_fallback(&self, base_data: &[u8]) -> Result<Vec<u8>> {
        let merger = Merger::default();
        merger
            .merge(&[base_data, self.fallback_data])
            .context("Failed to merge fonts")
    }

    pub fn merge_batch(
        &self,
        base_fonts: &[impl AsRef<Path> + Sync],
        output_dir: &Path,
    ) -> Result<()> {
        info!("Merging {} fonts with fallback", base_fonts.len());

        create_dir_all(output_dir)?;

        base_fonts.par_iter().try_for_each(|base_path| -> Result<()> {
            let base_path = base_path.as_ref();
            let base_data = read_font(base_path)?;
            let merged_data = self
                .merge_with_fallback(&base_data)
                .with_context(|| format!("Failed to merge {}", base_path.display()))?;

            let output = output_dir.join(base_path.file_name().unwrap());
            write_font(&output, &merged_data)?;

            info!("Merged: {}", output.display());
            Ok(())
        })?;

        info!("Merged {} fonts", base_fonts.len());
        Ok(())
    }
}

pub struct OwnedBatchMerger {
    fallback_data: Vec<u8>,
}

impl OwnedBatchMerger {
    pub fn as_batch_merger(&self) -> BatchMerger<'_> {
        BatchMerger::new(&self.fallback_data)
    }

    pub fn merge_batch(
        &self,
        base_fonts: &[impl AsRef<Path> + Sync],
        output_dir: &Path,
    ) -> Result<()> {
        self.as_batch_merger().merge_batch(base_fonts, output_dir)
    }
}

pub fn merge_fonts(inputs: &[impl AsRef<Path>], output: &Path) -> Result<()> {
    info!("Merging {} fonts:", inputs.len());
    for input in inputs {
        info!("  - {}", input.as_ref().display());
    }

    let mut merger = FontMerger::new();
    for input in inputs {
        merger.add_file(input)?;
    }

    merger.merge_to_file(output)
}

pub fn merge_batch(
    base_fonts: &[impl AsRef<Path> + Sync],
    fallback: &Path,
    output_dir: &Path,
) -> Result<()> {
    info!("Merging {} fonts with {}", base_fonts.len(), fallback.display());
    let batch_merger = BatchMerger::from_file(fallback)?;
    batch_merger.merge_batch(base_fonts, output_dir)
}
