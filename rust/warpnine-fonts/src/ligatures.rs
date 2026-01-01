use anyhow::{Context, Result};
use read_fonts::tables::gsub::SubstitutionSubtables;
use read_fonts::types::{GlyphId, GlyphId16 as ReadGlyphId16};
use read_fonts::{FontRef, TableProvider};
use std::fs;
use std::path::Path;
use write_fonts::tables::gsub::{SingleSubst, SubstitutionChainContext, SubstitutionLookup};
use write_fonts::tables::layout::{ChainedSequenceContext, ChainedSequenceContextFormat3, Lookup, LookupFlag};
use write_fonts::types::GlyphId16;
use write_fonts::FontBuilder;

fn find_glyph_id_for_name(font: &FontRef, name: &str) -> Option<GlyphId> {
    let post = font.post().ok()?;
    for gid in 0..font.maxp().ok()?.num_glyphs() {
        if let Some(glyph_name) = post.glyph_name(ReadGlyphId16::new(gid)) {
            if glyph_name == name {
                return Some(GlyphId::new(gid as u32));
            }
        }
    }
    None
}

fn coverage_contains_glyph(
    coverage: &read_fonts::tables::layout::CoverageTable,
    glyph: GlyphId,
) -> bool {
    coverage.iter().any(|g| g == glyph)
}

fn get_single_subst_mapping(
    subtable: &read_fonts::tables::gsub::SingleSubst,
    glyph: GlyphId,
) -> Option<GlyphId> {
    match subtable {
        read_fonts::tables::gsub::SingleSubst::Format1(s) => {
            let cov = s.coverage().ok()?;
            if cov.get(glyph).is_some() {
                let delta = s.delta_glyph_id();
                let new_gid = (glyph.to_u32() as i32 + delta as i32) as u32;
                Some(GlyphId::new(new_gid))
            } else {
                None
            }
        }
        read_fonts::tables::gsub::SingleSubst::Format2(s) => {
            let cov = s.coverage().ok()?;
            let idx = cov.get(glyph)?;
            let subs = s.substitute_glyph_ids();
            subs.get(idx as usize).map(|g| GlyphId::new(g.get().to_u32()))
        }
    }
}

fn get_single_subst_coverage<'a>(
    subtable: &'a read_fonts::tables::gsub::SingleSubst<'a>,
) -> Option<read_fonts::tables::layout::CoverageTable<'a>> {
    match subtable {
        read_fonts::tables::gsub::SingleSubst::Format1(s) => s.coverage().ok(),
        read_fonts::tables::gsub::SingleSubst::Format2(s) => s.coverage().ok(),
    }
}

