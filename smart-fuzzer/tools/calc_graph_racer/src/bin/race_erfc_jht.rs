//! j3432-hard 50 vs SPECFUN P/Q on mid and 1−erf as Q. CR already 0/50. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
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
const P: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const AS0: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW),
        CW,
    )
}
fn pq_x87(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(P[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(P[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(Q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(P[4]), CW), CW),
        &ext_add(&xden, &ef(Q[4]), CW),
        CW,
    );
    let s = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    ext_to_f64(&ext_div(&ext_sub(&s, &r, CW), &ye, CW), CW)
}
fn horner_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn aabb(x: f64) -> f64 {
    let xe = ef(x.abs());
    let z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    let t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    let n = horner_f(&AA, z);
    let d = horner_f(&BB, z);
    let mut acc = ext_div(&n, &d, CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E2), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E1), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E0), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
}
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
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
    let mut aj = AS0;
    aj[0] = poke(AS0[0], 4);
    aj[1] = poke(AS0[1], -2);
    aj[2] = poke(AS0[2], -5);
    let ev_j = |z: f64| qw(z, cody(z, &jc, &jd));
    let ev_pq = |z: f64| qw(z, pq_x87(z));
    let ev_1e = |z: f64| ext_to_f64(&ext_sub(&ef(1.0), &ef(erf_a(z, &aj)), CW), CW);
    let ev_1c = |z: f64| ext_to_f64(&ext_sub(&ef(1.0), &ef(erf_a(z, &AS0)), CW), CW);
    let mut hard = Vec::new();
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let d = ulp_distance(ev_j(r.z), want).unwrap_or(99);
        if d >= 2 && d <= ULP_CAP {
            hard.push((r.z, r.qbits, d));
        }
    }
    println!("j3432 hard direct n={}", hard.len());
    let ev_ab = |z: f64| qw(z, aabb(z));
    let ev_cc = |z: f64| qw(z, f::nswc_ccdd_f(z));
    let ev_j2 = |z: f64| {
        if z < 2.0 {
            ev_j(z)
        } else {
            ev_ab(z)
        }
    };
    let ev_jc = |z: f64| {
        if z < 2.0 {
            ev_j(z)
        } else {
            ev_cc(z)
        }
    };
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("j3432", Box::new(ev_j)),
        ("P/Q x87 SQRPI as F", Box::new(ev_pq)),
        ("1-erf A21 joint as Q", Box::new(ev_1e)),
        ("1-erf A21 CR as Q", Box::new(ev_1c)),
        ("AABB f64-wide as F", Box::new(ev_ab)),
        ("j3432 then AABB cut=2", Box::new(ev_j2)),
        ("CCDD as F", Box::new(ev_cc)),
        ("j3432 then CCDD cut=2", Box::new(ev_jc)),
    ];
    for (name, ev) in &graphs {
        let mut keep = 0usize;
        let mut hit = 0usize;
        let mut hmax = 0u64;
        let mut mex = 0usize;
        let mut mn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let d = ulp_distance(ev(r.z), want).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            mn += 1;
            if d == 0 {
                mex += 1;
            }
            if ulp_distance(ev_j(r.z), want).unwrap_or(99) == 0 && d == 0 {
                keep += 1;
            }
        }
        for &(z, qbits, _) in &hard {
            let d = ulp_distance(ev(z), f64::from_bits(qbits)).unwrap_or(u64::MAX);
            if d == 0 {
                hit += 1;
            } else if d <= ULP_CAP {
                hmax = hmax.max(d);
            }
        }
        let mut hit2 = 0usize;
        let mut n2 = 0usize;
        for &(z, qbits, _) in &hard {
            if z < 2.0 {
                continue;
            }
            n2 += 1;
            if ulp_distance(ev(z), f64::from_bits(qbits)).unwrap_or(99) == 0 {
                hit2 += 1;
            }
        }
        println!(
            "{name:24} mid {mex}/{mn} keep_j={keep}/3432 hit_hard={hit}/{} hardmax={hmax} hard_z>=2 {hit2}/{n2}",
            hard.len()
        );
    }
    println!("AABB/CCDD hits on all j3432-hard:");
    for &(z, qbits, dj) in &hard {
        let want = f64::from_bits(qbits);
        let da = ulp_distance(ev_ab(z), want).unwrap_or(99);
        let dc = ulp_distance(ev_cc(z), want).unwrap_or(99);
        if da == 0 || dc == 0 {
            println!("  z={z:.16} j={dj} aabb={da} ccdd={dc}");
        }
    }
    println!("implied ULP>=6 leftovers j3432/CR:");
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dj = ulp_distance(ev_j(r.z), want).unwrap_or(0);
        let dc = ulp_distance(qw(r.z, cody(r.z, &C0, &D0)), want).unwrap_or(0);
        if dj >= 6 || dc >= 6 {
            println!(
                "  z={:.16} direct={} j={} cr={} q={:016x}",
                r.z, r.direct, dj, dc, r.qbits
            );
        }
    }
}
