//! Merge context and related types
//!
//! This module provides the core data structures used throughout the merge process:
//! - `GlyphOrder`: The unified glyph ordering across all fonts
//! - `GidRemap`: Mapping from source font GIDs to merged GIDs
//! - `MergeContext`: The central context passed to all table mergers

use std::collections::HashMap;

use indexmap::{IndexMap, map::Entry};
use read_fonts::{FontRef, TableProvider, tables::post::Post};

use crate::{
    glyph_order::GlyphName,
    options::Options,
    tables::cmap::DuplicateGlyphInfo,
    types::{FontIndex, GlyphId, MegaGlyphId},
};

/// Unified glyph ordering across all fonts being merged
///
/// This struct encapsulates:
/// - The mega glyph order (all unique glyph names in merge order)
/// - Per-font mappings from original GID to glyph name
/// - Reverse lookup from glyph name to mega GID
#[derive(Debug, Clone)]
pub struct GlyphOrder {
    mega: Vec<GlyphName>,
    per_font: Vec<IndexMap<GlyphId, GlyphName>>,
    name_to_mega: HashMap<GlyphName, MegaGlyphId>,
}

impl GlyphOrder {
    /// Compute the unified glyph order from multiple fonts
    ///
    /// When the same glyph name appears in multiple fonts, later occurrences
    /// are renamed with a suffix (e.g., "A.1", "A.2")
    pub fn compute(fonts: &[FontRef]) -> Self {
        let mut mega_order: IndexMap<GlyphName, usize> = IndexMap::new();
        let mut per_font: Vec<IndexMap<GlyphId, GlyphName>> = Vec::with_capacity(fonts.len());

        for font in fonts {
            let glyph_order = get_glyph_order(font);
            let mut font_mapping = IndexMap::new();

            for (gid, name) in glyph_order.into_iter().enumerate() {
                let gid = GlyphId::new(gid as u16);

                match mega_order.entry(name.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(1);
                        font_mapping.insert(gid, name);
                    }
                    Entry::Occupied(mut entry) => {
                        let count = *entry.get();
                        *entry.get_mut() = count + 1;
                        let new_name = GlyphName::new(format!("{}.{count}", entry.key()));
                        mega_order.insert(new_name.clone(), 1);
                        font_mapping.insert(gid, new_name);
                    }
                }
            }

            per_font.push(font_mapping);
        }

        let mega: Vec<GlyphName> = mega_order.keys().cloned().collect();
        let name_to_mega: HashMap<GlyphName, MegaGlyphId> = mega
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), MegaGlyphId::new(i as u16)))
            .collect();

        Self { mega, per_font, name_to_mega }
    }

    /// Get the mega glyph order (all unique names)
    pub fn mega(&self) -> &[GlyphName] {
        &self.mega
    }

    /// Get the total number of glyphs in the merged font
    pub fn total_glyphs(&self) -> u16 {
        self.mega.len() as u16
    }

    /// Get the glyph mapping for a specific font
    pub fn font_mapping(&self, font_idx: usize) -> &IndexMap<GlyphId, GlyphName> {
        &self.per_font[font_idx]
    }

    /// Get all per-font glyph mappings
    pub fn all_mappings(&self) -> &[IndexMap<GlyphId, GlyphName>] {
        &self.per_font
    }

    /// Look up the mega GID for a glyph name
    pub fn mega_id(&self, name: &GlyphName) -> Option<MegaGlyphId> {
        self.name_to_mega.get(name).copied()
    }

    /// Look up the mega GID for a glyph name string
    pub fn mega_id_by_str(&self, name: &str) -> Option<MegaGlyphId> {
        self.name_to_mega.get(name).copied()
    }

    /// Get the name-to-mega-gid mapping
    pub fn name_to_mega_map(&self) -> &HashMap<GlyphName, MegaGlyphId> {
        &self.name_to_mega
    }

    /// Create a GidRemap for a specific font
    pub fn create_remap(&self, font_idx: usize) -> GidRemap {
        GidRemap::from_mapping(&self.per_font[font_idx], self)
    }
}

/// Extract glyph names from a font's post table
fn get_glyph_order(font: &FontRef) -> Vec<GlyphName> {
    let num_glyphs = font.maxp().map(|m| m.num_glyphs()).unwrap_or_default() as usize;
    let post = font.post().ok();

    (0..num_glyphs)
        .map(|gid| {
            let name = post
                .as_ref()
                .and_then(|p| get_glyph_name_from_post(p, gid as u16))
                .unwrap_or_else(|| format!("glyph{gid:05}"));
            GlyphName::new(name)
        })
        .collect()
}

fn get_glyph_name_from_post(post: &Post, gid: u16) -> Option<String> {
    post.glyph_name(read_fonts::types::GlyphId16::new(gid))
        .map(|s| s.to_string())
}

