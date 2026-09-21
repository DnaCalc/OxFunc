//! joint A21 two-mode last-store cover vs unpoked. Not an identity.
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
fn horner_w(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    acc
}
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    ext_to_f64(
        &ext_mul(&ef(z), &ext_add(&ef(1.0), &horner_w(z, a), CW), CW),
        CW,
    )
}
fn one_w(z: f64, a: &[f64; 21]) -> f64 {
    ext_to_f64(&ext_add(&ef(1.0), &horner_w(z, a), CW), CW)
}

fn cover(name: &str, map: &BTreeMap<u64, u64>, a: &[f64; 21]) {
    let mut n = 0usize;
    let mut up = [0usize; 6];
    let mut dn = [0usize; 6];
    let mut up_none = 0usize;
    let mut dn_none = 0usize;
    let mut tmu_ex = 0usize;
    let mut tmd_ex = 0usize;
    for (&zb, &eb) in map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        n += 1;
        let ff = erf_a(z, a);
        let s = one_w(z, a);
        let q_tmu = z.next_up() * s.next_up();
        let q_tmd = z.next_down() * s.next_down();
        if ulp_distance(q_tmu, t).unwrap_or(99) == 0 {
            tmu_ex += 1;
        }
        if ulp_distance(q_tmd, t).unwrap_or(99) == 0 {
            tmd_ex += 1;
        }
        let mut uok = false;
        let mut dok = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(ff, ak).to_bits() {
                up[ak as usize] += 1;
                uok = true;
            }
            if q_tmd.to_bits() == poke(ff, -ak).to_bits() {
                dn[ak as usize] += 1;
                dok = true;
            }
        }
        if !uok {
            up_none += 1;
        }
        if !dok {
            dn_none += 1;
        }
    }
    print!("{name} n={n} tmu_excel={tmu_ex} tmd_excel={tmd_ex} up==F+k");
    for (i, c) in up.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if up_none > 0 {
        print!(" none:{up_none}");
    }
    print!(" down==F-k");
    for (i, c) in dn.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if dn_none > 0 {
        print!(" none:{dn_none}");
    }
    println!();
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
    println!("P-side two-mode last-store cover of unpoked vs joint A21:");
    cover("unpoked", &map, &AS0);
    cover("joint", &map, &joint);
}
