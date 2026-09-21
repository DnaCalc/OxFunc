//! fdlibm PA/QA as Q on [0.84375,1.25) leftover-low. Not an identity.
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
const PA: [f64; 7] = [
    -2.36211856075265944077e-03,
    4.14856118683748331666e-01,
    -3.72207876035701323847e-01,
    3.18346619901161753674e-01,
    -1.10894694282396677476e-01,
    3.54783043256182359371e-02,
    -2.16637559486879084300e-03,
];
const QA: [f64; 6] = [
    1.06420880400844228286e-01,
    5.40397917702171048937e-01,
    7.18286544141962662868e-02,
    1.26171219808761642112e-01,
    1.36370839120290507362e-02,
    1.19844998467991074170e-02,
];
const ERX: f64 = 8.45062911510467529297e-01;

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
fn horner_lo(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn q_pa(z: f64) -> f64 {
    let s = z - 1.0;
    let p = horner_lo(&PA, s);
    let q = 1.0 + s * horner_lo(&QA, s);
    (1.0 - ERX) - p / q
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
    let qj = |z: f64| qw(z, cody(z, &jc, &jd, 0));
    println!("fdlibm PA as Q (erfc=(1-ERX)-P/Q):");
    let mut ex = 0usize;
    let mut nband = 0usize;
    let mut exb = 0usize;
    let mut n = 0usize;
    let mut mx = 0u64;
    let mut hit_lo = 0usize;
    let mut n_lo = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(q_pa(r.z), t).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        if d == 0 {
            ex += 1;
        } else {
            mx = mx.max(d);
        }
        if r.z >= 0.84375 && r.z < 1.25 {
            nband += 1;
            if d == 0 {
                exb += 1;
            }
        }
        if r.direct && r.z >= 0.84375 && r.z < 1.25 {
            let dj = ulp_distance(qj(r.z), t).unwrap_or(99);
            if dj >= 2 && qj(r.z) < t {
                n_lo += 1;
                if d == 0 {
                    hit_lo += 1;
                    println!("  HIT PA leftover-low z={:.16}", r.z);
                }
            }
        }
    }
    println!("  PA global mid {ex}/{n} max={mx}  [0.84375,1.25) {exb}/{nband}");
    println!("  leftover-low Q-hard in band hit={hit_lo}/{n_lo}");
    println!("j3432 vs PA cuts (bar 3432):");
    for &c in &[0.84375, 1.0, 1.25] {
        let mut jp = 0usize;
        let mut pj = 0usize;
        let mut nn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            nn += 1;
            let t = f64::from_bits(r.qbits);
            let g = if r.z < c { qj(r.z) } else { q_pa(r.z) };
            let h = if r.z < c { q_pa(r.z) } else { qj(r.z) };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                jp += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                pj += 1;
            }
        }
        println!("  cut={c:.5} j-then-PA={jp}/{nn} PA-then-j={pj}/{nn}");
    }
}
