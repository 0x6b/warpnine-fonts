//! Parallel file processing utilities.

use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;

/// Result of a parallel batch operation.
#[derive(Debug, Default)]
pub struct ParallelResult {
    pub succeeded: usize,
    pub failed: usize,
}

impl ParallelResult {
    pub fn total(&self) -> usize {
        self.succeeded + self.failed
    }

    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }
}

/// Run an operation on multiple files in parallel with consistent error reporting.
pub fn run_parallel<T, F>(label: &str, items: &[T], op: F) -> Result<ParallelResult>
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

    let mut result = ParallelResult::default();
    for r in &results {
        if let Err(e) = r {
            eprintln!("{e:?}");
            result.failed += 1;
        } else {
            result.succeeded += 1;
        }
    }

    println!("{label}: {} succeeded, {} failed", result.succeeded, result.failed);
    Ok(result)
}
