use anyhow::{Context, Result};
use read_fonts::tables::gsub::{ChainedSequenceContext, SubstitutionSubtables};
use read_fonts::tables::layout::ChainedSequenceRuleMarker;
use read_fonts::types::{BigEndian, GlyphId, GlyphId16 as ReadGlyphId16};
use read_fonts::{FontRef, TableProvider, TableRef};
use std::fs::read;
use std::fs::write;
use std::path::Path;

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

/// Remove the three-backtick ligature from a font.
///
/// The ligature in Recursive fonts uses:
/// - Type 6 Format 1 chaining contextual substitution
/// - Rules that match grave+grave+grave and call a ligature lookup
///
/// We clear the SubstLookupRecord by setting seq_lookup_count to 0.
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

    // Get the raw GSUB table data and its offset in the file
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

        // Type 6 is chaining contextual
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

            // Check if coverage contains grave
            let coverage = match subtable.coverage() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let grave_coverage_idx = match coverage.get(GlyphId::new(grave_gid as u32)) {
                Some(idx) => idx,
                None => continue,
            };

            // Get the rule sets
            let rule_sets = subtable.chained_seq_rule_sets();

            // Get the rule set for grave
            let rule_set = match rule_sets.get(grave_coverage_idx as usize) {
                Some(Ok(rs)) => rs,
                _ => continue,
            };

            // Check each rule
            for rule_result in rule_set.chained_seq_rules().iter() {
                let rule: TableRef<'_, ChainedSequenceRuleMarker> = match rule_result {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                // Check if this is a grave+grave+grave pattern
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

                // Check if there are SubstLookupRecords to clear
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

    // Since calculating exact offsets through the nested structure is complex,
    // let's search for the specific byte pattern and patch it.
    // The grave glyph ID is typically around 0x0265 (613).
    // We're looking for: grave grave (as input sequence), followed by seq_lookup_count > 0

    let grave_be = (grave_gid as u16).to_be_bytes();
    let mut modified_data = data.clone();
    let mut modifications = 0;

    // Search within GSUB table for the pattern
    let gsub_data = font
        .table_data(gsub_tag)
        .context("Failed to get GSUB data")?;
    let gsub_bytes = gsub_data.as_ref();

    // We need to find ChainedSequenceRule structures that have:
    // - input_glyph_count = 3 (meaning 2 glyphs in input_sequence since first is implicit)
    // - input_sequence = [grave, grave]
    // - seq_lookup_count > 0

    // Pattern to search for within GSUB:
    // ... [input_glyph_count=0x0003] [grave] [grave] [lookahead_count] ... [seq_lookup_count > 0]

    let input_count_pattern = 0x0003u16.to_be_bytes();

    for i in 0..gsub_bytes.len().saturating_sub(20) {
        // Check for input_glyph_count = 3
        if gsub_bytes[i..i + 2] != input_count_pattern {
            continue;
        }

        // Check for grave, grave following
        if i + 6 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 2..i + 4] != grave_be || gsub_bytes[i + 4..i + 6] != grave_be {
            continue;
        }

        // Read lookahead_glyph_count at i+6
        if i + 8 > gsub_bytes.len() {
            continue;
        }
        let lookahead_count = u16::from_be_bytes([gsub_bytes[i + 6], gsub_bytes[i + 7]]) as usize;

        // seq_lookup_count is at i + 8 + lookahead_count * 2
        let seq_lookup_count_offset = i + 8 + lookahead_count * 2;
        if seq_lookup_count_offset + 2 > gsub_bytes.len() {
            continue;
        }

        let seq_lookup_count = u16::from_be_bytes([
            gsub_bytes[seq_lookup_count_offset],
            gsub_bytes[seq_lookup_count_offset + 1],
        ]);

        if seq_lookup_count > 0 && seq_lookup_count < 10 {
            // Sanity check
            println!(
                "  Patching seq_lookup_count at GSUB offset 0x{seq_lookup_count_offset:x} (was {seq_lookup_count})"
            );

            // Patch in the file
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

    // We also need to look for backtrack patterns like [backtrack=grave] [input_count=3] [grave] [grave]
    // Pattern: [backtrack_count=1] [grave] [input_count=3] [grave] [grave] ...

    let backtrack_one_pattern = 0x0001u16.to_be_bytes();

    for i in 0..gsub_bytes.len().saturating_sub(20) {
        // Check for backtrack_glyph_count = 1
        if gsub_bytes[i..i + 2] != backtrack_one_pattern {
            continue;
        }

        // Check for grave in backtrack
        if i + 4 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 2..i + 4] != grave_be {
            continue;
        }

        // Check for input_glyph_count = 3
        if i + 6 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 4..i + 6] != input_count_pattern {
            continue;
        }

        // Check for grave, grave in input
        if i + 10 > gsub_bytes.len() {
            continue;
        }
        if gsub_bytes[i + 6..i + 8] != grave_be || gsub_bytes[i + 8..i + 10] != grave_be {
            continue;
        }

        // lookahead_count at i+10
        if i + 12 > gsub_bytes.len() {
            continue;
        }
        let lookahead_count = u16::from_be_bytes([gsub_bytes[i + 10], gsub_bytes[i + 11]]) as usize;

        // seq_lookup_count at i + 12 + lookahead_count * 2
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
