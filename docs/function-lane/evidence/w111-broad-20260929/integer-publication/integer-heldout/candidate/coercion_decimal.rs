//! Observable decimal parser arithmetic inferred from W111 live Excel captures.
//! Decimal powers are scaled in ascending base-16 exponent stages, each rounded
//! to 64 binary significand bits. This is a behavioral model, not a claim about
//! Excel's internal implementation. See numeric-text/nibble-candidate-freeze.json.

#[path = "coercion_decimal_powers.rs"]
mod powers;

fn round_integer(value: u128, shift: u32) -> u128 {
    if shift == 0 {
        return value;
    }
    let quotient = value >> shift;
    let remainder = value & ((1_u128 << shift) - 1);
    let half = 1_u128 << (shift - 1);
    quotient + u128::from(remainder > half || (remainder == half && quotient & 1 != 0))
}

/// `significand * 10^scale` using the independently frozen staged scale model.
/// Inputs have already passed lexical admission and the 15-digit retention rule.
pub(super) fn decimal_to_binary(significand: u64, scale: i32) -> Option<f64> {
    if significand == 0 {
        return Some(0.0);
    }
    let mut mantissa = significand;
    let mut exponent = 0_i32;
    let magnitude = scale.unsigned_abs();
    let table = if scale < 0 {
        &powers::NEGATIVE
    } else {
        &powers::POSITIVE
    };
    for power in [magnitude & 15, magnitude & 240, magnitude & 3840] {
        if power == 0 {
            continue;
        }
        let index = if power < 16 {
            power
        } else if power < 256 {
            15 + power / 16
        } else {
            31
        };
        // Lexical decimal point admission bounds scale to -323..=307.
        debug_assert!(power <= 256);
        let (factor, factor_exponent) = table[index as usize];
        let product = u128::from(mantissa) * u128::from(factor);
        let shift = (128 - product.leading_zeros()).saturating_sub(64);
        let rounded = round_integer(product, shift);
        exponent += i32::from(factor_exponent) + shift as i32;
        if rounded == 1_u128 << 64 {
            mantissa = 1_u64 << 63;
            exponent += 1;
        } else {
            mantissa = rounded as u64;
        }
    }
    // One final nearest-even rounding to binary64's 53 significand bits, then
    // construct the encoding so intermediate overflow/underflow cannot occur.
    let width = 64 - mantissa.leading_zeros();
    let mut final_mantissa = if width > 53 {
        exponent += (width - 53) as i32;
        round_integer(u128::from(mantissa), width - 53) as u64
    } else {
        exponent -= (53 - width) as i32;
        mantissa << (53 - width)
    };
    if final_mantissa == 1_u64 << 53 {
        final_mantissa >>= 1;
        exponent += 1;
    }
    let exponent = exponent + 52;
    if exponent < -1022 {
        return Some(0.0);
    }
    if exponent > 1023 {
        return None;
    }
    Some(f64::from_bits(
        ((exponent + 1023) as u64) << 52 | (final_mantissa & ((1_u64 << 52) - 1)),
    ))
}
