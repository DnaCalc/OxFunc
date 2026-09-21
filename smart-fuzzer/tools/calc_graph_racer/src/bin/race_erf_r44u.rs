//! remaining 44: unpoked last-store / two-mode-up / short A21. Not an identity.
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
const TWOSQRTPI: f64 = 1.1283791670955125738961589031215;

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
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}
fn a21n(z: f64, a: &[f64], n: usize) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a[..n].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn one_w_a(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn hit_ls_a(z: f64, t: f64, a: &[f64; 21]) -> bool {
    let ow = one_w_a(z, a);
    let owf = ext_to_f64(&ow, CW);
    [-2i32, -1, 1, 2].iter().any(|&k| {
        ulp_distance(ext_to_f64(&ext_mul(&ef(poke(z, k)), &ow, CW), CW), t).unwrap_or(99) == 0
            || ulp_distance(ext_to_f64(&ext_mul(&ef(z), &ef(poke(owf, k)), CW), CW), t)
                .unwrap_or(99)
                == 0
    })
}
fn hit_tm_a(z: f64, t: f64, a: &[f64; 21]) -> bool {
    let oy = one_w_a(z, a);
    let ru = ext_to_f64(&ext_mul(&ef(z.next_up()), &oy, CW_RU), CW_RU);
    let ow = ext_to_f64(&oy, CW).next_up();
    let tf = ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW);
    let rd = ext_to_f64(&ext_mul(&ef(z.next_down()), &oy, CW_RD), CW_RD);
    let ow2 = ext_to_f64(&oy, CW).next_down();
    let tfd = ext_to_f64(&ext_mul(&ef(z.next_down()), &ef(ow2), CW), CW);
    ulp_distance(ru, t).unwrap_or(99) == 0
        || ulp_distance(tf, t).unwrap_or(99) == 0
        || ulp_distance(rd, t).unwrap_or(99) == 0
        || ulp_distance(tfd, t).unwrap_or(99) == 0
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
    let ja = joint_a();
    let mut left: Vec<(f64, u64)> = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        if ulp_distance(a21n(z, &ja, 21), t).unwrap_or(99) == 0
            || hit_ls_a(z, t, &ja)
            || hit_tm_a(z, t, &ja)
        {
            continue;
        }
        if ulp_distance(a21n(z, &AS0, 21), t).unwrap_or(99) == 0 {
            continue;
        }
        left.push((z, bits));
    }
    let mut h_ls = 0usize;
    let mut h_tm = 0usize;
    let mut h_z1 = 0usize;
    let mut h_ow1 = 0usize;
    let mut h_pi = 0usize;
    let mut hn = [0usize; 8];
    println!("remaining 44 unpoked last-store / short A21:");
    for &(z, bits) in &left {
        let t = f64::from_bits(bits);
        let ow = one_w_a(z, &AS0);
        let owf = ext_to_f64(&ow, CW);
        let z1 = ulp_distance(ext_to_f64(&ext_mul(&ef(z.next_up()), &ow, CW), CW), t).unwrap_or(99)
            == 0;
        let ow1 = ulp_distance(
            ext_to_f64(&ext_mul(&ef(z), &ef(owf.next_up()), CW), CW),
            t,
        )
        .unwrap_or(99)
            == 0;
        let ls = hit_ls_a(z, t, &AS0);
        let tm = hit_tm_a(z, t, &AS0);
        let pi = ulp_distance(
            ext_to_f64(&ext_mul(&ef(TWOSQRTPI), &ef(z), CW), CW),
            t,
        )
        .unwrap_or(99)
            == 0;
        if ls {
            h_ls += 1;
        }
        if tm {
            h_tm += 1;
        }
        if z1 {
            h_z1 += 1;
        }
        if ow1 {
            h_ow1 += 1;
        }
        if pi {
            h_pi += 1;
        }
        print!("  z={z:.16} ls={ls} tm={tm} z+1={z1} ow+1={ow1} x87_2rpi={pi}");
        for n in 1..=7 {
            let hit = ulp_distance(a21n(z, &AS0, n), t).unwrap_or(99) == 0;
            if hit {
                hn[n] += 1;
            }
            print!(" n{n}={hit}");
        }
        println!();
    }
    println!(
        "unpoked-ls={h_ls}/{} tm={h_tm} z+1={h_z1} ow+1={h_ow1} x87_2rpi={h_pi} n1={} n2={} n3={} n4={} n5={} n6={} n7={}",
        left.len(),
        hn[1],
        hn[2],
        hn[3],
        hn[4],
        hn[5],
        hn[6],
        hn[7]
    );
    let mut k_ls = 0usize;
    let mut k_tm = 0usize;
    let mut k_n1 = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        if hit_ls_a(z, t, &AS0) {
            k_ls += 1;
        }
        if hit_tm_a(z, t, &AS0) {
            k_tm += 1;
        }
        if ulp_distance(a21n(z, &AS0, 1), t).unwrap_or(99) == 0 {
            k_n1 += 1;
        }
    }
    println!(
        "keep unpoked-ls={k_ls}/{} unpoked-tm={k_tm} n1={k_n1} (bar unpoked 635 joint 866)",
        rows.len()
    );
}
