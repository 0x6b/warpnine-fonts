use anyhow::{Context, Result};
use font_merger::Merger;
use std::fs;
use std::path::Path;

pub fn merge_fonts(inputs: &[impl AsRef<Path>], output: &Path) -> Result<()> {
    println!("Merging {} fonts:", inputs.len());
    for input in inputs {
        println!("  - {}", input.as_ref().display());
    }

    let font_data: Vec<Vec<u8>> = inputs
        .iter()
        .map(|path| {
            fs::read(path.as_ref())
                .with_context(|| format!("Failed to read {}", path.as_ref().display()))
        })
        .collect::<Result<Vec<_>>>()?;

    let font_refs: Vec<&[u8]> = font_data.iter().map(|v| v.as_slice()).collect();

    let merger = Merger::default();
    let merged_data = merger
        .merge(&font_refs)
        .with_context(|| "Failed to merge fonts")?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output, &merged_data)
        .with_context(|| format!("Failed to write {}", output.display()))?;

    let output_size = merged_data.len() as f64 / 1024.0 / 1024.0;
    println!("Merged font: {} ({:.2} MB)", output.display(), output_size);

    Ok(())
}
