use proc_macro_fixtures::fixtures;

#[test]
fn should_allow_key_value_pair() {
    let result = fixtures!("one", "two");
    let expected = vec![("one", "two")];
    assert_eq!(result, expected);
    // should not allow files to be above
}
