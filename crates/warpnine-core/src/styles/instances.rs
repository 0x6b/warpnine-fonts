//! Instance building helper.

use std::{
    fs::{create_dir_all, read, write},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result};
use font_instancer::instantiate;
use rayon::prelude::*;

use super::design::Style;

pub fn build_style_instances<F>(
    input: &Path,
    output_dir: &Path,
    styles: &[Style],
    output_prefix: &str,
    transform: F,
) -> Result<usize>
where
    F: Fn(&[u8], &Style) -> Result<Vec<u8>> + Sync,
{
    let data = read(input).context("Failed to read input font")?;
    create_dir_all(output_dir)?;

    let success = AtomicUsize::new(0);

    styles.par_iter().try_for_each(|style| -> Result<()> {
        let output = output_dir.join(format!("{output_prefix}{}.ttf", style.name));
        println!("Creating {}", style.name);

        let locations = style.axis_locations(0.0, 0.0);
        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", style.name))?;

        let final_data = transform(&static_data, style)?;

        write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success.fetch_add(1, Ordering::Relaxed);
        Ok(())
    })?;

    Ok(success.load(Ordering::Relaxed))
}
