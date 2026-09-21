//! Joint 3-coeff ±1 of unmask Cody C/D vs DIRECT [0.5,4). Not an identity.
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
fn cody210(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    // no store bit0/1
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if 0x210u32 & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if 0x210u32 & (1u32 << bd) != 0 {
            xden = ef(ext_to_f64(&xden, CW));
        }
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C0[7]), CW),
            &ext_add(&xden, &ef(D0[7]), CW),
            CW,
        ),
        CW,
    )
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
    // overlap C[0]+1 C[7]+1 vs 0x210
    let mut c = C0;
    let mut dd = D0;
    c[0] = poke(C0[0], 1);
    c[7] = poke(C0[7], 1);
    let mut both = 0usize;
    let mut only_p = 0usize;
    let mut only_m = 0usize;
    let mut miss_p: Vec<f64> = Vec::new();
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let ep = ulp_distance(mul(f::w_rn53(r.z), cody(r.z, &c, &dd)), t).unwrap_or(99) == 0;
        let em = ulp_distance(mul(f::w_rn53(r.z), cody210(r.z)), t).unwrap_or(99) == 0;
        match (ep, em) {
            (true, true) => both += 1,
            (true, false) => only_p += 1,
            (false, true) => {
                only_m += 1;
                if miss_p.len() < 8 {
                    miss_p.push(r.z);
                }
            }
            _ => {}
        }
    }
    print!("C[0]+1 C[7]+1 vs 0x210 both={both} only_2c={only_p} only_210={only_m} only_210 z");
    for z in &miss_p {
        print!(" {z:.6}");
    }
    println!();
    let base = score(&C0, &D0);
    let mut best_s = base;
    let mut best_desc = String::from("unmask");
    let mut nbeat = 0usize;
    let mut nge105 = 0usize;
    for i in 0..17usize {
        for j in (i + 1)..17 {
            for k in (j + 1)..17 {
                for &si in &[-1i32, 1] {
                    for &sj in &[-1i32, 1] {
                        for &sk in &[-1i32, 1] {
                            let mut cc = C0;
                            let mut ddd = D0;
                            apply(i, si, &mut cc, &mut ddd);
                            apply(j, sj, &mut cc, &mut ddd);
                            apply(k, sk, &mut cc, &mut ddd);
                            let s = score(&cc, &ddd);
                            if s > base {
                                nbeat += 1;
                            }
                            if s >= 105 {
                                nge105 += 1;
                            }
                            if s > best_s {
                                best_s = s;
                                best_desc = format!(
                                    "{}{si:+} {}{sj:+} {}{sk:+}",
                                    name(i),
                                    name(j),
                                    name(k)
                                );
                                println!("  new best dmid={s} {best_desc}");
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "3-coeff scan best={best_s} {best_desc} beat_unmask={nbeat} ge105={nge105} (bar 105 unmask {base}/{n})"
    );
}
