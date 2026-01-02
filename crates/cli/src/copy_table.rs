use anyhow::{Context, Result};
use read_fonts::FontRef;
use std::fs::read;
use std::fs::write;
use std::path::Path;
use write_fonts::FontBuilder;

pub fn copy_gsub(source: &Path, target: &Path) -> Result<()> {
    let source_data = read(source).context("Failed to read source font")?;
    let source_font = FontRef::new(&source_data).context("Failed to parse source font")?;

    let gsub_data = source_font
        .table_data(read_fonts::types::Tag::new(b"GSUB"))
        .context("Source font has no GSUB table")?;

    let target_data = read(target).context("Failed to read target font")?;
    let target_font = FontRef::new(&target_data).context("Failed to parse target font")?;

    let mut builder = FontBuilder::new();

    for record in target_font.table_directory.table_records() {
        let tag = record.tag();
        if tag == read_fonts::types::Tag::new(b"GSUB") {
            continue;
        }
        if let Some(data) = target_font.table_data(tag) {
            builder.add_raw(tag, data);
        }
    }

    builder.add_raw(read_fonts::types::Tag::new(b"GSUB"), gsub_data);

    let output = builder.build();
    write(target, output).context("Failed to write target font")?;

    println!(
        "Copied GSUB table from {} to {}",
        source.display(),
        target.display()
    );
    Ok(())
}
