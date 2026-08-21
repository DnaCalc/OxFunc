//! W109 G6-01 hypothesis (b): expm1 assembled from the fFEXP reduction's OWN
//! pieces (K, w=F2XM1(f)) with NO Kahan / NO ln(u) divide. For |x|<ln2, K=0 and
//! the reciprocal branch gives em = 1/(1+w) - 1 kept extended, one final round.
//! General: em = 2^K*(1+w) - 1 = (2^K - 1) + 2^K*w. Race vs pure em oracle.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e,
    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};

const RN53: u16 = CW_PC53_RN;
const RZ53: u16 = CW_PC53_RN | 0x0C00;

// returns (em_rn, em_rz, em_pieces_rn) assembled from the fFEXP reduction of x
fn assemble(x: f64) -> (u64, u64, u64) {
    let cw = CW_PC64_RN;
    let t = ext_mul(&ext_from_f64(x), &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let kf = ext_to_f64(&k, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw); // 2^|f| - 1
    let onepw = ext_add(&w, &ext_one(), cw); // 2^|f|
    // m = 2^f  (reciprocal on neg branch)
    let m = if neg {
        ext_div(&ext_one(), &onepw, cw)
    } else {
        onepw
    };
    // u = m * 2^K  (= e^x)
    let u = ext_scale(&m, &k, cw);
    // A) straightforward: em = u - 1, extended, one round
    let em_a = ext_sub(&u, &ext_one(), cw);
    // B) pieces: em = (2^K - 1) + 2^K * (m - 1)   [avoids catastrophic 2^K*(1+w)-1]
    //   but for reciprocal branch m-1 = 2^f-1 = 1/(1+w)-1. Use directly:
    let m_m1 = ext_sub(&m, &ext_one(), cw); // 2^f - 1
    let two_k = ext_scale(&ext_one(), &k, cw); // 2^K exact
    let two_k_m1 = ext_sub(&two_k, &ext_one(), cw); // 2^K - 1 exact for integer K
    let em_b = ext_add(&two_k_m1, &ext_mul(&two_k, &m_m1, cw), cw);
    let _ = kf;
    (
        ext_to_f64(&em_a, RN53).to_bits(),
        ext_to_f64(&em_a, RZ53).to_bits(),
        ext_to_f64(&em_b, RN53).to_bits(),
    )
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let (mut a_rn, mut a_rz, mut b_rn) = (0u32, 0u32, 0u32);
    let mut tot = 0u32;
    let mut miss_a: Vec<(i32, u32, i64)> = Vec::new();
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let x = f64::from_bits(u64::from_str_radix(f[2], 16).unwrap());
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let (ea, ez, eb) = assemble(x);
        if ea == pin {
            a_rn += 1;
        } else {
            miss_a.push((k, n, (ea as i64) - (pin as i64)));
        }
        if ez == pin {
            a_rz += 1;
        }
        if eb == pin {
            b_rn += 1;
        }
        tot += 1;
    }
    println!(
        "=== expm1 assembled from fFEXP reduction, pure em oracle N={} ===",
        tot
    );
    println!(
        "  A) u=2^K*m extended, em=u-1, RN53 : {}/{} ({:.1}%)",
        a_rn,
        tot,
        100.0 * a_rn as f64 / tot as f64
    );
    println!(
        "  A') same, RZ53 (chop)            : {}/{} ({:.1}%)",
        a_rz,
        tot,
        100.0 * a_rz as f64 / tot as f64
    );
    println!(
        "  B) pieces (2^K-1)+2^K*(m-1) RN53 : {}/{} ({:.1}%)",
        b_rn,
        tot,
        100.0 * b_rn as f64 / tot as f64
    );
    println!("\n  A misses (k,n,ulp) sample:");
    for m in miss_a.iter().take(20) {
        println!("    k={:3} n={:3} d={:+}", m.0, m.1, m.2);
    }
    let plus = miss_a.iter().filter(|m| m.2 > 0).count();
    let minus = miss_a.iter().filter(|m| m.2 < 0).count();
    println!(
        "  A miss dirs: +{} / -{} (total {})",
        plus,
        minus,
        miss_a.len()
    );
}
