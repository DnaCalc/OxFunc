//! A21 as Taylor erf/x: A[n]=2/sqrt(pi)*(-1)^n/(n!(2n+1)). Named vs A1-2 poke. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_chs, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80,
    CW_PC64_RN,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PI: f64 = 3.1415926535897932384626433832795;
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
fn two_rsqpi_x87() -> Ext80 {
    ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW)
}
fn kn_x87(n: usize) -> Ext80 {
    let mut acc = ef(1.0);
    for i in 1..=n {
        acc = ext_div(&acc, &ef(i as f64), CW);
    }
    acc = ext_div(&acc, &ef((2 * n + 1) as f64), CW);
    if n % 2 == 1 {
        ext_chs(&acc, CW)
    } else {
        acc
    }
}
fn taylor_x87() -> [Ext80; 21] {
    let c = two_rsqpi_x87();
    let mut a = [ef(0.0); 21];
    a[0] = ext_sub(&c, &ef(1.0), CW);
    for n in 1..21 {
        a[n] = ext_mul(&c, &kn_x87(n), CW);
    }
    a
}
fn taylor_f64() -> [f64; 21] {
    let c = 2.0 / PI.sqrt();
    let mut a = [0.0; 21];
    a[0] = c - 1.0;
    let mut fact = 1.0;
    for n in 1..21 {
        fact *= n as f64;
        let k = if n % 2 == 1 {
            -1.0 / (fact * (2 * n + 1) as f64)
        } else {
            1.0 / (fact * (2 * n + 1) as f64)
        };
        a[n] = c * k;
    }
    a
}
fn erf_f64a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn erf_80a(z: f64, a: &[Ext80; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), c, CW);
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
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    let tx = taylor_x87();
    let tf = taylor_f64();
    let a1p = poke(AS0[1], -2);
    let a1x = ext_to_f64(&tx[1], CW);
    let a1f = tf[1];
    println!(
        "A1 pub {:016x} A1-2 {:016x} tay_f64 {:016x} tay_x87_store {:016x}",
        AS0[1].to_bits(),
        a1p.to_bits(),
        a1f.to_bits(),
        a1x.to_bits()
    );
    println!(
        "A1-2==tay_f64 {} A1-2==tay_x87 {} pub==tay_f64 {}",
        a1p.to_bits() == a1f.to_bits(),
        a1p.to_bits() == a1x.to_bits(),
        AS0[1].to_bits() == a1f.to_bits()
    );
    let mut aj = AS0;
    aj[0] = poke(AS0[0], 4);
    aj[1] = poke(AS0[1], -2);
    aj[2] = poke(AS0[2], -5);
    let mut a0a1 = AS0;
    a0a1[0] = poke(AS0[0], 4);
    a0a1[1] = poke(AS0[1], -2);
    let mut a0_tay1 = AS0;
    a0_tay1[0] = poke(AS0[0], 4);
    a0_tay1[1] = a1f;
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 6] = [
        ("A21 CR published", Box::new(|z| erf_f64a(z, &AS0))),
        (
            "joint A0+4 A1-2 A2-5",
            Box::new({
                let a = aj;
                move |z| erf_f64a(z, &a)
            }),
        ),
        ("Taylor f64 2/sqrt(pi)*k_n", Box::new(move |z| erf_f64a(z, &tf))),
        ("Taylor x87 2/sqrt(pi)*k_n", Box::new(move |z| erf_80a(z, &tx))),
        (
            "A0+4 A1-2 rest pub",
            Box::new({
                let a = a0a1;
                move |z| erf_f64a(z, &a)
            }),
        ),
        (
            "A0+4 A1=tay_f64 rest pub",
            Box::new({
                let a = a0_tay1;
                move |z| erf_f64a(z, &a)
            }),
        ),
    ];
    println!("P-side n={}", rows.len());
    for (name, ev) in named {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu}");
    }
    let erf_lead = |z: f64| {
        let xe = ef(z.abs());
        let u = ext_mul(&xe, &xe, CW);
        let mut acc = ef(0.0);
        for &c in AS0[1..].iter().rev() {
            acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
        }
        let tail = ext_mul(&u, &acc, CW);
        ext_to_f64(
            &ext_mul(&xe, &ext_add(&two_rsqpi_x87(), &tail, CW), CW),
            CW,
        )
    };
    let mut both = 0usize;
    let mut only_t = 0usize;
    let mut only_l = 0usize;
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let dt = ulp_distance(erf_80a(z, &tx), want).unwrap_or(99);
        let dl = ulp_distance(erf_lead(z), want).unwrap_or(99);
        match (dt == 0, dl == 0) {
            (true, true) => both += 1,
            (true, false) => only_t += 1,
            (false, true) => only_l += 1,
            _ => {}
        }
    }
    println!(
        "Taylor-x87 vs lead both={} only_tay={} only_lead={} union={}",
        both,
        only_t,
        only_l,
        both + only_t + only_l
    );
}