/// Mapping from source font GIDs to merged mega GIDs
///
/// This is computed once per font and reused by all table mergers.
#[derive(Debug, Clone)]
pub struct GidRemap(HashMap<GlyphId, MegaGlyphId>);

impl GidRemap {
    /// Create a GidRemap from a font's glyph mapping and the global glyph order
    pub fn from_mapping(mapping: &IndexMap<GlyphId, GlyphName>, glyph_order: &GlyphOrder) -> Self {
        let inner = mapping
            .iter()
            .filter_map(|(old_gid, name)| {
                glyph_order.mega_id(name).map(|new_gid| (*old_gid, new_gid))
            })
            .collect();
        GidRemap(inner)
    }

    /// Look up the mega GID for a source font GID
    pub fn get(&self, old: GlyphId) -> Option<MegaGlyphId> {
        self.0.get(&old).copied()
    }

    /// Look up the mega GID for a raw u16 source GID
    pub fn get_u16(&self, old: u16) -> Option<u16> {
        self.get(GlyphId::new(old)).map(|m| m.to_u16())
    }

    /// Get the underlying HashMap for iteration
    pub fn iter(&self) -> impl Iterator<Item = (&GlyphId, &MegaGlyphId)> {
        self.0.iter()
    }
}

/// Central context for the merge operation
///
/// This bundles all the common data needed by table mergers, eliminating
/// the need to pass multiple slices and maps separately.
pub struct MergeContext<'a> {
    fonts: &'a [FontRef<'a>],
    glyph_order: GlyphOrder,
    remaps: Vec<GidRemap>,
    duplicate_info: DuplicateGlyphInfo,
    options: &'a Options,
}

impl<'a> MergeContext<'a> {
    /// Create a new merge context
    pub fn new(
        fonts: &'a [FontRef<'a>],
        glyph_order: GlyphOrder,
        duplicate_info: DuplicateGlyphInfo,
        options: &'a Options,
    ) -> Self {
        let remaps = (0..fonts.len()).map(|i| glyph_order.create_remap(i)).collect();

        Self {
            fonts,
            glyph_order,
            remaps,
            duplicate_info,
            options,
        }
    }

    /// Get the fonts being merged
    pub fn fonts(&self) -> &[FontRef<'a>] {
        self.fonts
    }

    /// Get the number of fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Get the glyph order
    pub fn glyph_order(&self) -> &GlyphOrder {
        &self.glyph_order
    }

    /// Get the mega glyph order
    pub fn mega(&self) -> &[GlyphName] {
        self.glyph_order.mega()
    }

    /// Get the total number of glyphs
    pub fn total_glyphs(&self) -> u16 {
        self.glyph_order.total_glyphs()
    }

    /// Get the GID remap for a specific font
    pub fn remap(&self, font_idx: usize) -> &GidRemap {
        &self.remaps[font_idx]
    }

    /// Get the glyph mapping for a specific font
    pub fn font_mapping(&self, font_idx: usize) -> &IndexMap<GlyphId, GlyphName> {
        self.glyph_order.font_mapping(font_idx)
    }

    /// Get the duplicate glyph info
    pub fn duplicate_info(&self) -> &DuplicateGlyphInfo {
        &self.duplicate_info
    }

    /// Get the options
    pub fn options(&self) -> &Options {
        self.options
    }

    /// Iterate over fonts with their index and remap
    pub fn fonts_with_remap(&self) -> impl Iterator<Item = (FontIndex, &FontRef<'a>, &GidRemap)> {
        self.fonts
            .iter()
            .zip(&self.remaps)
            .enumerate()
            .map(|(i, (font, remap))| (FontIndex::new(i), font, remap))
    }

    /// Get a specific font by index
    pub fn font(&self, idx: FontIndex) -> &FontRef<'a> {
        &self.fonts[idx.as_usize()]
    }

    /// Get the first font (used as template for many operations)
    pub fn first_font(&self) -> &FontRef<'a> {
        &self.fonts[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gid_remap() {
        let mut mapping = IndexMap::new();
        mapping.insert(GlyphId::new(0), GlyphName::new(".notdef"));
        mapping.insert(GlyphId::new(1), GlyphName::new("A"));

        let mut name_to_mega = HashMap::new();
        name_to_mega.insert(GlyphName::new(".notdef"), MegaGlyphId::new(0));
        name_to_mega.insert(GlyphName::new("A"), MegaGlyphId::new(1));

        let glyph_order = GlyphOrder {
            mega: vec![GlyphName::new(".notdef"), GlyphName::new("A")],
            per_font: vec![mapping.clone()],
            name_to_mega,
        };

        let remap = GidRemap::from_mapping(&mapping, &glyph_order);
        assert_eq!(remap.get(GlyphId::new(0)), Some(MegaGlyphId::new(0)));
        assert_eq!(remap.get(GlyphId::new(1)), Some(MegaGlyphId::new(1)));
        assert_eq!(remap.get(GlyphId::new(2)), None);
    }
}
