//! Frozen clean-room CONVERT v3 candidate shared by its scorer and the
//! oracle-blind publication-batch generator.
//!
//! The only v2 change is the length core's first-operation staging:
//! Excel's observed graph rounds `number * integer_angstrom_factor` through
//! x87 PC64 to binary64 before the binary64 division by the target factor.

#![allow(dead_code)]

use super::common;
use oxfunc_core::excel_numeric::research as rx;
use serde_json::{Value, json};

pub const FREEZE_ID: &str = "g4-05.convert.unified.20260809.v3";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prediction {
    Numeric(u64),
    ErrorNa,
}

impl Prediction {
    pub fn display(self) -> String {
        match self {
            Self::Numeric(bits) => format!("0x{bits:016x}"),
            Self::ErrorNa => "error:NA".to_string(),
        }
    }
}

#[derive(Clone)]
struct ResolvedUnit {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Clone, Copy)]
pub enum CoreVariant {
    /// Frozen v3: each generic linear arithmetic operation is evaluated under
    /// x87 PC64 and stored to binary64 before the next operation.
    FrozenV3,
    /// Retired v2 control: both core operations are binary64.
    RetiredV2,
    /// Control that stages only the empirically exposed length first product.
    LengthFirstProductPc64,
    /// Control for global first-product staging without a staged quotient.
    FirstProductPc64AllLinear,
    /// Control for length-only first-product and quotient staging.
    LengthSecondOperationPc64,
}

fn prefix_exponent(prefix: &str) -> i32 {
    match prefix {
        "Y" => 24,
        "Z" => 21,
        "E" => 18,
        "P" => 15,
        "T" => 12,
        "G" => 9,
        "M" => 6,
        "k" => 3,
        "h" => 2,
        "da" => 1,
        "d" => -1,
        "c" => -2,
        "m" => -3,
        "u" => -6,
        "n" => -9,
        "p" => -12,
        "f" => -15,
        other => panic!("unknown prefix {other}"),
    }
}

fn prefix_base(category: &str) -> Option<&'static str> {
    match category {
        "length" => Some("m"),
        "mass" => Some("g"),
        "time" => Some("sec"),
        "pressure" => Some("Pa"),
        "volume" => Some("l"),
        _ => None,
    }
}

fn resolve(unit: &str, category: &str) -> ResolvedUnit {
    if let Some(direct) = common::direct_unit(unit) {
        assert_eq!(
            direct.category.name(),
            category,
            "category drift for {unit}"
        );
        return ResolvedUnit {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = prefix_base(category).unwrap_or_else(|| panic!("no prefix base for {category}"));
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}{base}") {
            return ResolvedUnit {
                direct: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve {category} unit {unit}");
}

fn angstroms(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no angstrom factor for {other}"),
    }
}

fn physical_factor(unit: &str) -> f64 {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap_or_else(|| panic!("no physical factor for {unit}"))
        .factor_decimal
        .parse()
        .unwrap()
}

fn pressure_factor(unit: &str) -> f64 {
    match unit {
        "Pa" => 1.0,
        "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
        "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
        other => panic!("no supported pressure factor for {other}"),
    }
}

fn factor_to_base(category: &str, unit: &str) -> f64 {
    match category {
        "length" => angstroms(unit),
        "mass" | "time" | "volume" => physical_factor(unit),
        "pressure" => pressure_factor(unit),
        other => panic!("no linear factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn pc64_product_store(number: f64, factor: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(factor), cw),
        cw,
    )
}

fn pc64_quotient_store(number: f64, divisor: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(number), &rx::ext_from_f64(divisor), cw),
        cw,
    )
}

