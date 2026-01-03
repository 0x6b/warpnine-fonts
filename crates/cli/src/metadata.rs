//! Font metadata operations.

use std::path::Path;

use anyhow::Result;
use chrono::NaiveDate;
use log::info;

use crate::io::{read_font, write_font};

pub use warpnine_font_metadata::{FontVersion, MonospaceSettings};

/// Set monospace metadata on a font file.
pub fn set_monospace(path: &Path) -> Result<()> {
    let data = read_font(path)?;
    let new_data = warpnine_font_metadata::set_monospace(&data)?;
    write_font(path, new_data)?;
    info!("Updated monospace metadata: {}", path.display());
    Ok(())
}

/// Parse a version string (YYYY-MM-DD or YYYY-MM-DD.N) or use today's date.
pub fn parse_version_string(value: Option<&str>) -> Result<(NaiveDate, String)> {
    let version = FontVersion::parse(value)?;
    Ok((version.date, version.tag))
}

/// Set version metadata on a font file.
pub fn set_version(path: &Path, target_date: NaiveDate, version_tag: &str) -> Result<()> {
    let data = read_font(path)?;
    let new_data = warpnine_font_metadata::set_version(&data, target_date, version_tag)?;
    write_font(path, new_data)?;

    let version = FontVersion::new(target_date, version_tag);
    info!(
        "{}: version -> {}, revision -> {}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        version.version_string(),
        version.revision()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_string_none() {
        let (date, _tag) = parse_version_string(None).unwrap();
        assert_eq!(date, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_parse_version_string_date() {
        let (date, tag) = parse_version_string(Some("2024-12-01")).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(tag, "2024-12-01");
    }

    #[test]
    fn test_parse_version_string_with_build() {
        let (date, tag) = parse_version_string(Some("2024-12-01.1")).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(tag, "2024-12-01.1");
    }

    #[test]
    fn test_parse_version_string_invalid() {
        assert!(parse_version_string(Some("invalid")).is_err());
    }
}
