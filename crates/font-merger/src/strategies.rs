//! Merge strategies for combining values from multiple fonts
//!
//! These correspond to fontTools.merge.util functions

use crate::{MergeError, Result};

/// Assert all values are equal, return the first
pub fn equal<T: PartialEq + Clone>(
    values: &[T],
    table: &'static str,
    field: &'static str,
) -> Result<T> {
    let (first, rest) = values.split_first().ok_or(MergeError::NoFonts)?;
    if rest.iter().all(|v| v == first) {
        Ok(first.clone())
    } else {
        Err(MergeError::NotEqual { table, field })
    }
}

/// Return the first value
pub fn first<T: Clone>(values: &[T]) -> Result<T> {
    values.first().cloned().ok_or(MergeError::NoFonts)
}

/// Return the maximum value
pub fn max<T: Ord + Clone>(values: &[T]) -> Result<T> {
    values.iter().max().cloned().ok_or(MergeError::NoFonts)
}

/// Return the minimum value
pub fn min<T: Ord + Clone>(values: &[T]) -> Result<T> {
    values.iter().min().cloned().ok_or(MergeError::NoFonts)
}

/// Merge flags with a specific bit map
///
/// The bit_map specifies how to merge each bit:
/// - Some(true): bitwise OR
/// - Some(false): bitwise AND
/// - None: take from first font
pub fn merge_bits(values: &[u16], bit_map: &[Option<bool>; 16]) -> Result<u16> {
    let (first, _) = values.split_first().ok_or(MergeError::NoFonts)?;
    let first = *first;
    let mut result = 0u16;

    for (bit_idx, bit_mode) in bit_map.iter().enumerate() {
        let mask = 1u16 << bit_idx;
        match bit_mode {
            Some(true) => {
                if values.iter().any(|v| v & mask != 0) {
                    result |= mask;
                }
            }
            Some(false) => {
                if values.iter().all(|v| v & mask != 0) {
                    result |= mask;
                }
            }
            None => {
                result |= first & mask;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_success() {
        let values = vec![42, 42, 42];
        assert_eq!(equal(&values, "test", "field").unwrap(), 42);
    }

    #[test]
    fn test_equal_failure() {
        let values = vec![42, 43, 42];
        assert!(equal(&values, "test", "field").is_err());
    }

    #[test]
    fn test_first() {
        let values = vec![1, 2, 3];
        assert_eq!(first(&values).unwrap(), 1);
    }

    #[test]
    fn test_max() {
        let values = vec![1, 5, 3];
        assert_eq!(max(&values).unwrap(), 5);
    }

    #[test]
    fn test_min() {
        let values = vec![1, 5, 3];
        assert_eq!(min(&values).unwrap(), 1);
    }
}
