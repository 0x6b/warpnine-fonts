use std::path::Path;

use anyhow::Result;

use crate::font_ops::{map_name_records, modify_font_in_place};

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
    modify_font_in_place(path, |font, builder| {
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
    })?;

    println!(
        "{}: set name to '{}' ({})",
        path.file_name().unwrap_or_default().to_string_lossy(),
        naming.full_name(),
        naming.postscript_name()
    );

    Ok(())
}
