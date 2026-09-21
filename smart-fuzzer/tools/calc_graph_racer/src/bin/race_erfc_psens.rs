//! 0x5005 P/Q/R/S ±1 DIRECT sensitivity. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn polevl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
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
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], r: &[f64; 6], s: &[f64; 6]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let mask = 0x5005u32;
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (qq, b) = p1evl(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, r, mask, 0);
        let (ss, b) = p1evl(xe, s, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 4.0)
        .collect();
    let base_hit: Vec<bool> = dirs
        .iter()
        .map(|r| {
            ulp_distance(
                qwf(r.z, cephes(r.z, &CEPHES_P, &CEPHES_Q, &CEPHES_R, &CEPHES_S)),
                f64::from_bits(r.qbits),
            )
            .unwrap_or(99)
                == 0
        })
        .collect();
    let bcount = base_hit.iter().filter(|&&x| x).count();
    println!("base 0x5005 DIRECT {bcount}/{}", dirs.len());
    let run = |p: &[f64; 9], q: &[f64; 8], r: &[f64; 6], s: &[f64; 6], lab: &str| {
        let mut n = 0usize;
        let mut gain: Vec<f64> = Vec::new();
        let mut lose: Vec<f64> = Vec::new();
        for (i, row) in dirs.iter().enumerate() {
            let hit = ulp_distance(
                qwf(row.z, cephes(row.z, p, q, r, s)),
                f64::from_bits(row.qbits),
            )
            .unwrap_or(99)
                == 0;
            if hit {
                n += 1;
            }
            if hit && !base_hit[i] {
                gain.push(row.z);
            }
            if !hit && base_hit[i] {
                lose.push(row.z);
            }
        }
        let delta = n as i32 - bcount as i32;
        if delta != 0 {
            print!("  {lab:10} d={n} Δ={delta:+}");
            for z in gain.iter().take(4) {
                print!(" +{z:.4}");
            }
            for z in lose.iter().take(4) {
                print!(" -{z:.4}");
            }
            println!();
        }
    };
    for i in 0..9 {
        for k in [-1i32, 1] {
            let mut p = CEPHES_P;
            p[i] = poke(CEPHES_P[i], k);
            run(&p, &CEPHES_Q, &CEPHES_R, &CEPHES_S, &format!("P[{i}] {k:+}"));
        }
    }
    for i in 0..8 {
        for k in [-1i32, 1] {
            let mut q = CEPHES_Q;
            q[i] = poke(CEPHES_Q[i], k);
            run(&CEPHES_P, &q, &CEPHES_R, &CEPHES_S, &format!("Q[{i}] {k:+}"));
        }
    }
    for i in 0..6 {
        for k in [-1i32, 1] {
            let mut r = CEPHES_R;
            r[i] = poke(CEPHES_R[i], k);
            run(&CEPHES_P, &CEPHES_Q, &r, &CEPHES_S, &format!("R[{i}] {k:+}"));
        }
        for k in [-1i32, 1] {
            let mut s = CEPHES_S;
            s[i] = poke(CEPHES_S[i], k);
            run(&CEPHES_P, &CEPHES_Q, &CEPHES_R, &s, &format!("S[{i}] {k:+}"));
        }
    }
}
