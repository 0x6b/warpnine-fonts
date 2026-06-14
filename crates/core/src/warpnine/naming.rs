use std::path::Path;

use anyhow::{Result, anyhow};
use log::info;
use rayon::prelude::*;
use read_fonts::FontRef;
use warpnine_font_ops::{apply_style, map_name_records, rewrite_font};
use write_fonts::FontBuilder;

use crate::{
    io::{check_results, glob_fonts, transform_font_in_place},
    styles::Style,
};

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

/// Apply RIBBI naming and OS/2/head style bits to a single static instance.
///
/// Sets name IDs 1/2/4/6/16/17 plus copyright (0) and unique ID (3), and the
/// bold/italic/regular bits in OS/2 `fsSelection` and head `macStyle`.
pub fn set_ribbi_names(
    path: &Path,
    family: &str,
    ps_family: &str,
    copyright_extra: &str,
    style: &Style,
) -> Result<()> {
    let names = style.ribbi_names(family, ps_family);
    let bits = style.style_bits();
    let copyright = format!("{COPYRIGHT_TEMPLATE}{copyright_extra}");
    let unique_id = format!("1.0;WARPNINE;{}", names.postscript.replace('-', ""));
    let log_full = names.full_name.clone();
    let log_ps = names.postscript.clone();

    transform_font_in_place(path, move |data| {
        let data = apply_style(data, &names, &bits)?;
        rewrite_font(&data, |font: &FontRef, builder: &mut FontBuilder| {
            let name = map_name_records(font, |name_id, _current| match name_id {
                0 => Some(copyright.clone()),
                3 => Some(unique_id.clone()),
                _ => None,
            })?;
            builder.add_table(&name)?;
            Ok(())
        })
    })?;

    info!(
        "{}: set name to '{log_full}' ({log_ps})",
        path.file_name().unwrap_or_default().to_string_lossy(),
    );

    Ok(())
}

/// Apply RIBBI naming to every static instance matching `pattern`.
///
/// The style for each file is resolved from `styles` by matching the filename
/// suffix after `strip_prefix` against `Style::name`.
pub fn set_ribbi_names_for_pattern(
    dir: &Path,
    pattern: &str,
    family: &str,
    ps_family: &str,
    copyright_extra: &str,
    strip_prefix: &str,
    styles: &[Style],
) -> Result<usize> {
    let fonts: Vec<_> = glob_fonts(dir, pattern)?
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| !s.contains("-VF"))
        })
        .collect();
    if fonts.is_empty() {
        return Ok(0);
    }

    println!("  Setting names for {} fonts ({pattern})...", fonts.len());
    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let style_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.strip_prefix(strip_prefix).unwrap_or(s))
                .unwrap_or_default();

            let style = styles
                .iter()
                .find(|s| s.name == style_name)
                .ok_or_else(|| anyhow!("Unknown style '{style_name}' for {pattern}"))?;

            set_ribbi_names(path, family, ps_family, copyright_extra, style)
        })
        .collect();

    check_results(&results, &format!("set names ({pattern})"))?;
    Ok(fonts.len())
}
