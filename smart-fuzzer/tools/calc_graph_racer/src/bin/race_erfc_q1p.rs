//! ERFC Q for z<0.5 as RN53(1−P) from Schonfelder / A21 / A21-joint.
//! Tests complement consistency. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

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
fn erf_sch(z: f64) -> f64 {
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
fn erf_a21(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn q_from_p(p: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(p), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let nlo = rows.iter().filter(|r| r.z < 0.5).count();
    let nd = rows.iter().filter(|r| r.z < 0.5 && r.direct).count();
    println!("Q rows z<0.5: {nlo} direct={nd}");
    fn erf_joint(z: f64) -> f64 {
        let mut a = AS;
        a[0] = poke(AS[0], 4);
        a[1] = poke(AS[1], -2);
        a[2] = poke(AS[2], -5);
        a[3] = poke(AS[3], 1);
        erf_a21(z, &a)
    }
    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 4] = [
        ("1-Schonfelder x87", |z| q_from_p(erf_sch(z))),
        ("1-A21 CR", |z| q_from_p(erf_a21(z, &AS))),
        ("1-A21 joint866", |z| q_from_p(erf_joint(z))),
        ("1-libm erf", |z| q_from_p(libm::erf(z))),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut max_d = 0u64;
        let mut max_a = 0u64;
        for r in &rows {
            if r.z >= 0.5 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            max_a = max_a.max(d);
            if d == 0 {
                ex += 1;
            }
            if r.direct {
                dn += 1;
                max_d = max_d.max(d);
                if d == 0 {
                    dmid += 1;
                }
            }
        }
        println!("{name:22} {ex}/{n} maxa={max_a} d={dmid}/{dn} maxd={max_d}");
    }
}
