//! MATH77 ERFCS one-coeff ±16, keep-descent, DCSEVL HW=1,2 on P-side.
//! Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 24;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn erf_m77(z: f64, a: &[f64; 21], mask: u32) -> f64 {
    let xe = ef(z.abs());
    let mut t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    t = maybe(t, mask, 0);
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
        b0 = maybe(b0, mask, 1 + (a.len() - 1 - i) as u32);
    }
    let mut y = ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW);
    y = maybe(y, mask, 22);
    let mut s = ext_add(&ef(1.0), &y, CW);
    s = maybe(s, mask, 23);
    ext_to_f64(&ext_mul(&xe, &s, CW), CW)
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    n: usize,
    max_ulp: u64,
}
fn score(rows: &[(f64, u64)], a: &[f64; 21], mask: u32) -> Sc {
    let mut s = Sc::default();
    for &(z, pbits) in rows {
        let pg = erf_m77(z, a, mask);
        if !pg.is_finite() {
            continue;
        }
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        s.n += 1;
        if d == 0 {
            s.exact += 1;
        } else {
            s.max_ulp = s.max_ulp.max(d);
        }
    }
    s
}
fn better(a: Sc, b: Sc) -> bool {
    a.exact > b.exact || (a.exact == b.exact && a.max_ulp < b.max_ulp)
}
fn fmt(s: Sc) -> String {
    format!("{}/{} max={}", s.exact, s.n, s.max_ulp)
}

fn load_p(dir: &str) -> Vec<(f64, u64)> {
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
    map.into_values().collect()
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = load_p(&dir);
    let base = score(&rows, &A0, 0);
    println!("M77 ERFCS mask0 {}", fmt(base));

    println!("## one-coeff k=-16..=16");
    let mut best1 = base;
    let mut lab1 = String::from("CR");
    for i in 0..21 {
        for k in -16i32..=16 {
            if k == 0 {
                continue;
            }
            let mut a = A0;
            a[i] = poke(A0[i], k);
            let sc = score(&rows, &a, 0);
            if better(sc, best1) {
                best1 = sc;
                lab1 = format!("a[{i}]{k:+}");
                println!("HIT {lab1} {}", fmt(sc));
            }
        }
    }
    println!("best one-coeff {lab1} {}", fmt(best1));

    println!("## keep-descent |k|<=8");
    let mut a = A0;
    let mut best = base;
    let mut deltas = Vec::new();
    for k in 1..=8 {
        deltas.push(k);
        deltas.push(-k);
    }
    loop {
        let mut moved = false;
        for i in 0..21 {
            for &k in &deltas {
                let old = a[i];
                a[i] = poke(old, k);
                let sc = score(&rows, &a, 0);
                if better(sc, best) {
                    best = sc;
                    moved = true;
                    println!("  keep a[{i}] {k:+} {}", fmt(sc));
                } else {
                    a[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!("joint {}", fmt(best));

    println!("## HW=1");
    let mut besth = base;
    let mut labh = 0u32;
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, &A0, m);
        if better(sc, besth) {
            besth = sc;
            labh = m;
            println!("HIT bit={b} mask={m:#x} {}", fmt(sc));
        }
    }
    println!("best HW1 mask={labh:#x} {}", fmt(besth));
    println!("## HW=2");
    let mut best2 = besth;
    let mut lab2 = labh;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, &A0, m);
            if better(sc, best2) {
                best2 = sc;
                lab2 = m;
                println!("HIT bits={i},{j} mask={m:#x} {}", fmt(sc));
            }
        }
    }
    println!("best HW2 mask={lab2:#x} {}", fmt(best2));
}
