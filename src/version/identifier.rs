use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use derive_more::{Debug as DebugDerive, Display as DisplayDerive, IsVariant};
use num_bigint::BigUint;
use snafu::Snafu;

/// An identifier for a pre-release or build
#[derive(DebugDerive, Clone, PartialEq, Eq, Hash, IsVariant, PartialOrd, Ord, DisplayDerive)]
pub enum Identifier {
    Numeric(BigUint),
    Alpha(String),
}

impl Identifier {
    pub fn new<T: AsRef<str> + Display + Debug>(id: T) -> Result<Self, InvalidIdentifier<T>> {
        let s = id.as_ref();

        if s.is_empty() {
            return Err(InvalidIdentifier::Empty);
        }

        if s.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-') {
            return Err(InvalidIdentifier::InvalidCharacters { id });
        }

        if s.chars().all(|c| c.is_ascii_digit()) {
            if s.starts_with('0') && s.len() > 1 {
                return Err(InvalidIdentifier::LeadingZero { id });
            }
            return Ok(Self::Numeric(
                s.parse()
                    .expect("The conversion to BigUint should never fail"),
            ));
        }

        Ok(Self::Alpha(s.to_string()))
    }
}

impl<'a> TryFrom<&'a str> for Identifier {
    type Error = InvalidIdentifier<&'a str>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Identifier {
    type Error = InvalidIdentifier<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Identifier {
    type Err = InvalidIdentifier<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).map_err(|e| e.to_owned())
    }
}

#[derive(DebugDerive, Clone, Snafu)]
pub enum InvalidIdentifier<T: AsRef<str> + Display + Debug> {
    #[snafu(display("Identifier cannot be empty"))]
    Empty,
    #[snafu(display("Numeric identifiers must not start with zero: {id}"))]
    LeadingZero { id: T },
    #[snafu(display("Identifiers must be composed of alphanumeric characters or hyphens: `{id}`"))]
    InvalidCharacters { id: T },
}

impl<T: AsRef<str> + Display + Debug> InvalidIdentifier<T> {
    pub fn id(&self) -> &str {
        match self {
            InvalidIdentifier::Empty => "",
            InvalidIdentifier::LeadingZero { id } | InvalidIdentifier::InvalidCharacters { id } => {
                id.as_ref()
            }
        }
    }

    pub fn as_ref(&self) -> InvalidIdentifier<&T> {
        match self {
            InvalidIdentifier::Empty => InvalidIdentifier::Empty,
            InvalidIdentifier::LeadingZero { id } => InvalidIdentifier::LeadingZero { id },
            InvalidIdentifier::InvalidCharacters { id } => {
                InvalidIdentifier::InvalidCharacters { id }
            }
        }
    }

    pub fn map<U: AsRef<str> + Display + Debug, F: FnOnce(T) -> U>(
        self,
        f: F,
    ) -> InvalidIdentifier<U> {
        match self {
            InvalidIdentifier::Empty => InvalidIdentifier::Empty,
            InvalidIdentifier::LeadingZero { id } => InvalidIdentifier::LeadingZero { id: f(id) },
            InvalidIdentifier::InvalidCharacters { id } => {
                InvalidIdentifier::InvalidCharacters { id: f(id) }
            }
        }
    }

    pub fn to_owned(&self) -> InvalidIdentifier<String> {
        self.as_ref().map(ToString::to_string)
    }
}
