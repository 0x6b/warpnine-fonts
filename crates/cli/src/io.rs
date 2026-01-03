//! Shared font I/O utilities.

use std::{
    fs::{create_dir_all, read, write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use glob::glob;

/// A font file handle for I/O operations.
#[derive(Debug, Clone)]
pub struct FontFile {
    path: PathBuf,
}

impl FontFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read font data from the file.
    pub fn read(&self) -> Result<Vec<u8>> {
        read(&self.path).with_context(|| format!("Failed to read font: {}", self.path.display()))
    }

    /// Write font data to the file.
    pub fn write(&self, data: impl AsRef<[u8]>) -> Result<()> {
        write(&self.path, data)
            .with_context(|| format!("Failed to write font: {}", self.path.display()))
    }

    /// Read, transform, and write back to the same file.
    pub fn transform(&self, f: impl FnOnce(&[u8]) -> Result<Vec<u8>>) -> Result<()> {
        let data = self.read()?;
        let new_data = f(&data)?;
        self.write(new_data)
    }

    /// Create parent directory if it doesn't exist.
    pub fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }
        Ok(())
    }
}

impl AsRef<Path> for FontFile {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// Find fonts matching a glob pattern in a directory.
pub fn glob_fonts(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let pattern = dir.join(pattern);
    let pattern_str = pattern.to_str().context("Invalid pattern path")?;
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

// Keep free functions as convenience wrappers for backward compatibility
pub fn read_font(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    FontFile::new(path.as_ref()).read()
}

pub fn write_font(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<()> {
    FontFile::new(path.as_ref()).write(data)
}

pub fn transform_font_in_place(
    path: impl AsRef<Path>,
    f: impl FnOnce(&[u8]) -> Result<Vec<u8>>,
) -> Result<()> {
    FontFile::new(path.as_ref()).transform(f)
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    FontFile::new(path).ensure_parent_dir()
}
