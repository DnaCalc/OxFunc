//! Schonfelder Table 1 on ERF.PRECISE by band, including z in [0.5, 2).
//! Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const SCH: [f64; 18] = [
    1.4831105640848035818894480790578,
    -0.3010710733865949424707310463118,
    0.0689948306898315662466031807188,
    -0.0139162712647221876825465256678,
    0.0024207995224334636628916782398,
    -0.0003658639685848086446493825778,
    4.86209844323190482828875688e-5,
    -5.7492565580356848350542158e-6,
    6.113243578434764697067588e-7,
    -5.89910153129584343908468e-8,
    5.2070090920686482404558e-9,
    -4.232975879965543268108e-10,
    3.18811350664917497488e-11,
    -2.2361550188326842738e-12,
    1.467329847991084928e-13,
    -9.0440019853817478e-15,
    5.254813715470928e-16,
    -2.88742612228498e-17,
];
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
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn clenshaw_erf(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..SCH.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(SCH[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(SCH[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn a21_erf(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_ab_erf(z: f64) -> f64 {
    let xe = ef(z.abs());
    let ysq = ext_mul(&xe, &xe, CW);
    let n = {
        let mut acc = ef(0.0);
        for &c in [AA[4], AA[0], AA[1], AA[2], AA[3]].iter() {
            acc = ext_add(&ext_mul(&acc, &ysq, CW), &ef(c), CW);
        }
        acc
    };
    let d = {
        let mut acc = ef(1.0);
        for &c in BB.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &ysq, CW), &ef(c), CW);
        }
        acc
    };
    ext_to_f64(&ext_mul(&xe, &ext_div(&n, &d, CW), CW), CW)
}
fn cody_cd_f(z: f64) -> f64 {
    let ye = ef(z.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn p_from_q(q: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(q), CW), CW)
}

fn band(z: f64) -> usize {
    if z < 0.5 {
        0
    } else if z < 2.0 {
        1
    } else {
        2
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if x < 0.0 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    println!("ERF rows z>=0: {}", rows.len());

    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 5] = [
        ("Schonfelder Clenshaw", clenshaw_erf),
        ("NSWC A21", a21_erf),
        ("Cody A/B", cody_ab_erf),
        ("1-(Cody C/D F * w wait)", |z| p_from_q(cody_cd_f(z))), // this is 1-F not 1-Q
        ("libm::erf", libm::erf),
    ];
    // 1-Q with Q=w*F
    let p_from_wf = |z: f64| -> f64 {
        let w = excel_exp(-(z * z));
        let q = w * cody_cd_f(z);
        p_from_q(q)
    };
    println!("## by band [0,0.5) [0.5,2) [2,inf)");
    for (name, ev) in graphs {
        let mut ex = [0usize; 3];
        let mut n = [0usize; 3];
        let mut mx = [0u64; 3];
        for &(z, pbits) in &rows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            let b = band(z);
            n[b] += 1;
            if d == 0 {
                ex[b] += 1;
            } else {
                mx[b] = mx[b].max(d);
            }
        }
        println!(
            "{name:28} [0,0.5) {}/{} max={}  [0.5,2) {}/{} max={}  [2,) {}/{} max={}",
            ex[0], n[0], mx[0], ex[1], n[1], mx[1], ex[2], n[2], mx[2]
        );
    }
    let mut ex = [0usize; 3];
    let mut n = [0usize; 3];
    let mut mx = [0u64; 3];
    for &(z, pbits) in &rows {
        let pg = p_from_wf(z);
        if !pg.is_finite() {
            continue;
        }
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        let b = band(z);
        n[b] += 1;
        if d == 0 {
            ex[b] += 1;
        } else {
            mx[b] = mx[b].max(d);
        }
    }
    println!(
        "{:28} [0,0.5) {}/{} max={}  [0.5,2) {}/{} max={}  [2,) {}/{} max={}",
        "1-RN53(w*C/D) x87",
        ex[0], n[0], mx[0], ex[1], n[1], mx[1], ex[2], n[2], mx[2]
    );

    println!("## z<0.5 Schonfelder ulp=3 rows");
    for &(z, pbits) in &rows {
        if z >= 0.5 {
            continue;
        }
        let pg = clenshaw_erf(z);
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(99);
        if d >= 3 {
            println!("  z={:.17} ulp={d} p={:016x}", z, pbits);
        }
    }
}
