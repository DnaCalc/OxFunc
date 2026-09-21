//! Q z<0.5 as RN53(1−P) of x87 2/sqrt(pi) leading A21. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const PI: f64 = 3.1415926535897932384626433832795;
const TWO_RSQPI: f64 = 1.1283791670955125738961589031215;

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
fn horner_tail(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    acc
}
fn erf_lead(z: f64, lead: Ext80, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let tail = ext_mul(&u, &horner_tail(z, a), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&lead, &tail, CW), CW), CW)
}
fn erf_one_plus(z: f64, a: &[f64; 21]) -> f64 {
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
    let mut aj = AS0;
    aj[0] = poke(AS0[0], 4);
    aj[1] = poke(AS0[1], -2);
    aj[2] = poke(AS0[2], -5);
    aj[3] = poke(AS0[3], 1);
    let x87_lead = ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW);
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 4] = [
        (
            "1-A21 CR",
            Box::new(|z| q_from_p(erf_one_plus(z, &AS0))),
        ),
        (
            "1-A21 joint866",
            Box::new({
                let a = aj;
                move |z| q_from_p(erf_one_plus(z, &a))
            }),
        ),
        (
            "1-x87 2/sqrt(pi)+A[1:]",
            Box::new(move |z| q_from_p(erf_lead(z, x87_lead, &AS0))),
        ),
        (
            "1-TWO_RSQPI+A[1:]",
            Box::new(|z| q_from_p(erf_lead(z, ef(TWO_RSQPI), &AS0))),
        ),
    ];
    println!(
        "Q z<0.5 rows={}",
        rows.iter().filter(|r| r.z < 0.5).count()
    );
    for (name, ev) in named {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dhit = 0usize;
        let mut dn = 0usize;
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
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dhit += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:28} {ex}/{n} max={maxu} d={dhit}/{dn}");
    }
}
