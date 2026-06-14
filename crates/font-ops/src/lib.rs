//! Generic font table manipulation utilities.

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

/// Apply per-style naming and style bits to a font.
///
/// Rewrites name IDs 1/2/4/6/16/17, sets OS/2 `usWeightClass`, and updates the
/// bold/italic/regular bits in OS/2 `fsSelection` and head `macStyle` while
/// preserving all other bits. Name records that do not exist are left untouched.
pub fn apply_style(font_data: &[u8], names: &StyleNames, bits: &StyleBits) -> Result<Vec<u8>> {
    rewrite_font(font_data, |font, builder| {
        let new_name = map_name_records(font, |name_id, _current| match name_id {
            1 => Some(names.family.clone()),
            2 => Some(names.subfamily.clone()),
            4 => Some(names.full_name.clone()),
            6 => Some(names.postscript.clone()),
            16 => Some(names.typo_family.clone()),
            17 => Some(names.typo_subfamily.clone()),
            _ => None,
        })?;
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
