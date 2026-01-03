//! Font instance creation from variable fonts.

use std::{
    fs::create_dir_all,
    path::Path,
};

use anyhow::{Context, Result};
pub use font_instancer::AxisLocation;
use font_instancer::instantiate;
use log::info;
use rayon::prelude::*;

use crate::io::{read_font, write_font};

/// Instance definition for batch processing.
pub struct InstanceDef {
    pub name: String,
    pub axes: Vec<AxisLocation>,
}

impl InstanceDef {
    pub fn new(name: impl Into<String>, axes: Vec<AxisLocation>) -> Self {
        Self { name: name.into(), axes }
    }
}

/// Font instancer for creating static fonts from variable fonts.
pub struct Instancer<'a> {
    data: &'a [u8],
}

impl<'a> Instancer<'a> {
    /// Create an instancer from font data.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Load an instancer from a font file.
    pub fn from_file(path: &Path) -> Result<OwnedInstancer> {
        let data = read_font(path)?;
        Ok(OwnedInstancer { data })
    }

    /// Create a static instance with the given axis values.
    pub fn instantiate(&self, axes: &[AxisLocation]) -> Result<Vec<u8>> {
        instantiate(self.data, axes).context("Failed to instantiate font")
    }

    /// Create a static instance and write to a file.
    pub fn instantiate_to_file(&self, output: &Path, axes: &[AxisLocation]) -> Result<()> {
        let axis_desc: Vec<String> = axes.iter().map(|loc| format!("{}={}", loc.tag, loc.value)).collect();
        info!("Creating instance with axes: {}", axis_desc.join(", "));

        let static_data = self.instantiate(axes)?;

        if let Some(parent) = output.parent() {
            create_dir_all(parent)?;
        }

        write_font(output, &static_data)?;

        let input_size = self.data.len() as f64 / 1024.0 / 1024.0;
        let output_size = static_data.len() as f64 / 1024.0 / 1024.0;

        info!(
            "Instance created: ({input_size:.2} MB) -> {} ({output_size:.2} MB)",
            output.display()
        );

        Ok(())
    }

    /// Create multiple instances in parallel.
    pub fn instantiate_batch(&self, output_dir: &Path, instances: &[InstanceDef]) -> Result<()> {
        info!("Creating {} instances", instances.len());

        create_dir_all(output_dir)?;

        instances.par_iter().try_for_each(|inst| -> Result<()> {
            let static_data = self.instantiate(&inst.axes)
                .with_context(|| format!("Failed to instantiate {}", inst.name))?;

            let output = output_dir.join(format!("{}.ttf", inst.name));
            write_font(&output, &static_data)?;

            info!("Created: {}", output.display());
            Ok(())
        })?;

        info!("Created {} instances", instances.len());
        Ok(())
    }
}

/// Owned version of Instancer that holds its own data.
pub struct OwnedInstancer {
    data: Vec<u8>,
}

impl OwnedInstancer {
    pub fn as_instancer(&self) -> Instancer<'_> {
        Instancer::new(&self.data)
    }

    pub fn instantiate(&self, axes: &[AxisLocation]) -> Result<Vec<u8>> {
        self.as_instancer().instantiate(axes)
    }

    pub fn instantiate_to_file(&self, output: &Path, axes: &[AxisLocation]) -> Result<()> {
        self.as_instancer().instantiate_to_file(output, axes)
    }

    pub fn instantiate_batch(&self, output_dir: &Path, instances: &[InstanceDef]) -> Result<()> {
        self.as_instancer().instantiate_batch(output_dir, instances)
    }
}

// Convenience functions for backward compatibility

pub fn create_instance(input: &Path, output: &Path, axes: &[AxisLocation]) -> Result<()> {
    let data = read_font(input)?;
    let instancer = Instancer::new(&data);
    
    let axis_desc: Vec<String> = axes.iter().map(|loc| format!("{}={}", loc.tag, loc.value)).collect();
    info!("Creating instance with axes: {}", axis_desc.join(", "));
    
    let static_data = instancer.instantiate(axes)
        .with_context(|| format!("Failed to instantiate {}", input.display()))?;

    if let Some(parent) = output.parent() {
        create_dir_all(parent)?;
    }

    write_font(output, &static_data)?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = static_data.len() as f64 / 1024.0 / 1024.0;

    info!(
        "Instance created: {} ({input_size:.2} MB) -> {} ({output_size:.2} MB)",
        input.display(),
        output.display()
    );

    Ok(())
}

pub fn create_instances_batch(
    input: &Path,
    output_dir: &Path,
    instances: &[InstanceDef],
) -> Result<()> {
    info!("Creating {} instances from {}", instances.len(), input.display());
    let instancer = Instancer::from_file(input)?;
    instancer.instantiate_batch(output_dir, instances)
}
