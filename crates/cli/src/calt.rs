use anyhow::{Context, Result};
use read_fonts::tables::gsub::{FeatureList, ScriptList};
use read_fonts::tables::layout::LangSys;
use read_fonts::{FontRef, TableProvider};
use std::fs::read;
use std::fs::write;
use std::path::Path;
use write_fonts::FontBuilder;

pub fn fix_calt_registration(path: &Path) -> Result<()> {
    let data = read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let font = FontRef::new(&data)?;

    let gsub = match font.gsub() {
        Ok(gsub) => gsub,
        Err(_) => {
            println!("{}: no GSUB table, skipping", path.display());
            return Ok(());
        }
    };

    let feature_list = match gsub.feature_list() {
        Ok(fl) => fl,
        Err(_) => return Ok(()),
    };

    let script_list = match gsub.script_list() {
        Ok(sl) => sl,
        Err(_) => return Ok(()),
    };

    let (calt_indices, rclt_indices) = find_feature_indices(&feature_list);

    if calt_indices.is_empty() {
        println!(
            "{}: no calt feature found",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        return Ok(());
    }

    let gsub_data = font
        .table_data(read_fonts::types::Tag::new(b"GSUB"))
        .unwrap();
    let mut gsub_bytes = gsub_data.as_bytes().to_vec();

    let modifications =
        collect_modifications(&script_list, &gsub_bytes, &calt_indices, &rclt_indices)?;

    if modifications.is_empty() {
        println!(
            "{}: calt already registered to all scripts",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        return Ok(());
    }

    apply_modifications(&mut gsub_bytes, &modifications);

    let mut builder = FontBuilder::new();
    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if tag == read_fonts::types::Tag::new(b"GSUB") {
            builder.add_raw(tag, &gsub_bytes);
        } else if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }

    let new_font_data = builder.build();
    write(path, new_font_data)?;

    println!(
        "{}: registered calt/rclt to {} scripts",
        path.file_name().unwrap_or_default().to_string_lossy(),
        modifications.len()
    );

    Ok(())
}

fn find_feature_indices(feature_list: &FeatureList) -> (Vec<u16>, Vec<u16>) {
    let mut calt_indices = Vec::new();
    let mut rclt_indices = Vec::new();

    for (i, record) in feature_list.feature_records().iter().enumerate() {
        let tag = record.feature_tag();
        if tag == read_fonts::types::Tag::new(b"calt") {
            calt_indices.push(i as u16);
        } else if tag == read_fonts::types::Tag::new(b"rclt") {
            rclt_indices.push(i as u16);
        }
    }

    (calt_indices, rclt_indices)
}

struct LangSysModification {
    offset: usize,
    new_feature_count: u16,
    new_feature_indices: Vec<u16>,
}

fn collect_modifications(
    script_list: &ScriptList,
    gsub_bytes: &[u8],
    calt_indices: &[u16],
    rclt_indices: &[u16],
) -> Result<Vec<LangSysModification>> {
    let mut modifications = Vec::new();

    let script_list_offset = read_u16_be(gsub_bytes, 4) as usize;

    for script_record in script_list.script_records() {
        let script_offset = script_list_offset + script_record.script_offset().to_u32() as usize;

        let script = script_record.script(script_list.offset_data())?;

        if let Some(Ok(lang_sys)) = script.default_lang_sys() {
            let default_offset = read_u16_be(gsub_bytes, script_offset) as usize;
            if default_offset > 0 {
                let lang_sys_offset = script_offset + default_offset;
                if let Some(modification) =
                    check_lang_sys(&lang_sys, lang_sys_offset, calt_indices, rclt_indices)
                {
                    modifications.push(modification);
                }
            }
        }

        for lang_sys_record in script.lang_sys_records() {
            let lang_sys_rel_offset = lang_sys_record.lang_sys_offset().to_u32() as usize;
            let lang_sys_offset = script_offset + lang_sys_rel_offset;

            let lang_sys = lang_sys_record.lang_sys(script.offset_data())?;
            if let Some(modification) =
                check_lang_sys(&lang_sys, lang_sys_offset, calt_indices, rclt_indices)
            {
                modifications.push(modification);
            }
        }
    }

    Ok(modifications)
}

fn check_lang_sys(
    lang_sys: &LangSys,
    offset: usize,
    calt_indices: &[u16],
    rclt_indices: &[u16],
) -> Option<LangSysModification> {
    let current_indices: Vec<u16> = lang_sys.feature_indices().iter().map(|i| i.get()).collect();

    let mut new_indices = current_indices.clone();
    let mut modified = false;

    for &calt_idx in calt_indices {
        if !new_indices.contains(&calt_idx) {
            let insert_pos = if new_indices.len() > 1 {
                1
            } else {
                new_indices.len()
            };
            new_indices.insert(insert_pos, calt_idx);
            modified = true;
        }
    }

    for &rclt_idx in rclt_indices {
        if !new_indices.contains(&rclt_idx) {
            new_indices.push(rclt_idx);
            modified = true;
        }
    }

    if modified {
        Some(LangSysModification {
            offset,
            new_feature_count: new_indices.len() as u16,
            new_feature_indices: new_indices,
        })
    } else {
        None
    }
}

fn apply_modifications(gsub_bytes: &mut [u8], modifications: &[LangSysModification]) {
    for modification in modifications {
        let count_offset = modification.offset + 4;
        write_u16_be(gsub_bytes, count_offset, modification.new_feature_count);

        for (i, &idx) in modification.new_feature_indices.iter().enumerate() {
            let idx_offset = modification.offset + 6 + i * 2;
            if idx_offset + 2 <= gsub_bytes.len() {
                write_u16_be(gsub_bytes, idx_offset, idx);
            }
        }
    }
}

fn read_u16_be(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([data[offset], data[offset + 1]])
}

fn write_u16_be(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_be_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}
