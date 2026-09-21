//! A0+4 inside Horner vs named next_up(1+A0) / x87 1+(A0+4) lead. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PI: f64 = 3.1415926535897932384626433832795;
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
fn horner_full(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn horner_tail(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_mul(&u, &acc, CW)
}
fn erf_named_lead(z: f64, lead: Ext80, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    ext_to_f64(&ext_mul(&xe, &ext_add(&lead, &horner_tail(z, a), CW), CW), CW)
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
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    let a0p = poke(AS0[0], 4);
    let mut ap = AS0;
    ap[0] = a0p;
    let one_plus_pub = 1.0 + AS0[0];
    let one_plus_poke = 1.0 + a0p;
    let x87_1a0 = ext_add(&ef(1.0), &ef(a0p), CW);
    let x87_lead = ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW);
    println!(
        "1+A0_pub={:016x} 1+A0+4={:016x} next_up(1+A0)={:016x} x87_1+(A0+4) store={:016x}",
        one_plus_pub.to_bits(),
        one_plus_poke.to_bits(),
        one_plus_pub.next_up().to_bits(),
        ext_to_f64(&x87_1a0, CW).to_bits()
    );
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 6] = [
        ("1+Horner A0+4", Box::new(move |z| horner_full(z, &ap))),
        (
            "named (1+A0+4)+tail",
            Box::new(move |z| erf_named_lead(z, ef(one_plus_poke), &AS0)),
        ),
        (
            "named next_up(1+A0)+tail",
            Box::new(move |z| erf_named_lead(z, ef(one_plus_pub.next_up()), &AS0)),
        ),
        (
            "x87 1+(A0+4)+tail",
            Box::new(move |z| erf_named_lead(z, x87_1a0, &AS0)),
        ),
        (
            "x87 2/sqrt(pi)+tail",
            Box::new(move |z| erf_named_lead(z, x87_lead, &AS0)),
        ),
        ("1+Horner CR", Box::new(|z| horner_full(z, &AS0))),
    ];
    println!("P-side n={}", rows.len());
    for (name, ev) in named {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let pg = ev(z);
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
        println!("{name:28} {ex}/{n} max={maxu}");
    }
}
