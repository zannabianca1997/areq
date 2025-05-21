use std::{fmt::Debug, str::FromStr};

use derive_more::{Debug as DebugDerive, Display as DisplayDerive};
use lazy_regex::regex_if;
use snafu::Snafu;

/// An identifier for a build

#[derive(DebugDerive, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DisplayDerive)]
#[repr(transparent)]
pub struct BuildMetadata(String);

impl FromStr for BuildMetadata {
    type Err = InvalidBuildMetadata;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        regex_if!(r"^\d*[0-9a-zA-Z-]+$", s, BuildMetadata(s.to_owned()))
            .ok_or_else(|| debug_invalid_identifier(s))
    }
}

fn debug_invalid_identifier(s: &str) -> InvalidBuildMetadata {
    if s.is_empty() {
        return InvalidBuildMetadata::Empty;
    }

    if let Some(ch) = s.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '-') {
        return InvalidBuildMetadata::InvalidCharacters {
            id: s.to_string(),
            ch,
        };
    }

    unreachable!(
        "At least one error should match if the regular expression did not. The passing identifier is {s}"
    )
}

#[derive(DebugDerive, Clone, Snafu)]
pub enum InvalidBuildMetadata {
    #[snafu(display("Build metadata cannot be empty"))]
    Empty,
    #[snafu(display(
        "Build metadata must be composed of alphanumeric characters or hyphens, not '{ch}': `{id}`"
    ))]
    InvalidCharacters { id: String, ch: char },
}
