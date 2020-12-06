pub fn assert_on_decimal(data_to_check: f64, expected: f64) {
    let abs_diff = (data_to_check - expected).abs();
    assert!(abs_diff < 0.0005);
}
