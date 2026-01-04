//! Variation model for computing glyph deltas.
//!
//! Implements the core algorithm for computing how master contributions
//! are weighted at different locations in the design space.

use std::iter::once;

use crate::designspace::DesignSpace;

/// A region in the variation space, defined by (start, peak, end) tuples.
///
/// Each tuple defines the contribution curve for one axis.
/// The contribution is 0 at start, 1 at peak, and 0 at end.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// (min, peak, max) for each axis in normalized coordinates
    pub axes: Vec<(f32, f32, f32)>,
}

impl Region {
    /// Create a region from a peak location with neighbor-based tent boundaries.
    ///
    /// For each axis, the tent (min, peak, max) is computed following fontTools'
    /// VariationModel approach:
    /// - min: previous master's peak position (or 0 for first positive, -1 for negative)
    /// - max: axis maximum (1.0 for positive side, 0.0 for negative side)
    ///
    /// This "greedy" approach where each region extends to the axis max is what
    /// fontTools uses for delta computation.
    pub fn from_peak_with_neighbors(peak: &[f32], all_locations: &[Vec<f32>]) -> Self {
        let axes = peak
            .iter()
            .enumerate()
            .map(|(axis_idx, &p)| {
                if p == 0.0 {
                    // Default location on this axis - no contribution
                    (0.0, 0.0, 0.0)
                } else {
                    // Collect all unique positions on this axis, including default (0)
                    let mut positions: Vec<f32> = all_locations
                        .iter()
                        .map(|loc| loc.get(axis_idx).copied().unwrap_or(0.0))
                        .collect();
                    positions.push(0.0); // Ensure default is included
                    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    positions.dedup();

                    if p > 0.0 {
                        // Positive side: min is previous peak, max is 1.0
                        let pos_positions: Vec<f32> =
                            positions.iter().filter(|&&x| x >= 0.0).copied().collect();
                        let idx = pos_positions.iter().position(|&x| (x - p).abs() < 0.0001);

                        if let Some(i) = idx {
                            // min is the previous master's peak (0 for first positive master)
                            let min = if i == 0 { 0.0 } else { pos_positions[i - 1] };
                            // max is always the axis maximum
                            let max = 1.0;
                            (min, p, max)
                        } else {
                            (0.0, p, 1.0)
                        }
                    } else {
                        // Negative side: min is -1.0, max is next peak toward 0
                        let neg_positions: Vec<f32> =
                            positions.iter().filter(|&&x| x <= 0.0).copied().collect();
                        let idx = neg_positions.iter().position(|&x| (x - p).abs() < 0.0001);

                        if let Some(i) = idx {
                            // min is always the axis minimum
                            let min = -1.0;
                            // max is the next master's peak (0 for last negative master)
                            let max = if i >= neg_positions.len() - 1 {
                                0.0
                            } else {
                                neg_positions[i + 1]
                            };
                            (min, p, max)
                        } else {
                            (-1.0, p, 0.0)
                        }
                    }
                }
            })
            .collect();
        Self { axes }
    }

    /// Create a region from a peak location (simple case for corner masters only).
    ///
    /// DEPRECATED: Use from_peak_with_neighbors for proper intermediate master support.
    #[allow(dead_code)]
    pub fn from_peak(peak: &[f32]) -> Self {
        let axes = peak
            .iter()
            .map(|&p| {
                if p == 0.0 {
                    (0.0, 0.0, 0.0)
                } else if p > 0.0 {
                    (0.0, p, 1.0)
                } else {
                    (-1.0, p, 0.0)
                }
            })
            .collect();
        Self { axes }
    }

    /// Compute the scalar contribution of this region at a given location.
    ///
    /// Returns a value between 0 and 1.
    pub fn scalar_at(&self, location: &[f32]) -> f32 {
        let mut scalar = 1.0f32;

        for (i, &(min, peak, max)) in self.axes.iter().enumerate() {
            let loc = location.get(i).copied().unwrap_or(0.0);

            if peak == 0.0 {
                // No contribution on this axis
                continue;
            }

            if loc < min || loc > max {
                return 0.0;
            }

            if loc == peak {
                continue;
            }

            if loc < peak {
                scalar *= (loc - min) / (peak - min);
            } else {
                scalar *= (max - loc) / (max - peak);
            }
        }

        scalar
    }

