use anyhow::{Context, Result};
use font_instancer::{AxisLocation, instantiate};
use rayon::prelude::*;
use std::fs::create_dir_all;
use std::fs::read;
use std::fs::write;
use std::path::Path;

pub fn create_instance(input: &Path, output: &Path, axes: &[(String, f32)]) -> Result<()> {
    let data = read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    let locations: Vec<AxisLocation> = axes
        .iter()
        .map(|(tag, value)| AxisLocation::new(tag, *value))
        .collect();

    let axis_desc: Vec<String> = axes
        .iter()
        .map(|(tag, val)| format!("{tag}={val}"))
        .collect();
    println!("Creating instance with axes: {}", axis_desc.join(", "));

    let static_data = instantiate(&data, &locations)
        .with_context(|| format!("Failed to instantiate {}", input.display()))?;

    if let Some(parent) = output.parent() {
        create_dir_all(parent)?;
    }

    write(output, &static_data).with_context(|| format!("Failed to write {}", output.display()))?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = static_data.len() as f64 / 1024.0 / 1024.0;

    println!(
        "Instance created: {} ({input_size:.2} MB) -> {} ({output_size:.2} MB)",
        input.display(),
        output.display()
    );

    Ok(())
}

/// Instance definition for batch processing
pub struct InstanceDef {
    pub name: String,
    pub axes: Vec<(String, f32)>,
}

pub fn create_instances_batch(
    input: &Path,
    output_dir: &Path,
    instances: &[InstanceDef],
) -> Result<()> {
    println!(
        "Creating {} instances from {}",
        instances.len(),
        input.display()
    );

    let data = read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    create_dir_all(output_dir)?;

    instances.par_iter().try_for_each(|inst| -> Result<()> {
        let locations: Vec<AxisLocation> = inst
            .axes
            .iter()
            .map(|(tag, value)| AxisLocation::new(tag, *value))
            .collect();

        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", inst.name))?;

        let output = output_dir.join(format!("{}.ttf", inst.name));
        write(&output, &static_data)
            .with_context(|| format!("Failed to write {}", output.display()))?;

        println!("  Created: {}", output.display());
        Ok(())
    })?;

    println!("Created {} instances", instances.len());
    Ok(())
}
