use std::path::Path;

use anyhow::Result;
use log::info;
use rayon::prelude::*;
use read_fonts::FontRef;
use warpnine_font_ops::{map_name_records, rewrite_font};
use write_fonts::FontBuilder;

use crate::io::{check_results, glob_fonts, transform_font_in_place};

const COPYRIGHT_TEMPLATE: &str = "Copyright 2020 The Recursive Project Authors (https://github.com/arrowtype/recursive). \
Copyright 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'. ";

#[derive(Debug, Clone)]
pub struct FontNaming {
    pub family: String,
    pub style: String,
    pub postscript_family: Option<String>,
    pub copyright_extra: Option<String>,
}

impl FontNaming {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.family, self.style)
    }

    pub fn postscript_name(&self) -> String {
        let base = self
            .postscript_family
            .clone()
            .unwrap_or_else(|| self.family.replace(' ', ""));
        format!("{base}-{}", self.style.replace(' ', ""))
    }

    pub fn unique_id(&self) -> String {
        let ps_name = self.postscript_name().replace('-', "");
        format!("1.0;WARPNINE;{ps_name}")
    }

    pub fn copyright(&self) -> String {
        match &self.copyright_extra {
            Some(extra) => format!("{COPYRIGHT_TEMPLATE}{extra}"),
            None => COPYRIGHT_TEMPLATE.to_string(),
        }
    }
}

pub fn set_name(path: &Path, naming: &FontNaming) -> Result<()> {
    let naming = naming.clone();
    transform_font_in_place(path, |data| {
        rewrite_font(data, |font: &FontRef, builder: &mut FontBuilder| {
            let name = map_name_records(font, |name_id, _current| match name_id {
                0 => Some(naming.copyright()),
                1 => Some(format!("{} {}", naming.family, naming.style)),
                3 => Some(naming.unique_id()),
                4 => Some(naming.full_name()),
                6 => Some(naming.postscript_name()),
                16 => Some(naming.family.clone()),
                17 => Some(naming.style.clone()),
                _ => None,
            })?;
            builder.add_table(&name)?;
            Ok(())
        })
    })?;

    info!(
        "{}: set name to '{}' ({})",
        path.file_name().unwrap_or_default().to_string_lossy(),
        naming.full_name(),
        naming.postscript_name()
    );

    Ok(())
}

pub fn set_names_for_pattern(
    dir: &Path,
    pattern: &str,
    family: &str,
    ps_family: &str,
    copyright_extra: &str,
    strip_prefix: &str,
) -> Result<usize> {
    let fonts = glob_fonts(dir, pattern)?;
    if fonts.is_empty() {
        return Ok(0);
    }

    println!("  Setting names for {} fonts ({pattern})...", fonts.len());
    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let style = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.strip_prefix(strip_prefix).unwrap_or(s))
                .unwrap_or_default()
                .to_string();

            let naming = FontNaming {
                family: family.to_string(),
                style,
                postscript_family: Some(ps_family.to_string()),
                copyright_extra: Some(copyright_extra.to_string()),
            };

            set_name(path, &naming)
        })
        .collect();

    check_results(&results, &format!("set names ({pattern})"))?;
    Ok(fonts.len())
}
