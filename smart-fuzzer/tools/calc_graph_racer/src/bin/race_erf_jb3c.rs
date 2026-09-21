//! B[3]−1 A/B vs joint bands and z-cuts. Not an identity.
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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_ab(z: f64, a: &[f64; 5], b: &[f64; 4]) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(a[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(a[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(b[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(a[3]), CW), CW),
            &ext_add(&xden, &ef(b[3]), CW),
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
    let mut b3 = BB;
    b3[3] = poke(BB[3], -1);

    let mut only_b_z: Vec<f64> = Vec::new();
    let mut only_j_z: Vec<f64> = Vec::new();
    let mut band = [[0usize; 4]; 10];
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let dj = ulp_distance(erf_a(z, &joint), t).unwrap_or(99);
        let da = ulp_distance(cody_ab(z, &AA, &b3), t).unwrap_or(99);
        let bi = ((z * 20.0).floor() as usize).min(9);
        band[bi][3] += 1;
        match (dj == 0, da == 0) {
            (true, true) => band[bi][0] += 1,
            (true, false) => {
                band[bi][1] += 1;
                only_j_z.push(z);
            }
            (false, true) => {
                band[bi][2] += 1;
                only_b_z.push(z);
            }
            _ => {}
        }
    }
    println!("0.05-bands both/only_j/only_B3/n:");
    for i in 0..10 {
        let lo = i as f64 * 0.05;
        println!(
            "  [{lo:.2},{:.2}) {} {} {} {}",
            lo + 0.05,
            band[i][0],
            band[i][1],
            band[i][2],
            band[i][3]
        );
    }
    let p = |zs: &[f64], lab: &str| {
        if zs.is_empty() {
            println!("{lab} empty");
            return;
        }
        let n_tiny = zs.iter().filter(|&&z| z < 1e-6).count();
        let n_01 = zs.iter().filter(|&&z| z < 0.01).count();
        let n_1 = zs.iter().filter(|&&z| z < 0.1).count();
        println!(
            "{lab} n={} min={:.16} max={:.16} <1e-6={n_tiny} <0.01={n_01} <0.1={n_1}",
            zs.len(),
            zs[0],
            zs[zs.len() - 1]
        );
    };
    p(&only_b_z, "only_B3");
    p(&only_j_z, "only_j");

    println!("cuts (bar joint 866 / B3 789):");
    let cuts = [
        1e-8, 1e-6, 1e-4, 0.001, 0.01, 0.02, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45,
        0.48, 0.49,
    ];
    let mut best_abj = 0usize;
    let mut best_jab = 0usize;
    let mut lab_abj = String::new();
    let mut lab_jab = String::new();
    for &c in &cuts {
        let mut abj = 0usize;
        let mut jab = 0usize;
        let mut hit_abj = 0usize;
        let mut hit_jab = 0usize;
        for &(z, bits) in &rows {
            let t = f64::from_bits(bits);
            let gj = erf_a(z, &joint);
            let ga = cody_ab(z, &AA, &b3);
            let g = if z < c { ga } else { gj };
            let h = if z < c { gj } else { ga };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                abj += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                jab += 1;
            }
        }
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            let t = f64::from_bits(*bits);
            let g = if lz < c {
                cody_ab(lz, &AA, &b3)
            } else {
                erf_a(lz, &joint)
            };
            let h = if lz < c {
                erf_a(lz, &joint)
            } else {
                cody_ab(lz, &AA, &b3)
            };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                hit_abj += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                hit_jab += 1;
            }
        }
        println!("  cut={c} B3-then-j={abj} hit={hit_abj}/4 j-then-B3={jab} hit={hit_jab}/4");
        if abj > best_abj {
            best_abj = abj;
            lab_abj = format!("{c}");
        }
        if jab > best_jab {
            best_jab = jab;
            lab_jab = format!("{c}");
        }
    }
    println!("best B3-then-j {lab_abj} keep={best_abj}");
    println!("best j-then-B3 {lab_jab} keep={best_jab}");
}
