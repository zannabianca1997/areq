use std::{fmt::Display, str::FromStr};

use build::{BuildMetadata, InvalidBuildMetadata};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use lazy_regex::regex_captures;
use pure::{InvalidPureVersion, PureVersion};
use snafu::Snafu;

pub mod build;
pub mod pure;

/// A semantic version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, AsRef, AsMut, DerefMut)]
pub struct Version {
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub pure: PureVersion,
    pub build: Vec<BuildMetadata>,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pure.fmt(f)?;
        if !self.build.is_empty() {
            write!(f, "+{}", self.build[0])?;
            for build in &self.build[1..] {
                write!(f, ".{}", build)?;
            }
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = InvalidVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((_, major, minor, patch, pre, build)) = regex_captures!(
            r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$",
            s
        ) else {
            let (pure, build) = s.split_once('+').unwrap_or((s, ""));

            if let Err(source) = pure.parse::<PureVersion>() {
                return Err(InvalidVersion::InvalidPureVersion { source });
            }

            if !build.is_empty() {
                for build in build.split('.') {
                    if let Err(source) = build.parse::<BuildMetadata>() {
                        return Err(InvalidVersion::InvalidBuildMetadata { source });
                    }
                }
            }

            unreachable!(
                "At least one of the preceding conditions should fail if the regex failed. The passing version is {s}"
            );
        };

        let pure = PureVersion::from_checked_parts(major, minor, patch, pre)?;

        let build = if !build.is_empty() {
            build
                .split('.')
                .map(|p| {
                    p.parse()
                        .expect("The regex only matches valid build metadata")
                })
                .collect()
        } else {
            vec![]
        };

        Ok(Self { pure, build })
    }
}

#[derive(Debug, Clone, Snafu)]
pub enum InvalidVersion {
    #[snafu(transparent)]
    InvalidPureVersion { source: InvalidPureVersion },
    #[snafu(display("Invalid build metadata"))]
    InvalidBuildMetadata { source: InvalidBuildMetadata },
}
