use std::path::Path;

use anyhow::{Result, anyhow};
use chrono::{Datelike, NaiveDate};
use read_fonts::TableProvider;
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::{head::Head, os2::Os2, post::Post},
    types::Fixed,
};

use crate::font_ops::{map_name_records, modify_font_in_place};

// Monospace settings
const TARGET_WIDTH: i16 = 600;
const MONO_PROPORTION: u8 = 9; // PANOSE value for monospaced

// Name table IDs
const NAME_ID_VERSION: u16 = 5;
const NAME_ID_UNIQUE_ID: u16 = 3;

/// Set monospace flags in a font file.
pub fn set_monospace(path: &Path) -> Result<()> {
    modify_font_in_place(path, |font, builder| {
        if let Ok(post) = font.post() {
            let mut new_post: Post = post.to_owned_table();
            new_post.is_fixed_pitch = 1;
            builder.add_table(&new_post)?;
        }

        if let Ok(os2) = font.os2() {
            let mut new_os2: Os2 = os2.to_owned_table();
            new_os2.panose_10[3] = MONO_PROPORTION;
            new_os2.x_avg_char_width = TARGET_WIDTH;
            builder.add_table(&new_os2)?;
        }

        Ok(())
    })?;

    println!("Updated monospace metadata: {}", path.display());
    Ok(())
}

/// Parse a version string or return today's date.
pub fn parse_version_string(value: Option<&str>) -> Result<(NaiveDate, String)> {
    match value {
        None => {
            let today = chrono::Local::now().date_naive();
            Ok((today, today.format("%Y-%m-%d").to_string()))
        }
        Some(v) => {
            // Try YYYY-MM-DD.N format first
            if let Some((date_part, build_num)) = v.rsplit_once('.')
                && build_num.parse::<u32>().is_ok()
                && let Ok(parsed) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
            {
                return Ok((parsed, v.to_string()));
            }

            // Try plain YYYY-MM-DD format
            if let Ok(parsed) = NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                return Ok((parsed, v.to_string()));
            }

            Err(anyhow!("Invalid version '{v}'. Expected YYYY-MM-DD or YYYY-MM-DD.N."))
        }
    }
}

/// Set version date in a font file.
pub fn set_version(path: &Path, target_date: NaiveDate, version_tag: &str) -> Result<()> {
    let version_string = format!("Version {version_tag}");
    let revision_value = compute_font_revision(target_date);
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
                    new_parts.push(version_tag.to_string());
                    new_parts
                } else {
                    vec![version_tag.to_string()]
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

    println!(
        "{}: version -> {version_string}, revision -> {revision_value}, updated {updated} records",
        path.file_name().unwrap_or_default().to_string_lossy()
    );

    Ok(())
}

/// Compute font revision as YYYY.MMDD
fn compute_font_revision(date: NaiveDate) -> Fixed {
    let year = date.year() as f64;
    let month_day = date.format("%m%d").to_string().parse::<f64>().unwrap() / 10000.0;
    write_fonts::types::Fixed::from_f64(year + month_day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_string_none() {
        let (date, tag) = parse_version_string(None).unwrap();
        assert_eq!(date, chrono::Local::now().date_naive());
        assert_eq!(tag, date.format("%Y-%m-%d").to_string());
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
