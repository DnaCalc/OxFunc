//! P-side leading 2/sqrt(pi) in place of 1+A[0]. Named constant vs joint A0 poke. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_pi, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
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
const SQRPI: f64 = 5.6418958354775628695e-1;
const TWO_RSQPI: f64 = 1.1283791670955125738961589031215;
const PI: f64 = 3.1415926535897932384626433832795;

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
fn horner_tail(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    acc
}
fn erf_lead(z: f64, lead: Ext80, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let tail = ext_mul(&u, &horner_tail(z, a), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&lead, &tail, CW), CW), CW)
}
fn erf_one_plus(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn lead_const() -> Ext80 {
    ef(TWO_RSQPI)
}
fn lead_two_sqrpi() -> Ext80 {
    ext_mul(&ef(2.0), &ef(SQRPI), CW)
}
fn lead_f64_div() -> Ext80 {
    ef(2.0 / PI.sqrt())
}
fn lead_x87_div() -> Ext80 {
    ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW)
}
fn lead_x87_two_rsqrt() -> Ext80 {
    ext_mul(&ef(2.0), &ext_div(&ef(1.0), &ext_sqrt(&ef(PI), CW), CW), CW)
}
fn lead_x87_pi() -> Ext80 {
    ext_div(&ef(2.0), &ext_sqrt(&ext_pi(), CW), CW)
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
    let mut aj = AS0;
    aj[0] = poke(AS0[0], 4);
    aj[1] = poke(AS0[1], -2);
    aj[2] = poke(AS0[2], -5);
    aj[3] = poke(AS0[3], 1);
    let mut at = AS0;
    at[1] = poke(AS0[1], -2);
    at[2] = poke(AS0[2], -5);
    at[3] = poke(AS0[3], 1);
    println!(
        "1+A0_pub={:016x} 1+A0_j={:016x} TWO_RSQPI={:016x} 2*SQRPI={:016x} 2/sqrt(pi)f64={:016x}",
        (1.0 + AS0[0]).to_bits(),
        (1.0 + aj[0]).to_bits(),
        TWO_RSQPI.to_bits(),
        (2.0 * SQRPI).to_bits(),
        (2.0 / PI.sqrt()).to_bits()
    );
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 9] = [
        ("A21 CR 1+Horner", Box::new(|z| erf_one_plus(z, &AS0))),
        (
            "A21 joint 1+Horner",
            Box::new({
                let a = aj;
                move |z| erf_one_plus(z, &a)
            }),
        ),
        (
            "lead TWO_RSQPI + A[1:] CR",
            Box::new(|z| erf_lead(z, lead_const(), &AS0)),
        ),
        (
            "lead TWO_RSQPI + A[1:] joint",
            Box::new({
                let a = at;
                move |z| erf_lead(z, lead_const(), &a)
            }),
        ),
        (
            "lead 2*SQRPI + A[1:] CR",
            Box::new(|z| erf_lead(z, lead_two_sqrpi(), &AS0)),
        ),
        (
            "lead 2/sqrt(pi) f64 + CR",
            Box::new(|z| erf_lead(z, lead_f64_div(), &AS0)),
        ),
        (
            "lead x87 2/sqrt(pi) + CR",
            Box::new(|z| erf_lead(z, lead_x87_div(), &AS0)),
        ),
        (
            "lead x87 2*(1/sqrt(pi))+CR",
            Box::new(|z| erf_lead(z, lead_x87_two_rsqrt(), &AS0)),
        ),
        (
            "lead x87 2/sqrt(ext_pi)+CR",
            Box::new(|z| erf_lead(z, lead_x87_pi(), &AS0)),
        ),
    ];
    let x87_lead = lead_x87_div();
    let x87_lead_f64 = ext_to_f64(&x87_lead, CW);
    let extra: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        (
            "x87 2/sqrt(pi) + joint A[1:]",
            Box::new({
                let a = at;
                move |z| erf_lead(z, x87_lead, &a)
            }),
        ),
        (
            "store x87 lead f64 + CR",
            Box::new(move |z| erf_lead(z, ef(x87_lead_f64), &AS0)),
        ),
        (
            "store x87 lead f64 + joint",
            Box::new({
                let a = at;
                move |z| erf_lead(z, ef(x87_lead_f64), &a)
            }),
        ),
        (
            "next_up x87 lead + CR",
            Box::new(move |z| erf_lead(z, ef(x87_lead_f64.next_up()), &AS0)),
        ),
        (
            "next_up x87 lead + joint",
            Box::new({
                let a = at;
                move |z| erf_lead(z, ef(x87_lead_f64.next_up()), &a)
            }),
        ),
    ];
    println!("P-side rows={}", rows.len());
    let mut all = Vec::from(named);
    all.extend(extra);
    let mut scores: Vec<(String, usize)> = Vec::new();
    for (name, ev) in &all {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut tiny_ex = 0usize;
        let mut tiny_n = 0usize;
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
            if z < 1e-6 {
                tiny_n += 1;
                if d == 0 {
                    tiny_ex += 1;
                }
            }
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu} tiny(<1e-6) {tiny_ex}/{tiny_n}");
        scores.push((name.to_string(), ex));
    }
    let ev_j = |z| erf_one_plus(z, &aj);
    let ev_x = |z| erf_lead(z, x87_lead, &AS0);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_x = 0usize;
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let dj = ulp_distance(ev_j(z), want).unwrap_or(99);
        let dx = ulp_distance(ev_x(z), want).unwrap_or(99);
        match (dj == 0, dx == 0) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => only_x += 1,
            _ => {}
        }
    }
    println!(
        "overlap joint vs x87-lead: both={} only_joint={} only_x87={} union={}",
        both,
        only_j,
        only_x,
        both + only_j + only_x
    );
    let mut band_j = [0usize; 5];
    let mut band_x = [0usize; 5];
    let mut band_n = [0usize; 5];
    let mut band_only_x = [0usize; 5];
    let mut band_only_j = [0usize; 5];
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let dj = ulp_distance(ev_j(z), want).unwrap_or(99);
        let dx = ulp_distance(ev_x(z), want).unwrap_or(99);
        let b = ((z / 0.1) as usize).min(4);
        band_n[b] += 1;
        if dj == 0 {
            band_j[b] += 1;
        }
        if dx == 0 {
            band_x[b] += 1;
        }
        if dx == 0 && dj != 0 {
            band_only_x[b] += 1;
        }
        if dj == 0 && dx != 0 {
            band_only_j[b] += 1;
        }
    }
    for i in 0..5 {
        println!(
            "[{:.1},{:.1}) n={} joint={} x87lead={} only_j={} only_x={}",
            i as f64 * 0.1,
            i as f64 * 0.1 + 0.1,
            band_n[i],
            band_j[i],
            band_x[i],
            band_only_j[i],
            band_only_x[i]
        );
    }
    let mut best_xj = 0usize;
    let mut best_xj_cut = 0.0;
    let mut best_jx = 0usize;
    let mut best_jx_cut = 0.0;
    let cuts = [
        0.05, 0.0625, 0.1, 0.125, 0.15, 0.2, 0.25, 0.3, 0.375, 0.4, 0.46875, 0.5,
    ];
    for &c in &cuts {
        let mut xj = 0usize;
        let mut jx = 0usize;
        for &(z, pbits) in &rows {
            let want = f64::from_bits(pbits);
            let g = if z < c { ev_x(z) } else { ev_j(z) };
            let h = if z < c { ev_j(z) } else { ev_x(z) };
            if ulp_distance(g, want).unwrap_or(99) == 0 {
                xj += 1;
            }
            if ulp_distance(h, want).unwrap_or(99) == 0 {
                jx += 1;
            }
        }
        println!("cut={c:.5} x87-then-joint={xj} joint-then-x87={jx}");
        if xj > best_xj {
            best_xj = xj;
            best_xj_cut = c;
        }
        if jx > best_jx {
            best_jx = jx;
            best_jx_cut = c;
        }
    }
    println!(
        "best x87-then-joint {best_xj} @ {best_xj_cut} ; best joint-then-x87 {best_jx} @ {best_jx_cut}"
    );
    let _ = scores;
}
