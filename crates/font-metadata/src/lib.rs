//! Font metadata manipulation (monospace settings, versioning).

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use read_fonts::TableProvider;
use warpnine_font_ops::{map_name_records, rewrite_font};
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::{head::Head, os2::Os2, post::Post},
    types::Fixed,
};

/// Name table IDs.
const NAME_ID_UNIQUE_ID: u16 = 3;
const NAME_ID_VERSION: u16 = 5;

/// Monospace metadata settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct MonospaceSettings {
    /// Average character width.
    pub width: i16,
    /// PANOSE proportion value (9 = monospace).
    pub panose_proportion: u8,
}

impl MonospaceSettings {
    /// Default monospace settings: width=600, proportion=9.
    pub const DEFAULT: Self = Self { width: 600, panose_proportion: 9 };

    /// Apply monospace settings to font data.
    ///
    /// Modifies `post.is_fixed_pitch` and OS/2 table fields.
    pub fn apply(&self, data: &[u8]) -> Result<Vec<u8>> {
        let width = self.width;
        let proportion = self.panose_proportion;

        rewrite_font(data, |font, builder| {
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
        })
    }
}

/// Font version information.
#[derive(Debug, Clone)]
pub struct FontVersion {
    /// Version date.
    pub date: NaiveDate,
    /// Version tag (e.g., "2024-01-15" or "2024-01-15.1").
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

    /// Apply this version to font data.
    ///
    /// Updates `head.font_revision` and name IDs 3 (unique ID) and 5 (version).
    pub fn apply(&self, data: &[u8]) -> Result<Vec<u8>> {
        let version_string = self.version_string();
        let revision_value = self.revision();
        let version_tag = self.tag.clone();

        rewrite_font(data, |font, builder| {
            if let Ok(head) = font.head() {
                let mut new_head: Head = head.to_owned_table();
                new_head.font_revision = revision_value;
                builder.add_table(&new_head)?;
            }

            let new_name = map_name_records(font, |name_id, current| {
                if name_id == NAME_ID_VERSION {
                    Some(version_string.clone())
                } else if name_id == NAME_ID_UNIQUE_ID {
                    let parts: Vec<&str> =
                        current.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    let new_parts = if !parts.is_empty() {
                        let mut new_parts: Vec<String> =
                            parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect();
                        new_parts.push(version_tag.clone());
                        new_parts
                    } else {
                        vec![version_tag.clone()]
                    };
                    Some(new_parts.join("; "))
                } else {
                    None
                }
            })?;
            builder.add_table(&new_name)?;

            Ok(())
        })
    }
}

/// Apply default monospace settings to font data.
pub fn set_monospace(data: &[u8]) -> Result<Vec<u8>> {
    MonospaceSettings::DEFAULT.apply(data)
}

/// Apply version information to font data.
pub fn set_version(data: &[u8], date: NaiveDate, tag: &str) -> Result<Vec<u8>> {
    FontVersion::new(date, tag).apply(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_none() {
        let version = FontVersion::parse(None).unwrap();
        assert_eq!(version.date, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_parse_version_date() {
        let version = FontVersion::parse(Some("2024-12-01")).unwrap();
        assert_eq!(version.date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(version.tag, "2024-12-01");
    }

    #[test]
    fn test_parse_version_with_build() {
        let version = FontVersion::parse(Some("2024-12-01.1")).unwrap();
        assert_eq!(version.date, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(version.tag, "2024-12-01.1");
    }

    #[test]
    fn test_parse_version_invalid() {
        assert!(FontVersion::parse(Some("invalid")).is_err());
    }

    #[test]
    fn test_version_string() {
        let version = FontVersion::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "2024-01-15");
        assert_eq!(version.version_string(), "Version 2024-01-15");
    }

    #[test]
    fn test_revision() {
        let version = FontVersion::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "2024-01-15");
        let rev = version.revision();
        assert!((rev.to_f64() - 2024.0115).abs() < 0.0001);
    }

    #[test]
    fn test_monospace_default() {
        assert_eq!(MonospaceSettings::DEFAULT.width, 600);
        assert_eq!(MonospaceSettings::DEFAULT.panose_proportion, 9);
    }

    #[test]
    fn test_apply_monospace() {
        let data = font_test_data::CMAP12_FONT1;
        let result = set_monospace(data);
        assert!(result.is_ok());
    }
}
