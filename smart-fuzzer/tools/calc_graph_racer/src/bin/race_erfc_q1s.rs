//! Q z<0.5 as 1−P joint leftover signed-ULP. Complement tracking. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
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
    let qrows = f::load_q_rows_tagged(&dir);
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
    let prows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    println!("1-joint as Q z<0.5 leftover signed (bar 1520/1864):");
    let mut ex = 0usize;
    let mut dd = 0usize;
    let mut n = 0usize;
    let mut hi = 0usize;
    let mut lo = 0usize;
    let mut h1 = 0usize;
    let mut l1 = 0usize;
    let mut mx = 0u64;
    for r in &qrows {
        if r.z >= 0.5 || r.z <= 0.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let g = 1.0 - erf_a(r.z, &joint);
        let d = ulp_distance(g, t).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        if d == 0 {
            ex += 1;
            if r.direct {
                dd += 1;
            }
        } else {
            mx = mx.max(d);
            if g > t {
                hi += 1;
                if d == 1 {
                    h1 += 1;
                }
            } else {
                lo += 1;
                if d == 1 {
                    l1 += 1;
                }
            }
        }
    }
    println!("  1-joint Q {ex}/{n} d={dd} max={mx}  left>={hi} (ulp1={h1}) left<={lo} (ulp1={l1})");
    println!("P-side joint leftover signed (bar 866/1508) for comparison:");
    let mut pex = 0usize;
    let mut phi = 0usize;
    let mut plo = 0usize;
    let mut ph1 = 0usize;
    let mut pl1 = 0usize;
    for &(z, bits) in &prows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 0 {
            pex += 1;
        } else if g > t {
            phi += 1;
            if d == 1 {
                ph1 += 1;
            }
        } else {
            plo += 1;
            if d == 1 {
                pl1 += 1;
            }
        }
    }
    println!(
        "  joint P {pex}/{}  left>={phi} (ulp1={ph1}) left<={plo} (ulp1={pl1})",
        prows.len()
    );
    let mut both_exact = 0usize;
    let mut q_ex_p_hi = 0usize;
    let mut q_lo_p_hi = 0usize;
    for r in &qrows {
        if r.z >= 0.5 || r.z <= 0.0 {
            continue;
        }
        let Some(&pbits) = prows
            .iter()
            .find(|(z, _)| *z == r.z)
            .map(|(_, b)| b)
            .or(None)
        else {
            continue;
        };
        let p = erf_a(r.z, &joint);
        let q = 1.0 - p;
        let dp = ulp_distance(p, f64::from_bits(pbits)).unwrap_or(99);
        let dq = ulp_distance(q, f64::from_bits(r.qbits)).unwrap_or(99);
        if dp == 0 && dq == 0 {
            both_exact += 1;
        }
        if dp != 0 && p > f64::from_bits(pbits) && dq == 0 {
            q_ex_p_hi += 1;
        }
        if dp != 0 && p > f64::from_bits(pbits) && dq != 0 && q < f64::from_bits(r.qbits) {
            q_lo_p_hi += 1;
        }
    }
    println!(
        "paired rows: both_exact={both_exact} Qexact_while_P_high={q_ex_p_hi} Qlow_while_P_high={q_lo_p_hi}"
    );
}
