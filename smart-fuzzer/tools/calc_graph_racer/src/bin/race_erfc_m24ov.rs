//! 0x24a5 vs 0x5005 Q / F_or DIRECT overlap and XBIG trio. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

struct Ov {
    n: usize,
    both: usize,
    only_a: usize,
    only_b: usize,
    only_a_u1: usize,
    only_b_u1: usize,
}
impl Ov {
    fn new() -> Self {
        Self {
            n: 0,
            both: 0,
            only_a: 0,
            only_b: 0,
            only_a_u1: 0,
            only_b_u1: 0,
        }
    }
    fn add(&mut self, da: u64, db: u64) {
        self.n += 1;
        match (da == 0, db == 0) {
            (true, true) => self.both += 1,
            (true, false) => {
                self.only_a += 1;
                if db == 1 {
                    self.only_a_u1 += 1;
                }
            }
            (false, true) => {
                self.only_b += 1;
                if da == 1 {
                    self.only_b_u1 += 1;
                }
            }
            _ => {}
        }
    }
    fn union(&self) -> usize {
        self.both + self.only_a + self.only_b
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    const MA: u32 = 0x24a5;
    const MB: u32 = 0x5005;
    const MINT: u32 = 0x5;
    const MUNI: u32 = 0x74a5;

    println!("masks 0x24a5 bits 0,2,5,9,13  0x5005 bits 0,2,12,14  ∩=0x5 ∪=0x74a5");
    for (lab, m) in [
        ("0", 0u32),
        ("0x5", MINT),
        ("0x24a5", MA),
        ("0x5005", MB),
        ("0x74a5", MUNI),
    ] {
        let mut qe = 0usize;
        let mut qd = 0usize;
        let mut qn = 0usize;
        let mut qmx = 0u64;
        let mut fe = 0usize;
        let mut fd = 0usize;
        let mut fn_ = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let ff = cephes_mask(r.z, m);
            let dq = ulp_distance(qw(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if dq <= ULP_CAP {
                qn += 1;
                if dq == 0 {
                    qe += 1;
                    if r.direct {
                        qd += 1;
                    }
                } else {
                    qmx = qmx.max(dq);
                }
            }
            if let Some(fo) = f::f_or(r.z, r.qbits) {
                fn_ += 1;
                if ulp_distance(ff, fo).unwrap_or(99) == 0 {
                    fe += 1;
                    if r.direct {
                        fd += 1;
                    }
                }
            }
        }
        println!("  {lab:8} Q {qe}/{qn} d={qd} max={qmx}  F_or {fe}/{fn_} d={fd}");
    }

    let mut q_all = Ov::new();
    let mut q_dir = Ov::new();
    let mut q_48 = Ov::new();
    let mut q_8p = Ov::new();
    let mut f_all = Ov::new();
    let mut f_dir = Ov::new();
    let mut only_a_rows: Vec<(f64, bool, u64, u64, bool)> = Vec::new();
    let mut only_b_rows: Vec<(f64, bool, u64, u64, bool)> = Vec::new();
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let fa = cephes_mask(r.z, MA);
        let fb = cephes_mask(r.z, MB);
        let da = ulp_distance(qw(r.z, fa), t).unwrap_or(99);
        let db = ulp_distance(qw(r.z, fb), t).unwrap_or(99);
        q_all.add(da, db);
        if r.direct {
            q_dir.add(da, db);
            if r.z < 8.0 {
                q_48.add(da, db);
            } else {
                q_8p.add(da, db);
            }
            if da == 0 && db != 0 {
                only_a_rows.push((r.z, true, da, db, qw(r.z, fa) < t));
            }
            if db == 0 && da != 0 {
                only_b_rows.push((r.z, true, da, db, qw(r.z, fb) < t));
            }
        }
        if let Some(fo) = f::f_or(r.z, r.qbits) {
            let dfa = ulp_distance(fa, fo).unwrap_or(99);
            let dfb = ulp_distance(fb, fo).unwrap_or(99);
            f_all.add(dfa, dfb);
            if r.direct {
                f_dir.add(dfa, dfb);
            }
        }
    }
    let pr = |tag: &str, o: &Ov| {
        println!(
            "{tag} both={} only_24a5={} (u1={}) only_5005={} (u1={}) union={} n={}",
            o.both,
            o.only_a,
            o.only_a_u1,
            o.only_b,
            o.only_b_u1,
            o.union(),
            o.n
        );
    };
    pr("Q all", &q_all);
    pr("Q DIRECT", &q_dir);
    pr("Q DIRECT [4,8)", &q_48);
    pr("Q DIRECT [8,inf)", &q_8p);
    pr("F_or all", &f_all);
    pr("F_or DIRECT", &f_dir);

    println!("only_0x24a5 DIRECT n={}", only_a_rows.len());
    for (z, _, _, db, low) in &only_a_rows {
        println!(
            "  z={z:.16} 5005_ulp={db} 24a5 {} excel",
            if *low { "<" } else { ">" }
        );
    }
    println!("only_0x5005 DIRECT n={} (first 12)", only_b_rows.len());
    for (z, _, da, _, low) in only_b_rows.iter().take(12) {
        println!(
            "  z={z:.16} 24a5_ulp={da} 5005 {} excel",
            if *low { "<" } else { ">" }
        );
    }

    println!("0x5005 leftover-low hard (ulp>=2, graph<excel) vs 0x24a5:");
    for r in &rows {
        if !r.direct || r.z < 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let gb = qw(r.z, cephes_mask(r.z, MB));
        let db = ulp_distance(gb, t).unwrap_or(99);
        if db < 2 || db > ULP_CAP || gb >= t {
            continue;
        }
        let ga = qw(r.z, cephes_mask(r.z, MA));
        let da = ulp_distance(ga, t).unwrap_or(99);
        let d0 = ulp_distance(qw(r.z, cephes_mask(r.z, 0)), t).unwrap_or(99);
        let dc = ulp_distance(qw(r.z, f::cephes_f(r.z)), t).unwrap_or(99);
        let dl = ulp_distance(libm::erfc(r.z), t).unwrap_or(99);
        println!(
            "  z={:.16} 5005={db} 24a5={da} mask0={d0} cephes_f={dc} libm={dl} 24a5_vs={}",
            r.z,
            if ga < t { "low" } else { "high" }
        );
    }

    println!("XBIG trio + neighbors:");
    for r in &rows {
        if r.z < 26.0 || r.z > 27.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let da = ulp_distance(qw(r.z, cephes_mask(r.z, MA)), t).unwrap_or(99);
        let db = ulp_distance(qw(r.z, cephes_mask(r.z, MB)), t).unwrap_or(99);
        let d0 = ulp_distance(qw(r.z, cephes_mask(r.z, 0)), t).unwrap_or(99);
        let dc = ulp_distance(qw(r.z, f::cephes_f(r.z)), t).unwrap_or(99);
        let dl = ulp_distance(libm::erfc(r.z), t).unwrap_or(99);
        let dz = ulp_distance(0.0, t).unwrap_or(99);
        println!(
            "  z={:.16} dir={} excel_bits={:#x} 24a5={da} 5005={db} mask0={d0} cephes_f={dc} libm={dl} flush0={dz}",
            r.z,
            r.direct,
            r.qbits
        );
    }

    println!("piecewise 5005-then-24a5 / 24a5-then-5005 Q-tail cuts:");
    for (lo_m, hi_m, tag) in [(MB, MA, "5005-then-24a5"), (MA, MB, "24a5-then-5005")] {
        let mut best_ex = 0usize;
        let mut best_d = 0usize;
        let mut best_c = 0.0;
        for k in 0..80 {
            let c = 4.0 + k as f64 * 0.1;
            let mut ex = 0usize;
            let mut dd = 0usize;
            for r in &rows {
                if r.z < 4.0 {
                    continue;
                }
                let m = if r.z < c { lo_m } else { hi_m };
                let dq = ulp_distance(qw(r.z, cephes_mask(r.z, m)), f64::from_bits(r.qbits))
                    .unwrap_or(u64::MAX);
                if dq == 0 {
                    ex += 1;
                    if r.direct {
                        dd += 1;
                    }
                }
            }
            if ex > best_ex || (ex == best_ex && dd > best_d) {
                best_ex = ex;
                best_d = dd;
                best_c = c;
            }
        }
        println!("  {tag} best Q {best_ex} d={best_d} @ {best_c:.1} (bars 1572/53, 1507/60)");
    }
}
