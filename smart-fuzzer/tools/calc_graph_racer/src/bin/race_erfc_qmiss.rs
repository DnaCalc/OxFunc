//! Cody 0x74 Q direct-miss fingerprint. Conflict row. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C: [f64; 9] = [
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
const D: [f64; 8] = [
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
fn cody(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    f::w_rn53(z) * ff
}
fn signed_ulp(got: f64, or: f64) -> Option<i64> {
    let d = ulp_distance(got, or)? as i64;
    if d == 0 {
        Some(0)
    } else if got > or {
        Some(d)
    } else {
        Some(-d)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    const CONFLICT: f64 = 1.1842387490730035;

    println!("## conflict z≈1.1842387490730035 (direct bits win)");
    for r in &rows {
        if (r.z - CONFLICT).abs() > 1e-12 {
            continue;
        }
        let qo = f64::from_bits(r.qbits);
        println!(
            "  z={:.17} direct={} q={:016x}",
            r.z, r.direct, r.qbits
        );
        for (name, mask) in [("cody0", 0u32), ("cody74", 0x74), ("cody210", 0x210)] {
            let qg = qw(r.z, cody(r.z, mask));
            println!(
                "    {name} ulp={:?} qg={:016x}",
                signed_ulp(qg, qo),
                qg.to_bits()
            );
        }
        let qn = qw(r.z, f::nswc_derfc0(r.z));
        let qc = qw(r.z, f::cephes_f(r.z));
        let ql = libm::erfc(r.z);
        println!(
            "    nswc ulp={:?} cephes ulp={:?} libm ulp={:?}",
            signed_ulp(qn, qo),
            signed_ulp(qc, qo),
            signed_ulp(ql, qo)
        );
    }

    println!("\n## Cody 0x74 Q direct-mid misses (z, signed ulp)");
    let mut bucket = [0usize; 40]; // 0.5 + 0.1*k
    let mut n_dir = 0usize;
    let mut n_miss = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        n_dir += 1;
        let qg = qw(r.z, cody(r.z, 0x74));
        let qo = f64::from_bits(r.qbits);
        let Some(s) = signed_ulp(qg, qo) else {
            continue;
        };
        if s != 0 {
            n_miss += 1;
            println!("  z={:.17} ulp={:+} {}", r.z, s, if r.z >= 2.0 { "[2,4)" } else { "" });
            let b = ((r.z - 0.5) / 0.1).floor() as i32;
            if (0..40).contains(&b) {
                bucket[b as usize] += 1;
            }
        }
    }
    println!("direct-mid miss {n_miss}/{n_dir}");
    println!("0.1-buckets from 0.5:");
    for (i, &c) in bucket.iter().enumerate() {
        if c > 0 {
            println!("  [{:.1},{:.1}) {c}", 0.5 + i as f64 * 0.1, 0.6 + i as f64 * 0.1);
        }
    }
}
