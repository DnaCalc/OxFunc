//! Pin leftover-exclusive z of 0x210 vs 114. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
const SIX: [(&'static str, f64); 6] = [
    ("210-lo->114-1U", 0.8645833333333333),
    ("210-lo->114-1U", 0.8958333333333333),
    ("114-lo<-210-1U", 2.4687500000000000),
    ("114-lo<-210-1U", 3.8750000000000000),
    ("210-hi->114-fused", 3.3958333333333330),
    ("114-hi<-210-1U", 0.8437500000000000),
];
const C0: [f64; 9] = [
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c: &[f64; 9], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn k_of(z: f64, t: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}
fn grid(z: f64) -> &'static str {
    if z.fract() == 0.0 {
        "int"
    } else if (z * 2.0).fract() == 0.0 {
        "dyad2"
    } else if (z * 16.0).fract() == 0.0 {
        "dyad16"
    } else if (z * 48.0 - (z * 48.0).round()).abs() < 1e-12 {
        "48"
    } else if (z * 96.0 - (z * 96.0).round()).abs() < 1e-12 {
        "96"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    println!("pin leftover-exclusive z 0x210 vs 114:");
    for &(tag, lz) in &SIX {
        let Some(r) = rows
            .iter()
            .find(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0 && (rr.z - lz).abs() < 1e-12)
        else {
            println!("  {tag} z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(lz);
        let ff = cody(lz, &C0, 0);
        let g = mul(w, ff);
        let k = k_of(lz, t, ff);
        println!(
            "  {tag} z={lz:.16} bits={:#x} grid={} unmask_d={} k={k:?} last-mul up={:?} dn={:?} w+1={:?} w-1={:?}",
            lz.to_bits(),
            grid(lz),
            ulp_distance(g, t).unwrap_or(99),
            ulp_distance(g.next_up(), t),
            ulp_distance(g.next_down(), t),
            ulp_distance(mul(w.next_up(), ff), t),
            ulp_distance(mul(w.next_down(), ff), t)
        );
        let _ = (c114, MASK);
    }
    println!("scan leftover-high 0x210 where 114 fused:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let g210 = mul(w, cody(r.z, &C0, MASK));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let d114 = ulp_distance(mul(w, cody(r.z, &c114, MASK)), t).unwrap_or(99);
        if d210 >= 2 && g210 > t && d114 == 0 {
            let ff = cody(r.z, &C0, 0);
            let g = mul(w, ff);
            println!(
                "  CONV-HI z={:.16} bits={:#x} unmask_d={} k={:?} last-mul dn={:?}",
                r.z,
                r.z.to_bits(),
                ulp_distance(g, t).unwrap_or(99),
                k_of(r.z, t, ff),
                ulp_distance(g.next_down(), t)
            );
        }
    }
}