fn linear_prediction(
    number: f64,
    category: &str,
    from: &ResolvedUnit,
    to: &ResolvedUnit,
    variant: CoreVariant,
) -> f64 {
    let from_factor = factor_to_base(category, &from.direct);
    let to_factor = factor_to_base(category, &to.direct);
    let stage_first = match variant {
        CoreVariant::FrozenV3 | CoreVariant::FirstProductPc64AllLinear => true,
        CoreVariant::LengthFirstProductPc64 | CoreVariant::LengthSecondOperationPc64 => {
            category == "length"
        }
        CoreVariant::RetiredV2 => false,
    };
    let product = if stage_first {
        pc64_product_store(number, from_factor)
    } else {
        number * from_factor
    };
    let stage_second = matches!(variant, CoreVariant::FrozenV3)
        || (matches!(variant, CoreVariant::LengthSecondOperationPc64) && category == "length");
    let core = if stage_second {
        pc64_quotient_store(product, to_factor)
    } else {
        product / to_factor
    };
    let delta = pow10(from.prefix_exponent - to.prefix_exponent);
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(delta), cw),
        cw,
    )
}

fn temperature_prediction(number: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        (a, b) if a == b => number,
        ("K", "C") => number - 273.15,
        ("C", "K") => number + 273.15,
        ("K", "F") => (number - 273.15) * 1.8 + 32.0,
        ("F", "K") => (number - 32.0) / 1.8 + 273.15,
        ("C", "F") => number * 1.8 + 32.0,
        ("F", "C") => (number - 32.0) / 1.8,
        _ => panic!("unknown temperature pair {from}->{to}"),
    }
}

pub fn predict(
    number: f64,
    category: &str,
    from_unit: &str,
    to_unit: &str,
    variant: CoreVariant,
) -> Prediction {
    if category == "temperature" {
        return Prediction::Numeric(temperature_prediction(number, from_unit, to_unit).to_bits());
    }
    let from = resolve(from_unit, category);
    let to = resolve(to_unit, category);
    if category == "pressure" && (from.direct == "bar" || to.direct == "bar") {
        return Prediction::ErrorNa;
    }
    Prediction::Numeric(linear_prediction(number, category, &from, &to, variant).to_bits())
}

pub fn predict_frozen(number: f64, category: &str, from: &str, to: &str) -> Prediction {
    predict(number, category, from, to, CoreVariant::FrozenV3)
}

pub fn model_manifest() -> Value {
    json!({
        "freeze_id": FREEZE_ID,
        "selection_status": "fixed after discovery plus explicitly retired v1/v2 and refinement-only v3 discriminator; before disjoint v3 publication generation/capture",
        "linear_graph": [
            "from_factor = f64(category_table[from_direct])",
            "to_factor = f64(category_table[to_direct])",
            "product = f64(x87_pc64(number * from_factor))",
            "core = f64(x87_pc64(product / to_factor))",
            "delta = f64(decimal 10^(from_prefix_exponent-to_prefix_exponent))",
            "result = f64(x87_pc64(core * delta))"
        ],
        "tables": {
            "length_integer_angstroms_per_unit": {
                "m":"10000000000", "in":"254000000", "ft":"3048000000",
                "yd":"9144000000", "mi":"16093440000000", "Nmi":"18520000000000"
            },
            "mass_grams_per_unit": {"g":"1", "lbm":"453.59237", "ozm":"28.349523125"},
            "time_seconds_per_unit": {"sec":"1", "mn":"60", "hr":"3600", "day":"86400"},
            "pressure_factor_construction": {
                "Pa":"1", "atm":"f64(1 / f64(9.8692326671601280E-06))",
                "psi":"f64(1 / f64(1.4503773773020920E-04))", "bar":"unsupported=>error:NA"
            },
            "volume_liters_per_unit": {
                "l":"1", "tsp":"0.00492892159375", "tbs":"0.01478676478125",
                "oz":"0.0295735295625", "cup":"0.2365882365", "pt":"0.473176473",
                "qt":"0.946352946", "gal":"3.785411784"
            }
        },
        "temperature_graph": "direct pair binary64 affine formulas with 273.15, 1.8, and 32; identity passthrough"
    })
}
