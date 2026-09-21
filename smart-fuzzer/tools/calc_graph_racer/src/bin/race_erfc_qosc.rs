//! Signed-ULP equioscillation of SPECFUN C/D Q vs oracle. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C: [f64; 9] = [
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
const D: [f64; 8] = [
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
fn cody_cd(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn qw(z: f64, ff: f64) -> f64 {
    f::w_rn53(z) * ff
}
fn signed_ulp(got: f64, or: f64) -> Option<i64> {
    let d = ulp_distance(got, or)? as i64;
    if d == 0 {
        Some(0)
    } else if got > or {
        Some(d)
    } else {
        Some(-d)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let mut rows: Vec<_> = f::load_q_rows_tagged(&dir)
        .into_iter()
        .filter(|r| r.z >= 0.5 && r.z < 4.0)
        .collect();
    rows.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    let mut err = Vec::new();
    for r in &rows {
        let qg = qw(r.z, cody_cd(r.z));
        let Some(s) = signed_ulp(qg, f64::from_bits(r.qbits)) else {
            continue;
        };
        err.push((r.z, s, r.direct));
    }
    let mut sgn_changes = 0usize;
    let mut last = 0i64;
    for &(_, s, _) in &err {
        if s == 0 {
            continue;
        }
        if last != 0 && s.signum() != last.signum() {
            sgn_changes += 1;
        }
        last = s;
    }
    // local extrema of |s| among nonzero, requiring sign change
    let nz: Vec<_> = err.iter().filter(|e| e.1 != 0).cloned().collect();
    let mut extrema = 0usize;
    for i in 1..nz.len().saturating_sub(1) {
        let a = nz[i - 1].1.abs();
        let b = nz[i].1.abs();
        let c = nz[i + 1].1.abs();
        if b >= a && b >= c && b >= 1 {
            extrema += 1;
        }
    }
    println!("n={} nonzero={}", err.len(), nz.len());
    println!("sign_changes={sgn_changes} |ulp|-extrema≈{extrema}");

    let bands = [(0.5, 1.0), (1.0, 2.0), (2.0, 3.0), (3.0, 4.0)];
    for (lo, hi) in bands {
        let sl: Vec<_> = err
            .iter()
            .filter(|e| e.0 >= lo && e.0 < hi && e.1 != 0)
            .cloned()
            .collect();
        let mut sc = 0usize;
        let mut last = 0i64;
        for e in &sl {
            if last != 0 && e.1.signum() != last.signum() {
                sc += 1;
            }
            last = e.1;
        }
        let plus = sl.iter().filter(|e| e.1 > 0).count();
        let minus = sl.iter().filter(|e| e.1 < 0).count();
        let maxa = sl.iter().map(|e| e.1.abs()).max().unwrap_or(0);
        println!(
            "[{lo},{hi}) n={} +{} -{} sign_changes={sc} max|ulp|={maxa}",
            sl.len(),
            plus,
            minus
        );
    }

    // run-length of same sign (nonzero)
    let mut runs = Vec::new();
    let mut cur = 0i64;
    let mut len = 0usize;
    for e in &nz {
        let sg = e.1.signum();
        if sg == cur {
            len += 1;
        } else {
            if cur != 0 {
                runs.push(len);
            }
            cur = sg;
            len = 1;
        }
    }
    if cur != 0 {
        runs.push(len);
    }
    runs.sort();
    let nrun = runs.len();
    let med = if nrun == 0 {
        0
    } else {
        runs[nrun / 2]
    };
    println!(
        "sign_runs={} median_len={} min={} max={}",
        nrun,
        med,
        runs.first().copied().unwrap_or(0),
        runs.last().copied().unwrap_or(0)
    );
}
