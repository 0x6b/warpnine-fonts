//! Generic font table manipulation utilities.

use std::collections::HashSet;

use anyhow::{Context, Result};
use read_fonts::{
    FontRef, TableProvider,
    types::{NameId, Tag},
};
use write_fonts::{
    FontBuilder,
    from_obj::ToOwnedTable,
    tables::{
        head::{Head, MacStyle},
        name::{Name, NameRecord},
        os2::{Os2, SelectionFlags},
    },
};

/// Rewrite font data by applying a transformation function.
///
/// Copies all tables from the source font, then calls `f` to modify or add tables.
/// The function receives a reference to the source font and a mutable builder
/// that already contains all original tables.
pub fn rewrite_font(
    data: &[u8],
    f: impl FnOnce(&FontRef, &mut FontBuilder) -> Result<()>,
) -> Result<Vec<u8>> {
    let font = FontRef::new(data)?;
    let mut builder = FontBuilder::new();

    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }

    f(&font, &mut builder)?;
    Ok(builder.build())
}

/// Map name table records using a transformation function.
///
/// The mapper receives `(name_id, current_string)` and returns:
/// - `Some(new_string)` to replace the record's string
/// - `None` to keep the current string unchanged
pub fn map_name_records(
    font: &FontRef,
    mut mapper: impl FnMut(u16, &str) -> Option<String>,
) -> Result<Name> {
    let name = font.name()?;
    let mut new_records = Vec::new();

    for record in name.name_record() {
        let name_id = record.name_id().to_u16();
        let current = match record.string(name.string_data()) {
            Ok(s) => s.chars().collect::<String>(),
            Err(_) => continue,
        };

        let new_string = mapper(name_id, &current).unwrap_or(current);

        new_records.push(NameRecord::new(
            record.platform_id(),
            record.encoding_id(),
            record.language_id(),
            NameId::new(name_id),
            new_string.into(),
        ));
    }

    Ok(Name::new(new_records))
}

/// Human-readable name-table strings for a single static style.
///
/// Maps to name IDs: 1 (family), 2 (subfamily), 4 (full name),
/// 6 (PostScript name), 16 (typographic family), 17 (typographic subfamily).
#[derive(Debug, Clone)]
pub struct StyleNames {
    pub family: String,
    pub subfamily: String,
    pub full_name: String,
    pub postscript: String,
    pub typo_family: String,
    pub typo_subfamily: String,
}

/// OS/2 `fsSelection` / head `macStyle` / OS/2 `usWeightClass` settings for a style.
///
/// `regular` should be set only when the face is neither `bold` nor `italic`.
#[derive(Debug, Clone, Copy)]
pub struct StyleBits {
    pub italic: bool,
    pub bold: bool,
    pub regular: bool,
    pub weight_class: u16,
}

/// OpenType/CSS weight name for a `usWeightClass` value (300 → "Light" …
/// 1000 → "ExtraBlack"). Unknown values fall back to "Regular".
pub fn weight_name(weight: u16) -> &'static str {
    match weight {
        300 => "Light",
        500 => "Medium",
        600 => "SemiBold",
        700 => "Bold",
        800 => "ExtraBold",
        900 => "Black",
        1000 => "ExtraBlack",
        // 400 and any unknown value fall back to "Regular".
        _ => "Regular",
    }
}

/// Human-readable subfamily from a no-space style id
/// ("BoldItalic" → "Bold Italic", "Italic" → "Italic", "SemiBold" → "SemiBold").
pub fn style_display_name(style_id: &str) -> String {
    if style_id.ends_with("Italic") && style_id != "Italic" {
        let base = style_id.strip_suffix("Italic").unwrap();
        format!("{base} Italic")
    } else {
        style_id.to_string()
    }
}

/// OS/2 / head style bits for a weight + slant.
///
/// `bold` is the RIBBI bold member (weight 700); `regular` is set only when the
/// face is neither bold nor italic.
pub fn style_bits(weight: u16, italic: bool) -> StyleBits {
    let bold = weight == 700;
    StyleBits {
        italic,
        bold,
        regular: !italic && !bold,
        weight_class: weight,
    }
}

