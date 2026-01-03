use std::{
    fs::{create_dir_all, read, write},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result};
use font_instancer::instantiate;
use rayon::prelude::*;
use read_fonts::{FontRef, TableProvider};
use write_fonts::{FontBuilder, from_obj::ToOwnedTable, tables::os2::Os2};

use crate::{
    font_ops::{apply_family_style_names, rewrite_font},
    styles::SANS_STYLES,
};

fn update_weight_class(font_data: &[u8], weight: u16) -> Result<Vec<u8>> {
    rewrite_font(font_data, |font: &FontRef, builder: &mut FontBuilder| {
        if let Ok(os2) = font.os2() {
            let mut new_os2: Os2 = os2.to_owned_table();
            new_os2.us_weight_class = weight;
            builder.add_table(&new_os2)?;
        }
        Ok(())
    })
}

pub fn create_sans(input: &Path, output_dir: &Path) -> Result<()> {
    let data = read(input).context("Failed to read input font")?;
    create_dir_all(output_dir)?;

    let success = AtomicUsize::new(0);

    SANS_STYLES.par_iter().try_for_each(|style| -> Result<()> {
        let output = output_dir.join(format!("WarpnineSans-{}.ttf", style.name));
        println!("Creating {}", style.name);

        let locations = style.axis_locations(0.0, 0.0);

        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", style.name))?;

        let named_data = apply_family_style_names(&static_data, "Warpnine Sans", style.name)?;
        let final_data = update_weight_class(&named_data, style.weight.0 as u16)?;

        write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success.fetch_add(1, Ordering::Relaxed);
        Ok(())
    })?;

    println!("Created {} sans fonts in {}/", success.load(Ordering::Relaxed), output_dir.display());
    Ok(())
}
