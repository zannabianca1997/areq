//! Implementation of semantic versioning

use std::{fmt::Display, num::ParseIntError, str::FromStr};

use derive_more::Display;
use lazy_regex::regex_captures;
use snafu::{ResultExt, Snafu};

pub mod prerelease;
use prerelease::{InvalidPrerelease, Prerelease};

/// A semantic version with no metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PureVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Vec<Prerelease>,
}

impl PureVersion {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: vec![],
        }
    }

    pub fn is_major_zero(&self) -> bool {
        self.major == 0
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    pub(super) fn from_checked_parts(
        major: &str,
        minor: &str,
        patch: &str,
        pre: &str,
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

        let pre = pre
            .split('.')
            .map(|p| {
                p.parse()
                    .expect("The regex only matches valid prerelase identifiers")
            })
            .collect();

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
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre[0])?;
            for pre in &self.pre[1..] {
                write!(f, ".{}", pre)?;
            }
        }

        Ok(())
    }
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

    if let Err(source) = major.parse::<u64>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Major,
            value: major.to_string(),
            source,
        };
    };
    if let Err(source) = minor.parse::<u64>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Minor,
            value: minor.to_string(),
            source,
        };
    };
    if let Err(source) = patch.parse::<u64>() {
        return InvalidPureVersion::InvalidNumericPart {
            part: NumericPart::Patch,
            value: patch.to_string(),
            source,
        };
    };

    for pre in pre.split('.') {
        if let Err(source) = pre.parse::<Prerelease>() {
            return InvalidPureVersion::InvalidPrerelease { source };
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
