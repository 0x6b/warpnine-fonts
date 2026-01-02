//! Core types for font feature freezing configuration and results.

use read_fonts::types::Tag;
use std::collections::{BTreeSet, HashSet};
use std::fmt::{self, Formatter, Result};

/// Options controlling how features are frozen into the font.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FreezeOptions {
    pub features: Vec<String>,
    pub filter: ScriptLangFilter,
    pub suffix: SuffixConfig,
    pub replacenames: Option<String>,
    pub info: bool,
    pub zapnames: bool,
    /// Generate warnings for glyphs without unicode (expensive for large fonts)
    pub warnings: bool,
}

/// Restricts feature application to a specific OpenType script/language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptLangFilter {
    pub script: Option<String>,
    pub lang: Option<String>,
}

impl ScriptLangFilter {
    pub fn new(script: Option<String>, lang: Option<String>) -> Self {
        Self { script, lang }
    }

    pub fn is_active(&self) -> bool {
        self.script.is_some() || self.lang.is_some()
    }

    pub fn matches_script(&self, tag: &str) -> bool {
        self.script.as_deref().is_none_or(|s| s == tag)
    }

    pub fn matches_lang(&self, tag: &str) -> bool {
        self.lang.as_deref() == Some(tag)
    }
}

/// How to modify the font family name when freezing features.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SuffixConfig {
    #[default]
    None,
    Auto,
    Custom(String),
}

impl SuffixConfig {
    pub fn auto() -> Self {
        Self::Auto
    }

    pub fn custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn as_string(&self, features: &[String]) -> String {
        match self {
            Self::None => String::new(),
            Self::Custom(s) => format!(" {s}"),
            Self::Auto => {
                let sorted: BTreeSet<_> = features.iter().map(String::as_str).collect();
                format!(" {}", sorted.into_iter().collect::<Vec<_>>().join(" "))
            }
        }
    }
}

impl FreezeOptions {
    pub fn new<I, S>(features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            features: features.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn with_script(mut self, script: impl Into<String>) -> Self {
        self.filter.script = Some(script.into());
        self
    }

    pub fn with_lang(mut self, lang: impl Into<String>) -> Self {
        self.filter.lang = Some(lang.into());
        self
    }

    pub fn with_suffix(mut self) -> Self {
        self.suffix = SuffixConfig::Auto;
        self
    }

    pub fn with_usesuffix(mut self, s: impl Into<String>) -> Self {
        self.suffix = SuffixConfig::Custom(s.into());
        self
    }

    pub fn with_replacenames(mut self, r: impl Into<String>) -> Self {
        self.replacenames = Some(r.into());
        self
    }

    pub fn with_info(mut self) -> Self {
        self.info = true;
        self
    }

    pub fn with_zapnames(mut self) -> Self {
        self.zapnames = true;
        self
    }

    pub fn with_script_opt<S: Into<String>>(mut self, script: Option<S>) -> Self {
        self.filter.script = script.map(Into::into);
        self
    }

    pub fn with_lang_opt<S: Into<String>>(mut self, lang: Option<S>) -> Self {
        self.filter.lang = lang.map(Into::into);
        self
    }

    pub fn with_usesuffix_opt<S: Into<String>>(mut self, suffix: Option<S>) -> Self {
        if let Some(suffix) = suffix {
            self.suffix = SuffixConfig::Custom(suffix.into());
        }
        self
    }

    pub fn with_replacenames_opt<S: Into<String>>(mut self, replacements: Option<S>) -> Self {
        self.replacenames = replacements.map(Into::into);
        self
    }

    pub fn with_suffix_if(mut self, on: bool) -> Self {
        if on && matches!(self.suffix, SuffixConfig::None) {
            self.suffix = SuffixConfig::Auto;
        }
        self
    }

    pub fn with_info_if(mut self, on: bool) -> Self {
        self.info |= on;
        self
    }

    pub fn with_zapnames_if(mut self, on: bool) -> Self {
        self.zapnames |= on;
        self
    }

    pub fn with_warnings(mut self) -> Self {
        self.warnings = true;
        self
    }

    pub fn with_warnings_if(mut self, on: bool) -> Self {
        self.warnings |= on;
        self
    }

    pub fn wants_name_edits(&self) -> bool {
        self.suffix.is_enabled() || self.replacenames.is_some() || self.info
    }

    pub fn suffix_string(&self) -> String {
        self.suffix.as_string(&self.features)
    }

    pub fn feature_tags(&self) -> HashSet<Tag> {
        self.features
            .iter()
            .filter_map(|f| f.as_bytes().try_into().ok().map(Tag::new))
            .collect()
    }
}

/// Result of freezing features into a font.
#[derive(Debug, Clone)]
pub struct FreezeResult {
    pub data: Vec<u8>,
    pub stats: FreezeStats,
    pub warnings: Vec<String>,
    pub remapped_names: Vec<String>,
}

/// Statistics about the freeze operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FreezeStats {
    pub features_requested: usize,
    pub lookups_processed: usize,
    pub substitutions_applied: usize,
}

impl fmt::Display for FreezeStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "frozen {} features, {} substitutions",
            self.features_requested, self.substitutions_applied
        )
    }
}

/// Report of available scripts, languages, and features in a font.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontReport {
    pub scripts_langs: Vec<String>,
    pub features: Vec<String>,
}

impl fmt::Display for FontReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "# Scripts and languages:")?;
        for sl in &self.scripts_langs {
            writeln!(f, "{sl}")?;
        }
        writeln!(f, "# Features:")?;
        write!(f, "-f {}", self.features.join(","))
    }
}
