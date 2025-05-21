//! Implementation of semantic versioning

pub mod identifier;
use std::{fmt::Display, iter};

pub use identifier::Identifier;

/// A semantic version
#[derive(Debug, Clone)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Vec<Identifier>,
    pub build: Vec<Identifier>,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: vec![],
            build: vec![],
        }
    }

    pub fn is_major_zero(&self) -> bool {
        self.major == 0
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    pub fn has_build(&self) -> bool {
        !self.build.is_empty()
    }

    pub fn exact_eq(&self, other: &Self) -> bool {
        self.eq(other) && self.build == other.build
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre[0])?;
            for pre in &self.pre[1..] {
                write!(f, ".{}", pre)?;
            }
        }

        if !self.build.is_empty() {
            write!(f, "+{}", self.build[0])?;
            for build in &self.build[1..] {
                write!(f, ".{}", build)?;
            }
        }

        Ok(())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre == other.pre
    }
}
impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Version {
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

        if self.pre.is_empty() || other.pre.is_empty() {
            return self.pre.len().cmp(&other.pre.len()).reverse();
        }

        for (a, b) in iter::zip(&self.pre, &other.pre) {
            if a != b {
                return a.cmp(b);
            }
        }

        self.pre.len().cmp(&other.pre.len())
    }
}
