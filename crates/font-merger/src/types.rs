//! Domain-specific newtypes for type safety
//!
//! These types prevent mixing up different kinds of IDs and provide
//! self-documenting APIs.

use std::{
    fmt,
    fmt::{Display, Formatter, Result},
};

use read_fonts::types::Tag;

macro_rules! u16_id {
    ($(#[$meta:meta])* $name:ident, $label:literal) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u16);

        impl $name {
            pub const fn new(id: u16) -> Self {
                Self(id)
            }

            pub const fn to_u16(self) -> u16 {
                self.0
            }

            pub const fn to_u32(self) -> u32 {
                self.0 as u32
            }
        }

        impl From<u16> for $name {
            fn from(id: u16) -> Self {
                Self(id)
            }
        }

        impl From<$name> for u16 {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $label, self.0)
            }
        }
    };
}

/// Index into the fonts array being merged
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontIndex(pub usize);

impl FontIndex {
    pub const fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for FontIndex {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl From<FontIndex> for usize {
    fn from(FontIndex(idx): FontIndex) -> Self {
        idx
    }
}

impl Display for FontIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Font[{}]", self.0)
    }
}

u16_id!(
    /// A glyph ID from a source font (before merging)
    GlyphId,
    "GID"
);

u16_id!(
    /// A glyph ID in the merged mega glyph order
    MegaGlyphId,
    "MGID"
);

/// A Unicode codepoint
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Codepoint(pub u32);

impl Codepoint {
    pub const fn new(cp: u32) -> Self {
        Self(cp)
    }

    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Convert to a Rust char if valid
    pub fn to_char(self) -> Option<char> {
        char::from_u32(self.0)
    }
}

impl From<u32> for Codepoint {
    fn from(cp: u32) -> Self {
        Self(cp)
    }
}

impl From<Codepoint> for u32 {
    fn from(cp: Codepoint) -> Self {
        cp.0
    }
}

impl Display for Codepoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "U+{:04X}", self.0)
    }
}

/// A font table tag (always 4 bytes)
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TableTag(Tag);

impl TableTag {
    /// Create a TableTag from a 4-byte array
    pub fn new(bytes: &[u8; 4]) -> Self {
        Self(Tag::new(bytes))
    }

    /// Try to create a TableTag from a string
    ///
    /// Returns None if the string is longer than 4 bytes.
    /// Shorter strings are padded with spaces.
    pub fn parse(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        (bytes.len() <= 4).then(|| {
            let mut arr = [b' '; 4];
            arr[..bytes.len()].copy_from_slice(bytes);
            Self(Tag::new(&arr))
        })
    }

    /// Get the underlying Tag
    pub fn tag(&self) -> Tag {
        self.0
    }
}

impl From<Tag> for TableTag {
    fn from(tag: Tag) -> Self {
        Self(tag)
    }
}

impl From<TableTag> for Tag {
    fn from(tt: TableTag) -> Self {
        tt.0
    }
}

impl From<&TableTag> for Tag {
    fn from(tt: &TableTag) -> Self {
        tt.0
    }
}

impl Display for TableTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_id() {
        let gid = GlyphId::new(42);
        assert_eq!(gid.to_u16(), 42);
        assert_eq!(format!("{}", gid), "GID42");
    }

    #[test]
    fn test_mega_glyph_id() {
        let mgid = MegaGlyphId::new(100);
        assert_eq!(mgid.to_u16(), 100);
        assert_eq!(format!("{}", mgid), "MGID100");
    }

    #[test]
    fn test_codepoint() {
        let cp = Codepoint::new(0x0041);
        assert_eq!(cp.to_char(), Some('A'));
        assert_eq!(format!("{}", cp), "U+0041");
    }

    #[test]
    fn test_table_tag() {
        let tag = TableTag::parse("head").unwrap();
        assert_eq!(format!("{}", tag), "head");

        let tag = TableTag::parse("OS/2").unwrap();
        assert_eq!(format!("{}", tag), "OS/2");

        // Too long
        assert!(TableTag::parse("toolong").is_none());
    }
}
