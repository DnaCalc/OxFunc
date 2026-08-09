//! Select an oracle-blind ATANH cubic-vs-ratio discriminator inside a bit range.
//!
//! This tiny helper is used by the serialized COM bisection harness.  It never
//! sees an Excel answer: given two positive-f64 bit patterns, it returns a
//! midpoint-near input where the two frozen publication graphs disagree.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;

const CW: u16 = 0x133f;

fn ext(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn cubic(x: f64) -> f64 {
    x + (x * x * x) / 3.0
}

fn ratio_x87(x: f64) -> f64 {
    let ratio = (1.0 + x) / (1.0 - x);
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &ext(ratio), CW);
    to_f64(&rx::ext_mul(&logarithm, &ext(0.5), CW))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: atanh_boundary_point LOW_HEX HIGH_HEX"
    );
    let parse =
        |text: &str| u64::from_str_radix(text.trim_start_matches("0x"), 16).expect("hex f64 bits");
    let low = parse(&args[1]);
    let high = parse(&args[2]);
    assert!(low + 1 < high, "range must contain an interior f64");
    let midpoint = low + (high - low) / 2;

    let mut selected = None;
    for distance in 0_u64..=1_000_000 {
        for bits in [
            midpoint.saturating_sub(distance),
            midpoint.saturating_add(distance),
        ] {
            if bits <= low || bits >= high {
                continue;
            }
            let x = f64::from_bits(bits);
            let cubic_bits = cubic(x).to_bits();
            let ratio_bits = ratio_x87(x).to_bits();
            if cubic_bits != ratio_bits {
                selected = Some((bits, cubic_bits, ratio_bits));
                break;
            }
        }
        if selected.is_some() {
            break;
        }
    }

    let Some((bits, cubic_bits, ratio_bits)) = selected else {
        println!("{}", json!({"collapsed": true}));
        return;
    };
    println!(
        "{}",
        json!({
            "input_bits": format!("0x{bits:016x}"),
            "cubic_bits": format!("0x{cubic_bits:016x}"),
            "ratio_bits": format!("0x{ratio_bits:016x}")
        })
    );
}
