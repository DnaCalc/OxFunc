//! Joint on the 3074 graph + named small-piece skeletons (fdlibm PA/QA,
//! Cephes T/U, fdlibm PP/QQ, NSWC erfc1 A/B). Not an identity. No landing.
//!
//!   cargo run --release --bin race_erfc_3074 -- G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;
use std::io::Write;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const CUT3074: f64 = 1.347;

const AS: [f64; 21] = [
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
const C0: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
];
const FDLIBM_PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const FDLIBM_PA: [f64; 7] = [
    -2.36211856075265944077e-03,
    4.14856118683748331666e-01,
    -3.72207876035701323847e-01,
    3.18346619901161753674e-01,
    -1.10894694282396677476e-01,
    3.54783043256182359371e-02,
    -2.16637559486879084300e-03,
];
const FDLIBM_QA: [f64; 6] = [
    1.06420880400844228286e-01,
    5.40397917702171048937e-01,
    7.18286544141962662868e-02,
    1.26171219808761642112e-01,
    1.36370839120290507362e-02,
    1.19844998467991074170e-02,
];
const ERX: f64 = 8.45062911510467529297e-01;
const NSWC_ERFC1_A: [f64; 5] = [
    0.771058495001320e-04,
    -0.133733772997339e-02,
    0.323076579225834e-01,
    0.479137145607681e-01,
    0.128379167095513e+00,
];
const NSWC_ERFC1_B: [f64; 3] = [
    0.301048631703895e-02,
    0.538971687740286e-01,
    0.375795757275549e+00,
];

fn x87_horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn x87_horner_lo(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(cs[0]);
    for &c in &cs[1..] {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn horner_lo(cs: &[f64], x: f64) -> f64 {
    let mut a = cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
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

fn small_nswc(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner_hi(&AS, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &ww, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    ) / w
}

fn cody_cd(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(c[7]), CW),
            &ext_add(&xden, &ext_from_f64(d[7]), CW),
            CW,
        ),
        CW,
    )
}

fn polevl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn p1evl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = x + cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn cephes_tu_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let zz = z * z;
    let erf = z * polevl_hi(&CEPHES_T, zz) / p1evl_hi(&CEPHES_U, zz);
    (1.0 - erf) / w
}

fn fdlibm_pp_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let zz = z * z;
    let r = horner_lo(&FDLIBM_PP, zz);
    let s = 1.0 + zz * horner_lo(&FDLIBM_QQ, zz);
    let y = r / s;
    let erfc = 0.5 - (z * y + (z - 0.5));
    erfc / w
}

fn fdlibm_pa_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let s = z - 1.0;
    let p = horner_lo(&FDLIBM_PA, s);
    let q = 1.0 + s * horner_lo(&FDLIBM_QA, s);
    let erfc = (1.0 - ERX) - p / q;
    erfc / w
}

fn fdlibm_pa_x87(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let se = ext_sub(&ext_from_f64(z), &ext_from_f64(1.0), CW);
    let p = x87_horner_lo(&FDLIBM_PA, se);
    let q = ext_add(
        &ext_from_f64(1.0),
        &ext_mul(&se, &x87_horner_lo(&FDLIBM_QA, se), CW),
        CW,
    );
    let erfc = ext_sub(
        &ext_from_f64(1.0 - ERX),
        &ext_div(&p, &q, CW),
        CW,
    );
    ext_to_f64(&erfc, CW) / w
}

fn nswc_erfc1_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let t = z * z;
    let top = horner_lo(&NSWC_ERFC1_A, t) + 1.0;
    let bot = 1.0 + t * horner_lo(&NSWC_ERFC1_B, t);
    let erf = z * top / bot;
    (0.5 + (0.5 - erf)) / w
}

#[derive(Clone, Copy, Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
    dmid: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!(
        "{}/{} max={} sum={} dmid={}",
        a.exact, a.n, a.max_ulp, a.sum_ulp, a.dmid
    )
}

fn score_mid(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> Acc {
    let mut a = Acc::default();
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        a.add(d, r.direct);
    }
    a
}

fn lex_better(a: &Acc, b: &Acc) -> bool {
    a.exact > b.exact || (a.exact == b.exact && a.sum_ulp < b.sum_ulp)
}

