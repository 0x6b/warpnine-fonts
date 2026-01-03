//! Parallel file processing utilities.

use std::path::Path;

use anyhow::{Context, Result, bail};
use rayon::prelude::*;

/// Result of a parallel batch operation.
#[derive(Debug, Default)]
pub struct BatchResult {
    pub succeeded: usize,
    pub failed: usize,
}

#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use BatchResult instead")]
pub type ParallelResult = BatchResult;

impl BatchResult {
    pub fn total(&self) -> usize {
        self.succeeded + self.failed
    }

    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }

    pub fn ok_or_bail(&self, operation: &str) -> Result<()> {
        if self.failed > 0 {
            bail!("{operation} failed: {} succeeded, {} failed", self.succeeded, self.failed);
        }
        Ok(())
    }
}

/// Process items in parallel with consistent error reporting.
pub fn process_parallel_iter<T, R, F>(
    label: &str,
    items: impl IntoIterator<Item = T>,
    op: F,
) -> Result<BatchResult>
where
    T: Send,
    R: Send,
    F: Fn(T) -> Result<R> + Sync,
{
    let items: Vec<T> = items.into_iter().collect();
    let results: Vec<_> = items.into_par_iter().map(&op).collect();

    let mut result = BatchResult::default();
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

/// Collect results from parallel operations without printing.
pub fn collect_parallel<T, R, F>(items: &[T], op: F) -> Vec<Result<R>>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> Result<R> + Sync + Send,
{
    items.par_iter().map(op).collect()
}

/// Run an operation on multiple files in parallel with consistent error reporting.
pub fn run_parallel<T, F>(label: &str, items: &[T], op: F) -> Result<BatchResult>
where
    T: AsRef<Path> + Sync,
    F: Fn(&Path) -> Result<()> + Sync,
{
    process_parallel_iter(label, items.iter(), |item| {
        let path = item.as_ref();
        op(path).with_context(|| format!("Failed to process {}", path.display()))
    })
}