pub fn remove_grave_ligature(path: &Path) -> Result<bool> {
    let data = fs::read(path).context("Failed to read font")?;
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

    let grave3_gid = find_glyph_id_for_name(&font, "grave_grave_grave.code");

    let lookup_list = gsub.lookup_list().context("Failed to read lookup list")?;
    let lookups: Vec<_> = lookup_list.lookups().iter().collect();

    let mut type6_to_clear = Vec::new();
    let mut type1_to_modify = Vec::new();

    for (lookup_idx, lookup_result) in lookups.iter().enumerate() {
        let lookup = match lookup_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        match lookup.subtables() {
            Ok(SubstitutionSubtables::ChainContextual(subtables)) => {
                for subtable_result in subtables.iter() {
                    if let Ok(
                        read_fonts::tables::gsub::ChainedSequenceContext::Format3(subtable),
                    ) = subtable_result
                    {
                        let input_coverages: Vec<_> =
                            subtable.input_coverages().iter().flatten().collect();
                        let lookahead_coverages: Vec<_> =
                            subtable.lookahead_coverages().iter().flatten().collect();

                        if !input_coverages.is_empty()
                            && coverage_contains_glyph(&input_coverages[0], grave_gid)
                            && lookahead_coverages.len() == 2
                            && coverage_contains_glyph(&lookahead_coverages[0], grave_gid)
                            && coverage_contains_glyph(&lookahead_coverages[1], grave_gid)
                        {
                            println!(
                                "  Found three-backtick pattern in Lookup {}",
                                lookup_idx
                            );
                            type6_to_clear.push(lookup_idx);
                        }
                    }
                }
            }
            Ok(SubstitutionSubtables::Single(subtables)) => {
                if let Some(grave3) = grave3_gid {
                    for subtable_result in subtables.iter() {
                        if let Ok(subtable) = subtable_result {
                            if let Some(mapped) = get_single_subst_mapping(&subtable, grave_gid) {
                                if mapped == grave3 {
                                    println!(
                                        "  Found grave → grave_grave_grave.code in Lookup {}",
                                        lookup_idx
                                    );
                                    type1_to_modify.push((lookup_idx, subtable.clone()));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if type6_to_clear.is_empty() && type1_to_modify.is_empty() {
        println!("  No three-backtick ligature found");
        return Ok(false);
    }

    let mut new_lookups: Vec<SubstitutionLookup> = Vec::new();

    for (lookup_idx, lookup_result) in lookups.iter().enumerate() {
        let lookup = match lookup_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        if type6_to_clear.contains(&lookup_idx) {
            let empty_subtable = ChainedSequenceContextFormat3 {
                backtrack_coverages: vec![],
                input_coverages: vec![],
                lookahead_coverages: vec![],
                seq_lookup_records: vec![],
            };
            let inner = Lookup::<SubstitutionChainContext>::new(
                LookupFlag::from_bits_truncate(lookup.lookup_flag().to_bits()),
                vec![ChainedSequenceContext::Format3(empty_subtable).into()],
            );
            new_lookups.push(SubstitutionLookup::ChainContextual(inner));
        } else if type1_to_modify.iter().any(|(idx, _)| *idx == lookup_idx) {
            let (_, subtable) = type1_to_modify
                .iter()
                .find(|(idx, _)| *idx == lookup_idx)
                .unwrap();

            if let Some(cov) = get_single_subst_coverage(subtable) {
                let mut new_glyphs = Vec::new();
                let mut new_substitutes = Vec::new();

                for input_gid in cov.iter() {
                    if input_gid == grave_gid {
                        if let Some(grave3) = grave3_gid {
                            if let Some(mapped) = get_single_subst_mapping(subtable, grave_gid) {
                                if mapped == grave3 {
                                    continue;
                                }
                            }
                        }
                    }
                    if let Some(out_gid) =
                        get_single_subst_mapping(subtable, GlyphId::new(input_gid.to_u32()))
                    {
                        new_glyphs.push(GlyphId16::new(input_gid.to_u32() as u16));
                        new_substitutes.push(GlyphId16::new(out_gid.to_u32() as u16));
                    }
                }

                if !new_glyphs.is_empty() {
                    let new_coverage =
                        write_fonts::tables::layout::CoverageTable::format_1(new_glyphs);
                    let new_subtable =
                        SingleSubst::format_2(new_coverage, new_substitutes);
                    let inner = Lookup::<SingleSubst>::new(
                        LookupFlag::from_bits_truncate(lookup.lookup_flag().to_bits()),
                        vec![new_subtable],
                    );
                    new_lookups.push(SubstitutionLookup::Single(inner));
                }
            }
        } else {
            match lookup.subtables() {
                Ok(SubstitutionSubtables::Single(subtables)) => {
                    let mut write_subtables = Vec::new();
                    for subtable_result in subtables.iter() {
                        if let Ok(subtable) = subtable_result {
                            if let Some(cov) = get_single_subst_coverage(&subtable) {
                                let glyphs: Vec<_> = cov
                                    .iter()
                                    .map(|g| GlyphId16::new(g.to_u32() as u16))
                                    .collect();
                                let subs: Vec<_> = cov
                                    .iter()
                                    .filter_map(|g| {
                                        get_single_subst_mapping(&subtable, GlyphId::new(g.to_u32()))
                                    })
                                    .map(|g| GlyphId16::new(g.to_u32() as u16))
                                    .collect();
                                if glyphs.len() == subs.len() && !glyphs.is_empty() {
                                    let new_cov =
                                        write_fonts::tables::layout::CoverageTable::format_1(
                                            glyphs,
                                        );
                                    write_subtables
                                        .push(SingleSubst::format_2(new_cov, subs));
                                }
                            }
                        }
                    }
                    if !write_subtables.is_empty() {
                        let inner = Lookup::<SingleSubst>::new(
                            LookupFlag::from_bits_truncate(lookup.lookup_flag().to_bits()),
                            write_subtables,
                        );
                        new_lookups.push(SubstitutionLookup::Single(inner));
                    }
                }
                Ok(SubstitutionSubtables::ChainContextual(_)) => {
                    let empty_subtable = ChainedSequenceContextFormat3 {
                        backtrack_coverages: vec![],
                        input_coverages: vec![],
                        lookahead_coverages: vec![],
                        seq_lookup_records: vec![],
                    };
                    let inner = Lookup::<SubstitutionChainContext>::new(
                        LookupFlag::from_bits_truncate(lookup.lookup_flag().to_bits()),
                        vec![ChainedSequenceContext::Format3(empty_subtable).into()],
                    );
                    new_lookups.push(SubstitutionLookup::ChainContextual(inner));
                }
                _ => {}
            }
        }
    }

    let new_lookup_list = write_fonts::tables::gsub::SubstitutionLookupList::new(new_lookups);
    let new_gsub = rebuild_gsub(&gsub, new_lookup_list)?;

    let mut builder = FontBuilder::new();
    for record in font.table_directory.table_records() {
        let tag = record.tag();
        if tag == read_fonts::types::Tag::new(b"GSUB") {
            continue;
        }
        if let Some(table_data) = font.table_data(tag) {
            builder.add_raw(tag, table_data);
        }
    }
    builder.add_table(&new_gsub)?;

    let output = builder.build();
    fs::write(path, output).context("Failed to write modified font")?;

    println!("  Saved modified font");
    Ok(true)
}

fn rebuild_gsub(
    original: &read_fonts::tables::gsub::Gsub,
    lookup_list: write_fonts::tables::gsub::SubstitutionLookupList,
) -> Result<write_fonts::tables::gsub::Gsub> {
    let script_list = original.script_list()?;
    let feature_list = original.feature_list()?;

    let mut new_script_list = write_fonts::tables::layout::ScriptList::default();
    for script_record in script_list.script_records() {
        let script_tag = script_record.script_tag();
        let script = script_record.script(script_list.offset_data())?;

        let mut new_script = write_fonts::tables::layout::Script::default();

        if let Some(default_lang_sys) = script.default_lang_sys() {
            let dls = default_lang_sys?;
            let mut new_dls = write_fonts::tables::layout::LangSys::default();
            new_dls.required_feature_index = dls.required_feature_index();
            new_dls.feature_indices = dls.feature_indices().iter().map(|i| i.get()).collect();
            new_script.default_lang_sys = new_dls.into();
        }

        let mut lang_sys_records = Vec::new();
        for lang_sys_record in script.lang_sys_records() {
            let lang_tag = lang_sys_record.lang_sys_tag();
            let lang_sys = lang_sys_record.lang_sys(script.offset_data())?;

            let mut new_lang_sys = write_fonts::tables::layout::LangSys::default();
            new_lang_sys.required_feature_index = lang_sys.required_feature_index();
            new_lang_sys.feature_indices =
                lang_sys.feature_indices().iter().map(|i| i.get()).collect();

            lang_sys_records.push(write_fonts::tables::layout::LangSysRecord {
                lang_sys_tag: lang_tag,
                lang_sys: new_lang_sys.into(),
            });
        }
        new_script.lang_sys_records = lang_sys_records;

        new_script_list
            .script_records
            .push(write_fonts::tables::layout::ScriptRecord {
                script_tag,
                script: new_script.into(),
            });
    }

    let mut new_feature_list = write_fonts::tables::layout::FeatureList::default();
    for feature_record in feature_list.feature_records() {
        let feature_tag = feature_record.feature_tag();
        let feature = feature_record.feature(feature_list.offset_data())?;

        let mut new_feature = write_fonts::tables::layout::Feature::default();
        new_feature.feature_params = None.into();
        new_feature.lookup_list_indices =
            feature.lookup_list_indices().iter().map(|i| i.get()).collect();

        new_feature_list
            .feature_records
            .push(write_fonts::tables::layout::FeatureRecord {
                feature_tag,
                feature: new_feature.into(),
            });
    }

    Ok(write_fonts::tables::gsub::Gsub::new(
        new_script_list,
        new_feature_list,
        lookup_list,
    ))
}
