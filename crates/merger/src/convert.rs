//! Conversion traits for read-fonts → write-fonts types
//!
//! These traits centralize the conversion logic that was previously
//! scattered across table merger functions.

use read_fonts::{tables, tables::gpos as read_gpos};
use write_fonts::tables::gpos::{
    AnchorFormat1, AnchorFormat2, AnchorFormat3, AnchorTable, MarkArray, MarkRecord, ValueRecord,
};

/// Convert a read-fonts type to its write-fonts equivalent
pub trait ToWrite {
    type Output;
    fn to_write(&self) -> Self::Output;
}

impl ToWrite for read_gpos::ValueRecord {
    type Output = ValueRecord;

    fn to_write(&self) -> ValueRecord {
        let mut result = ValueRecord::new();
        if let Some(v) = self.x_placement {
            result = result.with_x_placement(v.get());
        }
        if let Some(v) = self.y_placement {
            result = result.with_y_placement(v.get());
        }
        if let Some(v) = self.x_advance {
            result = result.with_x_advance(v.get());
        }
        if let Some(v) = self.y_advance {
            result = result.with_y_advance(v.get());
        }
        result
    }
}

impl<'a> ToWrite for tables::gpos::AnchorTable<'a> {
    type Output = AnchorTable;

    fn to_write(&self) -> AnchorTable {
        match self {
            read_gpos::AnchorTable::Format1(a) => {
                AnchorTable::Format1(AnchorFormat1::new(a.x_coordinate(), a.y_coordinate()))
            }
            read_gpos::AnchorTable::Format2(a) => AnchorTable::Format2(AnchorFormat2::new(
                a.x_coordinate(),
                a.y_coordinate(),
                a.anchor_point(),
            )),
            read_gpos::AnchorTable::Format3(a) => AnchorTable::Format3(AnchorFormat3::new(
                a.x_coordinate(),
                a.y_coordinate(),
                None,
                None,
            )),
        }
    }
}

/// Extension trait for converting MarkArray
pub trait MarkArrayExt {
    fn to_write(&self) -> MarkArray;
}

impl<'a> MarkArrayExt for read_gpos::MarkArray<'a> {
    fn to_write(&self) -> MarkArray {
        let mark_records: Vec<MarkRecord> = self
            .mark_records()
            .iter()
            .map(|mr| {
                let anchor = mr
                    .mark_anchor(self.offset_data())
                    .ok()
                    .map(|a| a.to_write())
                    .unwrap_or_else(|| AnchorTable::Format1(AnchorFormat1::new(0, 0)));
                MarkRecord::new(mr.mark_class(), anchor)
            })
            .collect();
        MarkArray::new(mark_records)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use read_fonts::tables;
    #[cfg(test)]
    use read_fonts::tables::gpos;

    use super::*;

    #[test]
    fn test_value_record_trait_exists() {
        fn assert_impl<T: ToWrite>() {}
        assert_impl::<tables::gpos::ValueRecord>();
    }
}
