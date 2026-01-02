//! Shared font manipulation helpers.

use std::{
    fs::{read, write},
    path::Path,
};

use anyhow::Result;
use read_fonts::{FontRef, TableProvider};
use write_fonts::{
    FontBuilder,
    tables::name::{Name, NameRecord},
};

/// Rewrite font data by applying a transformation function.
///
/// Copies all tables from the source font, then calls `f` to modify/add tables.
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

/// Modify a font file in place.
pub fn modify_font_in_place(
    path: &Path,
    f: impl FnOnce(&FontRef, &mut FontBuilder) -> Result<()>,
) -> Result<()> {
    let data = read(path)?;
    let new_data = rewrite_font(&data, f)?;
    write(path, new_data)?;
    Ok(())
}

/// Map name table records using a transformation function.
///
/// The mapper receives (name_id, current_string) and returns the new string
/// (or None to keep the current string unchanged).
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

/// Apply family/style naming to a font, updating name IDs 1, 4, 6, 16, 17.
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
