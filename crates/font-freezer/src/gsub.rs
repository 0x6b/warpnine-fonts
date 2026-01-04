//! GSUB (Glyph Substitution) table processing.

use std::collections::{BTreeSet, HashMap};

use read_fonts::tables::{
    gsub::{Gsub, SingleSubst, SubstitutionLookup, SubstitutionSubtables},
    layout::CoverageTable,
};

use crate::Result;

/// A map of glyph substitutions extracted from GSUB lookups.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlyphSubstitutions(HashMap<u16, u16>);

impl GlyphSubstitutions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_lookups(&mut self, gsub: &Gsub, indices: &BTreeSet<u16>) -> Result<()> {
        let lookup_list = gsub.lookup_list()?;
        for &idx in indices {
            if let Ok(lookup) = lookup_list.lookups().get(idx as usize) {
                self.process_lookup(&lookup)?;
            }
        }
        Ok(())
    }

    fn process_lookup(&mut self, lookup: &SubstitutionLookup<'_>) -> Result<()> {
        match lookup.subtables()? {
            SubstitutionSubtables::Single(tables) => {
                for table in tables.iter().flatten() {
                    self.process_single(&table)?;
                }
            }
            SubstitutionSubtables::Alternate(tables) => {
                for table in tables.iter().flatten() {
                    for (i, alt) in table.alternate_sets().iter().enumerate() {
                        let Some(orig) = Coverage(table.coverage()?).get(i as u16) else {
                            continue;
                        };
                        let Ok(alt) = alt else { continue };
                        if let Some(gid) = alt.alternate_glyph_ids().first() {
                            self.apply(orig, gid.get().to_u32() as u16);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn process_single(&mut self, subtable: &SingleSubst<'_>) -> Result<()> {
        match subtable {
            SingleSubst::Format1(fmt) => {
                let delta = fmt.delta_glyph_id() as i32;
                for gid in Coverage(fmt.coverage()?).iter() {
                    self.apply(gid, (gid as i32 + delta) as u16);
                }
            }
            SingleSubst::Format2(fmt) => {
                let subs = fmt.substitute_glyph_ids();
                for (i, gid) in Coverage(fmt.coverage()?).iter().enumerate() {
                    if let Some(new) = subs.get(i) {
                        self.apply(gid, new.get().to_u32() as u16);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn apply(&mut self, from: u16, to: u16) {
        for k in self
            .0
            .iter()
            .filter(|&(_, v)| *v == from)
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
        {
            self.0.insert(k, to);
        }
        self.0.entry(from).or_insert(to);
    }

    pub fn remap(&self, gid: u16) -> u16 {
        self.0.get(&gid).copied().unwrap_or(gid)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u16, &u16)> {
        self.0.iter()
    }
}

struct Coverage<'a>(CoverageTable<'a>);

impl Coverage<'_> {
    fn get(&self, index: u16) -> Option<u16> {
        match &self.0 {
            CoverageTable::Format1(f) => {
                f.glyph_array().get(index as usize).map(|g| g.get().to_u32() as u16)
            }
            CoverageTable::Format2(f) => f.range_records().iter().find_map(|r| {
                let (start, end, base) = (
                    r.start_glyph_id().to_u32() as u16,
                    r.end_glyph_id().to_u32() as u16,
                    r.start_coverage_index(),
                );
                (index >= base && index < base + (end - start + 1)).then(|| start + (index - base))
            }),
        }
    }

    fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        match &self.0 {
            CoverageTable::Format1(f) => {
                Box::new(f.glyph_array().iter().map(|g| g.get().to_u32() as u16))
                    as Box<dyn Iterator<Item = u16>>
            }
            CoverageTable::Format2(f) => Box::new(f.range_records().iter().flat_map(|r| {
                let start = r.start_glyph_id().to_u32() as u16;
                let end = r.end_glyph_id().to_u32() as u16;
                start..=end
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_substitution_simple() {
        let mut subs = GlyphSubstitutions::new();
        subs.apply(1, 2);
        assert_eq!(subs.remap(1), 2);
        assert_eq!(subs.remap(99), 99);
    }

    #[test]
    fn test_apply_substitution_chain() {
        let mut subs = GlyphSubstitutions::new();
        subs.apply(1, 2);
        subs.apply(2, 3);
        assert_eq!(subs.remap(1), 3);
        assert_eq!(subs.remap(2), 3);
    }
}
