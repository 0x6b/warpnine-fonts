//! Variable font builder implementation.

use crate::designspace::DesignSpace;
use crate::error::{Error, Result};
use crate::variation_model::VariationModel;

use log::info;
use read_fonts::types::{F2Dot14, Fixed, GlyphId, NameId, Tag};
use read_fonts::{FontData, FontRef, TableProvider};
use std::collections::HashSet;
use write_fonts::from_obj::FromObjRef;
use write_fonts::tables::fvar::{AxisInstanceArrays, Fvar, InstanceRecord, VariationAxisRecord};
use write_fonts::tables::glyf::{CompositeGlyph as WriteCompositeGlyph, GlyfLocaBuilder, Glyph as WriteGlyph, SimpleGlyph as WriteSimpleGlyph};
use write_fonts::tables::gvar::{iup::iup_delta_optimize, GlyphDelta, GlyphDeltas, GlyphVariations, Gvar, Tent};
use write_fonts::tables::head::Head;
use write_fonts::FontBuilder;

/// Tables that should NOT be copied (variation-specific or rebuilt).
const SKIP_TABLES: &[Tag] = &[
    Tag::new(b"glyf"),
    Tag::new(b"loca"),
    Tag::new(b"head"),
    Tag::new(b"fvar"),
    Tag::new(b"gvar"),
    Tag::new(b"STAT"),
    Tag::new(b"DSIG"),
];

