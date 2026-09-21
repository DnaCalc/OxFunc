//! Q-metric 0x74 submasks and j3432 vs derfc0 0.5-bands. Not a cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn qw(z: f64, ff: f64) -> f64 {
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
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let mut hamc = C0;
    hamc[4] = poke(C0[4], -1);
    let mut hamd = D0;
    hamd[4] = poke(D0[4], -1);
    let qj = |z: f64| qw(z, cody(z, &jc, &jd, 0));
    let qh = |z: f64| qw(z, cody(z, &hamc, &hamd, 0));
    let q74 = |z: f64| qw(z, cody(z, &C0, &D0, 0x74));
    let qd = |z: f64| qw(z, f::nswc_derfc0(z));
    let qc = |z: f64| qw(z, f::nswc_ccdd_f(z));

    println!("Q-metric 0x74 submasks (bars CR 3386 / 0x74 3398 / j3432 3432):");
    const SB: [u32; 4] = [2, 4, 5, 6];
    let mut best = 0usize;
    let mut best_m = 0u32;
    for s in 0u32..16 {
        let mut mask = 0u32;
        let mut pop = 0u32;
        for i in 0..4 {
            if s & (1 << i) != 0 {
                mask |= 1 << SB[i];
                pop += 1;
            }
        }
        let mut ex = 0usize;
        let mut dd = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = qw(r.z, cody(r.z, &C0, &D0, mask));
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
                if r.direct {
                    dd += 1;
                }
            } else {
                mx = mx.max(d);
            }
        }
        if ex >= best {
            best = ex;
            best_m = mask;
        }
        if ex >= 3380 || pop <= 1 || mask == 0x74 {
            println!("  mask={mask:#06x} pop={pop} Q-mid {ex}/{n} d={dd} max={mx}");
        }
    }
    println!("  best {best_m:#06x} {best}");

    println!("Q-metric 0.5-bands (j3432 / ham2 / 0x74 / derfc0 / ccdd):");
    let mut bj = [0usize; 7];
    let mut bh = [0usize; 7];
    let mut b74 = [0usize; 7];
    let mut bd = [0usize; 7];
    let mut bc = [0usize; 7];
    let mut bn = [0usize; 7];
    let mut tot_j = 0usize;
    let mut tot_d = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(6);
        bn[b] += 1;
        let t = f64::from_bits(r.qbits);
        if ulp_distance(qj(r.z), t).unwrap_or(99) == 0 {
            bj[b] += 1;
            tot_j += 1;
        }
        if ulp_distance(qh(r.z), t).unwrap_or(99) == 0 {
            bh[b] += 1;
        }
        if ulp_distance(q74(r.z), t).unwrap_or(99) == 0 {
            b74[b] += 1;
        }
        if ulp_distance(qd(r.z), t).unwrap_or(99) == 0 {
            bd[b] += 1;
            tot_d += 1;
        }
        if ulp_distance(qc(r.z), t).unwrap_or(99) == 0 {
            bc[b] += 1;
        }
    }
    for i in 0..7 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "  [{:.1},{:.1}) n={} j={} ham={} 74={} derfc0={} ccdd={}",
            lo,
            lo + 0.5,
            bn[i],
            bj[i],
            bh[i],
            b74[i],
            bd[i],
            bc[i]
        );
    }
    println!("  totals j={tot_j} derfc0={tot_d}");

    println!("Q-metric j3432 vs derfc0 cuts:");
    for &c in &[1.0, 2.0, 2.5, 3.0, 3.5] {
        let mut jd = 0usize;
        let mut dj = 0usize;
        let mut n = 0usize;
        let mut dmid = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let t = f64::from_bits(r.qbits);
            let g = if r.z < c { qj(r.z) } else { qd(r.z) };
            let h = if r.z < c { qd(r.z) } else { qj(r.z) };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                jd += 1;
                if r.direct {
                    dmid += 1;
                }
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                dj += 1;
            }
        }
        println!("  cut={c:.1} j-then-derfc0={jd}/{n} d={dmid}  derfc0-then-j={dj}/{n} (bar 3432)");
    }

    println!("Q-metric 0x74 vs derfc0 cuts:");
    for &c in &[2.0, 2.5, 3.0] {
        let mut a = 0usize;
        let mut b = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let t = f64::from_bits(r.qbits);
            let g = if r.z < c { q74(r.z) } else { qd(r.z) };
            let h = if r.z < c { qd(r.z) } else { q74(r.z) };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                a += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                b += 1;
            }
        }
        println!("  cut={c:.1} 74-then-derfc0={a}/{n} derfc0-then-74={b}/{n} (bar 3398)");
    }
}
