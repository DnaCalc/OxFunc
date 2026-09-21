//! A0+4 plus each subset of A1/A2/A3 joint pokes. Which +13? Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
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
    let combos: [(&str, i32, i32, i32); 8] = [
        ("A0+4", 0, 0, 0),
        ("A0+4 A1-2", -2, 0, 0),
        ("A0+4 A2-5", 0, -5, 0),
        ("A0+4 A3+1", 0, 0, 1),
        ("A0+4 A1-2 A2-5", -2, -5, 0),
        ("A0+4 A1-2 A3+1", -2, 0, 1),
        ("A0+4 A2-5 A3+1", 0, -5, 1),
        ("A0+4 A1-2 A2-5 A3+1", -2, -5, 1),
    ];
    println!("P-side n={}", rows.len());
    for (name, p1, p2, p3) in combos {
        let mut a = AS0;
        a[0] = poke(AS0[0], 4);
        if p1 != 0 {
            a[1] = poke(AS0[1], p1);
        }
        if p2 != 0 {
            a[2] = poke(AS0[2], p2);
        }
        if p3 != 0 {
            a[3] = poke(AS0[3], p3);
        }
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let d = ulp_distance(erf_a(z, &a), f64::from_bits(pbits)).unwrap_or(u64::MAX);
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
        println!("{name:28} {ex}/{n} max={maxu} dlt={}", ex as i32 - 853);
    }
    let mut a862 = AS0;
    a862[0] = poke(AS0[0], 4);
    a862[1] = poke(AS0[1], -2);
    let mut a866 = a862;
    a866[2] = poke(AS0[2], -5);
    println!("exact under 866 not 862:");
    let mut n4 = 0usize;
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let d6 = ulp_distance(erf_a(z, &a866), want).unwrap_or(99);
        let d2 = ulp_distance(erf_a(z, &a862), want).unwrap_or(99);
        if d6 == 0 && d2 != 0 {
            n4 += 1;
            println!("  z={z:.16} 862_ulp={d2}");
        }
    }
    println!("n_866_not_862={n4}");
}
