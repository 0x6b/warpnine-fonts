//! Parallel file processing utilities.

use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;

/// Run an operation on multiple files in parallel with consistent error reporting.
///
/// Returns the number of successful operations.
pub fn run_parallel<T, F>(label: &str, items: &[T], op: F) -> Result<usize>
where
    T: AsRef<Path> + Sync,
    F: Fn(&Path) -> Result<()> + Sync,
{
    let results: Vec<_> = items
        .par_iter()
        .map(|item| {
            let path = item.as_ref();
            op(path).with_context(|| format!("Failed to process {}", path.display()))
        })
        .collect();

    let (success, failed) = results.iter().fold((0, 0), |(ok, err), r| {
        if let Err(e) = r {
            eprintln!("{e:?}");
            (ok, err + 1)
        } else {
            (ok + 1, err)
        }
    });

    println!("{label}: {success} succeeded, {failed} failed");
    Ok(success)
}
