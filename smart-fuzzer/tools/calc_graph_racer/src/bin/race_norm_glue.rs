//! NORM.S.DIST vs piecewise published-F last-store glue. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;
const AS0: [f64; 21] = [
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
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}
fn a21(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody0(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hit_q(a: f64, tq: f64) -> bool {
    if a < 0.5 {
        let ja = joint_a();
        let p = a21(a, &ja);
        if ulp_distance(1.0 - p, tq).unwrap_or(99) == 0 {
            return true;
        }
        let p0 = a21(a, &AS0);
        if ulp_distance(1.0 - p0, tq).unwrap_or(99) == 0 {
            return true;
        }
        let xe = ef(a);
        let tt = ext_mul(&xe, &xe, CW);
        let mut acc = ef(0.0);
        for &c in AS0.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &tt, CW), &ef(c), CW);
        }
        let ow = ext_add(&ef(1.0), &acc, CW);
        let owf = ext_to_f64(&ow, CW);
        for k in [-2i32, -1, 1, 2] {
            let p1 = ext_to_f64(&ext_mul(&ef(poke(a, k)), &ow, CW), CW);
            if ulp_distance(1.0 - p1, tq).unwrap_or(99) == 0 {
                return true;
            }
            let p2 = ext_to_f64(&ext_mul(&ef(a), &ef(poke(owf, k)), CW), CW);
            if ulp_distance(1.0 - p2, tq).unwrap_or(99) == 0 {
                return true;
            }
        }
        let pz = ext_to_f64(&ext_mul(&ef(a.next_up()), &ow, CW), CW);
        ulp_distance(1.0 - pz, tq).unwrap_or(99) == 0
    } else if a < 4.0 {
        let w = f::w_rn53(a);
        let ff = cody0(a);
        if ulp_distance(qmul(w, ff), tq).unwrap_or(99) == 0 {
            return true;
        }
        (-4i32..=4)
            .filter(|&k| k != 0)
            .any(|k| ulp_distance(qmul(w, poke(ff, k)), tq).unwrap_or(99) == 0)
    } else {
        let w = f::w_rn53(a);
        let ff = f::cephes_f(a);
        if ulp_distance(qmul(w, ff), tq).unwrap_or(99) == 0 {
            return true;
        }
        (-5i32..=5)
            .filter(|&k| k != 0)
            .any(|k| ulp_distance(qmul(w, poke(ff, k)), tq).unwrap_or(99) == 0)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut rows = Vec::new();
    for w in &bank.witnesses {
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let cum = match &w.args[1] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        if cum != 1.0 {
            continue;
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        rows.push((x, e.to_bits()));
    }
    let mut n = 0usize;
    let mut hit = 0usize;
    let mut n1 = 0usize;
    let mut h1 = 0usize;
    let mut n2 = 0usize;
    let mut h2 = 0usize;
    let mut n3 = 0usize;
    let mut h3 = 0usize;
    let mut miss = 0usize;
    println!("NORM.S.DIST last-store glue n={}", rows.len());
    for &(x, bits) in &rows {
        let t = f64::from_bits(bits);
        let z = x * INVSQRT2;
        let a = z.abs();
        n += 1;
        let tq = if z >= 0.0 { 2.0 * (1.0 - t) } else { 2.0 * t };
        let any = hit_q(a, tq);
        if a < 0.5 {
            n1 += 1;
            if any {
                h1 += 1;
            }
        } else if a < 4.0 {
            n2 += 1;
            if any {
                h2 += 1;
            }
        } else {
            n3 += 1;
            if any {
                h3 += 1;
            }
        }
        if any {
            hit += 1;
        } else {
            miss += 1;
            if a < 0.5 || miss <= 8 {
                println!("  MISS x={x:.16} z={z:.16} a={a:.16}");
            }
        }
    }
    println!(
        "hit={hit}/{n}  |z|<0.5 {h1}/{n1}  [0.5,4) {h2}/{n2}  |z|>=4 {h3}/{n3} miss={miss}"
    );
}
