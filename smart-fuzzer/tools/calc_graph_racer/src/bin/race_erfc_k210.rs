//! Keep-and-hit C/D pokes on 0x210 vs only-210 pair. Not an identity.
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
const ONLY: [f64; 2] = [3.2708333333333335, 3.46875];

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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let sc = |c: &[f64; 9], d: &[f64; 8]| {
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut only = 0usize;
        let mut mx = 0u64;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let dist = ulp_distance(qwf(r.z, cody(r.z, c, d, 0x210)), f64::from_bits(r.qbits))
                .unwrap_or(99);
            if dist == 0 {
                dmid += 1;
                if ONLY.iter().any(|&z| (r.z - z).abs() < 1e-12) {
                    only += 1;
                }
            } else {
                mx = mx.max(dist);
            }
        }
        (dmid, n, mx, only)
    };
    let base = sc(&C0, &D0);
    println!("base 0x210 dmid={}/{} maxd={} only={}/2", base.0, base.1, base.2, base.3);

    println!("one-coeff ±1..4 (print only>0 or dmid>105):");
    let mut best_o = 0usize;
    let mut best_d = base.0;
    let mut lab = "base".to_string();
    for i in 0..9 {
        for k in -4i32..=4 {
            if k == 0 {
                continue;
            }
            let mut c = C0;
            c[i] = poke(C0[i], k);
            let (dmid, n, mx, only) = sc(&c, &D0);
            if only > 0 || dmid > 105 {
                println!("  C[{i}] {k:+} dmid={dmid}/{n} maxd={mx} only={only}");
            }
            if only > best_o || (only == best_o && dmid > best_d) {
                best_o = only;
                best_d = dmid;
                lab = format!("C[{i}] {k:+}");
            }
        }
    }
    for i in 0..8 {
        for k in -4i32..=4 {
            if k == 0 {
                continue;
            }
            let mut d = D0;
            d[i] = poke(D0[i], k);
            let (dmid, n, mx, only) = sc(&C0, &d);
            if only > 0 || dmid > 105 {
                println!("  D[{i}] {k:+} dmid={dmid}/{n} maxd={mx} only={only}");
            }
            if only > best_o || (only == best_o && dmid > best_d) {
                best_o = only;
                best_d = dmid;
                lab = format!("D[{i}] {k:+}");
            }
        }
    }
    println!("best-by-only {lab} dmid={best_d} only={best_o}");

    println!("ham2 ±1 (print only>0 or dmid>=108):");
    for a in 0..17 {
        for b in (a + 1)..17 {
            for sa in [-1i32, 1] {
                for sb in [-1i32, 1] {
                    let mut c = C0;
                    let mut d = D0;
                    if a < 9 {
                        c[a] = poke(C0[a], sa);
                    } else {
                        d[a - 9] = poke(D0[a - 9], sa);
                    }
                    if b < 9 {
                        c[b] = poke(C0[b], sb);
                    } else {
                        d[b - 9] = poke(D0[b - 9], sb);
                    }
                    let (dmid, n, mx, only) = sc(&c, &d);
                    if only > 0 || dmid >= 108 {
                        println!(
                            "  a={a}{sa:+} b={b}{sb:+} dmid={dmid}/{n} maxd={mx} only={only}"
                        );
                    }
                    if only > best_o || (only == best_o && dmid > best_d) {
                        best_o = only;
                        best_d = dmid;
                        lab = format!("ham2 {a}{sa:+} {b}{sb:+}");
                    }
                }
            }
        }
    }
    println!("ham2 best-by-only {lab} dmid={best_d} only={best_o}");
}
