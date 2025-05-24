//! Implementation of semantic versioning

use std::{borrow::Cow, fmt::Display, num::ParseIntError, str::FromStr};

use chumsky::{Parser, prelude::*, text::digits};
use derive_more::Display;
use lazy_regex::regex_captures;
use snafu::{ResultExt, Snafu};

use crate::range::{self, ParserExtra};

pub mod prerelease;

use prerelease::{InvalidPrerelease, Prerelease};

#[cfg(test)]
mod tests;

pub type UInt = u64;

/// A semantic version with no metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PureVersion {
    pub major: UInt,
    pub minor: UInt,
    pub patch: UInt,
    pub pre: Cow<'static, [Prerelease]>,
}

impl PureVersion {
    const MIN: Self = Self {
        major: 0,
        minor: 0,
        patch: 0,
        pre: Cow::Borrowed({
            static V: [Prerelease; 1] = [Prerelease::MIN];
            &V
        }),
    };
    /// The maximum representable version
    const MAX: Self = Self {
        major: UInt::MAX,
        minor: UInt::MAX,
        patch: UInt::MAX,
        pre: Cow::Borrowed(&[]),
    };

    pub fn new(major: UInt, minor: UInt, patch: UInt) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Cow::Borrowed(&[]),
        }
    }

    pub fn is_major_zero(&self) -> bool {
        self.major == 0
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    /// Calculate the immediate successive version, such there are no version between this and that
    ///
    /// Note that this is not a "version bump", and normally generates nonsensical versions like `1.2.3-0.0.0.0`.
    /// The objective is simply to represent an exact version as a range [v, v.next()).
    fn next(mut self) -> Self {
        if !self.is_prerelease() {
            self.patch += 1;
        }
        self.pre.to_mut().push(Prerelease::MIN);
        self
    }

    /// Return if next is the next version
    fn compare_next_to(&self, other: &PureVersion) -> bool {
        other.has_prev()
            && self.major == other.major
            && self.minor == other.minor
            && if self.is_prerelease() {
                self.patch == other.patch && self.pre == other.pre.split_last().unwrap().1
            } else {
                self.patch + 1 == other.patch && other.pre.len() == 1
            }
    }

    /// Check if this version has a previous version, such there are no version between that and this
    fn has_prev(&self) -> bool {
        self.pre.last() == Some(&Prerelease::MIN)
            && if self.pre.len() == 1 {
                self.patch != UInt::MIN
            } else {
                true
            }
    }

    /*
        /// Calculate the immediate previous version if it exist, such there are no version between that and this
        ///
        /// If this version does not have a previous, return itself into the [`Err`] variant
        fn prev(mut self) -> Result<Self, Self> {
            if !self.has_prev() {
                return Err(self);
            }
            self.pre.to_mut().pop();
            if !self.is_prerelease() {
                self.patch -= 1
            }
            return Ok(self);
        }
    */

    /// Display the previous version without cloning
    ///
    /// Fails if [`has_pre`] is false
    fn display_prev(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.has_prev() {
            return std::fmt::Result::Err(std::fmt::Error);
        }
        display_impl(
            self.major,
            self.minor,
            self.patch - if self.pre.len() == 1 { 1 } else { 0 },
            self.pre.split_last().unwrap().1,
            f,
        )
    }

    pub(super) fn from_checked_parts(
        major: &str,
        minor: &str,
        patch: &str,
        pre: &str,
    ) -> Result<PureVersion, InvalidPureVersion> {
        let pre = if !pre.is_empty() {
            Cow::Owned(
                pre.split('.')
                    .map(|p| {
                        p.parse()
                            .expect("The regex only matches valid prerelase identifiers")
                    })
                    .collect(),
            )
        } else {
            Cow::Borrowed(&[] as &[_])
        };

        Self::from_checked_parts_splitted(major, minor, patch, pre)
    }
    pub(super) fn from_checked_parts_splitted(
        major: &str,
        minor: &str,
        patch: &str,
        pre: Cow<'static, [Prerelease]>,
    ) -> Result<PureVersion, InvalidPureVersion> {
        let major = major.parse().context(NumericPartTooLongSnafu {
            part: NumericPart::Major,
        })?;

        let minor = minor.parse().context(NumericPartTooLongSnafu {
            part: NumericPart::Minor,
        })?;

        let patch = patch.parse().context(NumericPartTooLongSnafu {
            part: NumericPart::Patch,
        })?;

        if patch == UInt::MAX && pre.is_empty() {
            return Err(InvalidPureVersion::PatchCannotBeUIntMax);
        }

        Ok(Self {
            major,
            minor,
            patch,
            pre,
        })
    }
}

impl Display for PureVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_impl(self.major, self.minor, self.patch, &*self.pre, f)
    }
}
fn display_impl(
    major: UInt,
    minor: UInt,
    patch: UInt,
    pre: &[Prerelease],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}.{}.{}", major, minor, patch)?;

    if !pre.is_empty() {
        write!(f, "-{}", pre[0])?;
        for pre in &pre[1..] {
            write!(f, ".{}", pre)?;
        }
    }

    Ok(())
}

