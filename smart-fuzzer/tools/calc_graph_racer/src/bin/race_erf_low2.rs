//! Remaining 2 P-side LOW (z=0.230, 0.452): 1-Q, Cephes T/U, f32 arg. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

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
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
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
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn polevl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}
fn p1evl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = x + coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}
fn cephes_erf(z: f64) -> f64 {
    let zz = z * z;
    z * polevl(zz, &CEPHES_T) / p1evl(zz, &CEPHES_U)
}
fn cephes_erf80(z: f64) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut num = ef(CEPHES_T[0]);
    for &c in &CEPHES_T[1..] {
        num = ext_add(&ext_mul(&num, &zz, CW), &ef(c), CW);
    }
    let mut den = ext_add(&zz, &ef(CEPHES_U[0]), CW);
    for &c in &CEPHES_U[1..] {
        den = ext_add(&ext_mul(&den, &zz, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&ze, &ext_div(&num, &den, CW), CW), CW)
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
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    let lows = [0.23046875f64, 0.4524739583333333];

    let one_q = |z: f64, ff: f64| 1.0 - f::w_rn53(z) * ff;
    let one_q80 = |z: f64, ff: f64| {
        ext_to_f64(
            &ext_sub(&ef(1.0), &ext_mul(&ef(f::w_rn53(z)), &ef(ff), CW), CW),
            CW,
        )
    };
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 14] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("j(z+1)", Box::new(|z| erf_a(z.next_up(), &joint))),
        ("j(z+2)", Box::new(|z| erf_a(z.next_up().next_up(), &joint))),
        ("j(f32 z)", Box::new(|z| erf_a(f64::from(z as f32), &joint))),
        ("cephes T/U", Box::new(cephes_erf)),
        ("cephes T/U80", Box::new(cephes_erf80)),
        ("1-libm erfc", Box::new(|z| 1.0 - libm::erfc(z))),
        ("1-w*cephesF", Box::new(|z| one_q(z, f::cephes_f(z)))),
        ("1-w*codyF", Box::new(|z| one_q(z, f::cody_erfcx_f(z)))),
        ("80(1-w*cephes)", Box::new(|z| one_q80(z, f::cephes_f(z)))),
        ("libm erf", Box::new(|z| libm::erf(z))),
        ("next_up(joint)", Box::new(|z| erf_a(z, &joint).next_up())),
        ("next_up2(joint)", Box::new(|z| erf_a(z, &joint).next_up().next_up())),
        ("1-next_down(libm erfc)", Box::new(|z| 1.0 - libm::erfc(z).next_down())),
    ];
    println!("remaining LOW n=2  bar joint 866");
    for &(z, bits) in &rows {
        if lows.iter().all(|&t| (z - t).abs() > 1e-14) {
            continue;
        }
        let t = f64::from_bits(bits);
        print!("z={z:.16} excel={bits:#x}");
        for (name, ev) in &graphs {
            let g = ev(z);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t { "L" } else if g > t { "H" } else { "=" };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut keep = 0usize;
        let mut hit = 0usize;
        let mut u1 = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &lz in &lows {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            let d = ulp_distance(ev(lz), f64::from_bits(*bits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={lz:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:22} hit={hit}/2 ulp1={u1} keep={keep}/{}", rows.len());
    }
}
