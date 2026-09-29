//! W111 current-baseline generic Number-to-Text rendering. The decimal pair is
//! shared with ROUND; COMPLEX keeps its distinct observed formatting policy.
//! Only finite normal/zero values were admitted by the live Value2 capture.

pub(crate) fn finite_number_text(number: f64) -> String {
    if number == 0.0 {
        return "0".to_string();
    }
    let (mut digits, mut scale) = super::round_fn::fifteen_significant_digits(number, false);
    while digits % 10 == 0 {
        digits /= 10;
        scale += 1;
    }
    let mut text = digits.to_string();
    let point = text.len() as i32 + scale;
    let fixed = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), text)
    } else if point >= text.len() as i32 {
        format!(
            "{}{}",
            text,
            "0".repeat((point - text.len() as i32) as usize)
        )
    } else {
        format!("{}.{}", &text[..point as usize], &text[point as usize..])
    };
    let unsigned = if fixed.len() <= 20 {
        fixed
    } else {
        let mut exponent = point - 1;
        // The 3-digit exponent format reserves one more mantissa position;
        // its observed transition includes exponent 99 on either side.
        if exponent.abs() >= 99 && text.len() > 14 {
            let divisor = 10_u128.pow(text.len() as u32 - 14);
            digits = digits / divisor + u128::from(digits % divisor >= divisor / 2);
            scale += (text.len() - 14) as i32;
            while digits % 10 == 0 {
                digits /= 10;
                scale += 1;
            }
            text = digits.to_string();
            exponent = text.len() as i32 + scale - 1;
        }
        let mantissa = if text.len() == 1 {
            text
        } else {
            format!("{}.{}", &text[..1], &text[1..])
        };
        format!(
            "{mantissa}E{}{:02}",
            if exponent < 0 { "-" } else { "+" },
            exponent.abs()
        )
    };
    if number < 0.0 {
        format!("-{unsigned}")
    } else {
        unsigned
    }
}
