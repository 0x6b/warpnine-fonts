//! Shared font I/O utilities.

use std::{
    fs::{create_dir_all, read, write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use glob::glob;

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

/// Find fonts matching a glob pattern in a directory.
pub fn glob_fonts(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let pattern = dir.join(pattern);
    let pattern_str = pattern
        .to_str()
        .context("Invalid pattern path")?;
    Ok(glob(pattern_str)
        .with_context(|| format!("Failed to glob pattern: {pattern_str}"))?
        .filter_map(Result::ok)
        .collect())
}

/// Check batch operation results and report failures.
pub fn check_results<T>(results: &[Result<T>], operation: &str) -> Result<()> {
    let failed_count = results.iter().filter(|r| r.is_err()).count();
    if failed_count > 0 {
        bail!("{operation} failed for {} files", failed_count);
    }
    Ok(())
}
