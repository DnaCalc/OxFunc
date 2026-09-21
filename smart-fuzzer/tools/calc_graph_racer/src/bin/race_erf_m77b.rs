//! MATH77 ERFCS descent from a[1]−4 (max-2 basin) and a0×a1 pair grid.
//! Prefer keeping max≤2. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn erf_m77(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
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
    }
    let y = ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    n: usize,
    max_ulp: u64,
}
fn score(rows: &[(f64, u64)], a: &[f64; 21]) -> Sc {
    let mut s = Sc::default();
    for &(z, pbits) in rows {
        let pg = erf_m77(z, a);
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
fn better_tight(a: Sc, b: Sc) -> bool {
    a.max_ulp < b.max_ulp
        || (a.max_ulp == b.max_ulp && a.exact > b.exact)
}
fn better_exact(a: Sc, b: Sc) -> bool {
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

fn descent(
    rows: &[(f64, u64)],
    mut a: [f64; 21],
    mut best: Sc,
    tight: bool,
) -> ([f64; 21], Sc) {
    let mut deltas = Vec::new();
    for k in 1..=8i32 {
        deltas.push(k);
        deltas.push(-k);
    }
    loop {
        let mut moved = false;
        for i in 0..21 {
            for &k in &deltas {
                let old = a[i];
                a[i] = poke(old, k);
                let sc = score(rows, &a);
                let ok = if tight {
                    better_tight(sc, best)
                } else {
                    better_exact(sc, best)
                };
                if ok {
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
    (a, best)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = load_p(&dir);
    let mut start = A0;
    start[1] = poke(A0[1], -4);
    let s0 = score(&rows, &start);
    println!("start a[1]-4 {}", fmt(s0));

    println!("## pair a[0]×a[1] k=-16..=16");
    let mut bp = s0;
    let mut lab = String::from("a1-4");
    for k0 in -16i32..=16 {
        for k1 in -16i32..=16 {
            if k0 == 0 && k1 == -4 {
                continue;
            }
            let mut a = A0;
            a[0] = poke(A0[0], k0);
            a[1] = poke(A0[1], k1);
            let sc = score(&rows, &a);
            if better_tight(sc, bp) || (sc.max_ulp <= 2 && better_exact(sc, bp)) {
                bp = sc;
                lab = format!("a0{k0:+} a1{k1:+}");
                println!("HIT {lab} {}", fmt(sc));
            }
        }
    }
    println!("best pair {lab} {}", fmt(bp));

    println!("## tight descent (keep max<=2) from a[1]-4");
    let (_, t1) = descent(&rows, start, s0, true);
    println!("tight {}", fmt(t1));

    println!("## exact descent from a[1]-4");
    let (_, t2) = descent(&rows, start, s0, false);
    println!("exact-from-a1-4 {}", fmt(t2));
}
