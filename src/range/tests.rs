use super::Ranges;

#[test]
fn empty() {
    assert!(Ranges::<u64>::EMPTY.is_empty());
}

#[test]
fn empty_eval_equal() {
    assert_eq!(Ranges::between(4, 2), Ranges::between(400, 20));
}
