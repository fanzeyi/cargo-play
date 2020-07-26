fn add(a: i16, b: i16) -> i16 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(1, 2), 3);
}
