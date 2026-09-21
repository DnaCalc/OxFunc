//! P-side x87 2/sqrt(pi) lead + 80-bit A[1:] decimals. Not an identity.
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
const A21_80: [Ext80; 21] = [
    Ext80([0x6c, 0x44, 0xdb, 0xa6, 0x10, 0xd4, 0x75, 0x83, 0xfc, 0x3f]),
    Ext80([0x12, 0x36, 0xcf, 0x1b, 0x58, 0xa3, 0x93, 0xc0, 0xfd, 0xbf]),
    Ext80([0xaf, 0x0d, 0x5f, 0x21, 0xd0, 0x90, 0x17, 0xe7, 0xfb, 0x3f]),
    Ext80([0xf0, 0x86, 0x5a, 0x44, 0x89, 0x71, 0x16, 0xdc, 0xf9, 0xbf]),
    Ext80([0x2c, 0xf7, 0x29, 0x35, 0x87, 0xe6, 0x2d, 0xab, 0xf7, 0x3f]),
    Ext80([0x4d, 0x2c, 0x5c, 0x20, 0x00, 0xda, 0x16, 0xe0, 0xf4, 0xbf]),
    Ext80([0xd2, 0xbb, 0x4d, 0xdc, 0x7c, 0x93, 0xd1, 0xfc, 0xf1, 0x3f]),
    Ext80([0x9b, 0x5a, 0x17, 0x06, 0x1f, 0x2e, 0x69, 0xfa, 0xee, 0xbf]),
    Ext80([0xeb, 0x01, 0x8d, 0x32, 0xee, 0x64, 0xf3, 0xdc, 0xeb, 0x3f]),
    Ext80([0x08, 0x04, 0x36, 0x1f, 0x62, 0x17, 0xba, 0xaf, 0xe8, 0xbf]),
    Ext80([0xaf, 0xdc, 0xb6, 0x11, 0x03, 0xb9, 0x62, 0xfe, 0xe4, 0x3f]),
    Ext80([0x1f, 0xbe, 0xdc, 0x1e, 0x0e, 0x8c, 0xeb, 0xa8, 0xe1, 0xbf]),
    Ext80([0xa0, 0x32, 0x67, 0x62, 0xf0, 0x6a, 0x35, 0xcf, 0xdd, 0x3f]),
    Ext80([0x41, 0xd9, 0x98, 0xf9, 0x14, 0x9e, 0x22, 0xec, 0xd9, 0xbf]),
    Ext80([0x2c, 0x96, 0x46, 0x33, 0x0a, 0xd1, 0x41, 0xfb, 0xd5, 0x3f]),
    Ext80([0xff, 0xf5, 0xa3, 0xe0, 0x81, 0x14, 0xb7, 0xfa, 0xd1, 0xbf]),
    Ext80([0x89, 0x76, 0x01, 0x4d, 0x52, 0x7c, 0x7f, 0xeb, 0xcd, 0x3f]),
    Ext80([0x97, 0x26, 0x78, 0x16, 0xc4, 0x91, 0xbe, 0xd0, 0xc9, 0xbf]),
    Ext80([0xa6, 0x39, 0xc0, 0xb1, 0x5c, 0x65, 0xb4, 0xad, 0xc5, 0x3f]),
    Ext80([0x2a, 0x49, 0x4f, 0xf3, 0xef, 0x6a, 0x1b, 0x81, 0xc1, 0xbf]),
    Ext80([0x89, 0x7f, 0xcf, 0x72, 0x6b, 0xd4, 0x0b, 0x85, 0xbc, 0x3f]),
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
fn horner_f_tail(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_mul(&u, &acc, CW)
}
fn horner_80_tail(z: f64) -> Ext80 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for c in A21_80[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), c, CW);
    }
    ext_mul(&u, &acc, CW)
}
fn erf_lead(z: f64, lead: Ext80, tail: Ext80) -> f64 {
    let xe = ef(z.abs());
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
    let mut aj = AS0;
    aj[0] = poke(AS0[0], 4);
    aj[1] = poke(AS0[1], -2);
    aj[2] = poke(AS0[2], -5);
    aj[3] = poke(AS0[3], 1);
    let x87_lead = ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW);
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("A21 CR 1+Horner", Box::new(|z| erf_one_plus(z, &AS0))),
        (
            "A21 joint 1+Horner",
            Box::new({
                let a = aj;
                move |z| erf_one_plus(z, &a)
            }),
        ),
        (
            "x87 2/sqrt(pi)+A[1:] f64",
            Box::new(move |z| erf_lead(z, x87_lead, horner_f_tail(z, &AS0))),
        ),
        (
            "x87 2/sqrt(pi)+A[1:] 80bit",
            Box::new(move |z| erf_lead(z, x87_lead, horner_80_tail(z))),
        ),
        (
            "A21 80bit 1+Horner",
            Box::new(|z| {
                let xe = ef(z.abs());
                let u = ext_mul(&xe, &xe, CW);
                let mut acc = ef(0.0);
                for c in A21_80.iter().rev() {
                    acc = ext_add(&ext_mul(&acc, &u, CW), c, CW);
                }
                ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
            }),
        ),
    ];
    println!("P-side rows={}", rows.len());
    for (name, ev) in named {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
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
        println!("{name:32} {ex}/{n} max={maxu}");
    }
}
