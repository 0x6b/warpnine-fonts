//! Options for font merging

use crate::types::TableTag;
use read_fonts::types::Tag;

/// Options for font merging
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Tables to drop from the merged font
    pub drop_tables: Vec<TableTag>,

    /// Whether to enable verbose logging
    pub verbose: bool,

    /// Whether to enable timing information
    pub timing: bool,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add tables to drop (accepts any iterable of string-like values)
    pub fn drop_tables(mut self, tables: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.drop_tables = tables
            .into_iter()
            .filter_map(|s| TableTag::parse(s.as_ref()))
            .collect();
        self
    }

    /// Add a single table to drop
    pub fn drop_table(mut self, table: impl AsRef<str>) -> Self {
        if let Some(tag) = TableTag::parse(table.as_ref()) {
            self.drop_tables.push(tag);
        }
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn timing(mut self, timing: bool) -> Self {
        self.timing = timing;
        self
    }

    /// Check if a table should be dropped
    pub fn should_drop(&self, tag: &TableTag) -> bool {
        self.drop_tables.contains(tag)
    }

    /// Check if a table should be dropped (by Tag)
    pub fn should_drop_tag(&self, tag: Tag) -> bool {
        self.should_drop(&tag.into())
    }
}
