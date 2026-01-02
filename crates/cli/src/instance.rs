use anyhow::{Context, Result};
use font_instancer::{AxisLocation, instantiate};
use std::fs;
use std::path::Path;

pub fn create_instance(input: &Path, output: &Path, axes: &[(String, f32)]) -> Result<()> {
    let data = fs::read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    let locations: Vec<AxisLocation> = axes
        .iter()
        .map(|(tag, value)| AxisLocation::new(tag, *value))
        .collect();

    let axis_desc: Vec<String> = axes
        .iter()
        .map(|(tag, val)| format!("{}={}", tag, val))
        .collect();
    println!("Creating instance with axes: {}", axis_desc.join(", "));

    let static_data = instantiate(&data, &locations)
        .with_context(|| format!("Failed to instantiate {}", input.display()))?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output, &static_data)
        .with_context(|| format!("Failed to write {}", output.display()))?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = static_data.len() as f64 / 1024.0 / 1024.0;

    println!(
        "Instance created: {} ({:.2} MB) -> {} ({:.2} MB)",
        input.display(),
        input_size,
        output.display(),
        output_size
    );

    Ok(())
}
