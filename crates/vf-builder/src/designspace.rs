//! Designspace model for variable font building.
//!
//! Mirrors the concepts from fontTools designspaceLib.

use std::{collections::HashMap, path::PathBuf};

/// A variation axis in the designspace.
#[derive(Debug, Clone)]
pub struct Axis {
    /// Four-character axis tag (e.g., "wght", "ital")
    pub tag: String,
    /// Human-readable axis name
    pub name: String,
    /// Minimum value on this axis
    pub minimum: f32,
    /// Default value on this axis
    pub default: f32,
    /// Maximum value on this axis
    pub maximum: f32,
}

impl Axis {
    /// Create a new axis.
    pub fn new(tag: &str, name: &str, minimum: f32, default: f32, maximum: f32) -> Self {
        Self {
            tag: tag.to_string(),
            name: name.to_string(),
            minimum,
            default,
            maximum,
        }
    }

    /// Create a standard weight axis (wght: 100-900, default 400).
    pub fn weight() -> Self {
        Self::new("wght", "Weight", 100.0, 400.0, 900.0)
    }

    /// Create a standard italic axis (ital: 0-1, default 0).
    pub fn italic() -> Self {
        Self::new("ital", "Italic", 0.0, 0.0, 1.0)
    }

    /// Normalize a user-space value to the range [-1, 1].
    ///
    /// Values below the default normalize to [-1, 0].
    /// Values above the default normalize to [0, 1].
    pub fn normalize(&self, value: f32) -> f32 {
        if value < self.default {
            if self.default == self.minimum {
                0.0
            } else {
                -((self.default - value) / (self.default - self.minimum))
            }
        } else if value > self.default {
            if self.default == self.maximum {
                0.0
            } else {
                (value - self.default) / (self.maximum - self.default)
            }
        } else {
            0.0
        }
    }
}

/// A source (master) font in the designspace.
#[derive(Debug, Clone)]
pub struct Source {
    /// Path to the source font file
    pub path: PathBuf,
    /// Location in the designspace as (axis_tag, value) pairs
    pub location: HashMap<String, f32>,
    /// Optional family name override
    pub family_name: Option<String>,
    /// Optional style name
    pub style_name: Option<String>,
}

impl Source {
    /// Create a new source with the given path and location.
    ///
    /// The path accepts any type that can be converted to a `PathBuf`,
    /// including `&str`, `String`, `&Path`, or `PathBuf`.
    pub fn new(path: impl Into<PathBuf>, location: impl IntoIterator<Item = (&'static str, f32)>) -> Self {
        Self {
            path: path.into(),
            location: location.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            family_name: None,
            style_name: None,
        }
    }

    /// Set the family name.
    pub fn with_family_name(mut self, name: &str) -> Self {
        self.family_name = Some(name.to_string());
        self
    }

    /// Set the style name.
    pub fn with_style_name(mut self, name: &str) -> Self {
        self.style_name = Some(name.to_string());
        self
    }

    /// Get the value for an axis, or the axis default if not specified.
    pub fn axis_value(&self, axis: &Axis) -> f32 {
        self.location.get(&axis.tag).copied().unwrap_or(axis.default)
    }

    /// Get the normalized location as a vector of F2Dot14-range values.
    pub fn normalized_location(&self, axes: &[Axis]) -> Vec<f32> {
        axes.iter()
            .map(|axis| axis.normalize(self.axis_value(axis)))
            .collect()
    }
}

/// A named instance in the designspace.
#[derive(Debug, Clone)]
pub struct Instance {
    /// Instance name (e.g., "Bold", "Light Italic")
    pub name: String,
    /// Location in the designspace as (axis_tag, value) pairs
    pub location: HashMap<String, f32>,
    /// Optional PostScript name
    pub postscript_name: Option<String>,
}

impl Instance {
    /// Create a new instance with the given name and location.
    pub fn new(name: &str, location: impl IntoIterator<Item = (&'static str, f32)>) -> Self {
        Self {
            name: name.to_string(),
            location: location.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            postscript_name: None,
        }
    }

    /// Set the PostScript name.
    pub fn with_postscript_name(mut self, name: &str) -> Self {
        self.postscript_name = Some(name.to_string());
        self
    }

    /// Get the value for an axis, or the axis default if not specified.
    pub fn axis_value(&self, axis: &Axis) -> f32 {
        self.location.get(&axis.tag).copied().unwrap_or(axis.default)
    }
}

/// A complete designspace defining a variable font.
#[derive(Debug, Clone)]
pub struct DesignSpace {
    /// Variation axes
    pub axes: Vec<Axis>,
    /// Source (master) fonts
    pub sources: Vec<Source>,
    /// Named instances
    pub instances: Vec<Instance>,
}

impl DesignSpace {
    /// Create a new designspace with the given axes and sources.
    pub fn new(axes: Vec<Axis>, sources: Vec<Source>) -> Self {
        Self { axes, sources, instances: Vec::new() }
    }

