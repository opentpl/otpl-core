pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[test]
#[ignore]
fn it_works() {
    assert_eq!(4, add_two(2));
}