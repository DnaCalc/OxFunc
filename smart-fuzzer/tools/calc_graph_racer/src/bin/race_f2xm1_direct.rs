//! W109 G6-01 (workflow structure lane): PMT's em = (1+r)^-n - 1 computed by the
//! x87 F2XM1 hardware instruction DIRECTLY (2^y - 1), y = -n*log2(1+r), NOT via
//! exp/ln Kahan. 2^(-n*log2(1+r)) - 1 = (1+r)^-n - 1 exactly; F2XM1 avoids the
//! cancellation. Real hardware F2XM1/FYL2XP1 (not Python emul). Excel-exact oracle.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_f2xm1, ext_from_f64, ext_fyl2xp1, ext_mul, ext_one,
    ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
const RN53: u16 = CW_PC53_RN;
const CW: u16 = CW_PC64_RN;

// log2(1+r) via FYL2XP1(1, r), extended
fn log2_1p(r: f64) -> Ext80 {
    ext_fyl2xp1(&ext_one(), &ext_from_f64(r), CW)
}

// em = 2^y - 1 for |y|<1 via F2XM1; for |y|>=1 reduce y = K + g, |g|<=0.5,
// em = 2^K*(1+F2XM1(g)) - 1.
fn f2xm1_full(y: &Ext80) -> Ext80 {
    let yv = ext_to_f64(y, CW);
    if yv.abs() < 1.0 {
        ext_f2xm1(y, CW)
    } else {
        let k = ext_rndint(y, CW);
        let g = ext_sub(y, &k, CW);
        let w = ext_f2xm1(&g, CW); // 2^g - 1
        let onepw = ext_add(&w, &ext_one(), CW);
        let scaled = ext_scale(&onepw, &k, CW); // 2^K * 2^g = 2^y
        ext_sub(&scaled, &ext_one(), CW)
    }
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let labels = [
        "Y1 y ext (fyl2xp1*-n), F2XM1, spill",
        "Y2 y=-n*fl(log2_1p) spilled, F2XM1(ext y), spill",
        "Y3 y ext, F2XM1, store PC64->f64",
        "Y4 y=(-n)*log2_1p all ext, direct only |y|<1",
    ];
    let mut score = [0u32; 4];
    let mut score_yl1 = [0u32; 4];
    let (mut tot, mut tot_yl1) = (0u32, 0u32);
    let mut miss1: Vec<(i32, u32, i64)> = Vec::new();
    for line in csv.lines().skip(1) {
        let ff: Vec<&str> = line.split(',').collect();
        if ff.len() < 6 {
            continue;
        }
        let k: i32 = ff[0].parse().unwrap();
        let n: u32 = ff[1].parse().unwrap();
        let r = 2f64.powi(k);
        let pin = u64::from_str_radix(ff[5], 16).unwrap();
        let l2 = log2_1p(r);
        // y1: y = -n * log2(1+r), all extended
        let y_ext = ext_mul(&ext_from_f64(-(n as f64)), &l2, CW);
        let yl1 = ext_to_f64(&y_ext, CW).abs() < 1.0;
        // Y1: em = F2XM1_full(y_ext), spill RN53
        let y1 = ext_to_f64(&f2xm1_full(&y_ext), RN53).to_bits();
        // Y2: y spilled to double first
        let y_sp = ext_to_f64(&y_ext, RN53);
        let y2 = ext_to_f64(&f2xm1_full(&ext_from_f64(y_sp)), RN53).to_bits();
        // Y3: like Y1 but final store at PC64 cw (still to f64 = RN53 effectively)
        let y3 = ext_to_f64(&f2xm1_full(&y_ext), CW).to_bits();
        // Y4: log2(1+r) spilled to double, *-n spilled, then F2XM1(ext) spill
        let l2sp = ext_to_f64(&l2, RN53);
        let y_sp2 = -(n as f64) * l2sp; // double product
        let y4 = ext_to_f64(&f2xm1_full(&ext_from_f64(y_sp2)), RN53).to_bits();

        let cands = [y1, y2, y3, y4];
        for i in 0..4 {
            if cands[i] == pin {
                score[i] += 1;
                if yl1 {
                    score_yl1[i] += 1;
                }
            }
        }
        if yl1 {
            tot_yl1 += 1;
        }
        if cands[0] != pin {
            miss1.push((k, n, (cands[0] as i64) - (pin as i64)));
        }
        tot += 1;
    }
    println!(
        "=== em via HARDWARE F2XM1 direct, N={} (|y|<1 subset={}) ===",
        tot, tot_yl1
    );
    for i in 0..4 {
        println!(
            "  {:44} all {:3}/{} ({:4.1}%)  |y|<1 {:3}/{} ({:4.1}%)",
            labels[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64,
            score_yl1[i],
            tot_yl1,
            100.0 * score_yl1[i] as f64 / tot_yl1 as f64
        );
    }
    println!(
        "\nY1 misses: {} ; +{}/-{}",
        miss1.len(),
        miss1.iter().filter(|m| m.2 > 0).count(),
        miss1.iter().filter(|m| m.2 < 0).count()
    );
    for m in miss1.iter().take(12) {
        println!("   k={:3} n={:3} d={:+}", m.0, m.1, m.2);
    }
}
