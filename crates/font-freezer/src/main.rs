use std::{
    ffi::OsString,
    fs::{read, write},
    io,
    path::{Path, PathBuf},
    process::ExitCode,
    result::Result,
};

use clap::Parser;
use font_feature_freezer::{FreezeOptions, FreezeResult, freeze, report};

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("{0}")]
    Message(String),
    #[error("read: {0}")]
    Read(#[source] io::Error),
    #[error("write: {0}")]
    Write(#[source] io::Error),
    #[error("{0}")]
    Font(#[from] font_feature_freezer::Error),
}

type CliResult<T> = Result<T, CliError>;

#[derive(Parser)]
#[command(name = "font-feature-freezer", version)]
#[command(about = "Permanently apply OpenType GSUB features by remapping the cmap table")]
#[command(long_about = "A Rust port of fonttools-opentype-feature-freezer.\n\n\
    With font-feature-freezer you can \"freeze\" some OpenType features into a font. \
    These features are then \"on by default\", even in apps that don't support OpenType features.")]
#[command(after_help = "Examples:\n  \
    font-feature-freezer -f 'c2sc,smcp' -S -U SC OpenSans.ttf OpenSansSC.ttf\n  \
    font-feature-freezer -R 'Lato/Otal' Lato-Regular.ttf Otal-Regular.ttf")]
struct Cli {
    /// Comma-separated feature tags, e.g. 'smcp,c2sc,onum'
    #[arg(short, long)]
    features: Option<String>,
    /// OpenType script tag, e.g. 'cyrl'
    #[arg(short, long)]
    script: Option<String>,
    /// OpenType language tag, e.g. 'SRB '
    #[arg(short, long)]
    lang: Option<String>,
    /// Zap glyphnames ('post' table version 3, .ttf only)
    #[arg(short, long)]
    zapnames: bool,
    /// Add suffix to font family name
    #[arg(short = 'S', long)]
    suffix: bool,
    /// Custom suffix (implies --suffix)
    #[arg(short = 'U', long)]
    usesuffix: Option<String>,
    /// Search/replace in names: 'old/new,old2/new2,...'
    #[arg(short = 'R', long)]
    replacenames: Option<String>,
    /// Update font version string
    #[arg(short, long)]
    info: bool,
    /// Report scripts, languages and features
    #[arg(short, long)]
    report: bool,
    /// Output names of remapped glyphs
    #[arg(short, long)]
    names: bool,
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Suppress output except errors
    #[arg(short, long)]
    quiet: bool,
    /// Input .otf or .ttf font file
    #[arg(value_name = "INPUT", required = true)]
    input: PathBuf,
    /// Output font file (default: <input>.featfreeze.<ext>)
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

impl Cli {
    fn run(&self) -> ExitCode {
        self.execute().map_or_else(
            |e| {
                if !e.to_string().is_empty() {
                    eprintln!("{e}");
                }
                ExitCode::FAILURE
            },
            |_| ExitCode::SUCCESS,
        )
    }

    fn execute(&self) -> CliResult<()> {
        if self.report {
            if self.output.is_some() {
                return Err(CliError::Message("output file is not used with --report".into()));
            }
            return self.run_report();
        }

        let features = self.parse_features()?;
        let output = self
            .output
            .clone()
            .unwrap_or_else(|| default_output_path(&self.input));
        let result = self.process_file(&self.input, &output, &features)?;

        if !self.quiet {
            for w in &result.warnings {
                eprintln!("WARNING: {w}");
            }
            println!(
                "{}: {}",
                self.input.file_name().unwrap_or_default().to_string_lossy(),
                result.stats
            );
        }

        Ok(())
    }

    fn run_report(&self) -> CliResult<()> {
        let data = read(&self.input).map_err(CliError::Read)?;
        print!("{}", report(&data)?);
        Ok(())
    }

    fn parse_features(&self) -> CliResult<Vec<String>> {
        let features = self.features.as_deref().ok_or_else(|| {
            CliError::Message("--features is required (unless using --report)".into())
        })?;
        let parsed: Vec<String> = features
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        if parsed.is_empty() {
            return Err(CliError::Message(
                "--features must include at least one feature tag".into(),
            ));
        }
        Ok(parsed)
    }

    fn freeze_options(&self, features: &[String]) -> FreezeOptions {
        FreezeOptions::new(features.iter().cloned())
            .with_script_opt(self.script.as_deref())
            .with_lang_opt(self.lang.as_deref())
            .with_suffix_if(self.suffix)
            .with_usesuffix_opt(self.usesuffix.as_deref())
            .with_replacenames_opt(self.replacenames.as_deref())
            .with_info_if(self.info)
            .with_zapnames_if(self.zapnames)
            .with_warnings_if(self.names)
    }

    fn process_file(
        &self,
        input: &Path,
        output: &Path,
        features: &[String],
    ) -> CliResult<FreezeResult> {
        let data = read(input).map_err(CliError::Read)?;
        let result = freeze(&data, &self.freeze_options(features))?;

        if self.names && !result.remapped_names.is_empty() {
            println!("{}", result.remapped_names.join(" "));
        }

        write(output, &result.data).map_err(CliError::Write)?;

        if self.verbose {
            eprintln!("[saveFont] Saved: {}", output.display());
        }
        Ok(result)
    }
}

fn default_output_path(input: &Path) -> PathBuf {
    let mut out = input.to_path_buf();
    let mut name = OsString::new();
    if let Some(stem) = input.file_stem() {
        name.push(stem);
    } else if let Some(file_name) = input.file_name() {
        name.push(file_name);
    } else {
        return out.with_extension("featfreeze");
    }
    name.push(".featfreeze");
    if let Some(ext) = input.extension() {
        name.push(".");
        name.push(ext);
    }
    out.set_file_name(name);
    out
}

fn main() -> ExitCode {
    Cli::parse().run()
}
