use crate::value::WorksheetErrorCode;

pub const BIT_MAX: u64 = 281_474_976_710_655;
pub const BIT_SHIFT_MAX: i32 = 53;

pub fn coerce_bit_operand(n: f64) -> Result<u64, WorksheetErrorCode> {
    if n < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let truncated = n.trunc();
    if truncated > BIT_MAX as f64 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(truncated as u64)
}

pub fn coerce_shift_count(n: f64) -> Result<i32, WorksheetErrorCode> {
    let truncated = n.trunc();
    if truncated.abs() > BIT_SHIFT_MAX as f64 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(truncated as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_common_bounds_match_excel_seed() {
        assert_eq!(coerce_bit_operand(281_474_976_710_655.0), Ok(BIT_MAX));
        assert_eq!(
            coerce_bit_operand(281_474_976_710_656.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(coerce_shift_count(53.0), Ok(53));
        assert_eq!(coerce_shift_count(54.0), Err(WorksheetErrorCode::Num));
    }
}
