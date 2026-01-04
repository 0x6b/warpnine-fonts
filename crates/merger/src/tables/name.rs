//! name table merging

use read_fonts::{FontRef, types::Tag};
use write_fonts::FontBuilder;

use crate::Result;

/// Merge name tables from multiple fonts
///
/// For now, we just copy the name table from the first font as raw bytes
/// A more sophisticated implementation would merge unique name records
pub fn merge_name(fonts: &[FontRef], builder: &mut FontBuilder) -> Result<()> {
    let first = match fonts.first() {
        Some(f) => f,
        None => return Ok(()),
    };

    if let Some(data) = first.table_data(Tag::new(b"name")) {
        builder.add_raw(Tag::new(b"name"), data.as_bytes().to_vec());
    }

    Ok(())
}
