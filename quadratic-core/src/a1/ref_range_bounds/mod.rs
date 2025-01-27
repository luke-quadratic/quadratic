use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ts_rs::TS;

use super::{range_might_intersect, A1Error, CellRefRangeEnd};
use crate::{Pos, Rect};

pub mod ref_range_bounds_contains;
pub mod ref_range_bounds_create;
pub mod ref_range_bounds_intersection;
pub mod ref_range_bounds_query;
pub mod ref_range_bounds_translate;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, TS)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct RefRangeBounds {
    pub start: CellRefRangeEnd,
    pub end: CellRefRangeEnd,
}

impl fmt::Debug for RefRangeBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RefRangeBounds(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl fmt::Display for RefRangeBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::ALL {
            write!(f, "*")?;
        } else if self.is_column_range() {
            if self.start.col() == self.end.col() {
                self.start.col.fmt_as_column(f)?;
            } else {
                self.start.col.fmt_as_column(f)?;
                write!(f, ":")?;
                self.end.col.fmt_as_column(f)?;
            }
        } else if self.is_row_range() {
            // handle special case of An: (show as An: instead of n:)
            if self.end.col.is_unbounded()
                && self.end.row.is_unbounded()
                && self.start.col.coord == 1
            {
                write!(f, "A")?;
                self.start.row.fmt_as_row(f)?;
                write!(f, ":")?;
            } else if self.start.row() == self.end.row() {
                write!(f, "A")?;
                self.start.row.fmt_as_row(f)?;
                write!(f, ":")?;
                self.start.row.fmt_as_row(f)?;
            } else {
                self.start.row.fmt_as_row(f)?;
                write!(f, ":")?;
                self.end.row.fmt_as_row(f)?;
            }
        } else {
            write!(f, "{}", self.start)?;
            // we don't need to print the end range if start == end
            if self.start != self.end {
                let end = self.end.to_string();
                write!(f, ":{end}")?;
            }
        }
        Ok(())
    }
}

impl RefRangeBounds {
    /// Range that contains the entire sheet.
    pub const ALL: Self = Self {
        start: CellRefRangeEnd::START,
        end: CellRefRangeEnd::UNBOUNDED,
    };

    /// Creates a new range bounds from relative coordinates.
    pub fn new_relative(start_col: i64, start_row: i64, end_col: i64, end_row: i64) -> Self {
        Self {
            start: CellRefRangeEnd::new_relative_xy(start_col, start_row),
            end: CellRefRangeEnd::new_relative_xy(end_col, end_row),
        }
    }

    /// Creates a new infinite row.
    pub fn new_infinite_row(row: i64) -> Self {
        Self {
            start: CellRefRangeEnd::new_relative_xy(1, row),
            end: CellRefRangeEnd::new_infinite_row_end(row),
        }
    }

    pub fn new_infinite_rows(start_row: i64, end_row: i64) -> Self {
        Self {
            start: CellRefRangeEnd::new_relative_xy(1, start_row),
            end: CellRefRangeEnd::new_infinite_row_end(end_row),
        }
    }

    /// Creates a new infinite column.
    pub fn new_infinite_col(col: i64) -> Self {
        Self {
            start: CellRefRangeEnd::new_relative_xy(col, 1),
            end: CellRefRangeEnd::new_infinite_col_end(col),
        }
    }

    pub fn new_infinite_cols(start_col: i64, end_col: i64) -> Self {
        Self {
            start: CellRefRangeEnd::new_relative_xy(start_col, 1),
            end: CellRefRangeEnd::new_infinite_col_end(end_col),
        }
    }
}

#[cfg(test)]
#[serial_test::parallel]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_cell_ref_range_parsing(ref_range_bounds: RefRangeBounds) {
            // We skip tests where start = end since we remove the end when parsing
            if ref_range_bounds.end != ref_range_bounds.start {
                assert_eq!(ref_range_bounds, ref_range_bounds.to_string().parse().unwrap());
            }
        }
    }

    #[test]
    fn test_new_relative() {
        let range = RefRangeBounds::new_relative(1, 2, 3, 4);
        assert_eq!(range.start.col.coord, 1);
        assert_eq!(range.start.row.coord, 2);
        assert_eq!(range.end.col.coord, 3);
        assert_eq!(range.end.row.coord, 4);
    }

    #[test]
    fn test_new_infinite_row() {
        let range = RefRangeBounds::new_infinite_row(1);
        assert_eq!(range.start.col.coord, 1);
        assert_eq!(range.start.row.coord, 1);
        assert_eq!(range.end.row.coord, 1);
        assert!(range.end.col.is_unbounded());
    }

    #[test]
    fn test_new_infinite_col() {
        let range = RefRangeBounds::new_infinite_col(1);
        assert_eq!(range.start.col.coord, 1);
        assert_eq!(range.start.row.coord, 1);
        assert_eq!(range.end.col.coord, 1);
        assert!(range.end.row.is_unbounded());
    }

    #[test]
    fn test_new_infinite_rows() {
        let range = RefRangeBounds::new_infinite_rows(1, 2);
        assert_eq!(range.start.col.coord, 1);
        assert_eq!(range.start.row.coord, 1);
        assert_eq!(range.end.row.coord, 2);
        assert!(range.end.col.is_unbounded());
    }

    #[test]
    fn test_new_infinite_cols() {
        let range = RefRangeBounds::new_infinite_cols(1, 2);
        assert_eq!(range.start.col.coord, 1);
        assert_eq!(range.start.row.coord, 1);
        assert_eq!(range.end.col.coord, 2);
        assert!(range.end.row.is_unbounded());
    }

    #[test]
    fn test_display_row_range() {
        let range = RefRangeBounds::new_infinite_row(1);
        assert_eq!(range.to_string(), "A1:1");

        let range = RefRangeBounds::new_infinite_rows(1, 2);
        assert_eq!(range.to_string(), "1:2");
    }

    #[test]
    fn test_display_infinite_a_n() {
        let range = RefRangeBounds::test_a1("15:");
        assert_eq!(range.to_string(), "A15:");
    }
}
