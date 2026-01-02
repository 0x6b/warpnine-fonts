use std::{
    fs::{create_dir_all, read, write},
    path::Path,
};

use anyhow::{Context, Result};
use font_instancer::{AxisLocation, instantiate};
use read_fonts::TableProvider;
use write_fonts::{from_obj::ToOwnedTable, tables::os2::Os2};

use crate::{
    font_ops::{map_name_records, rewrite_font},
    styles::{SANS_STYLES, Style},
};

fn update_sans_metadata(font_data: &[u8], family: &str, style: &Style) -> Result<Vec<u8>> {
    let postscript_family = family.replace(' ', "");
    let style_name = style.name;
    let weight = style.wght as u16;

    rewrite_font(font_data, |font, builder| {
        let new_name = map_name_records(font, |name_id, current| match name_id {
            1 => Some(format!("{family} {style_name}")),
            4 => Some(format!("{family} {style_name}")),
            6 => Some(format!("{postscript_family}-{style_name}")),
            16 => Some(family.to_string()),
            17 => Some(style_name.to_string()),
            _ => Some(current.to_string()),
        })?;
        builder.add_table(&new_name)?;

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

    let mut success = 0;

    for style in SANS_STYLES {
        let output = output_dir.join(format!("WarpnineSans-{}.ttf", style.name));
        println!("Creating {}", style.name);

        let locations = vec![
            AxisLocation::new("MONO", 0.0),
            AxisLocation::new("CASL", 0.0),
            AxisLocation::new("wght", style.wght),
            AxisLocation::new("slnt", style.slnt()),
            AxisLocation::new("CRSV", style.crsv()),
        ];

        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", style.name))?;

        let final_data = update_sans_metadata(&static_data, "Warpnine Sans", style)?;

        write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success += 1;
    }

    println!("Created {success} sans fonts in {}/", output_dir.display());
    Ok(())
}
