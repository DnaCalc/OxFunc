//! joint ∪ two-mode-up ∪ two-mode-down on P-side. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
const CW_RD: u16 = CW_PC64_RN | 0x0400;
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
fn one_w(z: f64) -> Ext80 {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
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
    let jn = |z: f64| ext_to_f64(&ext_mul(&ef(z), &one_w(z), CW), CW);
    let ru = |z: f64| ext_to_f64(&ext_mul(&ef(z.next_up()), &one_w(z), CW_RU), CW_RU);
    let tfu = |z: f64| {
        let ow = ext_to_f64(&one_w(z), CW).next_up();
        ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW)
    };
    let rd = |z: f64| ext_to_f64(&ext_mul(&ef(z.next_down()), &one_w(z), CW_RD), CW_RD);
    let tfd = |z: f64| {
        let ow = ext_to_f64(&one_w(z), CW).next_down();
        ext_to_f64(&ext_mul(&ef(z.next_down()), &ef(ow), CW), CW)
    };
    let mut n_j = 0usize;
    let mut n_u = 0usize;
    let mut n_d = 0usize;
    let mut n_any = 0usize;
    let mut only_u = 0usize;
    let mut only_d = 0usize;
    let mut ju = 0usize;
    let mut jd = 0usize;
    let mut ud = 0usize;
    let mut hit_low = 0usize;
    let mut n_low = 0usize;
    let mut hit_high = 0usize;
    let mut n_high = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = jn(z);
        let ej = ulp_distance(g, t).unwrap_or(99) == 0;
        let eu = ulp_distance(ru(z), t).unwrap_or(99) == 0
            || ulp_distance(tfu(z), t).unwrap_or(99) == 0;
        let ed = ulp_distance(rd(z), t).unwrap_or(99) == 0
            || ulp_distance(tfd(z), t).unwrap_or(99) == 0;
        if ej {
            n_j += 1;
        }
        if eu {
            n_u += 1;
        }
        if ed {
            n_d += 1;
        }
        if ej || eu || ed {
            n_any += 1;
        }
        if eu && !ej && !ed {
            only_u += 1;
        }
        if ed && !ej && !eu {
            only_d += 1;
        }
        if ej && eu {
            ju += 1;
        }
        if ej && ed {
            jd += 1;
        }
        if eu && ed {
            ud += 1;
        }
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            n_low += 1;
            if eu || ed {
                hit_low += 1;
            } else {
                println!("  LOW-MISS z={z:.16} d={d}");
            }
        }
        if d >= 2 && g > t {
            n_high += 1;
            if eu || ed {
                hit_high += 1;
            } else {
                println!("  HIGH-MISS z={z:.16} d={d}");
            }
        }
    }
    println!("4 LOW two-mode-up:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let eu = ulp_distance(ru(lz), t).unwrap_or(99) == 0
            || ulp_distance(tfu(lz), t).unwrap_or(99) == 0;
        let ed = ulp_distance(rd(lz), t).unwrap_or(99) == 0
            || ulp_distance(tfd(lz), t).unwrap_or(99) == 0;
        println!("  z={lz:.16} up={eu} down={ed}");
    }
    println!(
        "keep J={n_j} up={n_u} down={n_d} any={n_any}/{} ju={ju} jd={jd} ud={ud} only_up={only_u} only_down={only_d} leftover-LOW {hit_low}/{n_low} leftover-HIGH {hit_high}/{n_high}",
        rows.len()
    );
    for &lz in &[0.224609375, 0.38671875] {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let ow = one_w(lz);
        let owf = ext_to_f64(&ow, CW);
        print!("  miss z={lz:.16}");
        for k in 1..=3 {
            let g = ext_to_f64(&ext_mul(&ef(poke(lz, k)), &ow, CW), CW);
            print!(
                " z+{k}={}",
                ulp_distance(g, t).unwrap_or(99) == 0
            );
            let h = ext_to_f64(&ext_mul(&ef(poke(lz, -k)), &ow, CW), CW);
            print!(
                " z-{k}={}",
                ulp_distance(h, t).unwrap_or(99) == 0
            );
            let p = ext_to_f64(&ext_mul(&ef(lz), &ef(poke(owf, k)), CW), CW);
            print!(
                " ow+{k}={}",
                ulp_distance(p, t).unwrap_or(99) == 0
            );
            let q = ext_to_f64(&ext_mul(&ef(lz), &ef(poke(owf, -k)), CW), CW);
            print!(
                " ow-{k}={}",
                ulp_distance(q, t).unwrap_or(99) == 0
            );
        }
        println!();
    }
}
