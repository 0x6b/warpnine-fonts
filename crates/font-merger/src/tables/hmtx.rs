//! hmtx table merging

use indexmap::IndexMap;
use read_fonts::TableProvider;
use write_fonts::tables::hmtx::Hmtx;

use crate::{Result, context::MergeContext, glyph_order::GlyphName, types::GlyphId};

#[derive(Debug, Clone)]
pub struct GlyphMetrics {
    pub advance_width: u16,
    pub lsb: i16,
}

/// Merge hmtx tables from multiple fonts
pub fn merge_hmtx(ctx: &MergeContext) -> Result<Hmtx> {
    let mut metrics_map: IndexMap<GlyphName, GlyphMetrics> = IndexMap::new();

    for (i, font) in ctx.fonts().iter().enumerate() {
        let hhea = font.hhea()?;
        let hmtx = font.hmtx()?;
        let num_h_metrics = hhea.number_of_h_metrics() as usize;

        for (gid, glyph_name) in ctx.font_mapping(i) {
            let GlyphId(gid_val) = gid;
            let gid = *gid_val as usize;

            let (advance, lsb) = if gid < num_h_metrics {
                let lm = hmtx.h_metrics().get(gid).unwrap();
                (lm.advance.get(), lm.side_bearing.get())
            } else {
                // For glyphs beyond num_h_metrics, use the last advance width
                let last_advance = if num_h_metrics > 0 {
                    hmtx.h_metrics().get(num_h_metrics - 1).unwrap().advance.get()
                } else {
                    0
                };
                let lsb_idx = gid - num_h_metrics;
                let lsb = hmtx.left_side_bearings().get(lsb_idx).map(|b| b.get()).unwrap_or(0);
                (last_advance, lsb)
            };

            metrics_map.insert(glyph_name.clone(), GlyphMetrics { advance_width: advance, lsb });
        }
    }

    // Build the hmtx table in mega glyph order
    let mut h_metrics = Vec::with_capacity(ctx.mega().len());
    let left_side_bearings = Vec::new();

    for name in ctx.mega() {
        let metrics = metrics_map
            .get(name)
            .cloned()
            .unwrap_or(GlyphMetrics { advance_width: 0, lsb: 0 });

        h_metrics.push(write_fonts::tables::hmtx::LongMetric {
            advance: metrics.advance_width,
            side_bearing: metrics.lsb,
        });
    }

    Ok(Hmtx { h_metrics, left_side_bearings })
}
