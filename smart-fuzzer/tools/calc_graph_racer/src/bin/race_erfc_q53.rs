//! Named F on the 53 CR-direct ulp>=2 mid leftovers. Must also keep the 96 exacts.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
    let mut q = ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW);
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
    let mut hard = Vec::new();
    let mut exacts = Vec::new();
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody(r.z, &C, &D, 0));
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
        if d == 0 {
            exacts.push(*r);
        } else if d >= 2 {
            hard.push((*r, d));
        }
    }
    println!("CR direct exacts {} hard ulp>=2 {}", exacts.len(), hard.len());

    let mut hamc = C;
    hamc[4] = poke(C[4], -1);
    let mut hamd = D;
    hamd[4] = poke(D[4], -1);
    let mut jc = hamc;
    jc[0] = poke(C[0], 4);
    jc[1] = poke(C[1], 1);
    let mut jd = hamd;
    jd[0] = poke(D[0], -1);

    type Ev = Box<dyn Fn(f64) -> f64>;
    let graphs: [(&str, Ev); 9] = [
        ("CR 0x210", Box::new(|z| qw(z, cody(z, &C, &D, 0x210)))),
        ("CR 0x74", Box::new(|z| qw(z, cody(z, &C, &D, 0x74)))),
        ("CR 0x200", Box::new(|z| qw(z, cody(z, &C, &D, 0x200)))),
        ("ham2", Box::new(move |z| qw(z, cody(z, &hamc, &hamd, 0)))),
        ("j3432", Box::new(move |z| qw(z, cody(z, &jc, &jd, 0)))),
        ("nswc_derfc0", Box::new(|z| qw(z, f::nswc_derfc0(z)))),
        ("cephes_f", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("libm::erfc", Box::new(libm::erfc)),
        ("CR mask0", Box::new(|z| qw(z, cody(z, &C, &D, 0)))),
    ];
    for (name, ev) in &graphs {
        let mut hit_hard = 0usize;
        let mut keep_ex = 0usize;
        let mut hard_max = 0u64;
        let mut hard_sum = 0u128;
        for r in &exacts {
            let qg = ev(r.z);
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                keep_ex += 1;
            }
        }
        for (r, _) in &hard {
            let qg = ev(r.z);
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                hit_hard += 1;
            }
            hard_max = hard_max.max(d);
            if d < ULP_CAP {
                hard_sum += d as u128;
            }
        }
        println!(
            "{name:14} keep_ex={keep_ex}/{} hit_hard={hit_hard}/{} hard_max={hard_max} hard_sum={hard_sum}",
            exacts.len(),
            hard.len()
        );
    }
}
