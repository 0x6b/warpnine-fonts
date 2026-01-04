use std::path::Path;

use anyhow::Result;
use read_fonts::{FontRef, TableProvider};
use write_fonts::{FontBuilder, from_obj::ToOwnedTable, tables::os2::Os2};

use warpnine_font_ops::{apply_family_style_names, rewrite_font};

use crate::styles::{SANS_STYLES, Style, build_style_instances};

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

fn transform_sans(font_data: &[u8], style: &Style) -> Result<Vec<u8>> {
    let named_data = apply_family_style_names(font_data, "Warpnine Sans", style.name)?;
    update_weight_class(&named_data, style.weight.value() as u16)
}

pub fn create_sans(input: &Path, output_dir: &Path) -> Result<()> {
    let count =
        build_style_instances(input, output_dir, SANS_STYLES, "WarpnineSans-", transform_sans)?;
    println!("Created {} sans fonts in {}/", count, output_dir.display());
    Ok(())
}
