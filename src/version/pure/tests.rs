use std::str::FromStr;

use super::PureVersion;

static SORTED: &[&'static str] = &[
    "1.0.0-alpha",
    "1.0.0-alpha.1",
    "1.0.0-alpha.beta",
    "1.0.0-beta",
    "1.0.0-beta.2",
    "1.0.0-beta.11",
    "1.0.0-rc.1",
    "1.0.0",
];

#[test]
fn can_parse() {
    for v in SORTED {
        PureVersion::from_str(v).unwrap();
    }
}

#[test]
fn prereleases_are_sorted() {
    assert!(SORTED.is_sorted_by_key(|v| PureVersion::from_str(v).unwrap()))
}

#[test]
fn roundtrips() {
    for v in SORTED {
        let back = PureVersion::from_str(v).unwrap().to_string();
        assert_eq!(v, &back)
    }
}