/// RIBBI-grouped name-table strings for a style.
///
/// Regular (400) and Bold (700) plus their italics share the base `family` as
/// name ID 1, distinguished by name ID 2 (Regular/Bold/Italic/Bold Italic).
/// Every other weight becomes its own `"{family} {Weight}"` sub-family with
/// name ID 2 of Regular/Italic. `style_id` is the no-space style identifier
/// (e.g. "BoldItalic") used for the PostScript name and ID 17.
pub fn ribbi_names(
    family: &str,
    ps_family: &str,
    style_id: &str,
    weight: u16,
    italic: bool,
) -> StyleNames {
    let bits = style_bits(weight, italic);
    let in_base_quad = weight == 400 || bits.bold;

    let subfamily = match (bits.bold, bits.italic) {
        (true, true) => "Bold Italic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    }
    .to_string();

    let id1 =
        if in_base_quad { family.to_string() } else { format!("{family} {}", weight_name(weight)) };

    let full_name = if subfamily == "Regular" { id1.clone() } else { format!("{id1} {subfamily}") };

    StyleNames {
        family: id1,
        subfamily,
        full_name,
        postscript: format!("{ps_family}-{style_id}"),
        typo_family: family.to_string(),
        typo_subfamily: style_display_name(style_id),
    }
}

/// Build a name table with per-style strings applied.
///
/// Rewrites name IDs 1/2/4/6/16/17 in place. The typographic family (16) and
/// subfamily (17) are *synthesized* on every platform record that carries a
/// legacy family (ID 1) but lacks them, so typographic grouping works even when
/// the donor font ships only the legacy RIBBI name IDs (0-6).
fn build_style_name_table(font: &FontRef, names: &StyleNames) -> Result<Name> {
    let name = font.name()?;
    let mut records: Vec<NameRecord> = Vec::new();

    // Platform records (platformID, encodingID, languageID) carrying ID 1, and
    // which of those already carry ID 16 / 17.
    let mut id1_platforms: HashSet<(u16, u16, u16)> = HashSet::new();
    let mut have_typo_family: HashSet<(u16, u16, u16)> = HashSet::new();
    let mut have_typo_subfamily: HashSet<(u16, u16, u16)> = HashSet::new();

    for record in name.name_record() {
        let id = record.name_id().to_u16();
        let key = (record.platform_id(), record.encoding_id(), record.language_id());
        let current = match record.string(name.string_data()) {
            Ok(s) => s.chars().collect::<String>(),
            Err(_) => continue,
        };
        let value = match id {
            1 => {
                id1_platforms.insert(key);
                names.family.clone()
            }
            2 => names.subfamily.clone(),
            4 => names.full_name.clone(),
            6 => names.postscript.clone(),
            16 => {
                have_typo_family.insert(key);
                names.typo_family.clone()
            }
            17 => {
                have_typo_subfamily.insert(key);
                names.typo_subfamily.clone()
            }
            _ => current,
        };
        records.push(NameRecord::new(key.0, key.1, key.2, NameId::new(id), value.into()));
    }

    // Synthesize typographic names wherever the legacy family exists without them.
    for key in &id1_platforms {
        if !have_typo_family.contains(key) {
            let value = names.typo_family.clone();
            records.push(NameRecord::new(key.0, key.1, key.2, NameId::new(16), value.into()));
        }
        if !have_typo_subfamily.contains(key) {
            let value = names.typo_subfamily.clone();
            records.push(NameRecord::new(key.0, key.1, key.2, NameId::new(17), value.into()));
        }
    }

    records.sort_by_key(|r| (r.platform_id, r.encoding_id, r.language_id, r.name_id));
    Ok(Name::new(records))
}

/// Apply per-style naming and style bits to a font.
///
/// Rewrites name IDs 1/2/4/6/16/17 (synthesizing 16/17 when the donor lacks
/// them — see [`build_style_name_table`]), sets OS/2 `usWeightClass`, and
/// updates the bold/italic/regular bits in OS/2 `fsSelection` and head
/// `macStyle` while preserving all other bits.
pub fn apply_style(font_data: &[u8], names: &StyleNames, bits: &StyleBits) -> Result<Vec<u8>> {
    rewrite_font(font_data, |font, builder| {
        let new_name = build_style_name_table(font, names)?;
        builder.add_table(&new_name)?;

        if let Ok(os2) = font.os2() {
            let mut new_os2: Os2 = os2.to_owned_table();
            new_os2.us_weight_class = bits.weight_class;

            let mut fs = new_os2.fs_selection;
            fs.remove(SelectionFlags::ITALIC | SelectionFlags::BOLD | SelectionFlags::REGULAR);
            if bits.italic {
                fs.insert(SelectionFlags::ITALIC);
            }
            if bits.bold {
                fs.insert(SelectionFlags::BOLD);
            }
            if bits.regular {
                fs.insert(SelectionFlags::REGULAR);
            }
            new_os2.fs_selection = fs;

            builder.add_table(&new_os2)?;
        }

        if let Ok(head) = font.head() {
            let mut new_head: Head = head.to_owned_table();
            let mut mac = new_head.mac_style;
            mac.remove(MacStyle::BOLD | MacStyle::ITALIC);
            if bits.bold {
                mac.insert(MacStyle::BOLD);
            }
            if bits.italic {
                mac.insert(MacStyle::ITALIC);
            }
            new_head.mac_style = mac;

            builder.add_table(&new_head)?;
        }

        Ok(())
    })
}

/// Copy a table from source font to target font.
///
/// Returns the new font data with the specified table replaced (or added)
/// from the source font.
pub fn copy_table(source_data: &[u8], target_data: &[u8], tag: Tag) -> Result<Vec<u8>> {
    let source_font = FontRef::new(source_data).context("Failed to parse source font")?;

    let table_data = source_font
        .table_data(tag)
        .with_context(|| format!("Source font has no {tag} table"))?;

    let target_font = FontRef::new(target_data).context("Failed to parse target font")?;

    let mut builder = FontBuilder::new();

    for record in target_font.table_directory.table_records() {
        let record_tag = record.tag();
        if record_tag == tag {
            continue;
        }
        if let Some(data) = target_font.table_data(record_tag) {
            builder.add_raw(record_tag, data);
        }
    }

    builder.add_raw(tag, table_data);

    Ok(builder.build())
}

/// Copy GSUB table from source font to target font, removing FeatureVariations.
///
/// FeatureVariations may reference axis indices that don't exist in the target
/// font (e.g., source has 5 axes, target has 2). This causes OTS validation errors.
pub fn copy_gsub_without_feature_variations(
    source_data: &[u8],
    target_data: &[u8],
) -> Result<Vec<u8>> {
    use write_fonts::{from_obj::ToOwnedTable, tables::gsub::Gsub};

    let source_font = FontRef::new(source_data).context("Failed to parse source font")?;
    let gsub = source_font.gsub().context("Source font has no GSUB table")?;

    let script_list = gsub.script_list()?.to_owned_table();
    let feature_list = gsub.feature_list()?.to_owned_table();
    let lookup_list = gsub.lookup_list()?.to_owned_table();

    let new_gsub = Gsub::new(script_list, feature_list, lookup_list);

    let target_font = FontRef::new(target_data).context("Failed to parse target font")?;
    let mut builder = FontBuilder::new();

    for record in target_font.table_directory.table_records() {
        let record_tag = record.tag();
        if record_tag == Tag::new(b"GSUB") {
            continue;
        }
        if let Some(data) = target_font.table_data(record_tag) {
            builder.add_raw(record_tag, data);
        }
    }

    builder.add_table(&new_gsub)?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_names() {
        assert_eq!(weight_name(300), "Light");
        assert_eq!(weight_name(400), "Regular");
        assert_eq!(weight_name(600), "SemiBold");
        assert_eq!(weight_name(1000), "ExtraBlack");
        assert_eq!(weight_name(123), "Regular");
    }

    #[test]
    fn display_names() {
        assert_eq!(style_display_name("Regular"), "Regular");
        assert_eq!(style_display_name("Italic"), "Italic");
        assert_eq!(style_display_name("SemiBold"), "SemiBold");
        assert_eq!(style_display_name("BoldItalic"), "Bold Italic");
        assert_eq!(style_display_name("ExtraBlackItalic"), "ExtraBlack Italic");
    }

    #[test]
    fn bits_regular_bold_italic() {
        let r = style_bits(400, false);
        assert!(r.regular && !r.bold && !r.italic && r.weight_class == 400);
        let b = style_bits(700, false);
        assert!(b.bold && !b.regular && !b.italic);
        let bi = style_bits(700, true);
        assert!(bi.bold && bi.italic && !bi.regular);
        let heavy = style_bits(900, false);
        assert!(heavy.regular && !heavy.bold && heavy.weight_class == 900);
        let si = style_bits(600, true);
        assert!(si.italic && !si.bold && !si.regular);
    }

    #[test]
    fn ribbi_base_quad() {
        let n = ribbi_names("Warpnine Mono", "WarpnineMono", "Regular", 400, false);
        assert_eq!(n.family, "Warpnine Mono");
        assert_eq!(n.subfamily, "Regular");
        assert_eq!(n.full_name, "Warpnine Mono");
        assert_eq!(n.postscript, "WarpnineMono-Regular");
        assert_eq!(n.typo_family, "Warpnine Mono");
        assert_eq!(n.typo_subfamily, "Regular");

        let bi = ribbi_names("Warpnine Mono", "WarpnineMono", "BoldItalic", 700, true);
        assert_eq!(bi.family, "Warpnine Mono");
        assert_eq!(bi.subfamily, "Bold Italic");
        assert_eq!(bi.full_name, "Warpnine Mono Bold Italic");
        assert_eq!(bi.postscript, "WarpnineMono-BoldItalic");
        assert_eq!(bi.typo_subfamily, "Bold Italic");
    }

    #[test]
    fn ribbi_sub_family() {
        let n = ribbi_names("Warpnine Mono", "WarpnineMono", "SemiBoldItalic", 600, true);
        assert_eq!(n.family, "Warpnine Mono SemiBold");
        assert_eq!(n.subfamily, "Italic");
        assert_eq!(n.full_name, "Warpnine Mono SemiBold Italic");
        assert_eq!(n.typo_family, "Warpnine Mono");
        assert_eq!(n.typo_subfamily, "SemiBold Italic");
    }
}
