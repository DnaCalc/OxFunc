//! Joint 2-coeff ±1 of unmask Cody C/D vs DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(c[7]), CW),
            &ext_add(&xden, &ef(d[7]), CW),
            CW,
        ),
        CW,
    )
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
                ulp_distance(mul(f::w_rn53(r.z), cody(r.z, c, d)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let base = score(&C0, &D0);
    println!("unmask fused dmid={base}/{n} (bar 0x210=105)");
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
    best.sort_by(|a, b| b.0.cmp(&a.0));
    println!("top 12 joint 2-coeff ±1:");
    for (s, i, si, j, sj) in best.iter().take(12) {
        println!(
            "  dmid={s} {}{si:+} {}{sj:+} Δ={:+}",
            name(*i),
            name(*j),
            *s as i32 - base as i32
        );
    }
    let nbeat = best.iter().filter(|t| t.0 > base).count();
    let ntie = best.iter().filter(|t| t.0 == base).count();
    println!(
        "pairs*signs={} beat_unmask={nbeat} tie={ntie} best={} (bar 105)",
        best.len(),
        best[0].0
    );
}
