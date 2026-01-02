//! Variable font builder implementation.

use crate::designspace::DesignSpace;
use crate::error::{Error, Result};
use crate::variation_model::VariationModel;

use log::info;
use read_fonts::types::{F2Dot14, Fixed, GlyphId, NameId, Tag};
use read_fonts::{FontData, FontRef, TableProvider};
use std::collections::HashSet;
use std::time::Instant;
use write_fonts::FontBuilder;
use write_fonts::from_obj::FromObjRef;
use write_fonts::tables::fvar::{AxisInstanceArrays, Fvar, InstanceRecord, VariationAxisRecord};
use write_fonts::tables::glyf::{
    CompositeGlyph as WriteCompositeGlyph, GlyfLocaBuilder, Glyph as WriteGlyph,
    SimpleGlyph as WriteSimpleGlyph,
};
use write_fonts::tables::gvar::{
    GlyphDelta, GlyphDeltas, GlyphVariations, Gvar, Tent, iup::iup_delta_optimize,
};
use write_fonts::tables::head::Head;

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
    designspace.validate().map_err(Error::InvalidDesignspace)?;

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
    let gvar_start = Instant::now();
    let gvar = build_gvar(designspace, &masters, &model, num_glyphs)?;
    info!("Built gvar table in {:.2}s", gvar_start.elapsed().as_secs_f64());

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

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

static TOTAL_POINTS: AtomicUsize = AtomicUsize::new(0);
static REQUIRED_POINTS: AtomicUsize = AtomicUsize::new(0);
static OPTIONAL_POINTS: AtomicUsize = AtomicUsize::new(0);
static DELTA_COMPUTE_NS: AtomicU64 = AtomicU64::new(0);
static IUP_OPTIMIZE_NS: AtomicU64 = AtomicU64::new(0);

