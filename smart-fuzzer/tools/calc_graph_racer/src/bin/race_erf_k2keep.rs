//! k=2 leftover-low keep vs convert of joint. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
fn k_of(z: f64, t: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(poke(ff, k), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
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
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    fn grid(z: f64) -> &'static str {
        if z.fract() == 0.0 {
            "int"
        } else if (z * 2.0).fract() == 0.0 {
            "dyad2"
        } else if (z * 16.0).fract() == 0.0 {
            "dyad16"
        } else if (z * 48.0 - (z * 48.0).round()).abs() < 1e-12 {
            "48"
        } else if (z * 96.0 - (z * 96.0).round()).abs() < 1e-12 {
            "96"
        } else {
            "other"
        }
    }
    fn is555(z: f64) -> bool {
        let h = format!("{:x}", z.to_bits());
        h.contains("55555555") || h.contains("aaaaaaaa")
    }
    let mut keep_g: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut conv_g: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut k3_g: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut keep_555 = 0usize;
    let mut conv_555 = 0usize;
    let mut k3_555 = 0usize;
    let mut nkeep = 0usize;
    let mut nconv = 0usize;
    let mut nk3 = 0usize;
    let mut keep_tz = [0usize; 53];
    let mut conv_tz = [0usize; 53];
    let mut k3_tz = [0usize; 53];
    println!("k=2 leftover-low keep vs convert; k=3 leftover-low:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        let d0 = ulp_distance(ff, t).unwrap_or(99);
        if !(d0 >= 2 && ff < t) {
            continue;
        }
        let k = k_of(z, t, ff);
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let gname = grid(z);
        let s555 = is555(z);
        let tz = ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        match k {
            Some(2) if dj >= 2 && gj < t => {
                nkeep += 1;
                *keep_g.entry(gname).or_insert(0) += 1;
                keep_tz[tz] += 1;
                if s555 {
                    keep_555 += 1;
                }
                println!(
                    "  KEEP k=2 z={:.16} bits={:#x} grid={gname} 555={s555} j_d={dj}",
                    z,
                    z.to_bits()
                );
            }
            Some(2) if dj == 1 => {
                nconv += 1;
                *conv_g.entry(gname).or_insert(0) += 1;
                conv_tz[tz] += 1;
                if s555 {
                    conv_555 += 1;
                }
            }
            Some(3) if dj >= 2 && gj < t => {
                nk3 += 1;
                *k3_g.entry(gname).or_insert(0) += 1;
                k3_tz[tz] += 1;
                if s555 {
                    k3_555 += 1;
                }
                println!(
                    "  KEEP k=3 z={:.16} bits={:#x} grid={gname} 555={s555} j_d={dj}",
                    z,
                    z.to_bits()
                );
            }
            _ => {}
        }
    }
    print!("KEEP k=2 n={nkeep} 555={keep_555} grid");
    for (k, v) in &keep_g {
        print!(" {k}:{v}");
    }
    print!("\nCONV k=2 n={nconv} 555={conv_555} grid");
    for (k, v) in &conv_g {
        print!(" {k}:{v}");
    }
    print!("\nKEEP k=3 n={nk3} 555={k3_555} grid");
    for (k, v) in &k3_g {
        print!(" {k}:{v}");
    }
    println!();
    let prtz = |name: &str, h: &[usize; 53]| {
        print!("{name} tz");
        for (i, c) in h.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        println!();
    };
    prtz("KEEP k=2", &keep_tz);
    prtz("CONV k=2", &conv_tz);
    prtz("KEEP k=3", &k3_tz);
}