    /// Check if this region is the default (all peaks at 0).
    #[allow(dead_code)]
    pub fn is_default(&self) -> bool {
        self.axes.iter().all(|(_, peak, _)| *peak == 0.0)
    }
}

/// Variation model for computing deltas from master values.
#[derive(Debug)]
pub struct VariationModel {
    /// Regions for each master (excluding default)
    pub regions: Vec<Region>,
    /// Index of the default master in the original source list
    pub default_idx: usize,
    /// Order in which to process masters for delta computation
    pub master_order: Vec<usize>,
    /// Precomputed scalars: region_scalars[i][j] = scalar of region j at region i's peak
    /// Only lower triangle is used (j < i)
    region_scalars: Vec<Vec<f32>>,
}

impl VariationModel {
    /// Create a variation model from a designspace.
    pub fn new(designspace: &DesignSpace) -> Option<Self> {
        let default_idx = designspace.default_source_index()?;
        let locations = designspace.master_locations();

        // Build regions for each master, using neighbor-based tent computation
        let mut regions_with_idx: Vec<(usize, Region)> = Vec::new();

        for (idx, loc) in locations.iter().enumerate() {
            if idx == default_idx {
                continue;
            }
            // Use neighbor-aware region computation for proper intermediate master support
            let region = Region::from_peak_with_neighbors(loc, &locations);
            regions_with_idx.push((idx, region));
        }

        // Sort masters by "support" - masters with fewer non-zero axes come first
        // This ensures proper delta accumulation
        regions_with_idx.sort_by_key(|(_, region)| {
            region.axes.iter().filter(|(_, peak, _)| *peak != 0.0).count()
        });

        let master_order: Vec<usize> = once(default_idx)
            .chain(regions_with_idx.iter().map(|(idx, _)| *idx))
            .collect();

        let regions: Vec<Region> = regions_with_idx.into_iter().map(|(_, r)| r).collect();

        // Precompute scalars between all region pairs
        // region_scalars[i][j] = scalar of region j at region i's peak location
        let region_scalars: Vec<Vec<f32>> = regions
            .iter()
            .enumerate()
            .map(|(i, region_i)| {
                let peak_i: Vec<f32> = region_i.axes.iter().map(|(_, p, _)| *p).collect();
                regions[..i]
                    .iter()
                    .map(|region_j| region_j.scalar_at(&peak_i))
                    .collect()
            })
            .collect();

        Some(Self { regions, default_idx, master_order, region_scalars })
    }

