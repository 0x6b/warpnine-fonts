//! Generic font table manipulation utilities.

use anyhow::{Context, Result};
use read_fonts::{FontRef, TableProvider, types::Tag};
use write_fonts::{
    FontBuilder,
    tables::name::{Name, NameRecord},
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
            read_fonts::types::NameId::new(name_id),
            new_string.into(),
        ));
    }

    Ok(Name::new(new_records))
}

/// Apply family and style naming to a font.
///
/// Updates the following name IDs:
/// - 1 (Family): `"{family} {style}"`
/// - 4 (Full name): `"{family} {style}"`
/// - 6 (PostScript name): `"{family}-{style}"` (spaces removed from family)
/// - 16 (Typographic family): `"{family}"`
/// - 17 (Typographic subfamily): `"{style}"`
pub fn apply_family_style_names(font_data: &[u8], family: &str, style: &str) -> Result<Vec<u8>> {
    let postscript_family = family.replace(' ', "");

    rewrite_font(font_data, |font, builder| {
        let new_name = map_name_records(font, |name_id, _current| match name_id {
            1 | 4 => Some(format!("{family} {style}")),
            6 => Some(format!("{postscript_family}-{style}")),
            16 => Some(family.to_string()),
            17 => Some(style.to_string()),
            _ => None,
        })?;
        builder.add_table(&new_name)?;
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
        .with_context(|| format!("Source font has no {} table", tag))?;

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
