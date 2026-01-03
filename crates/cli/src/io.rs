//! Shared font I/O utilities.

use std::{
    fs::{create_dir_all, read, write},
    path::Path,
};

use anyhow::{Context, Result};

/// Read font data from a file.
pub fn read_font(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    read(path).with_context(|| format!("Failed to read font: {}", path.display()))
}

/// Write font data to a file.
pub fn write_font(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    write(path, data).with_context(|| format!("Failed to write font: {}", path.display()))
}

/// Read a font, transform it, and write back to the same file.
pub fn transform_font_in_place(
    path: impl AsRef<Path>,
    f: impl FnOnce(&[u8]) -> Result<Vec<u8>>,
) -> Result<()> {
    let path = path.as_ref();
    let data = read_font(path)?;
    let new_data = f(&data)?;
    write_font(path, new_data)
}

/// Create parent directory if it doesn't exist.
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    Ok(())
}