    /// Add named instances to the designspace.
    pub fn with_instances(mut self, instances: Vec<Instance>) -> Self {
        self.instances = instances;
        self
    }

    /// Find the default source (the one at the default location for all axes).
    pub fn default_source(&self) -> Option<&Source> {
        self.sources.iter().find(|source| {
            self.axes
                .iter()
                .all(|axis| (source.axis_value(axis) - axis.default).abs() < 0.001)
        })
    }

    /// Find the index of the default source.
    pub fn default_source_index(&self) -> Option<usize> {
        self.sources.iter().position(|source| {
            self.axes
                .iter()
                .all(|axis| (source.axis_value(axis) - axis.default).abs() < 0.001)
        })
    }

    /// Get all unique axis locations from sources as normalized coordinates.
    pub fn master_locations(&self) -> Vec<Vec<f32>> {
        self.sources
            .iter()
            .map(|source| source.normalized_location(&self.axes))
            .collect()
    }

    /// Validate the designspace.
    pub fn validate(&self) -> Result<(), String> {
        if self.axes.is_empty() {
            return Err("Designspace must have at least one axis".to_string());
        }
        if self.sources.is_empty() {
            return Err("Designspace must have at least one source".to_string());
        }
        if self.default_source().is_none() {
            return Err("Designspace must have a source at the default location".to_string());
        }

        // Validate axis tags are 4 characters or less
        for axis in &self.axes {
            if axis.tag.len() > 4 {
                return Err(format!("Axis tag '{}' must be 4 characters or less", axis.tag));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_normalize() {
        let axis = Axis::new("wght", "Weight", 300.0, 400.0, 900.0);

        assert_eq!(axis.normalize(400.0), 0.0);
        assert_eq!(axis.normalize(300.0), -1.0);
        assert_eq!(axis.normalize(900.0), 1.0);
        assert!((axis.normalize(350.0) - (-0.5)).abs() < 0.001);
        assert!((axis.normalize(650.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn axis_normalize_symmetric() {
        // Italic axis: 0-1 with default 0
        let axis = Axis::new("ital", "Italic", 0.0, 0.0, 1.0);

        assert_eq!(axis.normalize(0.0), 0.0);
        assert_eq!(axis.normalize(1.0), 1.0);
        assert_eq!(axis.normalize(0.5), 0.5);
    }

    #[test]
    fn source_normalized_location() {
        let axes = vec![
            Axis::new("wght", "Weight", 300.0, 400.0, 900.0),
            Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
        ];

        let source = Source::new("test.ttf", vec![("wght", 900.0), ("ital", 1.0)]);
        let normalized = source.normalized_location(&axes);

        assert_eq!(normalized, vec![1.0, 1.0]);
    }

    #[test]
    fn designspace_default_source() {
        let axes = vec![
            Axis::new("wght", "Weight", 300.0, 400.0, 900.0),
            Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
        ];

        let sources = vec![
            Source::new("Regular.ttf", vec![("wght", 400.0), ("ital", 0.0)]),
            Source::new("Bold.ttf", vec![("wght", 700.0), ("ital", 0.0)]),
        ];

        let ds = DesignSpace::new(axes, sources);
        let default = ds.default_source().unwrap();

        assert_eq!(default.path, PathBuf::from("Regular.ttf"));
    }
}
