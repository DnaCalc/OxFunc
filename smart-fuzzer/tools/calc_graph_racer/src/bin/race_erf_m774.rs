//! MATH77 a[1]−4 / max-2 basin vs 4 LOW and joint. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const A0: [f64; 21] = [
    -0.49046121234691808039984544033376e-1,
    -0.14226120510371364237824741899631e+0,
    0.10035582187599795575754676712933e-1,
    -0.57687646997674847650827025509167e-3,
    0.27419931252196061034422160791471e-4,
    -0.11043175507344507604135381295905e-5,
    0.38488755420345036949961311498174e-7,
    -0.11808582533875466969631751801581e-8,
    0.32334215826050909646402930953354e-10,
    -0.79910159470045487581607374708595e-12,
    0.17990725113961455611967245486634e-13,
    -0.37186354878186926382316828209493e-15,
    0.71035990037142529711689908394666e-17,
    -0.12612455119155225832495424853333e-18,
    0.20916406941769294369170500266666e-20,
    -0.32539731029314072982364160000000e-22,
    0.47668672097976748332373333333333e-24,
    -0.65980120782851343155199999999999e-26,
    0.86550114699637626197333333333333e-28,
    -0.10788925177498064213333333333333e-29,
    0.12811883993017002666666666666666e-31,
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
fn erf_m77(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let twox = ext_mul(&ef(2.0), &t, CW);
    let mut b0 = ef(0.0);
    let mut b1 = ef(0.0);
    let mut b2 = ef(0.0);
    for i in (0..a.len()).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(
            &ext_sub(&ext_mul(&twox, &b1, CW), &b2, CW),
            &ef(a[i]),
            CW,
        );
    }
    let y = ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW);
    let s = ext_add(&ef(1.0), &y, CW);
    ext_to_f64(&ext_mul(&xe, &s, CW), CW)
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
    let mcr = |z: f64| erf_m77(z, &A0);
    let ma14 = |z: f64| {
        let mut a = A0;
        a[1] = poke(A0[1], -4);
        erf_m77(z, &a)
    };
    let m856 = |z: f64| {
        let mut a = A0;
        a[1] = poke(A0[1], -4);
        a[0] = poke(A0[0], 1);
        a[3] = poke(A0[3], 1);
        erf_m77(z, &a)
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
        ("m77CR", Box::new(mcr)),
        ("a1-4", Box::new(ma14)),
        ("m856", Box::new(m856)),
        ("joint", Box::new(jf)),
    ];
    println!("4 LOW MATH77 max-2:");
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
        let mut mx = 0u64;
        for &(z, bits) in &rows {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                keep += 1;
            } else {
                mx = mx.max(d);
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
        println!("{name:8} hit={hit}/4 keep={keep}/{} max={mx}", rows.len());
    }
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_m = 0usize;
    let mut only_m_u1j = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let dj = ulp_distance(jf(z), t).unwrap_or(99);
        let dm = ulp_distance(m856(z), t).unwrap_or(99);
        match (dj == 0, dm == 0) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_m += 1;
                if dj == 1 {
                    only_m_u1j += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "m856 vs joint both={both} only_j={only_j} only_m={only_m} (j ulp1={only_m_u1j}) union={}",
        both + only_j + only_m
    );
}
