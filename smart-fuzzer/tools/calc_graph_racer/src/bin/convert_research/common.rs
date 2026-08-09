//! Shared clean-room models for the W109 CONVERT constant/operation race.
//!
//! This module deliberately models only the unit table and arithmetic graph
//! visible from OxFunc plus exact decimal interpretations of those public
//! constants.  It does not inspect Excel binaries or any Microsoft-shipped
//! implementation artifact.

#![allow(dead_code)]

use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODEL_NAMES: [&str; 12] = [
    "f64_mul_div",
    "f64_ratio_mul",
    "f64_mul_recip",
    "x87_f64_cont_pc64",
    "x87_decimal_cont_pc64",
    "x87_decimal_mul_store_pc64",
    "x87_decimal_ratio_cont_pc64",
    "x87_decimal_ratio_store_f64_mul",
    "x87_decimal_inverse_table_pc64",
    "x87_decimal_cont_pc53",
    "x87_decimal_cont_pc64_identity_shortcut",
    "x87_f64_cont_pc64_identity_shortcut",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Length,
    Mass,
    Time,
    Pressure,
    Volume,
}

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Pressure => "pressure",
            Self::Volume => "volume",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitSpec {
    pub name: &'static str,
    pub category: Category,
    /// Exact decimal spelling represented by the current public OxFunc table.
    pub factor_decimal: &'static str,
}

pub const DIRECT_UNITS: [UnitSpec; 25] = [
    UnitSpec {
        name: "m",
        category: Category::Length,
        factor_decimal: "1",
    },
    UnitSpec {
        name: "in",
        category: Category::Length,
        factor_decimal: "0.0254",
    },
    UnitSpec {
        name: "ft",
        category: Category::Length,
        factor_decimal: "0.3048",
    },
    UnitSpec {
        name: "yd",
        category: Category::Length,
        factor_decimal: "0.9144",
    },
    UnitSpec {
        name: "mi",
        category: Category::Length,
        factor_decimal: "1609.344",
    },
    UnitSpec {
        name: "Nmi",
        category: Category::Length,
        factor_decimal: "1852",
    },
    UnitSpec {
        name: "g",
        category: Category::Mass,
        factor_decimal: "1",
    },
    UnitSpec {
        name: "lbm",
        category: Category::Mass,
        factor_decimal: "453.59237",
    },
    UnitSpec {
        name: "ozm",
        category: Category::Mass,
        factor_decimal: "28.349523125",
    },
    UnitSpec {
        name: "sec",
        category: Category::Time,
        factor_decimal: "1",
    },
    UnitSpec {
        name: "mn",
        category: Category::Time,
        factor_decimal: "60",
    },
    UnitSpec {
        name: "hr",
        category: Category::Time,
        factor_decimal: "3600",
    },
    UnitSpec {
        name: "day",
        category: Category::Time,
        factor_decimal: "86400",
    },
    UnitSpec {
        name: "Pa",
        category: Category::Pressure,
        factor_decimal: "1",
    },
    UnitSpec {
        name: "bar",
        category: Category::Pressure,
        factor_decimal: "100000",
    },
    UnitSpec {
        name: "atm",
        category: Category::Pressure,
        factor_decimal: "101325",
    },
    UnitSpec {
        name: "psi",
        category: Category::Pressure,
        factor_decimal: "6894.757293168",
    },
    UnitSpec {
        name: "l",
        category: Category::Volume,
        factor_decimal: "1",
    },
    UnitSpec {
        name: "tsp",
        category: Category::Volume,
        factor_decimal: "0.00492892159375",
    },
    UnitSpec {
        name: "tbs",
        category: Category::Volume,
        factor_decimal: "0.01478676478125",
    },
    UnitSpec {
        name: "oz",
        category: Category::Volume,
        factor_decimal: "0.0295735295625",
    },
    UnitSpec {
        name: "cup",
        category: Category::Volume,
        factor_decimal: "0.2365882365",
    },
    UnitSpec {
        name: "pt",
        category: Category::Volume,
        factor_decimal: "0.473176473",
    },
    UnitSpec {
        name: "qt",
        category: Category::Volume,
        factor_decimal: "0.946352946",
    },
    UnitSpec {
        name: "gal",
        category: Category::Volume,
        factor_decimal: "3.785411784",
    },
];

pub const PREFIXES: [(&str, &str); 17] = [
    ("Y", "1000000000000000000000000"),
    ("Z", "1000000000000000000000"),
    ("E", "1000000000000000000"),
    ("P", "1000000000000000"),
    ("T", "1000000000000"),
    ("G", "1000000000"),
    ("M", "1000000"),
    ("k", "1000"),
    ("h", "100"),
    ("da", "10"),
    ("d", "0.1"),
    ("c", "0.01"),
    ("m", "0.001"),
    ("u", "0.000001"),
    ("n", "0.000000001"),
    ("p", "0.000000000001"),
    ("f", "0.000000000000001"),
];

