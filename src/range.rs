use std::{
    cmp::Reverse,
    convert::identity,
    fmt::{Debug, Display},
};

use chumsky::{Parser, error::Rich};
use itertools::Itertools;

mod numeric_impls;
mod parse;

pub use parse::Extra as ParserExtra;

#[cfg(test)]
mod tests;

/// Required functions for being a range extreme
pub trait RangeExtreme: Ord + Clone {
    /// Minimum value
    ///
    /// Every value should greather or equal of this
    const MIN: Self;
    /// Maximum value
    ///
    /// Every value should less or equal of this
    const MAX: Self;

    /// Next value
    ///
    /// There should be no value `v` that satisfy `a < v < a.next()`
    fn next(self) -> Self;

    /// Compare value to next value
    ///
    /// `a.compare_next_to(b)` should be true if and only if `a.next() == b`
    fn compare_next_to(&self, other: &Self) -> bool {
        &self.clone().next() == other
    }
}

/// Required functions for the range to be displayed
pub trait RangeExtremeDisplay: RangeExtreme + Display {
    /// Check if this value has a previous value
    ///
    /// `a.has_prev()` shoud be true if and only if it exist a value `b` so `b.next() == a`
    fn has_prev(&self) -> bool;

    /// Display the previous value
    ///
    /// See [`RangeExtremeDisplay::has_prev`] for the definition of previous value
    fn display_prev(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

/// Marker trait for ranges extremes that can be parser
///
/// Implementors must ensure that valid string representations must not
///  - contain `||` or `&&`
///  - start with `==`, `>`, `<`, `>=`, `<=`, `!` or `(`
///  - end with `)`
///  - have surrounding whitespace
///  - be `*`.
pub trait RangeExtremeParseable: RangeExtreme {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, ParserExtra<'a>> + Clone;
}

/// A range of versions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranges<T> {
    /// Sorted list of range extremes, in descending order
    ///
    /// If the number of elements is odd, the last range is considered half-open
    extremes: Vec<T>,
}

impl<T> Ranges<T>
where
    T: RangeExtreme,
{
    /// Empty range
    pub const EMPTY: Self = Self { extremes: vec![] };

    /// Create a new range from `start` to `end`, including `start` and excluding `end`
    pub fn between(start: T, end: T) -> Self {
        // Ensure all empty ranges are considered equal
        if start >= end {
            return Self::EMPTY;
        }
        Self {
            extremes: vec![end, start],
        }
    }

    /// Create a new range from `start` to `end`, excluding `start` and excluding `end`
    pub fn between_exclude_start(start: T, end: T) -> Self {
        Self::between(start.next(), end)
    }

    /// Create a new range from `start` to `end`, including `start` and including `end`
    pub fn between_include_end(start: T, end: T) -> Self {
        Self::between(start, end.next())
    }

    /// Create a new range from `start` to `end`, excluding `start` and including `end`
    pub fn between_exclude_start_include_end(start: T, end: T) -> Self {
        Self::between(start.next(), end.next())
    }

    /// Create a new range from `start` to infinity, including `start`
    pub fn from(start: T) -> Self {
        Self {
            extremes: vec![start],
        }
    }

    /// Create a new range from `start` to infinity, excluding `start`
    pub fn from_exclusive(start: T) -> Self {
        Self::from(start.next())
    }

    /// Create a new range from negative infinity to `end`, excluding `end`
    pub fn to(end: T) -> Self {
        Self::between(T::MIN, end)
    }

    /// Create a new range from negative infinity to `end`, including `end`
    pub fn to_inclusive(end: T) -> Self {
        Self::to(end.next())
    }

    /// Create a range containing only one value.
    pub fn single(value: T) -> Self {
        Self::between_include_end(value.clone(), value)
    }

    /// Create a range containing all values except one.
    pub fn except(value: T) -> Self {
        Self::single(value).not()
    }

    /// Return whether the range contains exactly one element.
    pub fn is_single(&self) -> bool {
        self.extremes.len() == 2 && self.extremes[1].compare_next_to(&self.extremes[0])
    }

    /// Return whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self == &Self::EMPTY
    }

