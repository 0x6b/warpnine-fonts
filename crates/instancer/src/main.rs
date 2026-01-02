use std::{
    fs::{read, write},
    io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use font_instancer::{AxisLocation, instantiate};
use skrifa::MetadataProvider;

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Instancer(#[from] font_instancer::Error),
    #[error("expected TAG=VALUE: {0}")]
    InvalidSpec(String),
    #[error("axis tag must be 4 characters: {0}")]
    InvalidTag(String),
    #[error("invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Parser)]
#[command(name = "font-instancer", version)]
#[command(about = "Instantiate a variable font at specific axis locations")]
struct Cli {
    /// Input variable TTF file
    input: PathBuf,

    /// Axis locations as TAG=VALUE (e.g., wght=700)
    locations: Vec<String>,

    /// Output file (default: INPUT-instance.ttf)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Quiet output
    #[arg(short, long)]
    quiet: bool,

    /// Show font axes info
    #[arg(long)]
    info: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.info {
        return show_info(&cli.input);
    }

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    let locations = parse_locations(&cli.locations)?;

    let data = read(&cli.input)?;
    let result = instantiate(&data, &locations)?;

    let output = cli.output.unwrap_or_else(|| {
        let stem = cli.input.file_stem().unwrap().to_string_lossy();
        PathBuf::from(format!("{stem}-instance.ttf"))
    });

    write(&output, result)?;
    if !cli.quiet {
        println!("Wrote {}", output.display());
    }

    Ok(())
}

fn parse_locations(args: &[String]) -> Result<Vec<AxisLocation>, CliError> {
    args.iter().map(String::as_str).map(parse_axis_spec).collect()
}

fn parse_axis_spec(s: &str) -> Result<AxisLocation, CliError> {
    let (tag, value) = s
        .split_once('=')
        .ok_or_else(|| CliError::InvalidSpec(s.to_string()))?;

    if tag.len() != 4 {
        return Err(CliError::InvalidTag(tag.to_string()));
    }

    let value: f32 = value.parse().map_err(|_| CliError::InvalidValue(value.to_string()))?;

    Ok(AxisLocation::new(tag, value))
}

fn show_info(path: &Path) -> ExitCode {
    let data = match read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading file: {e}");
            return ExitCode::FAILURE;
        }
    };

    let font = match skrifa::FontRef::new(&data) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error parsing font: {e}");
            return ExitCode::FAILURE;
        }
    };

    let axes = font.axes();
    if axes.is_empty() {
        println!("Not a variable font");
        return ExitCode::SUCCESS;
    }

    println!("Variable font axes:");
    for axis in axes.iter() {
        println!(
            "  {:4}  {:6.0} .. {:6.0} (default: {:6.0})",
            axis.tag(),
            axis.min_value(),
            axis.max_value(),
            axis.default_value(),
        );
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_spec() {
        let loc = parse_axis_spec("wght=700").unwrap();
        assert_eq!(loc.value, 700.0);
    }

    #[test]
    fn parse_float_value() {
        let loc = parse_axis_spec("slnt=-12.5").unwrap();
        assert_eq!(loc.value, -12.5);
    }

    #[test]
    fn parse_rejects_missing_equals() {
        assert!(parse_axis_spec("wght700").is_err());
    }

    #[test]
    fn parse_rejects_short_tag() {
        assert!(parse_axis_spec("wg=700").is_err());
    }

    #[test]
    fn parse_rejects_invalid_value() {
        assert!(parse_axis_spec("wght=bold").is_err());
    }
}
