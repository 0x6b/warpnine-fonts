use anyhow::{Context, Result};
use font_instancer::{AxisLocation, instantiate};
use read_fonts::tables::glyf::CurvePoint;
use read_fonts::types::GlyphId;
use read_fonts::{FontRef, TableProvider};
use std::path::Path;
use write_fonts::{
    FontBuilder,
    from_obj::ToOwnedTable,
    tables::{
        glyf::{
            Anchor, Bbox, Component, ComponentFlags, CompositeGlyph, Contour, GlyfLocaBuilder,
            Glyph, SimpleGlyph, Transform,
        },
        head::Head,
        hhea::Hhea,
        hmtx::{Hmtx, LongMetric},
        name::{Name, NameRecord},
        os2::Os2,
    },
};

use crate::sans::SANS_INSTANCES;
use read_fonts::tables;
use std::fs::create_dir_all;
use std::fs::read;
use std::fs::write;

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
            1 => format!("{family} {style}"),
            4 => format!("{family} {style}"),
            6 => format!("{postscript_family}-{style}"),
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

fn scale_simple_glyph(glyph: &read_fonts::tables::glyf::SimpleGlyph, scale_x: f32) -> SimpleGlyph {
    let mut contours = Vec::new();
    let end_pts: Vec<u16> = glyph
        .end_pts_of_contours()
        .iter()
        .map(|e| e.get())
        .collect();
    let all_points: Vec<CurvePoint> = glyph.points().collect();

    let mut start = 0usize;
    for end in end_pts {
        let end_idx = end as usize + 1;
        let scaled_points: Vec<CurvePoint> = all_points[start..end_idx]
            .iter()
            .map(|p| CurvePoint::new((p.x as f32 * scale_x).round() as i16, p.y, p.on_curve))
            .collect();
        contours.push(Contour::from(scaled_points));
        start = end_idx;
    }

    let bbox = Bbox {
        x_min: (glyph.x_min() as f32 * scale_x).round() as i16,
        y_min: glyph.y_min(),
        x_max: (glyph.x_max() as f32 * scale_x).round() as i16,
        y_max: glyph.y_max(),
    };

    SimpleGlyph {
        bbox,
        contours,
        instructions: glyph.instructions().to_vec(),
    }
}

fn scale_composite_glyph(glyph: &tables::glyf::CompositeGlyph, scale_x: f32) -> CompositeGlyph {
    let mut components = Vec::new();

    for c in glyph.components() {
        let new_anchor = match c.anchor {
            Anchor::Offset { x, y } => Anchor::Offset {
                x: (x as f32 * scale_x).round() as i16,
                y,
            },
            Anchor::Point { base, component } => Anchor::Point { base, component },
        };

        let new_transform = Transform {
            xx: c.transform.xx,
            yx: c.transform.yx,
            xy: c.transform.xy,
            yy: c.transform.yy,
        };

        components.push(Component {
            glyph: c.glyph,
            anchor: new_anchor,
            flags: ComponentFlags {
                round_xy_to_grid: c
                    .flags
                    .contains(read_fonts::tables::glyf::CompositeGlyphFlags::ROUND_XY_TO_GRID),
                use_my_metrics: c
                    .flags
                    .contains(read_fonts::tables::glyf::CompositeGlyphFlags::USE_MY_METRICS),
                scaled_component_offset: c.flags.contains(
                    read_fonts::tables::glyf::CompositeGlyphFlags::SCALED_COMPONENT_OFFSET,
                ),
                unscaled_component_offset: c.flags.contains(
                    read_fonts::tables::glyf::CompositeGlyphFlags::UNSCALED_COMPONENT_OFFSET,
                ),
                overlap_compound: c
                    .flags
                    .contains(read_fonts::tables::glyf::CompositeGlyphFlags::OVERLAP_COMPOUND),
            },
            transform: new_transform,
        });
    }

    let bbox = Bbox {
        x_min: (glyph.x_min() as f32 * scale_x).round() as i16,
        y_min: glyph.y_min(),
        x_max: (glyph.x_max() as f32 * scale_x).round() as i16,
        y_max: glyph.y_max(),
    };

    let first = components.remove(0);
    let mut composite = CompositeGlyph::new(first, bbox);
    for c in components {
        composite.add_component(c, bbox);
    }
    composite
}

fn apply_horizontal_scale(font_data: &[u8], scale_x: f32, weight_class: u16) -> Result<Vec<u8>> {
    let font = FontRef::new(font_data)?;

    let mut builder = FontBuilder::new();

    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if tag == read_fonts::types::Tag::new(b"glyf")
            || tag == read_fonts::types::Tag::new(b"loca")
            || tag == read_fonts::types::Tag::new(b"hmtx")
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

    if let Ok(glyf) = font.glyf() {
        if let Ok(loca) = font.loca(None) {
            let num_glyphs = loca.len();
            let mut glyf_builder = GlyfLocaBuilder::new();

            for gid in 0..num_glyphs {
                let glyph_data = loca.get_glyf(GlyphId::new(gid as u32), &glyf);
                let glyph = match glyph_data {
                    Ok(Some(read_fonts::tables::glyf::Glyph::Simple(simple))) => {
                        Glyph::Simple(scale_simple_glyph(&simple, scale_x))
                    }
                    Ok(Some(read_fonts::tables::glyf::Glyph::Composite(composite))) => {
                        Glyph::Composite(scale_composite_glyph(&composite, scale_x))
                    }
                    _ => Glyph::Empty,
                };
                glyf_builder.add_glyph(&glyph)?;
            }

            let (new_glyf, new_loca, loca_format) = glyf_builder.build();
            builder.add_table(&new_glyf)?;
            builder.add_table(&new_loca)?;

            if let Ok(head) = font.head() {
                let mut new_head: Head = head.to_owned_table();
                new_head.x_min = (new_head.x_min as f32 * scale_x).round() as i16;
                new_head.x_max = (new_head.x_max as f32 * scale_x).round() as i16;
                new_head.index_to_loc_format = loca_format as i16;
                builder.add_table(&new_head)?;
            }
        }
    } else if let Ok(head) = font.head() {
        let mut new_head: Head = head.to_owned_table();
        new_head.x_min = (new_head.x_min as f32 * scale_x).round() as i16;
        new_head.x_max = (new_head.x_max as f32 * scale_x).round() as i16;
        builder.add_table(&new_head)?;
    }

    if let Ok(hmtx) = font.hmtx() {
        let num_glyphs = font.maxp().map(|m| m.num_glyphs()).unwrap_or(0) as usize;
        let num_long_metrics = font.hhea().map(|h| h.number_of_h_metrics()).unwrap_or(0) as usize;

        let mut new_h_metrics = Vec::with_capacity(num_long_metrics);
        let mut new_lsbs = Vec::new();

        for gid in 0..num_glyphs {
            let glyph_id = GlyphId::new(gid as u32);
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
        new_os2.us_weight_class = weight_class;
        builder.add_table(&new_os2)?;
    }

    Ok(builder.build())
}

pub fn create_condensed(input: &Path, output_dir: &Path, scale: f32) -> Result<()> {
    let data = read(input).context("Failed to read input font")?;
    create_dir_all(output_dir)?;

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

        let scaled_data = apply_horizontal_scale(&static_data, scale, instance.wght as u16)?;

        let final_data =
            update_condensed_name_table(&scaled_data, "Warpnine Sans Condensed", instance.style)?;

        write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success += 1;
    }

    println!(
        "Created {success} condensed fonts in {}/",
        output_dir.display()
    );
    Ok(())
}
