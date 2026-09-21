//! 0x210 Cody Horner + C/D coeff ±1 DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn apply(idx: usize, k: i32, c: &mut [f64; 9], d: &mut [f64; 8]) {
    if idx < 9 {
        c[idx] = poke(C0[idx], k);
    } else {
        d[idx - 9] = poke(D0[idx - 9], k);
    }
}
fn name(idx: usize) -> String {
    if idx < 9 {
        format!("C[{idx}]")
    } else {
        format!("D[{}]", idx - 9)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let n = dirs.len();
    let score = |c: &[f64; 9], d: &[f64; 8]| -> usize {
        dirs.iter()
            .filter(|r| {
                let t = f64::from_bits(r.qbits);
                ulp_distance(mul(f::w_rn53(r.z), cody(r.z, c, d, MASK)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let base = score(&C0, &D0);
    println!("0x210 fused DIRECT [0.5,4)={base}/{n} (unmask 3-coeff bar 107)");
    println!("1-coeff C/D ±1 on 0x210:");
    let mut best1 = base;
    for i in 0..17usize {
        for &si in &[-1i32, 1] {
            let mut c = C0;
            let mut d = D0;
            apply(i, si, &mut c, &mut d);
            let s = score(&c, &d);
            let delta = s as i32 - base as i32;
            if delta != 0 {
                println!("  {}{si:+} fused={s} Δ={delta:+}", name(i));
            }
            if s > best1 {
                best1 = s;
            }
        }
    }
    println!("best 1-coeff={best1}");
    let mut best = Vec::new();
    for i in 0..17usize {
        for j in (i + 1)..17 {
            for &si in &[-1i32, 1] {
                for &sj in &[-1i32, 1] {
                    let mut c = C0;
                    let mut d = D0;
                    apply(i, si, &mut c, &mut d);
                    apply(j, sj, &mut c, &mut d);
                    let s = score(&c, &d);
                    best.push((s, i, si, j, sj));
                }
            }
        }
    }
    best.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("top 8 C/D 2-coeff ±1 on 0x210:");
    for (s, i, si, j, sj) in best.iter().take(8) {
        println!(
            "  fused={s} {}{si:+} {}{sj:+} Δ={:+}",
            name(*i),
            name(*j),
            *s as i32 - base as i32
        );
    }
    let nbeat = best.iter().filter(|t| t.0 > base).count();
    let ngt107 = best.iter().filter(|t| t.0 > 107).count();
    let nge107 = best.iter().filter(|t| t.0 >= 107).count();
    println!(
        "2-coeff on 0x210 n={} beat105={nbeat} best={} ge107={nge107} gt107={ngt107}",
        best.len(),
        best[0].0
    );
}
