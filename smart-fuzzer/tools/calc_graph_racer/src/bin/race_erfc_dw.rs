//! Decimal-width of fitted C[0]/R[0]/A[0] vs IEEE last bits. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: f64 = 0.564188496988670089;
const R0: f64 = 5.64189583547755073984e-1;
const A0: f64 = 0.1283791670955125738961589031215;
fn a0_joint() -> f64 {
    let mut v: f64 = 0.1283791670955125738961589031215;
    v = v.next_up();
    v = v.next_up();
    v = v.next_up();
    v.next_up()
}
const C0S: &str = "0.564188496988670089";
const R0S: &str = "0.564189583547755073984";
const A0S: &str = "0.1283791670955125738961589031215";
const C0TAB: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c0: f64) -> f64 {
    let ye = ef(y.abs());
    let mut c = C0TAB;
    c[0] = c0;
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn prefixes(s: &str) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    let digits: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let nd: usize = digits.chars().filter(|c| c.is_ascii_digit()).count();
    for n in 6..=nd {
        let mut seen = 0usize;
        let mut p = String::new();
        for ch in digits.chars() {
            if ch.is_ascii_digit() {
                if seen == n {
                    break;
                }
                p.push(ch);
                seen += 1;
            } else {
                p.push(ch);
            }
        }
        if let Ok(v) = p.parse::<f64>() {
            out.push((n, v));
        }
    }
    out
}

fn main() {
    println!("C[0] IEEE={:#x}", C0.to_bits());
    for (n, v) in prefixes(C0S) {
        let d = if v.to_bits() == C0.to_bits() {
            0
        } else {
            ulp_distance(v, C0).unwrap_or(99)
        };
        if d <= 4 || n >= 15 {
            println!("  C0 {n} digits {v:.20} bits={:#x} ulp_vs_ieee={d}", v.to_bits());
        }
    }
    println!("R[0] IEEE={:#x}", R0.to_bits());
    for (n, v) in prefixes(R0S) {
        let d = if v.to_bits() == R0.to_bits() {
            0
        } else {
            ulp_distance(v, R0).unwrap_or(99)
        };
        if d <= 4 || n >= 15 {
            println!("  R0 {n} digits {v:.20} bits={:#x} ulp_vs_ieee={d}", v.to_bits());
        }
    }
    let a0j = a0_joint();
    println!("A[0] pub={:#x} joint={:#x}", A0.to_bits(), a0j.to_bits());
    for (n, v) in prefixes(A0S) {
        let dp = ulp_distance(v, A0).unwrap_or(99);
        let dj = ulp_distance(v, a0j).unwrap_or(99);
        if dp <= 4 || dj <= 4 || n >= 16 {
            println!("  A0 {n} digits ulp_pub={dp} ulp_joint={dj} bits={:#x}", v.to_bits());
        }
    }
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("0x210 dmid with C[0] from k-digit parse (bar 105):");
    for (n, v) in prefixes(C0S) {
        if n < 9 {
            continue;
        }
        let mut dmid = 0usize;
        let mut nd = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            nd += 1;
            if ulp_distance(qwf(r.z, cody(r.z, v)), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        if dmid >= 90 || n >= 15 {
            println!("  {n} digits dmid={dmid}/{nd}");
        }
    }
}
