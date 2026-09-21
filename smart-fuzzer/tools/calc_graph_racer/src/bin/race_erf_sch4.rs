//! Schonfelder CR / a[2]+5 / joint-862 vs 4 LOW and A21-joint. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const A0: [f64; 18] = [
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
const LOWS: [f64; 4] = [
    0.23046875,
    0.37109375,
    0.4524739583333333,
    0.4716796875,
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
fn erf_sch(z: f64, a: &[f64; 18]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(a[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
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
    let a2f = |z: f64| {
        let mut a = A0;
        a[2] = poke(A0[2], 5);
        erf_sch(z, &a)
    };
    let a862f = |z: f64| {
        let mut a = A0;
        a[0] = f64::from_bits(0x3ff7bad224918aa1);
        a[1] = f64::from_bits(0xbfd344bf9b7de400);
        a[3] = f64::from_bits(0xbf8c80224fb6c6ce);
        a[4] = f64::from_bits(0x3f63d4c8d8edc41a);
        a[5] = f64::from_bits(0xbf37fa2dc7a4ad66);
        erf_sch(z, &a)
    };
    let jf = |z: f64| {
        let mut a = AS0;
        a[0] = poke(AS0[0], 4);
        a[1] = poke(AS0[1], -2);
        a[2] = poke(AS0[2], -5);
        a[3] = poke(AS0[3], 1);
        erf_a(z, &a)
    };
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 4] = [
        ("schCR", Box::new(|z| erf_sch(z, &A0))),
        ("a2+5", Box::new(a2f)),
        ("sch862", Box::new(a862f)),
        ("joint", Box::new(jf)),
    ];
    println!("4 LOW Schonfelder:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        print!("  z={lz:.16}");
        for (name, ev) in &graphs {
            let g = ev(lz);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t {
                "L"
            } else if g > t {
                "H"
            } else {
                "="
            };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut keep = 0usize;
        let mut hit = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(*bits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:8} hit={hit}/4 keep={keep}/{}", rows.len());
    }
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_s = 0usize;
    let mut only_s_u1j = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let dj = ulp_distance(jf(z), t).unwrap_or(99);
        let ds = ulp_distance(a862f(z), t).unwrap_or(99);
        match (dj == 0, ds == 0) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_s += 1;
                if dj == 1 {
                    only_s_u1j += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "sch862 vs joint both={both} only_j={only_j} only_s={only_s} (j ulp1={only_s_u1j}) union={}",
        both + only_j + only_s
    );
}
