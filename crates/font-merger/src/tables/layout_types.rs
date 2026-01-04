//! Types for layout table (GSUB/GPOS) merging
//!
//! These types encapsulate the state management for merging OpenType
//! layout tables, replacing raw HashMaps and Vecs.

use std::collections::HashMap;

use read_fonts::types::Tag;

/// Index into the merged feature list
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FeatureIndex(pub u16);

impl FeatureIndex {
    pub fn new(idx: u16) -> Self {
        Self(idx)
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Index into the merged lookup list
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LookupIndex(pub u16);

impl LookupIndex {
    pub fn new(idx: u16) -> Self {
        Self(idx)
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Script tag wrapper for type safety
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScriptTag(pub Tag);

impl ScriptTag {
    pub fn new(tag: Tag) -> Self {
        Self(tag)
    }

    pub fn tag(&self) -> Tag {
        self.0
    }

    pub fn is_dflt(&self) -> bool {
        self.0 == Tag::new(b"DFLT")
    }
}

/// Language tag wrapper for type safety
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LangTag(pub Tag);

impl LangTag {
    pub fn new(tag: Tag) -> Self {
        Self(tag)
    }

    pub fn dflt() -> Self {
        Self(Tag::new(b"dflt"))
    }

    pub fn tag(&self) -> Tag {
        self.0
    }
}

/// Map of scripts → languages → feature indices
///
/// This replaces `HashMap<Tag, HashMap<Tag, Vec<u16>>>` with a proper type.
#[derive(Debug, Default)]
pub struct ScriptLangFeatureMap {
    inner: HashMap<ScriptTag, HashMap<LangTag, Vec<FeatureIndex>>>,
}

impl ScriptLangFeatureMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add feature indices for a script/language combination
    pub fn add_features(
        &mut self,
        script: ScriptTag,
        lang: LangTag,
        features: impl IntoIterator<Item = FeatureIndex>,
    ) {
        self.inner
            .entry(script)
            .or_default()
            .entry(lang)
            .or_default()
            .extend(features);
    }

    /// Add a feature to all non-DFLT scripts and all their languages
    pub fn add_feature_to_all_scripts(&mut self, feature: FeatureIndex) {
        for (script, lang_map) in self.inner.iter_mut() {
            if !script.is_dflt() {
                for features in lang_map.values_mut() {
                    features.push(feature);
                }
            }
        }
    }

    /// Convert to raw format for building
    pub fn into_raw(self) -> HashMap<Tag, HashMap<Tag, Vec<u16>>> {
        self.inner
            .into_iter()
            .map(|(script, lang_map)| {
                let lang_raw = lang_map
                    .into_iter()
                    .map(|(lang, features)| {
                        (lang.tag(), features.into_iter().map(|f| f.as_u16()).collect())
                    })
                    .collect();
                (script.tag(), lang_raw)
            })
            .collect()
    }
}

/// A merged feature with its tag and lookup indices
#[derive(Debug, Clone)]
pub struct MergedFeature {
    pub tag: Tag,
    pub lookup_indices: Vec<LookupIndex>,
}

impl MergedFeature {
    pub fn new(tag: Tag, lookup_indices: Vec<LookupIndex>) -> Self {
        Self { tag, lookup_indices }
    }
}

/// List of merged features
#[derive(Debug, Default)]
pub struct MergedFeatureList {
    features: Vec<MergedFeature>,
}

impl MergedFeatureList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a feature and return its index
    pub fn add(&mut self, tag: Tag, lookup_indices: Vec<LookupIndex>) -> FeatureIndex {
        let idx = FeatureIndex::new(self.features.len() as u16);
        self.features.push(MergedFeature::new(tag, lookup_indices));
        idx
    }

    /// Get the current count (for offset calculation)
    pub fn len(&self) -> u16 {
        self.features.len() as u16
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// Convert to raw format for building
    pub fn into_raw(self) -> Vec<(Tag, Vec<u16>)> {
        self.features
            .into_iter()
            .map(|f| (f.tag, f.lookup_indices.into_iter().map(|l| l.as_u16()).collect()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_lang_feature_map() {
        let mut map = ScriptLangFeatureMap::new();

        let latn = ScriptTag::new(Tag::new(b"latn"));
        let dflt_lang = LangTag::dflt();

        map.add_features(
            latn.clone(),
            dflt_lang.clone(),
            [FeatureIndex::new(0), FeatureIndex::new(1)],
        );

        let raw = map.into_raw();
        assert!(raw.contains_key(&Tag::new(b"latn")));
    }

    #[test]
    fn test_merged_feature_list() {
        let mut list = MergedFeatureList::new();

        let idx = list.add(Tag::new(b"liga"), vec![LookupIndex::new(0), LookupIndex::new(1)]);
        assert_eq!(idx.as_u16(), 0);

        let idx2 = list.add(Tag::new(b"kern"), vec![LookupIndex::new(2)]);
        assert_eq!(idx2.as_u16(), 1);

        assert_eq!(list.len(), 2);
    }
}