/// Build a variable font from a designspace.
///
/// This function:
/// 1. Loads all master fonts
/// 2. Verifies glyph compatibility across masters
/// 3. Computes glyph deltas using the variation model
/// 4. Builds fvar, gvar, and other required tables
/// 5. Copies other tables from the default master
pub fn build_variable_font(designspace: &DesignSpace) -> Result<Vec<u8>> {
    designspace
        .validate()
        .map_err(Error::InvalidDesignspace)?;

    info!(
        "Building variable font from {} masters",
        designspace.sources.len()
    );

    // Load all master fonts
    let master_data: Vec<Vec<u8>> = designspace
        .sources
        .iter()
        .map(|source| {
            std::fs::read(&source.path).map_err(|e| Error::ReadFont {
                path: source.path.clone(),
                source: e,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let masters: Vec<FontRef> = master_data
        .iter()
        .zip(designspace.sources.iter())
        .map(|(data, source)| {
            FontRef::new(data).map_err(|e| Error::ParseFont {
                path: source.path.clone(),
                message: e.to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let default_idx = designspace
        .default_source_index()
        .ok_or(Error::NoDefaultSource)?;
    let default_font = &masters[default_idx];

    // Verify glyph compatibility
    verify_glyph_compatibility(designspace, &masters)?;

    // Build variation model
    let model = VariationModel::new(designspace).ok_or(Error::NoDefaultSource)?;

    info!("Variation model: {} regions", model.regions.len());

    // Ensure we have glyf table
    let _ = default_font.glyf().map_err(|_| Error::MissingTable {
        path: designspace.sources[default_idx].path.clone(),
        table: "glyf".to_string(),
    })?;

    let num_glyphs = default_font.maxp()?.num_glyphs();
    info!("Processing {} glyphs", num_glyphs);

    // Build gvar table
    let gvar = build_gvar(designspace, &masters, &model, num_glyphs)?;
    info!("Built gvar table");

    // Build glyf/loca tables (copy from default)
    let (new_glyf, new_loca, loca_format) = build_glyf_loca(default_font)?;

    // Build fvar table
    let fvar = build_fvar(designspace)?;
    info!("Built fvar table with {} axes", designspace.axes.len());

    // Build head table
    let head = build_head(default_font, loca_format)?;

    // Assemble the font
    let mut builder = FontBuilder::new();

    builder.add_table(&fvar)?;
    builder.add_table(&gvar)?;
    builder.add_table(&new_glyf)?;
    builder.add_table(&new_loca)?;
    builder.add_table(&head)?;

    // Copy tables from default master
    let skip_set: HashSet<Tag> = SKIP_TABLES.iter().copied().collect();
    for record in default_font.table_directory.table_records() {
        let tag = record.tag();
        if !skip_set.contains(&tag) {
            if let Some(data) = default_font.table_data(tag) {
                builder.add_raw(tag, data);
            }
        }
    }

    Ok(builder.build())
}

fn verify_glyph_compatibility(designspace: &DesignSpace, masters: &[FontRef]) -> Result<()> {
    let default_idx = designspace.default_source_index().unwrap();
    let default_font = &masters[default_idx];
    let expected_glyphs = default_font.maxp()?.num_glyphs();

    for (idx, master) in masters.iter().enumerate() {
        if idx == default_idx {
            continue;
        }

        let actual_glyphs = master.maxp()?.num_glyphs();
        if actual_glyphs != expected_glyphs {
            return Err(Error::GlyphCountMismatch {
                path: designspace.sources[idx].path.clone(),
                expected: expected_glyphs,
                actual: actual_glyphs,
            });
        }
    }

    Ok(())
}

fn build_fvar(designspace: &DesignSpace) -> Result<Fvar> {
    let axes: Vec<VariationAxisRecord> = designspace
        .axes
        .iter()
        .enumerate()
        .map(|(idx, axis)| {
            let mut tag_bytes = [b' '; 4];
            for (i, b) in axis.tag.bytes().take(4).enumerate() {
                tag_bytes[i] = b;
            }

            VariationAxisRecord {
                axis_tag: Tag::new(&tag_bytes),
                min_value: Fixed::from_f64(f64::from(axis.minimum)),
                default_value: Fixed::from_f64(f64::from(axis.default)),
                max_value: Fixed::from_f64(f64::from(axis.maximum)),
                flags: 0u16,
                axis_name_id: NameId::new(256 + idx as u16),
            }
        })
        .collect();

    let instances: Vec<InstanceRecord> = designspace
        .instances
        .iter()
        .enumerate()
        .map(|(idx, instance)| {
            let coordinates: Vec<Fixed> = designspace
                .axes
                .iter()
                .map(|axis| Fixed::from_f64(f64::from(instance.axis_value(axis))))
                .collect();

            InstanceRecord {
                subfamily_name_id: NameId::new(258 + idx as u16),
                flags: 0,
                coordinates,
                post_script_name_id: None,
            }
        })
        .collect();

    Ok(Fvar {
        axis_instance_arrays: AxisInstanceArrays { axes, instances }.into(),
    })
}

fn build_gvar(
    designspace: &DesignSpace,
    masters: &[FontRef],
    model: &VariationModel,
    num_glyphs: u16,
) -> Result<Gvar> {
    // Load glyf/loca for all masters
    let master_glyfs: Vec<_> = masters
        .iter()
        .map(|m| m.glyf())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let master_locas: Vec<_> = masters
        .iter()
        .map(|m| m.loca(None))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let axis_count = designspace.axes.len() as u16;
    let mut all_variations: Vec<GlyphVariations> = Vec::with_capacity(num_glyphs as usize);

    for glyph_idx in 0..num_glyphs {
        let gid = GlyphId::new(glyph_idx as u32);

        let variations =
            build_glyph_variations(gid, designspace, &master_glyfs, &master_locas, model)?;

        all_variations.push(variations);
    }

    Gvar::new(all_variations, axis_count).map_err(Error::GvarBuild)
}

fn build_glyph_variations(
    gid: GlyphId,
    designspace: &DesignSpace,
    master_glyfs: &[read_fonts::tables::glyf::Glyf],
    master_locas: &[read_fonts::tables::loca::Loca],
    model: &VariationModel,
) -> Result<GlyphVariations> {
    use read_fonts::tables::glyf::Glyph;

    let default_idx = model.default_idx;

    // Get the default glyph
    let default_glyph = master_locas[default_idx]
        .get_glyf(gid, &master_glyfs[default_idx])
        .ok()
        .flatten();

    let Some(default_glyph) = default_glyph else {
        // Empty glyph - no variations needed
        return Ok(GlyphVariations::new(gid, vec![]));
    };

    match default_glyph {
        Glyph::Simple(simple) => build_simple_glyph_variations(
            gid,
            &simple,
            designspace,
            master_glyfs,
            master_locas,
            model,
        ),
        Glyph::Composite(composite) => build_composite_glyph_variations(
            gid,
            &composite,
            designspace,
            master_glyfs,
            master_locas,
            model,
        ),
    }
}

fn build_simple_glyph_variations(
    gid: GlyphId,
    default_simple: &read_fonts::tables::glyf::SimpleGlyph,
    designspace: &DesignSpace,
    master_glyfs: &[read_fonts::tables::glyf::Glyf],
    master_locas: &[read_fonts::tables::loca::Loca],
    model: &VariationModel,
) -> Result<GlyphVariations> {
    use read_fonts::tables::glyf::Glyph;

    let num_points = default_simple.num_points();

    // Collect points from all masters
    let mut master_points: Vec<Vec<(i16, i16)>> = Vec::with_capacity(designspace.sources.len());

    for (master_idx, (glyf, loca)) in master_glyfs.iter().zip(master_locas.iter()).enumerate() {
        let glyph = loca.get_glyf(gid, glyf).ok().flatten();

        let points: Vec<(i16, i16)> = match glyph {
            Some(Glyph::Simple(simple)) => {
                if simple.num_points() != num_points {
                    return Err(Error::PointCountMismatch {
                        path: designspace.sources[master_idx].path.clone(),
                        glyph_id: gid.to_u32(),
                        expected: num_points,
                        actual: simple.num_points(),
                    });
                }
                simple.points().map(|p| (p.x, p.y)).collect()
            }
            _ => {
                // If a master has no glyph or a different type, use default points
                default_simple.points().map(|p| (p.x, p.y)).collect()
            }
        };

        master_points.push(points);
    }

    // Get default master coordinates for IUP optimization
    let default_coords: Vec<kurbo::Point> = default_simple
        .points()
        .map(|p| kurbo::Point::new(f64::from(p.x), f64::from(p.y)))
        .collect();

    // Get contour end points for IUP
    let contour_ends: Vec<usize> = default_simple
        .end_pts_of_contours()
        .iter()
        .map(|v| v.get() as usize)
        .collect();

    // Build GlyphDeltas for each region
    let mut glyph_deltas: Vec<GlyphDeltas> = Vec::with_capacity(model.regions.len());

    for region_idx in 0..model.regions.len() {
        let peak = model.region_peak(region_idx);
        let tents: Vec<Tent> = peak
            .iter()
            .map(|&v| Tent::new(F2Dot14::from_f32(v), None))
            .collect();

        // Compute raw deltas for all points
        let mut raw_deltas: Vec<kurbo::Vec2> = Vec::with_capacity(num_points);

        for point_idx in 0..num_points {
            let point_values: Vec<(i16, i16)> =
                master_points.iter().map(|points| points[point_idx]).collect();

            let (_, point_deltas) = model.compute_deltas_2d(&point_values);
            let delta = point_deltas[region_idx];

            raw_deltas.push(kurbo::Vec2::new(f64::from(delta.0), f64::from(delta.1)));
        }

        // Add 4 phantom point deltas (set to zero for now)
        // Phantom points: LSB origin, advance width, top origin, advance height
        for _ in 0..4 {
            raw_deltas.push(kurbo::Vec2::ZERO);
        }

        // Extend coordinates with phantom points
        let mut coords_with_phantom = default_coords.clone();
        for _ in 0..4 {
            coords_with_phantom.push(kurbo::Point::ZERO);
        }

        // Apply IUP optimization with tolerance of 0.5 (half a unit)
        let deltas = match iup_delta_optimize(raw_deltas.clone(), coords_with_phantom, 0.5, &contour_ends) {
            Ok(optimized) => {
                // Remove phantom point deltas from result
                optimized.into_iter().take(num_points).collect()
            }
            Err(_) => {
                // Fall back to marking all as required
                raw_deltas
                    .into_iter()
                    .take(num_points)
                    .map(|d| GlyphDelta::required(d.x as i16, d.y as i16))
                    .collect()
            }
        };

        glyph_deltas.push(GlyphDeltas::new(tents, deltas));
    }

    Ok(GlyphVariations::new(gid, glyph_deltas))
}

fn build_composite_glyph_variations(
    gid: GlyphId,
    default_composite: &read_fonts::tables::glyf::CompositeGlyph,
    designspace: &DesignSpace,
    master_glyfs: &[read_fonts::tables::glyf::Glyf],
    master_locas: &[read_fonts::tables::loca::Loca],
    model: &VariationModel,
) -> Result<GlyphVariations> {
    use read_fonts::tables::glyf::Glyph;

    // Count components with offsets that can vary
    let num_components = default_composite.components().count();

    // Collect component offsets from all masters
    let mut master_offsets: Vec<Vec<(i16, i16)>> = Vec::with_capacity(designspace.sources.len());

    for (_master_idx, (glyf, loca)) in master_glyfs.iter().zip(master_locas.iter()).enumerate() {
        let glyph = loca.get_glyf(gid, glyf).ok().flatten();

        let offsets: Vec<(i16, i16)> = match glyph {
            Some(Glyph::Composite(composite)) => composite
                .components()
                .map(|c| {
                    let anchor = c.anchor;
                    match anchor {
                        read_fonts::tables::glyf::Anchor::Offset { x, y } => (x, y),
                        _ => (0, 0),
                    }
                })
                .collect(),
            _ => {
                // Use default offsets if master doesn't have this glyph
                default_composite
                    .components()
                    .map(|c| match c.anchor {
                        read_fonts::tables::glyf::Anchor::Offset { x, y } => (x, y),
                        _ => (0, 0),
                    })
                    .collect()
            }
        };

        master_offsets.push(offsets);
    }

    // Build GlyphDeltas for each region
    let mut glyph_deltas: Vec<GlyphDeltas> = Vec::with_capacity(model.regions.len());

    for region_idx in 0..model.regions.len() {
        let peak = model.region_peak(region_idx);
        let tents: Vec<Tent> = peak
            .iter()
            .map(|&v| Tent::new(F2Dot14::from_f32(v), None))
            .collect();

        let mut deltas: Vec<GlyphDelta> = Vec::with_capacity(num_components);

        for comp_idx in 0..num_components {
            let offset_values: Vec<(i16, i16)> = master_offsets
                .iter()
                .map(|offsets| offsets.get(comp_idx).copied().unwrap_or((0, 0)))
                .collect();

            let (_, offset_deltas) = model.compute_deltas_2d(&offset_values);
            let delta = offset_deltas[region_idx];

            deltas.push(GlyphDelta::required(delta.0, delta.1));
        }

        glyph_deltas.push(GlyphDeltas::new(tents, deltas));
    }

    Ok(GlyphVariations::new(gid, glyph_deltas))
}

fn build_glyf_loca(
    default_font: &FontRef,
) -> Result<(
    write_fonts::tables::glyf::Glyf,
    write_fonts::tables::loca::Loca,
    write_fonts::tables::loca::LocaFormat,
)> {
    use read_fonts::tables::glyf::Glyph;

    let glyf = default_font.glyf()?;
    let loca = default_font.loca(None)?;
    let num_glyphs = default_font.maxp()?.num_glyphs();

    let mut builder = GlyfLocaBuilder::new();

    for glyph_idx in 0..num_glyphs {
        let gid = GlyphId::new(glyph_idx as u32);

        let glyph = loca.get_glyf(gid, &glyf).ok().flatten();

        let write_glyph: WriteGlyph = match glyph {
            None => WriteGlyph::Empty,
            Some(Glyph::Simple(simple)) => {
                let write_simple =
                    WriteSimpleGlyph::from_obj_ref(&simple, FontData::new(&[]));
                WriteGlyph::Simple(write_simple)
            }
            Some(Glyph::Composite(composite)) => {
                let write_composite =
                    WriteCompositeGlyph::from_obj_ref(&composite, FontData::new(&[]));
                WriteGlyph::Composite(write_composite)
            }
        };

        builder.add_glyph(&write_glyph)?;
    }

    Ok(builder.build())
}

fn build_head(
    default_font: &FontRef,
    loca_format: write_fonts::tables::loca::LocaFormat,
) -> Result<Head> {
    let head = default_font.head()?;

    Ok(Head::new(
        head.font_revision(),
        head.checksum_adjustment(),
        head.flags(),
        head.units_per_em(),
        head.created(),
        head.modified(),
        head.x_min(),
        head.y_min(),
        head.x_max(),
        head.y_max(),
        head.mac_style(),
        head.lowest_rec_ppem(),
        match loca_format {
            write_fonts::tables::loca::LocaFormat::Short => 0,
            write_fonts::tables::loca::LocaFormat::Long => 1,
        },
    ))
}
