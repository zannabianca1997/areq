use super::Range;

#[test]
fn empty() {
    assert!(Range::<u64>::EMPTY.is_empty());
}

#[test]
fn empty_eval_equal() {
    assert_eq!(Range::between(4, 2), Range::between(400, 20));
}

#[test]
fn intersect_extremals() {
    assert!(Range::between_include_end(4, 6).intersect(&Range::between(6, 7)));
    assert!(!Range::between(4, 6).intersect(&Range::between(6, 9)));
}