    /// Compute deltas from master values.
    ///
    /// Given values at each master location, compute the deltas needed
    /// to reconstruct those values through variation interpolation.
    ///
    /// # Arguments
    ///
    /// * `master_values` - Values at each master, indexed by original source index
    ///
    /// # Returns
    ///
    /// A tuple of (default_value, deltas) where deltas correspond to `self.regions`.
    #[allow(dead_code)]
    pub fn compute_deltas(&self, master_values: &[i16]) -> (i16, Vec<i16>) {
        let default_value = master_values[self.default_idx];
        let mut deltas = Vec::with_capacity(self.regions.len());

        // For each non-default master, compute its delta
        for region_idx in 0..self.regions.len() {
            let master_idx = self.master_order[region_idx + 1];
            let master_value = master_values[master_idx];

            // Start with the raw difference from default
            let mut delta = i32::from(master_value) - i32::from(default_value);

            // Subtract contributions from previous masters using precomputed scalars
            for (prev_region_idx, &scalar) in self.region_scalars[region_idx].iter().enumerate() {
                if scalar != 0.0 {
                    delta -= (f32::from(deltas[prev_region_idx]) * scalar) as i32;
                }
            }

            deltas.push(delta.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
        }

        (default_value, deltas)
    }

    /// Compute 2D deltas (x, y) from master values.
    pub fn compute_deltas_2d(&self, master_values: &[(i16, i16)]) -> ((i16, i16), Vec<(i16, i16)>) {
        let default_value = master_values[self.default_idx];
        let mut deltas = Vec::with_capacity(self.regions.len());

        for region_idx in 0..self.regions.len() {
            let master_idx = self.master_order[region_idx + 1];
            let master_value = master_values[master_idx];

            let mut delta_x = i32::from(master_value.0) - i32::from(default_value.0);
            let mut delta_y = i32::from(master_value.1) - i32::from(default_value.1);

            // Use precomputed scalars instead of recomputing
            for (prev_region_idx, &scalar) in self.region_scalars[region_idx].iter().enumerate() {
                if scalar != 0.0 {
                    let prev_delta: (i16, i16) = deltas[prev_region_idx];
                    delta_x -= (f32::from(prev_delta.0) * scalar) as i32;
                    delta_y -= (f32::from(prev_delta.1) * scalar) as i32;
                }
            }

            deltas.push((
                delta_x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                delta_y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            ));
        }

        (default_value, deltas)
    }

    /// Compute 2D delta for a single region (more efficient for per-point calls).
    ///
    /// This avoids allocating a Vec for all regions when only one is needed.
    #[inline]
    pub fn compute_delta_2d_for_region(
        &self,
        master_values: &[(i16, i16)],
        region_idx: usize,
        prev_deltas: &[(i16, i16)],
    ) -> (i16, i16) {
        let default_value = master_values[self.default_idx];
        let master_idx = self.master_order[region_idx + 1];
        let master_value = master_values[master_idx];

        let mut delta_x = i32::from(master_value.0) - i32::from(default_value.0);
        let mut delta_y = i32::from(master_value.1) - i32::from(default_value.1);

        for (prev_region_idx, &scalar) in self.region_scalars[region_idx].iter().enumerate() {
            if scalar != 0.0 {
                let prev_delta = prev_deltas[prev_region_idx];
                delta_x -= (f32::from(prev_delta.0) * scalar) as i32;
                delta_y -= (f32::from(prev_delta.1) * scalar) as i32;
            }
        }

        (
            delta_x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            delta_y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::designspace::{Axis, Source};

    fn make_2axis_designspace() -> DesignSpace {
        let axes = vec![
            Axis::new("wght", "Weight", 300.0, 400.0, 900.0),
            Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
        ];

        // 4 corner masters
        let sources = vec![
            Source::new(PathBuf::from("Regular.ttf"), vec![("wght", 400.0), ("ital", 0.0)]),
            Source::new(PathBuf::from("Bold.ttf"), vec![("wght", 900.0), ("ital", 0.0)]),
            Source::new(PathBuf::from("Italic.ttf"), vec![("wght", 400.0), ("ital", 1.0)]),
            Source::new(PathBuf::from("BoldItalic.ttf"), vec![("wght", 900.0), ("ital", 1.0)]),
        ];

        DesignSpace::new(axes, sources)
    }

    #[test]
    fn region_scalar_at_peak() {
        let region = Region::from_peak(&[1.0, 0.0]);
        assert_eq!(region.scalar_at(&[1.0, 0.0]), 1.0);
    }

    #[test]
    fn region_scalar_interpolated() {
        let region = Region::from_peak(&[1.0, 0.0]);
        assert!((region.scalar_at(&[0.5, 0.0]) - 0.5).abs() < 0.001);
    }

    #[test]
    fn region_scalar_outside() {
        let region = Region::from_peak(&[1.0, 0.0]);
        assert_eq!(region.scalar_at(&[-0.5, 0.0]), 0.0);
    }

    #[test]
    fn variation_model_creation() {
        let ds = make_2axis_designspace();
        let model = VariationModel::new(&ds).unwrap();

        assert_eq!(model.default_idx, 0);
        assert_eq!(model.regions.len(), 3); // 3 non-default masters
    }

    #[test]
    fn compute_simple_deltas() {
        let ds = make_2axis_designspace();
        let model = VariationModel::new(&ds).unwrap();

        // Master values: Regular=100, Bold=200, Italic=110, BoldItalic=220
        let values = [100i16, 200, 110, 220];
        let (default, deltas) = model.compute_deltas(&values);

        assert_eq!(default, 100);
        // The deltas should reconstruct the values when applied
        // This is a simplified test - actual values depend on master ordering
        assert_eq!(deltas.len(), 3);
    }
}
