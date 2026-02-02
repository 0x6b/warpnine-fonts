//! Glyph name types and utilities
//!
//! This module provides the `GlyphName` type which wraps glyph name strings
//! with proper trait implementations for use in collections.

use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

/// A glyph name that may have been disambiguated during merging
///
/// This type wraps a String and provides convenient trait implementations
/// for use in HashMaps and other collections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphName(String);

impl GlyphName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for GlyphName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for GlyphName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for GlyphName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for GlyphName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for GlyphName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for GlyphName {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

impl Display for GlyphName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for GlyphName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for GlyphName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<GlyphName> for String {
    fn from(GlyphName(name): GlyphName) -> Self {
        name
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_glyph_name_display() {
        let name = GlyphName::new("A");
        assert_eq!(format!("{}", name), "A");
    }

    #[test]
    fn test_glyph_name_deref() {
        let name = GlyphName::new("hello");
        assert_eq!(name.len(), 5);
        assert!(name.starts_with("hel"));
    }

    #[test]
    fn test_glyph_name_equality() {
        let name = GlyphName::new("test");
        assert_eq!(name, "test");
        assert_eq!(name, String::from("test"));
        assert_eq!(name, GlyphName::new("test"));
    }

    #[test]
    fn test_glyph_name_in_hashmap() {
        let mut map: HashMap<GlyphName, i32> = HashMap::new();
        map.insert(GlyphName::new("A"), 1);
        map.insert(GlyphName::new("B"), 2);

        assert_eq!(map.get(&GlyphName::new("A")), Some(&1));
        assert_eq!(map.get(&GlyphName::new("B")), Some(&2));
    }
}