pub const PREFIX_BASES: [(&str, Category); 5] = [
    ("sec", Category::Time),
    ("Pa", Category::Pressure),
    ("m", Category::Length),
    ("g", Category::Mass),
    ("l", Category::Volume),
];

#[derive(Clone, Debug)]
pub struct OwnedUnitSpec {
    pub name: String,
    pub category: Category,
    pub factor_decimal: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaRow {
    pub id: String,
    pub split: String,
    pub class: String,
    pub category: String,
    pub number_bits: String,
    pub from_unit: String,
    pub to_unit: String,
    pub informative: bool,
    pub predictions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaDocument {
    pub schema_version: String,
    pub function: String,
    pub selection_note: String,
    pub model_names: Vec<String>,
    pub rows: Vec<MetaRow>,
}

pub fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

pub fn f64_from_hex(raw: &str) -> Result<f64, String> {
    let digits = raw
        .strip_prefix("0x")
        .ok_or_else(|| format!("missing 0x prefix in {raw}"))?;
    let bits = u64::from_str_radix(digits, 16).map_err(|e| format!("bad f64 bits {raw}: {e}"))?;
    Ok(f64::from_bits(bits))
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn pow10(n: usize) -> u128 {
    (0..n).fold(1_u128, |acc, _| acc.checked_mul(10).unwrap())
}

/// Parse a finite positive decimal spelling as an exact rational.
fn decimal_rational(raw: &str) -> (u128, u128) {
    assert!(!raw.starts_with('-'), "unit factors must be positive");
    let (mantissa, exponent10) = match raw.find(['e', 'E']) {
        Some(index) => (&raw[..index], raw[index + 1..].parse::<i32>().unwrap()),
        None => (raw, 0),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some(parts) => parts,
        None => (mantissa, ""),
    };
    let digits = format!("{whole}{fraction}");
    let mut numerator = digits.parse::<u128>().unwrap();
    let mut denominator = pow10(fraction.len());
    if exponent10 >= 0 {
        numerator = numerator.checked_mul(pow10(exponent10 as usize)).unwrap();
    } else {
        denominator = denominator
            .checked_mul(pow10((-exponent10) as usize))
            .unwrap();
    }
    let divisor = gcd(numerator, denominator);
    (numerator / divisor, denominator / divisor)
}

/// Correctly round an exact positive decimal to an x87 80-bit value with a
/// 64-bit significand (round-to-nearest, ties-to-even).
pub fn ext_from_decimal(raw: &str) -> rx::Ext80 {
    let (numerator, denominator) = decimal_rational(raw);
    if numerator == 0 {
        return rx::Ext80([0; 10]);
    }

    // The f64 estimate is used only to locate the binary binade.  The exact
    // rational arithmetic below supplies every significand bit and performs
    // the actual rounding.
    let estimate = raw.parse::<f64>().unwrap();
    let mut exponent = estimate.log2().floor() as i32;
    let shift = 63 - exponent;
    // Do not materialize `numerator << shift`: even modest 17-digit decimal
    // literals below one can need more than 128 temporary bits although the
    // final significand is only 64 bits.  Binary long division keeps the
    // remainder bounded by the (small decimal) denominator throughout.
    let (quotient, remainder, scaled_denominator) = if shift >= 0 {
        let mut quotient = numerator / denominator;
        let mut remainder = numerator % denominator;
        for _ in 0..shift {
            quotient = quotient.checked_shl(1).expect("decimal quotient shift");
            remainder = remainder
                .checked_mul(2)
                .expect("decimal long-division remainder");
            if remainder >= denominator {
                remainder -= denominator;
                quotient += 1;
            }
        }
        (quotient, remainder, denominator)
    } else {
        let scaled_denominator = denominator
            .checked_shl((-shift) as u32)
            .expect("decimal denominator shift");
        (
            numerator / scaled_denominator,
            numerator % scaled_denominator,
            scaled_denominator,
        )
    };
    let twice_remainder = remainder.checked_mul(2).unwrap();
    let mut rounded = quotient
        + u128::from(
            twice_remainder > scaled_denominator
                || (twice_remainder == scaled_denominator && quotient & 1 == 1),
        );
    if rounded == (1_u128 << 64) {
        rounded >>= 1;
        exponent += 1;
    }
    assert!((1_u128 << 63..1_u128 << 64).contains(&rounded));
    let significand = rounded as u64;
    let biased = u16::try_from(exponent + 16_383).unwrap();
    let mut bytes = [0_u8; 10];
    bytes[..8].copy_from_slice(&significand.to_le_bytes());
    bytes[8..].copy_from_slice(&biased.to_le_bytes());
    rx::Ext80(bytes)
}

pub fn direct_units_in(category: Category) -> impl Iterator<Item = &'static UnitSpec> {
    DIRECT_UNITS.iter().filter(move |u| u.category == category)
}

pub fn direct_unit(name: &str) -> Option<OwnedUnitSpec> {
    DIRECT_UNITS
        .iter()
        .find(|u| u.name == name)
        .map(|u| OwnedUnitSpec {
            name: u.name.to_string(),
            category: u.category,
            factor_decimal: u.factor_decimal.to_string(),
        })
}

pub fn prefix_unit(prefix: &str, base: &str) -> Option<OwnedUnitSpec> {
    let (_, category) = PREFIX_BASES
        .iter()
        .find(|(candidate, _)| *candidate == base)?;
    let (_, decimal) = PREFIXES
        .iter()
        .find(|(candidate, _)| *candidate == prefix)?;
    Some(OwnedUnitSpec {
        name: format!("{prefix}{base}"),
        category: *category,
        factor_decimal: (*decimal).to_string(),
    })
}

fn f64_factor(unit: &OwnedUnitSpec) -> f64 {
    unit.factor_decimal.parse::<f64>().unwrap()
}

fn x87_cont(number: f64, from: &rx::Ext80, to: &rx::Ext80, control_word: u16) -> f64 {
    let product = rx::ext_mul(&rx::ext_from_f64(number), from, control_word);
    let quotient = rx::ext_div(&product, to, control_word);
    rx::ext_to_f64(&quotient, control_word)
}

pub fn predictions(
    number: f64,
    from: &OwnedUnitSpec,
    to: &OwnedUnitSpec,
) -> BTreeMap<String, String> {
    assert_eq!(from.category, to.category);
    let ff = f64_factor(from);
    let tf = f64_factor(to);
    let fe = ext_from_decimal(&from.factor_decimal);
    let te = ext_from_decimal(&to.factor_decimal);
    let x = rx::ext_from_f64(number);
    let one = rx::ext_from_f64(1.0);
    let cw = rx::CW_PC64_RN;

    let f64_mul_div = number * ff / tf;
    let f64_ratio_mul = number * (ff / tf);
    let f64_mul_recip = (number * ff) * (1.0 / tf);
    let x87_f64_cont_pc64 = x87_cont(number, &rx::ext_from_f64(ff), &rx::ext_from_f64(tf), cw);
    let x87_decimal_cont_pc64 = x87_cont(number, &fe, &te, cw);
    let decimal_product = rx::ext_mul(&x, &fe, cw);
    let decimal_product_stored = rx::ext_from_f64(rx::ext_to_f64(&decimal_product, cw));
    let x87_decimal_mul_store_pc64 =
        rx::ext_to_f64(&rx::ext_div(&decimal_product_stored, &te, cw), cw);
    let decimal_ratio = rx::ext_div(&fe, &te, cw);
    let x87_decimal_ratio_cont_pc64 = rx::ext_to_f64(&rx::ext_mul(&x, &decimal_ratio, cw), cw);
    let decimal_ratio_stored = rx::ext_to_f64(&decimal_ratio, cw);
    let x87_decimal_ratio_store_f64_mul = number * decimal_ratio_stored;

    // Alternate table orientation: store units-per-base values (rounded to
    // 64-bit extended) and evaluate x / inverse(from) * inverse(to).
    let inverse_from = rx::ext_div(&one, &fe, cw);
    let inverse_to = rx::ext_div(&one, &te, cw);
    let from_base = rx::ext_div(&x, &inverse_from, cw);
    let x87_decimal_inverse_table_pc64 =
        rx::ext_to_f64(&rx::ext_mul(&from_base, &inverse_to, cw), cw);
    let x87_decimal_cont_pc53 = x87_cont(number, &fe, &te, rx::CW_PC53_RN);
    let x87_decimal_cont_pc64_identity_shortcut = if from.name == to.name {
        number
    } else {
        x87_decimal_cont_pc64
    };
    let x87_f64_cont_pc64_identity_shortcut = if from.name == to.name {
        number
    } else {
        x87_f64_cont_pc64
    };

    let values = [
        f64_mul_div,
        f64_ratio_mul,
        f64_mul_recip,
        x87_f64_cont_pc64,
        x87_decimal_cont_pc64,
        x87_decimal_mul_store_pc64,
        x87_decimal_ratio_cont_pc64,
        x87_decimal_ratio_store_f64_mul,
        x87_decimal_inverse_table_pc64,
        x87_decimal_cont_pc53,
        x87_decimal_cont_pc64_identity_shortcut,
        x87_f64_cont_pc64_identity_shortcut,
    ];
    MODEL_NAMES
        .iter()
        .zip(values)
        .map(|(name, value)| ((*name).to_string(), hex(value)))
        .collect()
}

pub fn predictions_are_informative(predictions: &BTreeMap<String, String>) -> bool {
    let mut values = predictions.values();
    let Some(first) = values.next() else {
        return false;
    };
    values.any(|value| value != first)
}

/// Monotone integer key used for signed ULP residuals.
pub fn ordered_bits(bits: u64) -> i128 {
    if bits & (1_u64 << 63) != 0 {
        i128::from(!bits)
    } else {
        i128::from(bits | (1_u64 << 63))
    }
}
