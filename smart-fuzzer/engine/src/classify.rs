//! Typed outcome comparison and the ODR-FN-005 severity classes, defined once.
//!
//! Distance between two finite doubles is the ordinal distance on the IEEE total order
//! (the number of representable doubles between them). Severity of a single row:
//!
//! * `Structural`: the outcomes differ in kind (number vs error vs text vs logical), or are
//!   both errors/texts/logicals with different payloads.
//! * `Gross`: both numbers, but opposite sign (non-zero), one non-finite, or > 1024 ULP apart.
//! * `Numeric`: 5..=1024 ULP.
//! * `LastBit`: 1..=4 ULP, or +0 vs -0.
//!
//! A function carries its worst row's class.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outcome {
    Number { bits: u64 },
    Error { code: String },
    Logical { value: bool },
    Text { value: String },
    /// Anything the comparison does not model (arrays, references, rich values).
    Other { detail: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    LastBit,
    Numeric,
    Gross,
    Structural,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RowVerdict {
    pub agree: bool,
    pub severity: Option<Severity>,
    /// Ordinal ULP distance when both sides are finite numbers.
    pub ulp: Option<u64>,
}

/// Map a double's bits onto a monotone integer line (the IEEE total order), so that the
/// difference of two mapped values is the ordinal ULP distance. +0 and -0 both map to 0.
fn ordinal(bits: u64) -> i128 {
    let signed = bits as i64 as i128;
    if signed < 0 {
        -(signed & 0x7fff_ffff_ffff_ffff)
    } else {
        signed
    }
}

pub fn ulp_distance(a: u64, b: u64) -> u64 {
    (ordinal(a) - ordinal(b)).unsigned_abs() as u64
}

pub fn compare(expected: &Outcome, actual: &Outcome) -> RowVerdict {
    let agree = |ulp| RowVerdict { agree: true, severity: None, ulp };
    let differ = |severity, ulp| RowVerdict { agree: false, severity: Some(severity), ulp };
    match (expected, actual) {
        (Outcome::Number { bits: e }, Outcome::Number { bits: a }) => {
            if e == a {
                return agree(Some(0));
            }
            let (fe, fa) = (f64::from_bits(*e), f64::from_bits(*a));
            if fe.is_nan() || fa.is_nan() || fe.is_infinite() || fa.is_infinite() {
                return differ(Severity::Gross, None);
            }
            if fe == 0.0 && fa == 0.0 {
                // +0 vs -0: same value, different bits.
                return differ(Severity::LastBit, Some(0));
            }
            if (fe < 0.0) != (fa < 0.0) && fe != 0.0 && fa != 0.0 {
                return differ(Severity::Gross, Some(ulp_distance(*e, *a)));
            }
            let ulp = ulp_distance(*e, *a);
            let severity = match ulp {
                1..=4 => Severity::LastBit,
                5..=1024 => Severity::Numeric,
                _ => Severity::Gross,
            };
            differ(severity, Some(ulp))
        }
        (e, a) if e == a => agree(None),
        _ => differ(Severity::Structural, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(x: f64) -> Outcome {
        Outcome::Number { bits: x.to_bits() }
    }

    #[test]
    fn ulp_distance_is_ordinal_across_zero_and_sign() {
        assert_eq!(ulp_distance(1.0f64.to_bits(), 1.0f64.to_bits()), 0);
        assert_eq!(ulp_distance(1.0f64.to_bits(), (1.0f64.to_bits() + 1)), 1);
        assert_eq!(ulp_distance(0.0f64.to_bits(), (-0.0f64).to_bits()), 0);
        // smallest positive subnormal to smallest negative subnormal: 2 steps via zero
        assert_eq!(ulp_distance(1u64, (1u64 | (1 << 63))), 2);
    }

    #[test]
    fn severity_bands_follow_odr_fn_005() {
        let base = 1.5f64.to_bits();
        let at = |d: u64| Outcome::Number { bits: base + d };
        assert!(compare(&at(0), &at(0)).agree);
        assert_eq!(compare(&at(0), &at(4)).severity, Some(Severity::LastBit));
        assert_eq!(compare(&at(0), &at(5)).severity, Some(Severity::Numeric));
        assert_eq!(compare(&at(0), &at(1024)).severity, Some(Severity::Numeric));
        assert_eq!(compare(&at(0), &at(1025)).severity, Some(Severity::Gross));
        assert_eq!(compare(&n(1.0), &n(-1.0)).severity, Some(Severity::Gross));
        assert_eq!(compare(&n(0.0), &n(-0.0)).severity, Some(Severity::LastBit));
        assert_eq!(compare(&n(1.0), &n(f64::INFINITY)).severity, Some(Severity::Gross));
        let num_err = Outcome::Error { code: "Num".into() };
        assert_eq!(compare(&n(1.0), &num_err).severity, Some(Severity::Structural));
        let val_err = Outcome::Error { code: "Value".into() };
        assert_eq!(compare(&num_err, &val_err).severity, Some(Severity::Structural));
        assert!(compare(&num_err, &num_err.clone()).agree);
        assert!(Severity::LastBit < Severity::Structural);
    }
}
