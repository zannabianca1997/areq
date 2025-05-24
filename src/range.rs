use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};

mod numeric_impls;

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
/// Implementors must ensure that valid string representations must not contains
/// the character `,`, and must not start with `=`.
pub trait RangeExtremeFromStr: RangeExtreme + FromStr {}

/// A range of versions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range<T> {
    start: T,
    end: T,
}

impl<T> Range<T>
where
    T: RangeExtreme,
{
    /// Empty range
    pub const EMPTY: Self = Self {
        start: T::MAX,
        end: T::MAX,
    };

    /// Full range
    pub const FULL: Self = Self {
        start: T::MIN,
        end: T::MAX,
    };

    /// Create a new range from `start` to `end`, including `start` and excluding `end`
    pub fn between(start: T, end: T) -> Self {
        // Ensure all empty ranges are considered equal
        if start >= end {
            return Self::EMPTY;
        }
        Self { start, end }
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
        Self::between(start, T::MAX)
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

    /// Return whether the range contains exactly one element.
    pub fn is_single(&self) -> bool {
        self.start.compare_next_to(&self.end)
    }

    /// Return whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self == &Self::EMPTY
    }

    /// Return whether `value` is inside the range.
    pub fn contains(&self, value: &T) -> bool {
        &self.start <= value && value < &self.end
    }

    /// Return whether the range completely contains another range.
    ///
    /// True if and only if all values belonging to other also belong to self
    pub fn contains_range(&self, other: &Self) -> bool {
        if other.is_empty() {
            return true;
        }
        self.start <= other.start && other.end <= self.end
    }

    /// Return whether two ranges intersect
    ///
    /// True if and only if there exists a value such that it belongs to both ranges
    pub fn intersect(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.start < other.end && other.start < self.end
    }

    /// Return the intersection of two ranges
    pub fn intersection(&self, other: &Self) -> Self {
        if !self.intersect(other) {
            return Self::EMPTY;
        }
        Self::between(
            std::cmp::max(&self.start, &other.start).clone(),
            std::cmp::min(&self.end, &other.end).clone(),
        )
    }
}

impl<T> Display for Range<T>
where
    T: RangeExtremeDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_impl(&self.start, &self.end, f)
    }
}

/// Display a range in a human-readable format.
///
/// The range is displayed as either:
/// - `==<value>` if the range contains only one element
/// - `>=[start],<[end]` if the range contains all elements from `start` to `end` (inclusive or exclusive)
/// - `>=[start]` if the range contains all elements strictly greater than `start`
/// - `<[end]` if the range contains all elements strictly less than `end`
///
/// Inclusive variant of end and exclusive variant of start are displayied similarly
fn display_impl<T>(start: &T, end: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
    T: RangeExtremeDisplay,
{
    if start == end {
        return write!(f, "!");
    }

    if start.compare_next_to(&end) {
        return write!(f, "=={}", start);
    }

    let mut need_comma = false;

    if start != &T::MIN {
        need_comma = true;

        if start.has_prev() {
            write!(f, ">")?;
            start.display_prev(f)?;
        } else {
            write!(f, ">={}", start)?;
        }
    }

    if end != &T::MAX {
        if need_comma {
            write!(f, ",")?;
        }

        if end.has_prev() {
            write!(f, "<=")?;
            end.display_prev(f)?;
        } else {
            write!(f, "<{}", end)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RangeParseError<TErr> {
    ParseExtreme { source: TErr },
    UnrecognizedConstraintOperator { constraint: String },
}

impl<T> std::fmt::Display for RangeParseError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeParseError::ParseExtreme { .. } => write!(f, "Cannot parse range extreme"),
            RangeParseError::UnrecognizedConstraintOperator { constraint } => {
                write!(
                    f,
                    "Unrecognized constraint operator (expected one of `<=`, `<`, `>=`, `>`, `==`): {constraint}"
                )
            }
        }
    }
}

impl<TErr> Error for RangeParseError<TErr>
where
    TErr: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RangeParseError::ParseExtreme { source } => Some(source),
            RangeParseError::UnrecognizedConstraintOperator { .. } => None,
        }
    }
}

impl<T> FromStr for Range<T>
where
    T: RangeExtremeFromStr,
{
    type Err = RangeParseError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut range = Range::FULL;

        for constraint in s.split(',') {
            let constraint = constraint.trim();

            let (fun, extreme): (fn(T) -> Range<T>, &str) = if constraint.is_empty() {
                // Equivalent to (|_| Range::FULL, ""), but faster
                continue;
            } else if constraint == "!" {
                // Equivalent to (|_| Range::EMPTY, ""), but faster
                return Ok(Range::EMPTY);
            } else if let Some(constraint) = constraint.strip_prefix("<=") {
                (Range::to_inclusive, constraint)
            } else if let Some(constraint) = constraint.strip_prefix("<") {
                (Range::to, constraint)
            } else if let Some(constraint) = constraint.strip_prefix(">=") {
                (Range::from, constraint)
            } else if let Some(constraint) = constraint.strip_prefix(">") {
                (Range::from_exclusive, constraint)
            } else if let Some(constraint) = constraint.strip_prefix("==") {
                (Range::single, constraint)
            } else {
                return Err(RangeParseError::UnrecognizedConstraintOperator {
                    constraint: constraint.to_string(),
                });
            };

            let extreme = extreme
                .trim_start()
                .parse()
                .map_err(|source| RangeParseError::ParseExtreme { source })?;

            let constraint = fun(extreme);

            range = range.intersection(&constraint);
            if range.is_empty() {
                break;
            }
        }

        Ok(range)
    }
}
