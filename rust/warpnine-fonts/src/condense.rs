use anyhow::{Context, Result};
use font_instancer::{instantiate, AxisLocation};
use read_fonts::{FontRef, TableProvider};
use std::fs;
use std::path::Path;
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::{
        head::Head,
        hhea::Hhea,
        hmtx::{Hmtx, LongMetric},
        name::{Name, NameRecord},
        os2::Os2,
    },
    FontBuilder,
};

use crate::sans::SANS_INSTANCES;

const WIDTH_CLASS_CONDENSED: u16 = 3;

fn update_condensed_name_table(font_data: &[u8], family: &str, style: &str) -> Result<Vec<u8>> {
    let font = FontRef::new(font_data)?;
    let name = font.name()?;

    let postscript_family = family.replace(' ', "");

    let mut new_records: Vec<NameRecord> = Vec::new();

    for record in name.name_record() {
        let name_id = record.name_id().to_u16();
        let platform_id = record.platform_id();
        let encoding_id = record.encoding_id();
        let language_id = record.language_id();

        let current_string = match record.string(name.string_data()) {
            Ok(s) => s.chars().collect::<String>(),
            Err(_) => continue,
        };

        let new_string = match name_id {
            1 => format!("{} {}", family, style),
            4 => format!("{} {}", family, style),
            6 => format!("{}-{}", postscript_family, style),
            16 => family.to_string(),
            17 => style.to_string(),
            _ => current_string,
        };

        new_records.push(NameRecord::new(
            platform_id,
            encoding_id,
            language_id,
            read_fonts::types::NameId::new(name_id),
            new_string.into(),
        ));
    }

    let new_name = Name::new(new_records);

    let mut builder = FontBuilder::new();
    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }
    builder.add_table(&new_name)?;

    Ok(builder.build())
}

fn apply_horizontal_scale(font_data: &[u8], scale_x: f32) -> Result<Vec<u8>> {
    let font = FontRef::new(font_data)?;

    let mut builder = FontBuilder::new();

    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if tag == read_fonts::types::Tag::new(b"hmtx")
            || tag == read_fonts::types::Tag::new(b"head")
            || tag == read_fonts::types::Tag::new(b"hhea")
            || tag == read_fonts::types::Tag::new(b"OS/2")
        {
            continue;
        }
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }

    if let Ok(hmtx) = font.hmtx() {
        let num_glyphs = font.maxp().map(|m| m.num_glyphs()).unwrap_or(0) as usize;
        let num_long_metrics = font.hhea().map(|h| h.number_of_h_metrics()).unwrap_or(0) as usize;

        let mut new_h_metrics = Vec::with_capacity(num_long_metrics);
        let mut new_lsbs = Vec::new();

        for gid in 0..num_glyphs {
            let glyph_id = read_fonts::types::GlyphId::new(gid as u32);
            let advance = hmtx.advance(glyph_id).unwrap_or(0);
            let lsb = hmtx.side_bearing(glyph_id).unwrap_or(0);

            if gid < num_long_metrics {
                new_h_metrics.push(LongMetric {
                    advance: (advance as f32 * scale_x).round() as u16,
                    side_bearing: (lsb as f32 * scale_x).round() as i16,
                });
            } else {
                new_lsbs.push((lsb as f32 * scale_x).round() as i16);
            }
        }

        let new_hmtx = Hmtx::new(new_h_metrics, new_lsbs);
        builder.add_table(&new_hmtx)?;
    }

    if let Ok(head) = font.head() {
        let mut new_head: Head = head.to_owned_table();
        new_head.x_min = (new_head.x_min as f32 * scale_x).round() as i16;
        new_head.x_max = (new_head.x_max as f32 * scale_x).round() as i16;
        builder.add_table(&new_head)?;
    }

    if let Ok(hhea) = font.hhea() {
        let mut new_hhea: Hhea = hhea.to_owned_table();
        let adv_max = new_hhea.advance_width_max.to_u16();
        let min_lsb = new_hhea.min_left_side_bearing.to_i16();
        let min_rsb = new_hhea.min_right_side_bearing.to_i16();
        let x_max = new_hhea.x_max_extent.to_i16();

        new_hhea.advance_width_max = ((adv_max as f32 * scale_x).round() as u16).into();
        new_hhea.min_left_side_bearing = ((min_lsb as f32 * scale_x).round() as i16).into();
        new_hhea.min_right_side_bearing = ((min_rsb as f32 * scale_x).round() as i16).into();
        new_hhea.x_max_extent = ((x_max as f32 * scale_x).round() as i16).into();
        builder.add_table(&new_hhea)?;
    }

    if let Ok(os2) = font.os2() {
        let mut new_os2: Os2 = os2.to_owned_table();
        new_os2.x_avg_char_width = (new_os2.x_avg_char_width as f32 * scale_x).round() as i16;
        new_os2.us_width_class = WIDTH_CLASS_CONDENSED;
        builder.add_table(&new_os2)?;
    }

    Ok(builder.build())
}

pub fn create_condensed(input: &Path, output_dir: &Path, scale: f32) -> Result<()> {
    let data = fs::read(input).context("Failed to read input font")?;
    fs::create_dir_all(output_dir)?;

    let mut success = 0;

    for instance in SANS_INSTANCES {
        let output = output_dir.join(format!("WarpnineSansCondensed-{}.ttf", instance.style));
        println!(
            "Creating {} condensed ({:.0}%)",
            instance.style,
            scale * 100.0
        );

        let locations = vec![
            AxisLocation::new("MONO", instance.mono()),
            AxisLocation::new("CASL", instance.casl()),
            AxisLocation::new("wght", instance.wght),
            AxisLocation::new("slnt", instance.slnt()),
            AxisLocation::new("CRSV", instance.crsv()),
        ];

        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", instance.style))?;

        let scaled_data = apply_horizontal_scale(&static_data, scale)?;

        let final_data =
            update_condensed_name_table(&scaled_data, "Warpnine Sans Condensed", instance.style)?;

        fs::write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success += 1;
    }

    println!(
        "Created {} condensed fonts in {}/",
        success,
        output_dir.display()
    );
    Ok(())
}