impl FromStr for PureVersion {
    type Err = InvalidPureVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((_, major, minor, patch, pre)) = regex_captures!(
            r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?$",
            s
        ) else {
            return Err(debug_invalid_pure_version(s));
        };

        Self::from_checked_parts(major, minor, patch, pre)
    }
}

fn debug_invalid_pure_version(s: &str) -> InvalidPureVersion {
    let (version, pre) = s.split_once('-').unwrap_or((s, ""));

    let mut version = version.splitn(4, '.');
    let Some(major) = version.next() else {
        return InvalidPureVersion::MissingNumericPart {
            part: NumericPart::Major,
        };
    };
    let Some(minor) = version.next() else {
        return InvalidPureVersion::MissingNumericPart {
            part: NumericPart::Minor,
        };
    };
    let Some(patch) = version.next() else {
        return InvalidPureVersion::MissingNumericPart {
            part: NumericPart::Patch,
        };
    };
    if let Some(rest) = version.next() {
        return InvalidPureVersion::ExtraBeforePrereleases {
            extra: rest.to_string(),
        };
    }

    if let Err(source) = major.parse::<UInt>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Major,
            value: major.to_string(),
            source,
        };
    };
    if let Err(source) = minor.parse::<UInt>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Minor,
            value: minor.to_string(),
            source,
        };
    };
    if let Err(source) = patch.parse::<UInt>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Patch,
            value: patch.to_string(),
            source,
        };
    };

    if !pre.is_empty() {
        for pre in pre.split('.') {
            if let Err(source) = pre.parse::<Prerelease>() {
                return InvalidPureVersion::InvalidPrerelease { source };
            }
        }
    }

    unreachable!(
        "At least one of the preceding conditions should fail if the regex failed. The passing version is {s}"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum NumericPart {
    #[display("major")]
    Major,
    #[display("minor")]
    Minor,
    #[display("patch")]
    Patch,
}

#[derive(Debug, Clone, Snafu)]
pub enum InvalidPureVersion {
    #[snafu(display("The {part} version is too big to fit inside a 64 bit unsigned int"))]
    NumericPartTooLong {
        part: NumericPart,
        source: ParseIntError,
    },
    #[snafu(display("The {part} version is missing"))]
    MissingNumericPart { part: NumericPart },
    #[snafu(display("Additional data between numeric parts and prerelase: `{extra}`"))]
    ExtraBeforePrereleases { extra: String },
    #[snafu(display("Invalid {part} version: `{value}`"))]
    InvalidNumericPart {
        part: NumericPart,
        value: String,
        source: ParseIntError,
    },
    #[snafu(display("Invalid prerelease"))]
    InvalidPrerelease { source: InvalidPrerelease },
    #[snafu(display(
        "The patch version cannot be the maximum 64 bit unsigned int unless prerelease"
    ))]
    PatchCannotBeUIntMax,
}

impl PartialOrd for PureVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PureVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.major != other.major {
            return self.major.cmp(&other.major);
        }

        if self.minor != other.minor {
            return self.minor.cmp(&other.minor);
        }

        if self.patch != other.patch {
            return self.patch.cmp(&other.patch);
        }

        match (self.is_prerelease(), other.is_prerelease()) {
            (true, true) => self.pre.cmp(&other.pre),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => std::cmp::Ordering::Equal,
        }
    }
}

impl range::RangeExtreme for PureVersion {
    const MIN: Self = PureVersion::MIN;

    const MAX: Self = PureVersion::MAX;

    fn next(self) -> Self {
        PureVersion::next(self)
    }

    fn compare_next_to(&self, other: &Self) -> bool {
        PureVersion::compare_next_to(&self, other)
    }
}

impl range::RangeExtremeDisplay for PureVersion {
    fn has_prev(&self) -> bool {
        PureVersion::has_prev(&self)
    }

    fn display_prev(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        PureVersion::display_prev(&self, f)
    }
}

impl range::RangeExtremeParseable for PureVersion {
    fn parser<'a>() -> impl chumsky::Parser<'a, &'a str, Self, ParserExtra<'a>> + Clone {
        let numeric = digits(10).to_slice();

        numeric
            .clone()
            .labelled("major")
            .then_ignore(just('.'))
            .then(numeric.clone().labelled("minor"))
            .then_ignore(just('.'))
            .then(numeric.labelled("patch"))
            .then(
                just('-')
                    .ignore_then(
                        Prerelease::parser()
                            .separated_by(just('.'))
                            .at_least(1)
                            .collect::<Vec<_>>(),
                    )
                    .or_not(),
            )
            .try_map(|(((major, minor), patch), pre), span| {
                PureVersion::from_checked_parts_splitted(
                    major,
                    minor,
                    patch,
                    pre.map(Cow::Owned).unwrap_or(Cow::Borrowed(&[])),
                )
                .map_err(|err| Rich::custom(span, err))
            })
    }
}
