//! vmtx table merging

use indexmap::IndexMap;
use read_fonts::TableProvider;
use write_fonts::tables::vmtx::Vmtx;

use crate::{Result, context::MergeContext, glyph_order::GlyphName, types::GlyphId};

#[derive(Debug, Clone)]
pub struct VerticalGlyphMetrics {
    pub advance_height: u16,
    pub tsb: i16,
}

/// Merge vmtx tables from multiple fonts
pub fn merge_vmtx(ctx: &MergeContext) -> Result<Option<Vmtx>> {
    // Check if any font has vmtx
    let has_vmtx = ctx.fonts().iter().any(|f| f.vmtx().is_ok());
    if !has_vmtx {
        return Ok(None);
    }

    let mut metrics_map: IndexMap<GlyphName, VerticalGlyphMetrics> = IndexMap::new();

    for (i, font) in ctx.fonts().iter().enumerate() {
        let vmtx = match font.vmtx() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let vhea = match font.vhea() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let num_v_metrics = vhea.number_of_long_ver_metrics() as usize;

        for (gid, glyph_name) in ctx.font_mapping(i) {
            let GlyphId(gid_val) = gid;
            let gid = *gid_val as usize;

            let (advance, tsb) = if gid < num_v_metrics {
                let lm = vmtx.v_metrics().get(gid).unwrap();
                (lm.advance.get(), lm.side_bearing.get())
            } else {
                let last_advance = if num_v_metrics > 0 {
                    vmtx.v_metrics().get(num_v_metrics - 1).unwrap().advance.get()
                } else {
                    0
                };
                let tsb_idx = gid - num_v_metrics;
                let tsb = vmtx.top_side_bearings().get(tsb_idx).map(|b| b.get()).unwrap_or(0);
                (last_advance, tsb)
            };

            metrics_map
                .insert(glyph_name.clone(), VerticalGlyphMetrics { advance_height: advance, tsb });
        }
    }

    // Build the vmtx table in mega glyph order
    let mut v_metrics = Vec::with_capacity(ctx.mega().len());

    for name in ctx.mega() {
        let metrics = metrics_map
            .get(name)
            .cloned()
            .unwrap_or(VerticalGlyphMetrics { advance_height: 0, tsb: 0 });

        v_metrics.push(write_fonts::tables::vmtx::LongMetric {
            advance: metrics.advance_height,
            side_bearing: metrics.tsb,
        });
    }

    Ok(Some(Vmtx { v_metrics, top_side_bearings: Vec::new() }))
}
