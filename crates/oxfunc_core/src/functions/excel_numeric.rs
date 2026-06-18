use crate::value::WorksheetErrorCode;

pub(crate) fn excel_underflow_to_zero(value: f64) -> f64 {
    if value.is_finite() && value != 0.0 && value.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        value
    }
}

/// Excel never publishes a non-finite numeric result: a kernel that overflows to
/// `±Inf` or produces `NaN` surfaces as `#NUM!`. Use this to guard kernels whose
/// output can run off to infinity (SINH/COSH, PERMUTATIONA, ...).
///
/// Functions that *saturate* in Excel (e.g. COTH/TANH/FISHERINV return `±1` for
/// large arguments) must NOT use this guard — they should produce the saturated
/// value directly.
pub(crate) fn finite_or_num(value: f64) -> Result<f64, WorksheetErrorCode> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_or_num_passes_finite_and_rejects_non_finite() {
        assert_eq!(finite_or_num(1.5), Ok(1.5));
        assert_eq!(finite_or_num(0.0), Ok(0.0));
        assert_eq!(finite_or_num(f64::INFINITY), Err(WorksheetErrorCode::Num));
        assert_eq!(
            finite_or_num(f64::NEG_INFINITY),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(finite_or_num(f64::NAN), Err(WorksheetErrorCode::Num));
    }
}
