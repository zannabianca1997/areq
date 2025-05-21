use std::{fmt::Debug, str::FromStr};

use derive_more::{Debug as DebugDerive, Display as DisplayDerive, IsVariant};
use lazy_regex::regex_switch;
use num_bigint::BigUint;
use snafu::Snafu;

/// An identifier for a pre-release
#[derive(DebugDerive, Clone, PartialEq, Eq, Hash, IsVariant, PartialOrd, Ord, DisplayDerive)]
pub enum Prerelease {
    Numeric(NumericPrerelease),
    Alpha(AlphaPrerelease),
}

#[derive(DebugDerive, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DisplayDerive)]
#[repr(transparent)]
pub struct NumericPrerelease(BigUint);

#[derive(DebugDerive, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DisplayDerive)]
#[repr(transparent)]
pub struct AlphaPrerelease(String);

impl FromStr for Prerelease {
    type Err = InvalidPrerelease;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        regex_switch!(
         s,
         r"^0|[1-9]\d*$" => Prerelease::Numeric(NumericPrerelease(
             s.parse()
                 .expect("The conversion to BigUint should be infallible"),
          ) ),
         r"^\d*[a-zA-Z-][0-9a-zA-Z-]*$" => Prerelease::Alpha(AlphaPrerelease(s.to_owned()))
        )
        .ok_or_else(|| debug_invalid_identifier(s))
    }
}

fn debug_invalid_identifier(s: &str) -> InvalidPrerelease {
    if s.is_empty() {
        return InvalidPrerelease::Empty;
    }

    if let Some(ch) = s.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '-') {
        return InvalidPrerelease::InvalidCharacters {
            id: s.to_string(),
            ch,
        };
    }

    if s.chars().all(|c| c.is_ascii_digit()) {
        if s.starts_with('0') && s.len() > 1 {
            return InvalidPrerelease::LeadingZeros { id: s.to_string() };
        }
    }

    unreachable!(
        "At least one error should match if the regular expression did not. The passing identifier is {s}"
    )
}

#[derive(DebugDerive, Clone, Snafu)]
pub enum InvalidPrerelease {
    #[snafu(display("Prerelease cannot be empty"))]
    Empty,
    #[snafu(display("Numeric prerelease must not start with zero: `{id}`"))]
    LeadingZeros { id: String },
    #[snafu(display(
        "Prerelease must be composed of alphanumeric characters or hyphens, not '{ch}': `{id}`"
    ))]
    InvalidCharacters { id: String, ch: char },
}
