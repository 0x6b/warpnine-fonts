//! Main Merger implementation

use crate::context::{GlyphOrder, MergeContext};
use crate::options::Options;
use crate::tables::cff::{check_cff, merge_cff, merge_cff2};
use crate::tables::cmap::merge_cmap;
use crate::tables::glyf::merge_glyf;
use crate::tables::head::merge_head;
use crate::tables::hhea::merge_hhea;
use crate::tables::hint::{check_hint_compatibility, log_hint_info};
use crate::tables::hmtx::merge_hmtx;
use crate::tables::layout::{merge_gpos, merge_gsub};
use crate::tables::maxp::merge_maxp;
use crate::tables::name::merge_name;
use crate::tables::os2::merge_os2;
use crate::tables::post::merge_post;
use crate::tables::vhea::merge_vhea;
use crate::tables::vmtx::merge_vmtx;

use crate::{MergeError, Result};
use log::info;
use read_fonts::{types::Tag, FontRef, TableProvider};
use std::collections::HashSet;
use write_fonts::FontBuilder;

/// Constant table tags to avoid repeated construction
const HANDLED_TABLES: &[[u8; 4]] = &[
    *b"head", *b"maxp", *b"cmap", *b"hmtx", *b"hhea", *b"post", *b"OS/2", *b"name", *b"vhea",
    *b"vmtx", *b"glyf", *b"loca", *b"CFF ", *b"CFF2", *b"GSUB", *b"GPOS",
];

/// Font merger that combines multiple fonts into one
#[derive(Default)]
pub struct Merger {
    options: Options,
}

impl Merger {
    /// Create a new Merger with the given options
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    /// Merge multiple font files into one
    pub fn merge(&self, font_data: &[&[u8]]) -> Result<Vec<u8>> {
        if font_data.is_empty() {
            return Err(MergeError::NoFonts);
        }

        let fonts: Vec<_> = font_data
            .iter()
            .map(|data| FontRef::new(data))
            .collect::<std::result::Result<_, _>>()?;

        self.merge_fonts(&fonts)
    }

    /// Merge multiple FontRef instances
    pub fn merge_fonts(&self, fonts: &[FontRef]) -> Result<Vec<u8>> {
        if fonts.is_empty() {
            return Err(MergeError::NoFonts);
        }

        self.validate_units_per_em(fonts)?;

        check_hint_compatibility(fonts);
        log_hint_info(fonts);

        let glyph_order = GlyphOrder::compute(fonts);
        let total_glyphs = glyph_order.total_glyphs();

        info!(
            "Merging {} fonts with {total_glyphs} total glyphs",
            fonts.len()
        );

        let (cmap, duplicate_info) = merge_cmap(fonts, &glyph_order)?;

        let ctx = MergeContext::new(fonts, glyph_order, duplicate_info, &self.options);

        let mut head = merge_head(ctx.fonts())?;
        let maxp = merge_maxp(ctx.fonts(), total_glyphs)?;
        let hmtx = merge_hmtx(&ctx)?;
        let hhea = merge_hhea(ctx.fonts(), total_glyphs)?;
        let post = merge_post(&ctx)?;

        let os2 = merge_os2(ctx.fonts())?;
        let vhea = merge_vhea(ctx.fonts(), total_glyphs)?;
        let vmtx = merge_vmtx(&ctx)?;

        let has_cff = check_cff(ctx.fonts())?;
        let (glyf_loca, cff_data) = if has_cff {
            (None, merge_cff(&ctx)?)
        } else {
            (merge_glyf(&ctx)?, None)
        };
        let cff2_data = merge_cff2(&ctx)?;

        let gsub = merge_gsub(&ctx)?;
        let gpos = merge_gpos(&ctx)?;

        let mut builder = FontBuilder::new();

        if let Some((_, _, format)) = glyf_loca.as_ref() {
            head.index_to_loc_format = match *format {
                write_fonts::tables::loca::LocaFormat::Short => 0,
                write_fonts::tables::loca::LocaFormat::Long => 1,
            };
        }

        builder.add_table(&head)?;
        builder.add_table(&maxp)?;
        builder.add_table(&cmap)?;
        builder.add_table(&hmtx)?;
        builder.add_table(&hhea)?;
        builder.add_table(&post)?;

        if let Some(os2) = os2.filter(|_| !self.options.should_drop_tag(Tag::new(b"OS/2"))) {
            builder.add_table(&os2)?;
        }
        if !self.options.should_drop_tag(Tag::new(b"name")) {
            merge_name(ctx.fonts(), &mut builder)?;
        }
        if let Some(vhea) = vhea.filter(|_| !self.options.should_drop_tag(Tag::new(b"vhea"))) {
            builder.add_table(&vhea)?;
        }
        if let Some(vmtx) = vmtx.filter(|_| !self.options.should_drop_tag(Tag::new(b"vmtx"))) {
            builder.add_table(&vmtx)?;
        }

        if let Some((glyf, loca, _format)) = glyf_loca {
            builder.add_table(&glyf)?;
            builder.add_table(&loca)?;
        }
        if let Some(cff_data) = cff_data {
            builder.add_raw(Tag::new(b"CFF "), cff_data);
        }
        if let Some(cff2_data) = cff2_data {
            builder.add_raw(Tag::new(b"CFF2"), cff2_data);
        }

        if let Some(gsub) = gsub {
            builder.add_table(&gsub)?;
        }
        if let Some(gpos) = gpos {
            builder.add_table(&gpos)?;
        }

        self.copy_other_tables(&mut builder, ctx.first_font())?;

        Ok(builder.build())
    }

    fn validate_units_per_em(&self, fonts: &[FontRef]) -> Result<()> {
        let (first, rest) = fonts.split_first().ok_or(MergeError::NoFonts)?;
        let first_upem = first.head()?.units_per_em();
        rest.iter().try_for_each(|font| {
            let upem = font.head()?.units_per_em();
            if upem == first_upem {
                Ok(())
            } else {
                Err(MergeError::IncompatibleUnitsPerEm {
                    expected: first_upem,
                    actual: upem,
                })
            }
        })
    }

    fn copy_other_tables(&self, builder: &mut FontBuilder, font: &FontRef) -> Result<()> {
        let handled_tables: HashSet<Tag> = HANDLED_TABLES.iter().map(Tag::new).collect();
        let drop_tables: HashSet<Tag> = self.options.drop_tables.iter().map(|t| t.tag()).collect();

        for record in font.table_directory.table_records() {
            let tag = record.tag();
            if handled_tables.contains(&tag) || drop_tables.contains(&tag) || builder.contains(tag)
            {
                continue;
            }
            if let Some(data) = font.table_data(tag) {
                builder.add_raw(tag, data.as_bytes().to_vec());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merger_no_fonts() {
        let merger = Merger::default();
        let result = merger.merge(&[]);
        assert!(matches!(result, Err(MergeError::NoFonts)));
    }
}
