pub(crate) fn excel_underflow_to_zero(value: f64) -> f64 {
    if value.is_finite() && value != 0.0 && value.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        value
    }
}
