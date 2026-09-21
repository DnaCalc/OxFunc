//! Actual-bits XBIG leftover: z²/w/0.5Q-row last-store. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research::{excel_exp, x87_mul};

const INVSQRT2: f64 = 0.7071067811865476;
const XBITS: u64 = 0xc03d99406238a476;
const TBITS: u64 = 0x18b8b3d4c3e0a000; // placeholder, filled from run if wrong

fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let x = f64::from_bits(XBITS);
    let dir = std::env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let text = std::fs::read_to_string(&path).unwrap();
    // find expected bits for this x
    let mut t = f64::from_bits(TBITS);
    let mut found = false;
    for cap in text.split("expected_bits") {
        if cap.contains("c03d99406238a476") || cap.contains(&format!("{XBITS:x}")) {
            // fall through to serde
        }
    }
    let bank: calc_graph_racer::score::WitnessSet =
        serde_json::from_str(&text).unwrap();
    use calc_graph_racer::eval::parse_bits_hex;
    use calc_graph_racer::score::WitnessArg;
    for w in &bank.witnesses {
        let xx = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        if xx.to_bits() == XBITS {
            t = parse_bits_hex(&w.expected_bits).unwrap();
            found = true;
            break;
        }
    }
    println!("found={found} x={x:.16} t={t:e} tbits={:016x}", t.to_bits());
    let a = (x * INVSQRT2).abs();
    let ff = f::cephes_f(a);
    let w0 = f::w_rn53(a);
    let phi0 = 0.5 * qmul(w0, ff);
    println!(
        "a={a:.16} fused={}{}",
        ulp_distance(phi0, t).unwrap_or(99),
        if phi0 < t { "L" } else { "H" }
    );
    let tt0 = x87_mul(a, a);
    print!("  t±:");
    for k in -24i32..=24 {
        if k == 0 {
            continue;
        }
        let p = 0.5 * qmul(excel_exp(-poke(tt0, k)), ff);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" t{k:+}");
        }
    }
    println!();
    print!("  w±:");
    for k in -64i32..=64 {
        if k == 0 {
            continue;
        }
        if ulp_distance(0.5 * qmul(poke(w0, k), ff), t).unwrap_or(99) == 0 {
            print!(" w{k:+}");
        }
    }
    println!();
    let rows = f::load_q_rows_tagged(&dir);
    for r in rows.iter().filter(|rr| (rr.z - a).abs() < 1e-15) {
        let q = f64::from_bits(r.qbits);
        let half = 0.5 * q;
        println!(
            "  Q z={:.16} direct={} 0.5Q={} 0.5Q_ulp={}",
            r.z,
            r.direct,
            half,
            ulp_distance(half, t).unwrap_or(99)
        );
    }
    println!(
        "  0.5*libm_erfc(a)={}",
        ulp_distance(0.5 * libm::erfc(a), t).unwrap_or(99)
    );
}
