use std::{
    fs::{read, write},
    path::Path,
};

use anyhow::{Context, Result};
use read_fonts::{
    FontRef, TableProvider, TableRef,
    tables::{
        gsub::{ChainedSequenceContext, SubstitutionSubtables},
        layout::ChainedSequenceRuleMarker,
    },
    types::{BigEndian, GlyphId, GlyphId16 as ReadGlyphId16},
};

fn find_glyph_id_for_name(font: &FontRef, name: &str) -> Option<u16> {
    let post = font.post().ok()?;
    let num_glyphs = font.maxp().ok()?.num_glyphs();
    for gid in 0..num_glyphs {
        if let Some(glyph_name) = post.glyph_name(ReadGlyphId16::new(gid))
            && glyph_name == name
        {
            return Some(gid);
        }
    }
    None
}

pub fn remove_grave_ligature(path: &Path) -> Result<bool> {
    let data = read(path).context("Failed to read font")?;
    let font = FontRef::new(&data).context("Failed to parse font")?;

    let gsub = match font.gsub() {
        Ok(gsub) => gsub,
        Err(_) => {
            println!("  No GSUB table found");
            return Ok(false);
        }
    };

    let grave_gid = match find_glyph_id_for_name(&font, "grave") {
        Some(gid) => gid,
        None => {
            println!("  No 'grave' glyph found");
            return Ok(false);
        }
    };

    let gsub_tag = read_fonts::types::Tag::new(b"GSUB");
    let gsub_record = font
        .table_directory
        .table_records()
        .iter()
        .find(|r| r.tag() == gsub_tag)
        .context("GSUB table not found in directory")?;
    let gsub_offset = gsub_record.offset() as usize;

    let lookup_list = gsub.lookup_list().context("Failed to read lookup list")?;

    for (lookup_idx, lookup_result) in lookup_list.lookups().iter().enumerate() {
        let lookup = match lookup_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        if lookup.lookup_type() != 6 {
            continue;
        }

        let subtables = match lookup.subtables() {
            Ok(SubstitutionSubtables::ChainContextual(s)) => s,
            _ => continue,
        };

        for subtable_result in subtables.iter() {
            let subtable = match subtable_result {
                Ok(ChainedSequenceContext::Format1(s)) => s,
                _ => continue,
            };

            let coverage = match subtable.coverage() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let grave_coverage_idx = match coverage.get(GlyphId::new(grave_gid as u32)) {
                Some(idx) => idx,
                None => continue,
            };

            let rule_sets = subtable.chained_seq_rule_sets();

            let rule_set = match rule_sets.get(grave_coverage_idx as usize) {
                Some(Ok(rs)) => rs,
                _ => continue,
            };

            for rule_result in rule_set.chained_seq_rules().iter() {
                let rule: TableRef<'_, ChainedSequenceRuleMarker> = match rule_result {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let input_seq = rule.input_sequence();
                if input_seq.len() != 2 {
                    continue;
                }

                let all_grave = input_seq
                    .iter()
                    .all(|g: &BigEndian<ReadGlyphId16>| g.get().to_u32() == grave_gid as u32);

                if !all_grave {
                    continue;
                }

                let lookup_count = rule.seq_lookup_count();
                if lookup_count == 0 {
                    continue;
                }

                println!(
                    "  Found three-backtick pattern in Lookup {lookup_idx} (rule has {lookup_count} lookup records)"
                );
            }
        }
    }

    let grave_be = (grave_gid as u16).to_be_bytes();
    let mut modified_data = data.clone();
    let mut modifications = 0;

    let gsub_data = font.table_data(gsub_tag).context("Failed to get GSUB data")?;
    let gsub_bytes = gsub_data.as_ref();

    let input_count_pattern = 0x0003u16.to_be_bytes();

    for i in 0..gsub_bytes.len().saturating_sub(20) {
        if gsub_bytes[i..i + 2] != input_count_pattern {
            continue;
        }

        if i + 6 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 2..i + 4] != grave_be || gsub_bytes[i + 4..i + 6] != grave_be {
            continue;
        }

        if i + 8 > gsub_bytes.len() {
            continue;
        }
        let lookahead_count = u16::from_be_bytes([gsub_bytes[i + 6], gsub_bytes[i + 7]]) as usize;

        let seq_lookup_count_offset = i + 8 + lookahead_count * 2;
        if seq_lookup_count_offset + 2 > gsub_bytes.len() {
            continue;
        }

        let seq_lookup_count = u16::from_be_bytes([
            gsub_bytes[seq_lookup_count_offset],
            gsub_bytes[seq_lookup_count_offset + 1],
        ]);

        if seq_lookup_count > 0 && seq_lookup_count < 10 {
            println!(
                "  Patching seq_lookup_count at GSUB offset 0x{seq_lookup_count_offset:x} (was {seq_lookup_count})"
            );

            let file_offset = gsub_offset + seq_lookup_count_offset;
            modified_data[file_offset] = 0;
            modified_data[file_offset + 1] = 0;
            modifications += 1;
        }
    }

    if modifications == 0 {
        println!("  No matching patterns found to patch");
        return Ok(false);
    }

    let backtrack_one_pattern = 0x0001u16.to_be_bytes();

    for i in 0..gsub_bytes.len().saturating_sub(20) {
        if gsub_bytes[i..i + 2] != backtrack_one_pattern {
            continue;
        }

        if i + 4 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 2..i + 4] != grave_be {
            continue;
        }

        if i + 6 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 4..i + 6] != input_count_pattern {
            continue;
        }

        if i + 10 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 6..i + 8] != grave_be || gsub_bytes[i + 8..i + 10] != grave_be {
            continue;
        }

        if i + 12 > gsub_bytes.len() {
            continue;
        }
        let lookahead_count = u16::from_be_bytes([gsub_bytes[i + 10], gsub_bytes[i + 11]]) as usize;

        let seq_lookup_count_offset = i + 12 + lookahead_count * 2;
        if seq_lookup_count_offset + 2 > gsub_bytes.len() {
            continue;
        }

        let seq_lookup_count = u16::from_be_bytes([
            gsub_bytes[seq_lookup_count_offset],
            gsub_bytes[seq_lookup_count_offset + 1],
        ]);

        if seq_lookup_count > 0 && seq_lookup_count < 10 {
            println!(
                "  Patching seq_lookup_count at GSUB offset 0x{seq_lookup_count_offset:x} (was {seq_lookup_count}, backtrack pattern)"
            );

            let file_offset = gsub_offset + seq_lookup_count_offset;
            if modified_data[file_offset] != 0 || modified_data[file_offset + 1] != 0 {
                modified_data[file_offset] = 0;
                modified_data[file_offset + 1] = 0;
                modifications += 1;
            }
        }
    }

    write(path, &modified_data).context("Failed to write modified font")?;
    println!("  Saved modified font ({modifications} patches applied)");

    Ok(true)
}
