use anyhow::{Context, Result};
use font_instancer::{instantiate, AxisLocation};
use read_fonts::{FontRef, TableProvider};
use std::fs;
use std::path::Path;
use write_fonts::{
    tables::name::{Name, NameRecord},
    FontBuilder,
};

#[derive(Clone, Copy)]
pub struct SansInstance {
    pub style: &'static str,
    pub wght: f32,
    pub italic: bool,
}

impl SansInstance {
    const fn new(style: &'static str, wght: f32, italic: bool) -> Self {
        Self { style, wght, italic }
    }

    pub fn mono(&self) -> f32 {
        0.0
    }

    pub fn casl(&self) -> f32 {
        0.0
    }

    pub fn slnt(&self) -> f32 {
        if self.italic { -15.0 } else { 0.0 }
    }

    pub fn crsv(&self) -> f32 {
        if self.italic { 1.0 } else { 0.5 }
    }
}

pub const SANS_INSTANCES: &[SansInstance] = &[
    SansInstance::new("Light", 300.0, false),
    SansInstance::new("LightItalic", 300.0, true),
    SansInstance::new("Regular", 400.0, false),
    SansInstance::new("Italic", 400.0, true),
    SansInstance::new("Medium", 500.0, false),
    SansInstance::new("MediumItalic", 500.0, true),
    SansInstance::new("SemiBold", 600.0, false),
    SansInstance::new("SemiBoldItalic", 600.0, true),
    SansInstance::new("Bold", 700.0, false),
    SansInstance::new("BoldItalic", 700.0, true),
    SansInstance::new("ExtraBold", 800.0, false),
    SansInstance::new("ExtraBoldItalic", 800.0, true),
    SansInstance::new("Black", 900.0, false),
    SansInstance::new("BlackItalic", 900.0, true),
];

fn update_sans_name_table(font_data: &[u8], family: &str, style: &str) -> Result<Vec<u8>> {
    let font = FontRef::new(font_data)?;
    let name = font.name()?;

    let postscript_family = family.replace(' ', "");

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
            1 => format!("{} {}", family, style),
            4 => format!("{} {}", family, style),
            6 => format!("{}-{}", postscript_family, style),
            16 => family.to_string(),
            17 => style.to_string(),
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

    let mut builder = FontBuilder::new();
    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }
    builder.add_table(&new_name)?;

    Ok(builder.build())
}

pub fn create_sans(input: &Path, output_dir: &Path) -> Result<()> {
    let data = fs::read(input).context("Failed to read input font")?;
    fs::create_dir_all(output_dir)?;

    let mut success = 0;

    for instance in SANS_INSTANCES {
        let output = output_dir.join(format!("WarpnineSans-{}.ttf", instance.style));
        println!("Creating {}", instance.style);

        let locations = vec![
            AxisLocation::new("MONO", instance.mono()),
            AxisLocation::new("CASL", instance.casl()),
            AxisLocation::new("wght", instance.wght),
            AxisLocation::new("slnt", instance.slnt()),
            AxisLocation::new("CRSV", instance.crsv()),
        ];

        let static_data = instantiate(&data, &locations)
            .with_context(|| format!("Failed to instantiate {}", instance.style))?;

        let final_data = update_sans_name_table(&static_data, "Warpnine Sans", instance.style)?;

        fs::write(&output, final_data)?;
        println!("  Created: {}", output.display());
        success += 1;
    }

    println!("Created {} sans fonts in {}/", success, output_dir.display());
    Ok(())
}
