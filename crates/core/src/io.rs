//! Shared font I/O utilities.

use std::{
    fs::{create_dir_all, read, write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use glob::glob;
use log::error;

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

    pub fn read(&self) -> Result<Vec<u8>> {
        read(&self.path).with_context(|| format!("Failed to read font: {}", self.path.display()))
    }

    pub fn write(&self, data: impl AsRef<[u8]>) -> Result<()> {
        write(&self.path, data)
            .with_context(|| format!("Failed to write font: {}", self.path.display()))
    }

    pub fn transform(&self, f: impl FnOnce(&[u8]) -> Result<Vec<u8>>) -> Result<()> {
        let data = self.read()?;
        let new_data = f(&data)?;
        self.write(new_data)
    }

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

pub fn glob_fonts(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let pattern = dir.join(pattern);
    let pattern_str = pattern.to_str().context("Invalid pattern path")?;
    Ok(glob(pattern_str)
        .with_context(|| format!("Failed to glob pattern: {pattern_str}"))?
        .filter_map(Result::ok)
        .collect())
}

/// Check batch operation results, log failures, and bail if any failed.
///
/// For operations that don't have associated paths, use this simpler version.
pub fn check_results<T>(results: &[Result<T>], operation: &str) -> Result<()> {
    let errors: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();

    if !errors.is_empty() {
        for err in &errors {
            error!("{operation}: {err:#}");
        }
        bail!("{operation} failed for {} files", errors.len());
    }
    Ok(())
}

/// Check batch operation results with paths, log failures with file names, and bail if any failed.
pub fn check_results_with_paths<T, P: AsRef<Path>>(
    results: &[(P, Result<T>)],
    operation: &str,
) -> Result<()> {
    let errors: Vec<_> = results
        .iter()
        .filter_map(|(path, r)| r.as_ref().err().map(|e| (path.as_ref(), e)))
        .collect();

    if !errors.is_empty() {
        for (path, err) in &errors {
            error!("{}: {err:#}", path.display());
        }
        bail!("{operation} failed for {} files", errors.len());
    }
    Ok(())
}

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
