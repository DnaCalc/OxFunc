//! Exhaustive 3-bit Cody Horner stores vs 0x210 DIRECT [0.5,4). Not an identity.
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if mask & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if mask & (1u32 << bd) != 0 {
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
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn label(bit: u32) -> String {
    let i = (bit - 2) / 2;
    let side = if bit % 2 == 0 { "xnum" } else { "xden" };
    format!("i={i}{side}")
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let n = dirs.len();
    let score = |mask: u32| -> usize {
        dirs.iter()
            .filter(|r| {
                let t = f64::from_bits(r.qbits);
                ulp_distance(mul(f::w_rn53(r.z), cody_mask(r.z, mask)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let bits: Vec<u32> = (2..16).collect();
    let mut ranked: Vec<(usize, u32)> = Vec::new();
    for i in 0..bits.len() {
        for j in (i + 1)..bits.len() {
            for k in (j + 1)..bits.len() {
                let mask = (1u32 << bits[i]) | (1u32 << bits[j]) | (1u32 << bits[k]);
                ranked.push((score(mask), mask));
            }
        }
    }
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    println!("3-bit triples={} 0x210={} n={n}", ranked.len(), score(0x210));
    println!("top 8:");
    for (s, mask) in ranked.iter().take(8) {
        print!("  fused={s} mask={mask:#x}");
        for b in 2..16u32 {
            if mask & (1 << b) != 0 {
                print!(" {}", label(b));
            }
        }
        println!();
    }
    let nge105 = ranked.iter().filter(|t| t.0 >= 105).count();
    let ngt105 = ranked.iter().filter(|t| t.0 > 105).count();
    println!(
        "best={} mask={:#x} ge105={nge105} gt105={ngt105}",
        ranked[0].0, ranked[0].1
    );
}
