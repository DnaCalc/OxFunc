//! 0.5 / 0.46875 k-ULP neighborhood Excel bits. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn q1p(p: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(p), CW), CW)
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
    let mut map: BTreeMap<u64, (bool, u64)> = BTreeMap::new();
    for r in &rows {
        map.insert(r.z.to_bits(), (r.direct, r.qbits));
    }
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    println!("0.5 + k ulp:");
    let mut z = 0.5f64;
    for _ in 0..3 {
        z = z.next_down();
    }
    for k in -3i32..=6 {
        match map.get(&z.to_bits()) {
            Some(&(dir, excel)) => {
                let t = f64::from_bits(excel);
                let d210 = ulp_distance(qwf(z, cody_cd(z, 0x210)), t).unwrap_or(99);
                let dj = ulp_distance(q1p(erf_a(z, &joint)), t).unwrap_or(99);
                let dl = ulp_distance(libm::erfc(z), t).unwrap_or(99);
                println!(
                    "  k={k:+} z={z:.20} dir={dir} excel={excel:#x} 210={d210} 1-joint={dj} libm={dl}"
                );
            }
            None => println!("  k={k:+} z={z:.20} missing"),
        }
        z = z.next_up();
    }
    println!("0.46875 + k ulp (Cody A/B cut):");
    let mut z = 0.46875f64;
    for _ in 0..2 {
        z = z.next_down();
    }
    for k in -2i32..=3 {
        match map.get(&z.to_bits()) {
            Some(&(dir, excel)) => {
                let t = f64::from_bits(excel);
                let d210 = ulp_distance(qwf(z, cody_cd(z, 0x210)), t).unwrap_or(99);
                let dj = ulp_distance(q1p(erf_a(z, &joint)), t).unwrap_or(99);
                let dl = ulp_distance(libm::erfc(z), t).unwrap_or(99);
                println!(
                    "  k={k:+} z={z:.20} dir={dir} excel={excel:#x} 210={d210} 1-joint={dj} libm={dl}"
                );
            }
            None => println!("  k={k:+} z={z:.20} missing"),
        }
        z = z.next_up();
    }
}
