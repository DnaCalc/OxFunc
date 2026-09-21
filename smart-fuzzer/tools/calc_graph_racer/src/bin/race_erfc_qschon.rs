//! Schonfelder 1978 Math.Comp. Table 1 Chebyshev erf(x)/x, x∈(0,2).
//! NAG S15 / Clenshaw-style. Distinct from MATH77. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const A1: [f64; 18] = [
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn clenshaw_prime_f64(a: &[f64], t: f64) -> f64 {
    let n = a.len() - 1;
    let mut d1 = 0.0;
    let mut d2 = 0.0;
    for k in (1..=n).rev() {
        let dk = 2.0 * t * d1 - d2 + a[k];
        d2 = d1;
        d1 = dk;
    }
    t * d1 - d2 + 0.5 * a[0]
}
fn clenshaw_prime_x87(a: &[f64], te: Ext80) -> f64 {
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &te, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    ext_to_f64(
        &ext_add(
            &ext_sub(&ext_mul(&te, &d1, CW), &d2, CW),
            &ext_mul(&ef(0.5), &ef(a[0]), CW),
            CW,
        ),
        CW,
    )
}
fn t_map(x: f64) -> f64 {
    0.5 * x * x - 1.0
}
fn erf_sch_f64(x: f64) -> f64 {
    x * clenshaw_prime_f64(&A1, t_map(x))
}
fn erf_sch_x87(x: f64) -> f64 {
    let xe = ef(x.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = clenshaw_prime_x87(&A1, t);
    ext_to_f64(&ext_mul(&xe, &ef(y), CW), CW)
}
fn a21_erf(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let mut acc = ef(0.0);
    for &c in AS.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &zz, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &acc, CW), CW), CW)
}
fn cody_cd(y: f64) -> f64 {
    let ye = ef(y.abs());
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
fn q_from_p(p: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(p), CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn load_erf_p(dir: &str) -> Vec<(f64, u64)> {
    let names = [
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b7erf.json",
        "answers-b8erf.json",
        "answers-b10.json",
        "answers-b11.json",
        "answers-b9train.json",
    ];
    let mut rows = BTreeMap::new();
    for name in names {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            if z < 0.0 || z >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            rows.insert(z.to_bits(), (z, p.to_bits()));
        }
    }
    rows.into_values().collect()
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let qrows = f::load_q_rows_tagged(&dir);
    let prows = load_erf_p(&dir);

    println!("## ERF P-side z<0.5 term trunc");
    for n in [8usize, 10, 12, 14, 16, 18] {
        let mut ex = 0usize;
        let mut ntot = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &prows {
            let xe = ef(z.abs());
            let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
            let y = clenshaw_prime_x87(&A1[..n], t);
            let pg = ext_to_f64(&ext_mul(&xe, &ef(y), CW), CW);
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            ntot += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("n={n:2} {ex}/{ntot} max={maxu}");
    }
    println!("## ERF P-side z<0.5");
    for (name, ev) in [
        ("Schonfelder T1 f64", erf_sch_f64 as fn(f64) -> f64),
        ("Schonfelder T1 x87", erf_sch_x87),
        ("NSWC A21 x87", a21_erf),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &prows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:22} {ex}/{n} max={maxu}");
    }

    println!("## Q-mid: 1-erf Schonfelder z<cut else Cody C/D w*F");
    for &cut in &[0.5f64, 1.0, 2.0] {
        let mut exact = 0usize;
        let mut n = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut max_d = 0u64;
        for r in &qrows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = if r.z < cut {
                q_from_p(erf_sch_x87(r.z))
            } else {
                qw(r.z, cody_cd(r.z))
            };
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                exact += 1;
            }
            if r.direct {
                dn += 1;
                max_d = max_d.max(d);
                if d == 0 {
                    dmid += 1;
                }
            }
        }
        println!("cut={cut} exact={exact}/{n} dmid={dmid}/{dn} maxd={max_d}");
    }
    let mut exact = 0usize;
    let mut n = 0usize;
    let mut dmid = 0usize;
    let mut dn = 0usize;
    for r in &qrows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody_cd(r.z));
        if !qg.is_finite() {
            continue;
        }
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        if d == 0 {
            exact += 1;
        }
        if r.direct {
            dn += 1;
            if d == 0 {
                dmid += 1;
            }
        }
    }
    println!("Cody C/D only     exact={exact}/{n} dmid={dmid}/{dn}");

    println!("## P-side t-map / last-mul");
    let maps: [(&str, fn(f64) -> f64); 4] = [
        ("0.5*x*x-1 x87 Clenshaw x*y80", erf_sch_x87),
        ("f64 t then x87 Clenshaw", |z| {
            let t = 0.5 * z * z - 1.0;
            let y = clenshaw_prime_x87(&A1, ef(t));
            z * y
        }),
        ("f64 Clenshaw x87 last mul", |z| {
            let y = clenshaw_prime_f64(&A1, 0.5 * z * z - 1.0);
            ext_to_f64(&ext_mul(&ef(z), &ef(y), CW), CW)
        }),
        ("t=2*(x/2)^2-1 x87", |z| {
            let xe = ef(z.abs());
            let half = ext_div(&xe, &ef(2.0), CW);
            let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&half, &half, CW), CW), &ef(1.0), CW);
            let y = clenshaw_prime_x87(&A1, t);
            ext_to_f64(&ext_mul(&xe, &ef(y), CW), CW)
        }),
    ];
    for (name, ev) in maps {
        let mut ex = 0usize;
        let mut ntot = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &prows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            ntot += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:36} {ex}/{ntot} max={maxu}");
    }
}