fn build_gvar(
    designspace: &DesignSpace,
    masters: &[FontRef],
    model: &VariationModel,
    num_glyphs: u16,
) -> Result<Gvar> {
    // Reset counters
    TOTAL_POINTS.store(0, Ordering::Relaxed);
    REQUIRED_POINTS.store(0, Ordering::Relaxed);
    OPTIONAL_POINTS.store(0, Ordering::Relaxed);
    DELTA_COMPUTE_NS.store(0, Ordering::Relaxed);
    IUP_OPTIMIZE_NS.store(0, Ordering::Relaxed);

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

    let variations_start = Instant::now();
    let all_variations: Vec<GlyphVariations> = (0..num_glyphs)
        .map(|glyph_idx| {
            let gid = GlyphId::new(glyph_idx as u32);
            build_glyph_variations(gid, designspace, &master_glyfs, &master_locas, model)
        })
        .collect::<Result<Vec<_>>>()?;
    let variations_elapsed = variations_start.elapsed().as_secs_f64();

    let total = TOTAL_POINTS.load(Ordering::Relaxed);
    let required = REQUIRED_POINTS.load(Ordering::Relaxed);
    let optional = OPTIONAL_POINTS.load(Ordering::Relaxed);
    info!(
        "Glyph variations computed in {:.2}s ({} glyphs, {:.0} glyphs/sec)",
        variations_elapsed,
        num_glyphs,
        num_glyphs as f64 / variations_elapsed
    );
    info!(
        "IUP statistics: {} total points, {} required ({:.1}%), {} optional ({:.1}%)",
        total,
        required,
        required as f64 / total as f64 * 100.0,
        optional,
        optional as f64 / total as f64 * 100.0
    );
    
    let delta_secs = DELTA_COMPUTE_NS.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
    let iup_secs = IUP_OPTIMIZE_NS.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
    info!("Time breakdown: delta_compute={:.2}s, iup_optimize={:.2}s", delta_secs, iup_secs);

    let gvar_build_start = Instant::now();
    let gvar = Gvar::new(all_variations, axis_count).map_err(Error::GvarBuild)?;
    info!("Gvar::new() took {:.2}s", gvar_build_start.elapsed().as_secs_f64());
    
    Ok(gvar)
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

    // Precompute tents for all regions (these are constant per glyph)
    let all_tents: Vec<Vec<Tent>> = model
        .regions
        .iter()
        .map(|region| {
            region
                .axes
                .iter()
                .map(|&(min, peak, max)| {
                    let peak_f2d14 = F2Dot14::from_f32(peak);
                    let intermediate = Some((F2Dot14::from_f32(min), F2Dot14::from_f32(max)));
                    Tent::new(peak_f2d14, intermediate)
                })
                .collect()
        })
        .collect();

    // Compute all deltas for all points across all regions
    // all_deltas[region_idx][point_idx] = (dx, dy)
    let delta_start = Instant::now();
    let num_regions = model.regions.len();
    let num_masters = master_points.len();
    
    // Pre-allocate point_values buffer to avoid repeated allocations
    let mut point_values: Vec<(i16, i16)> = vec![(0, 0); num_masters];
    
    // all_raw_deltas[region_idx] = Vec of deltas for that region
    let mut all_raw_deltas: Vec<Vec<kurbo::Vec2>> = (0..num_regions)
        .map(|_| Vec::with_capacity(num_points + 4))
        .collect();
    
    // For each point, compute deltas across all regions
    for point_idx in 0..num_points {
        // Fill point_values buffer (no allocation)
        for (master_idx, points) in master_points.iter().enumerate() {
            point_values[master_idx] = points[point_idx];
        }
        
        // Compute deltas for all regions for this point
        let mut prev_deltas: Vec<(i16, i16)> = Vec::with_capacity(num_regions);
        for region_idx in 0..num_regions {
            let delta = model.compute_delta_2d_for_region(&point_values, region_idx, &prev_deltas);
            prev_deltas.push(delta);
            all_raw_deltas[region_idx].push(kurbo::Vec2::new(f64::from(delta.0), f64::from(delta.1)));
        }
    }
    DELTA_COMPUTE_NS.fetch_add(delta_start.elapsed().as_nanos() as u64, Ordering::Relaxed);

    // Build GlyphDeltas for each region
    let mut glyph_deltas: Vec<GlyphDeltas> = Vec::with_capacity(num_regions);

    for region_idx in 0..num_regions {
        let tents = all_tents[region_idx].clone();
        let raw_deltas = &mut all_raw_deltas[region_idx];

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
        // Note: We keep all deltas including phantom points - gvar requires them
        let iup_start = Instant::now();
        let deltas =
            match iup_delta_optimize(raw_deltas.clone(), coords_with_phantom, 0.5, &contour_ends) {
                Ok(optimized) => {
                    // Track IUP statistics (outline points only, not phantom)
                    let outline_deltas = &optimized[..num_points];
                    let required_count = outline_deltas.iter().filter(|d| d.required).count();
                    let optional_count = outline_deltas.iter().filter(|d| !d.required).count();
                    TOTAL_POINTS.fetch_add(num_points, Ordering::Relaxed);
                    REQUIRED_POINTS.fetch_add(required_count, Ordering::Relaxed);
                    OPTIONAL_POINTS.fetch_add(optional_count, Ordering::Relaxed);

                    // Force all deltas to be required to work around write-fonts bug
                    optimized
                        .into_iter()
                        .map(|d| GlyphDelta::required(d.x, d.y))
                        .collect()
                }
                Err(e) => {
                    // Log the error for debugging
                    log::warn!(
                        "IUP optimization failed for glyph {}: {:?}",
                        gid.to_u32(),
                        e
                    );
                    // Fall back to marking all as required (including phantom points)
                    raw_deltas
                        .iter()
                        .map(|d| GlyphDelta::required(d.x as i16, d.y as i16))
                        .collect()
                }
            };
        IUP_OPTIMIZE_NS.fetch_add(iup_start.elapsed().as_nanos() as u64, Ordering::Relaxed);

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
        // Get the full region (min, peak, max) for proper tent encoding
        let region = &model.regions[region_idx];
        let tents: Vec<Tent> = region
            .axes
            .iter()
            .map(|&(min, peak, max)| {
                let peak_f2d14 = F2Dot14::from_f32(peak);
                let intermediate = Some((F2Dot14::from_f32(min), F2Dot14::from_f32(max)));
                Tent::new(peak_f2d14, intermediate)
            })
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

        // Add 4 phantom point deltas (zero - composite metrics don't vary here)
        for _ in 0..4 {
            deltas.push(GlyphDelta::required(0, 0));
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
                let write_simple = WriteSimpleGlyph::from_obj_ref(&simple, FontData::new(&[]));
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
