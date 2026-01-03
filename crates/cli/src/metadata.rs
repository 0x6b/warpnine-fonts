//! Font metadata operations.

use std::path::Path;

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use log::info;
use read_fonts::TableProvider;
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::{head::Head, os2::Os2, post::Post},
    types::Fixed,
};

use crate::font_ops::{map_name_records, modify_font_in_place};

/// Monospace settings
const TARGET_WIDTH: i16 = 600;
const MONO_PROPORTION: u8 = 9;

/// Name table IDs
const NAME_ID_VERSION: u16 = 5;
const NAME_ID_UNIQUE_ID: u16 = 3;

/// Monospace metadata settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct MonospaceSettings {
    pub width: i16,
    pub panose_proportion: u8,
}

impl MonospaceSettings {
    pub const DEFAULT: Self = Self {
        width: TARGET_WIDTH,
        panose_proportion: MONO_PROPORTION,
    };

    /// Apply monospace settings to a font file.
    pub fn apply(&self, path: &Path) -> Result<()> {
        let width = self.width;
        let proportion = self.panose_proportion;

        modify_font_in_place(path, |font, builder| {
            if let Ok(post) = font.post() {
                let mut new_post: Post = post.to_owned_table();
                new_post.is_fixed_pitch = 1;
                builder.add_table(&new_post)?;
            }

            if let Ok(os2) = font.os2() {
                let mut new_os2: Os2 = os2.to_owned_table();
                new_os2.panose_10[3] = proportion;
                new_os2.x_avg_char_width = width;
                builder.add_table(&new_os2)?;
            }

            Ok(())
        })?;

        info!("Updated monospace metadata: {}", path.display());
        Ok(())
    }
}

/// Font version information.
#[derive(Debug, Clone)]
pub struct FontVersion {
    pub date: NaiveDate,
    pub tag: String,
}

impl FontVersion {
    /// Create a version from a date and tag.
    pub fn new(date: NaiveDate, tag: impl Into<String>) -> Self {
        Self { date, tag: tag.into() }
    }

    /// Parse a version string (YYYY-MM-DD or YYYY-MM-DD.N) or use today's date.
    pub fn parse(value: Option<&str>) -> Result<Self> {
        match value {
            None => {
                let today = chrono::Local::now().date_naive();
                Ok(Self::new(today, today.format("%Y-%m-%d").to_string()))
            }
            Some(v) => {
                // Try YYYY-MM-DD.N format first
                if let Some((date_part, build_num)) = v.rsplit_once('.')
                    && build_num.parse::<u32>().is_ok()
                    && let Ok(parsed) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                {
                    return Ok(Self::new(parsed, v));
                }

                // Try plain YYYY-MM-DD format
                if let Ok(parsed) = NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                    return Ok(Self::new(parsed, v));
                }

                Err(anyhow!("Invalid version '{v}'. Expected YYYY-MM-DD or YYYY-MM-DD.N."))
            }
        }
    }

    /// Get the version string (e.g., "Version 2024-01-15").
    pub fn version_string(&self) -> String {
        format!("Version {}", self.tag)
    }

    /// Compute font revision as YYYY.MMDD.
    pub fn revision(&self) -> Fixed {
        let year = self.date.year() as f64;
        let month_day = self.date.format("%m%d").to_string().parse::<f64>().unwrap() / 10000.0;
        Fixed::from_f64(year + month_day)
    }

    /// Apply this version to a font file.
    pub fn apply(&self, path: &Path) -> Result<()> {
        let version_string = self.version_string();
        let revision_value = self.revision();
        let version_tag = self.tag.clone();
        let mut updated = 0;

        modify_font_in_place(path, |font, builder| {
            if let Ok(head) = font.head() {
                let mut new_head: Head = head.to_owned_table();
                new_head.font_revision = revision_value;
                builder.add_table(&new_head)?;
            }

            let new_name = map_name_records(font, |name_id, current| {
                if name_id == NAME_ID_VERSION {
                    updated += 1;
                    Some(version_string.clone())
                } else if name_id == NAME_ID_UNIQUE_ID {
                    let parts: Vec<&str> = current
                        .split(';')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let new_parts = if !parts.is_empty() {
                        let mut new_parts: Vec<String> =
                            parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect();
                        new_parts.push(version_tag.clone());
                        new_parts
                    } else {
                        vec![version_tag.clone()]
                    };
                    updated += 1;
                    Some(new_parts.join("; "))
                } else {
                    None
                }
            })?;
            builder.add_table(&new_name)?;

            Ok(())
        })?;

        info!(
            "{}: version -> {version_string}, revision -> {revision_value}, updated {updated} records",
            path.file_name().unwrap_or_default().to_string_lossy()
        );

        Ok(())
    }
}

// Convenience functions for backward compatibility

pub fn set_monospace(path: &Path) -> Result<()> {
    MonospaceSettings::DEFAULT.apply(path)
}

pub fn parse_version_string(value: Option<&str>) -> Result<(NaiveDate, String)> {
    let version = FontVersion::parse(value)?;
    Ok((version.date, version.tag))
}

pub fn set_version(path: &Path, target_date: NaiveDate, version_tag: &str) -> Result<()> {
    FontVersion::new(target_date, version_tag).apply(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_string_none() {
        let version = FontVersion::parse(None).unwrap();
        assert_eq!(version.date, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_parse_version_string_date() {
        let version = FontVersion::parse(Some("2024-12-01")).unwrap();
        assert_eq!(version.date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(version.tag, "2024-12-01");
    }

    #[test]
    fn test_parse_version_string_with_build() {
        let version = FontVersion::parse(Some("2024-12-01.1")).unwrap();
        assert_eq!(version.date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(version.tag, "2024-12-01.1");
    }

    #[test]
    fn test_parse_version_string_invalid() {
        assert!(FontVersion::parse(Some("invalid")).is_err());
    }
}