fn joint(rows: &[f::QRow], mut c: [f64; 9], mut d: [f64; 8], cut: f64) -> ([f64; 9], [f64; 8], Acc) {
    let score_now = |c: &[f64; 9], d: &[f64; 8]| {
        score_mid(rows, |z| {
            if z < cut {
                small_nswc(z)
            } else {
                cody_cd(z, c, d)
            }
        })
    };
    let mut best = score_now(&c, &d);
    let deltas = [1i32, -1, 2, -2, 4, -4];
    loop {
        let mut moved = false;
        for i in 0..9 {
            for &k in &deltas {
                let old = c[i];
                c[i] = poke(old, k);
                let sc = score_now(&c, &d);
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                    println!("  keep C[{i}] {k:+} mid {}", fmt(&sc));
                } else {
                    c[i] = old;
                }
            }
        }
        for i in 0..8 {
            for &k in &deltas {
                let old = d[i];
                d[i] = poke(old, k);
                let sc = score_now(&c, &d);
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                    println!("  keep D[{i}] {k:+} mid {}", fmt(&sc));
                } else {
                    d[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    (c, d, best)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let out = env::args()
        .nth(2)
        .unwrap_or_else(|| ".".into());
    let rows = f::load_q_rows_tagged(&dir);

    println!("## named small-piece + Cody C/D");
    let mut c_cr = C0;
    let mut d_cr = D0;
    for (name, cut, sm) in [
        (
            "NSWC-small cut=1 CR Cody",
            1.0,
            small_nswc as fn(f64) -> f64,
        ),
        ("NSWC-small cut=1.347 CR Cody", 1.347, small_nswc),
        ("Cephes T/U cut=1 CR Cody", 1.0, cephes_tu_f),
        ("Cephes T/U cut=1.347 CR Cody", 1.347, cephes_tu_f),
        ("fdlibm PP/QQ cut=0.84375 CR Cody", 0.84375, fdlibm_pp_f),
        ("fdlibm PP/QQ cut=1 CR Cody", 1.0, fdlibm_pp_f),
        ("fdlibm PA/QA native cut=1.25 CR Cody", 1.25, fdlibm_pa_f),
        ("fdlibm PA/QA x87 cut=1.25 CR Cody", 1.25, fdlibm_pa_x87),
        ("fdlibm PA/QA x87 cut=1.347 CR Cody", 1.347, fdlibm_pa_x87),
        ("NSWC erfc1 A/B cut=1 CR Cody", 1.0, nswc_erfc1_f),
        ("NSWC erfc1 A/B cut=0.5 CR Cody", 0.5, nswc_erfc1_f),
    ] {
        let sc = score_mid(&rows, |z| {
            if z < cut {
                sm(z)
            } else {
                cody_cd(z, &c_cr, &d_cr)
            }
        });
        println!("{name:44} {}", fmt(&sc));
    }

    let mut c_h = C0;
    let mut d_h = D0;
    c_h[6] = c_h[6].next_up();
    d_h[4] = d_h[4].next_up();
    let base3074 = score_mid(&rows, |z| {
        if z < CUT3074 {
            small_nswc(z)
        } else {
            cody_cd(z, &c_h, &d_h)
        }
    });
    println!("\n## joint from 3074 graph cut=1.347 C6+1 D4+1");
    println!("base {}", fmt(&base3074));
    let (cj, dj, best) = joint(&rows, c_h, d_h, CUT3074);
    println!("joint-from-3074 {}", fmt(&best));
    for i in 0..9 {
        if cj[i].to_bits() != C0[i].to_bits() {
            println!("  C[{i}] bits {:016x} CR {:016x}", cj[i].to_bits(), C0[i].to_bits());
        }
    }
    for i in 0..8 {
        if dj[i].to_bits() != D0[i].to_bits() {
            println!("  D[{i}] bits {:016x} CR {:016x}", dj[i].to_bits(), D0[i].to_bits());
        }
    }

    println!("\n## joint from CR at documented cut=1 (control)");
    let (_c1, _d1, best1) = joint(&rows, c_cr, d_cr, 1.0);
    println!("joint-from-CR-cut1 {}", fmt(&best1));

    let path = format!("{out}/JOINT_3074.md");
    let mut w = fs::File::create(&path).unwrap();
    writeln!(
        w,
        "# Joint on 3074 graph\n\nNot an identity. Not landed.\n\nbase 3074 {}\njoint-from-3074 {}\njoint-from-CR-cut1 {}\n",
        fmt(&base3074),
        fmt(&best),
        fmt(&best1)
    )
    .ok();
    println!("wrote {path}");
}