    /// Return a range containing all values.
    pub fn full() -> Self {
        Self::from(T::MIN)
    }

    /// Return whether the range contains all possible values.
    pub fn is_full(&self) -> bool {
        self.extremes.len() == 1 && self.extremes[0] == T::MIN
    }

    /// Return whether `value` is inside the range.
    pub fn contains(&self, value: &T) -> bool {
        (self.extremes.len()
            - self
                .extremes
                .binary_search_by_key(&Reverse(value), Reverse)
                .unwrap_or_else(identity))
            % 2
            == 1
    }

    /// Return an iterator over the ranges in the range set
    ///
    /// If the end is missing, the range is half-infinite
    fn ranges(&self) -> impl IntoIterator<Item = (&T, Option<&T>)> {
        let full_ranges = self.extremes.as_slice().rchunks_exact(2);
        let remainder = full_ranges.remainder().first();
        full_ranges
            .map(|chunk| (&chunk[1], Some(&chunk[0])))
            .chain(remainder.map(|chunk| (chunk, None)))
    }

    /// Return the negation of the range set
    pub fn not(mut self) -> Self {
        if self.extremes.last() == Some(&T::MIN) {
            self.extremes.pop();
        } else {
            self.extremes.push(T::MIN);
        }
        self
    }

    /// Return the union of two ranges set
    pub fn or(mut self, other: &Self) -> Self {
        for (start, end) in other.ranges() {
            let add_start = (!self.contains(start)).then_some(start);
            let add_end = end.and_then(|end| (!self.contains(end)).then_some(end));

            let i_start = self
                .extremes
                .binary_search_by_key(&Reverse(start), Reverse)
                .unwrap_or_else(identity);
            let i_end = end.map_or(0, |end| {
                self.extremes
                    .binary_search_by_key(&Reverse(end), Reverse)
                    .unwrap_or_else(identity)
            });

            self.extremes.splice(
                i_end..i_start,
                [add_end, add_start].into_iter().flatten().cloned(),
            );
        }
        self
    }

    /// Return the intersection of two ranges set
    pub fn xor<'a>(ranges: impl IntoIterator<Item = &'a Self>) -> Self
    where
        T: 'a,
    {
        // Each extreme is a point where the predicate passes from true to false.
        // Xor changes value each time it changes, so we can simply merge all the points
        // and deduplicate them
        Self {
            extremes: ranges
                .into_iter()
                .map(|r| r.extremes.iter().map(Reverse))
                .kmerge()
                .map(|Reverse(extreme)| extreme)
                .dedup_with_count()
                .filter_map(|(count, item)| (count % 2 == 1).then_some(item))
                .cloned()
                .collect(),
        }
    }

    /// Return the intersection of two ranges set
    pub fn and(self, other: &Self) -> Self {
        // Using the identity `a && b = a ^ b ^ (a || b)`
        let or = self.clone().or(other);
        Self::xor([&self, &other, &or])
    }

    pub fn from_str<'a>(s: &'a str) -> Result<Self, Vec<Rich<'a, char>>>
    where
        T: RangeExtremeParseable + 'a,
    {
        parse::parser().parse(s).into_result()
    }
}

impl<T> Display for Ranges<T>
where
    T: RangeExtremeDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "-");
        }

        if self.is_full() {
            return write!(f, "*");
        }

        for (i, (start, end)) in self.ranges().into_iter().enumerate() {
            if i > 0 {
                write!(f, " || ")?;
            }

            if start.compare_next_to(end.unwrap_or(&T::MAX)) {
                write!(f, "=={}", start)?;
                continue;
            }

            if start != &T::MIN {
                if start.has_prev() {
                    write!(f, ">")?;
                    start.display_prev(f)?;
                } else {
                    write!(f, ">={}", start)?;
                }

                if end.is_some() {
                    write!(f, " && ")?;
                }
            }

            if let Some(end) = end {
                if end.has_prev() {
                    write!(f, "<=")?;
                    end.display_prev(f)?;
                } else {
                    write!(f, "<{}", end)?;
                }
            }
        }
        Ok(())
    }
}
