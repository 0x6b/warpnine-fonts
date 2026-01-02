use anyhow::Result;
use read_fonts::{FontRef, TableProvider};
use std::fs::read;
use std::fs::write;
use std::path::Path;
use write_fonts::{
    FontBuilder,
    tables::name::{Name, NameRecord},
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
    let data = read(path)?;
    let font = FontRef::new(&data)?;

    let mut builder = FontBuilder::new();

    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }

    if let Ok(name) = font.name() {
        let mut new_records: Vec<NameRecord> = Vec::new();

        for record in name.name_record() {
            let name_id = record.name_id().to_u16();
            let platform_id = record.platform_id();
            let encoding_id = record.encoding_id();
            let language_id = record.language_id();

            let current_string = match record.string(name.string_data()) {
                Ok(s) => s.chars().collect::<String>(),
                Err(_) => continue,
            };

            let new_string = match name_id {
                0 => naming.copyright(),
                1 => format!("{} {}", naming.family, naming.style),
                3 => naming.unique_id(),
                4 => naming.full_name(),
                6 => naming.postscript_name(),
                16 => naming.family.clone(),
                17 => naming.style.clone(),
                _ => current_string,
            };

            new_records.push(NameRecord::new(
                platform_id,
                encoding_id,
                language_id,
                read_fonts::types::NameId::new(name_id),
                new_string.into(),
            ));
        }

        let new_name = Name::new(new_records);
        builder.add_table(&new_name)?;
    }

    let new_font_data = builder.build();
    write(path, new_font_data)?;

    println!(
        "{}: set name to '{}' ({})",
        path.file_name().unwrap_or_default().to_string_lossy(),
        naming.full_name(),
        naming.postscript_name()
    );

    Ok(())
}
