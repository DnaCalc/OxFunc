//! 0x210 leftover-low: w*up(F) analog of 1+w-up. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}
fn qwf(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn kind(z: f64) -> &'static str {
    let m = z.to_bits() & ((1u64 << 52) - 1);
    if m.trailing_zeros() >= 20 {
        "dyad"
    } else if (z.to_bits() & 0xfff) == 0x555 {
        "555"
    } else if (z.to_bits() & 0xfff) == 0xaaa {
        "aaa"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("0x210 DIRECT leftover-low (ulp>=2 graph<excel):");
    let mut n = 0usize;
    let mut hit_uf = 0usize;
    let mut hit_uw = 0usize;
    let mut hit_uq = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let ff = cody(r.z);
        let w = f::w_rn53(r.z);
        let g = qwf(w, ff);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d < 2 || g >= t {
            continue;
        }
        n += 1;
        let duf = ulp_distance(qwf(w, ff.next_up()), t).unwrap_or(99);
        let duw = ulp_distance(qwf(w.next_up(), ff), t).unwrap_or(99);
        let duq = ulp_distance(g.next_up(), t).unwrap_or(99);
        if duf == 0 {
            hit_uf += 1;
        }
        if duw == 0 {
            hit_uw += 1;
        }
        if duq == 0 {
            hit_uq += 1;
        }
        if d >= 3 || duf == 0 {
            println!(
                "  z={:.16} kind={} ulp={d} w*upF={duf} upw*F={duw} upQ={duq}",
                r.z,
                kind(r.z)
            );
        }
    }
    println!("n={n} w*upF hit={hit_uf} upw*F hit={hit_uw} upQ hit={hit_uq}");
    let mut k0 = 0usize;
    let mut kuf = 0usize;
    let mut nd = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        nd += 1;
        let t = f64::from_bits(r.qbits);
        let ff = cody(r.z);
        let w = f::w_rn53(r.z);
        if ulp_distance(qwf(w, ff), t).unwrap_or(99) == 0 {
            k0 += 1;
        }
        if ulp_distance(qwf(w, ff.next_up()), t).unwrap_or(99) == 0 {
            kuf += 1;
        }
    }
    println!("DIRECT keep base={k0}/{nd} w*upF={kuf}/{nd} (bar 105)");
}
