//! Cody A/B directed on joint ulp=3 singleton and +2 LOW 13. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
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
fn cody_ab(z: f64) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
            &ext_add(&xden, &ef(BB[3]), CW),
            CW,
        ),
        CW,
    )
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

    let mut low2: Vec<(f64, u64)> = Vec::new();
    let mut u3: Vec<(f64, u64)> = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 2 && g < t {
            low2.push((z, bits));
        }
        if d >= 3 {
            u3.push((z, bits));
        }
    }
    println!("joint +2 LOW n={} ulp>=3 n={}", low2.len(), u3.len());

    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("AB", Box::new(cody_ab)),
        ("dn(AB)", Box::new(|z| cody_ab(z).next_down())),
        ("dn2(AB)", Box::new(|z| cody_ab(z).next_down().next_down())),
        ("up(AB)", Box::new(|z| cody_ab(z).next_up())),
        ("A21pub", Box::new(|z| erf_a(z, &AS0))),
        ("libm", Box::new(|z| libm::erf(z))),
        ("1-erfc", Box::new(|z| 1.0 - libm::erfc(z))),
    ];
    println!("ulp>=3 named:");
    for &(z, bits) in &u3 {
        let t = f64::from_bits(bits);
        print!("  z={z:.16}");
        for (name, ev) in &graphs {
            let g = ev(z);
            print!(
                " {name}={}{}",
                ulp_distance(g, t).unwrap_or(99),
                if g < t { "L" } else if g > t { "H" } else { "=" }
            );
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut keep = 0usize;
        let mut hit2 = 0usize;
        let mut hit3 = 0usize;
        let mut u1_2 = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits) in &low2 {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                hit2 += 1;
            } else if d == 1 {
                u1_2 += 1;
            }
        }
        for &(z, bits) in &u3 {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit3 += 1;
            }
        }
        println!(
            "{name:8} keep={keep}/{}  +2LOW hit={hit2}/{} ulp1={u1_2}  u3 hit={hit3}/{}",
            rows.len(),
            low2.len(),
            u3.len()
        );
    }
}
