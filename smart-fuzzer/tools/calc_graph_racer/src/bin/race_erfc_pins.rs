//! F-body pin residuals for the human-era leaders. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const AS: [f64; 21] = [
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

fn x87_horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn small_nswc(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner_hi(&AS, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &ww, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    ) / w
}
fn cody_cd(y: f64, c0_off: i32, c6_off: i32, d4_off: i32, d7_off: i32) -> f64 {
    let mut cc = C;
    let mut dd = D;
    for _ in 0..c0_off.abs() {
        cc[0] = if c0_off > 0 { cc[0].next_up() } else { cc[0].next_down() };
    }
    for _ in 0..c6_off.abs() {
        cc[6] = if c6_off > 0 { cc[6].next_up() } else { cc[6].next_down() };
    }
    for _ in 0..d4_off.abs() {
        dd[4] = if d4_off > 0 { dd[4].next_up() } else { dd[4].next_down() };
    }
    for _ in 0..d7_off.abs() {
        dd[7] = if d7_off > 0 { dd[7].next_up() } else { dd[7].next_down() };
    }
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(cc[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(cc[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(dd[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(cc[7]), CW),
            &ext_add(&xden, &ext_from_f64(dd[7]), CW),
            CW,
        ),
        CW,
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("z pin F_or nswc_derfc0 nswc_small cody_x87 cut1 cut1.347 j3074 cephes as714");
    for &pz in &f::PIN_Z {
        let Some(r) = rows.iter().find(|r| r.z == pz) else {
            println!("{pz} missing");
            continue;
        };
        let fo = f::f_or(pz, r.qbits).unwrap();
        let graphs: [(&str, f64); 8] = [
            ("nswc0", f::nswc_derfc0(pz)),
            ("small", small_nswc(pz)),
            ("cody", cody_cd(pz, 0, 0, 0, 0)),
            ("cut1", if pz < 1.0 { small_nswc(pz) } else { cody_cd(pz, 0, 0, 0, 0) }),
            ("c1347", if pz < 1.347 { small_nswc(pz) } else { cody_cd(pz, 0, 1, 1, 0) }),
            ("j3074", if pz < 1.347 { small_nswc(pz) } else { cody_cd(pz, 12, 1, 1, 1) }),
            ("cephes", f::cephes_f(pz)),
            ("as714", f::cf_as714_x87_n(pz, 80)),
        ];
        print!("{pz:.5}");
        for (name, fg) in graphs {
            let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
            print!("  {name}={d}");
        }
        println!();
    }
}
