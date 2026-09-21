//! Full NORMSDIST 16495: piecewise published-F last-store + 0.5 last-store + w+4.
//! Not an identity.
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
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
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn one_w(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
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
fn phi_from(z: f64, q: f64) -> f64 {
    if z >= 0.0 {
        1.0 - 0.5 * q
    } else {
        0.5 * q
    }
}
fn hit_phi(z: f64, t: f64, q: f64) -> bool {
    let phi = phi_from(z, q);
    if ulp_distance(phi, t).unwrap_or(99) == 0 {
        return true;
    }
    for k in -5i32..=5 {
        if k == 0 {
            continue;
        }
        if ulp_distance(poke(phi, k), t).unwrap_or(99) == 0 {
            return true;
        }
    }
    false
}
fn hit_q_list(z: f64, t: f64, qs: impl IntoIterator<Item = f64>) -> bool {
    qs.into_iter().any(|q| hit_phi(z, t, q))
}
fn qs_small(a: f64) -> Vec<f64> {
    let ja = joint_a();
    let mut v = Vec::new();
    let mut pushp = |p: f64| v.push(1.0 - p);
    pushp(a21(a, &ja));
    pushp(a21(a, &AS0));
    let owj = one_w(a, &ja);
    let owjf = ext_to_f64(&owj, CW);
    let owc = one_w(a, &AS0);
    let owcf = ext_to_f64(&owc, CW);
    for k in [-2i32, -1, 1, 2] {
        pushp(ext_to_f64(&ext_mul(&ef(poke(a, k)), &owj, CW), CW));
        pushp(ext_to_f64(&ext_mul(&ef(a), &ef(poke(owjf, k)), CW), CW));
        pushp(ext_to_f64(&ext_mul(&ef(poke(a, k)), &owc, CW), CW));
        pushp(ext_to_f64(&ext_mul(&ef(a), &ef(poke(owcf, k)), CW), CW));
    }
    pushp(ext_to_f64(&ext_mul(&ef(a.next_up()), &owc, CW), CW));
    v
}
fn qs_mid(a: f64) -> Vec<f64> {
    let w = f::w_rn53(a);
    let ff = cody0(a);
    let mut v = vec![qmul(w, ff)];
    for k in -4i32..=4 {
        if k != 0 {
            v.push(qmul(w, poke(ff, k)));
        }
    }
    v
}
fn qs_tail(a: f64) -> Vec<f64> {
    let w = f::w_rn53(a);
    let mut v = Vec::new();
    for ff in [f::cephes_f(a), cephes_mask(a, 0)] {
        v.push(qmul(w, ff));
        for k in -5i32..=5 {
            if k != 0 {
                v.push(qmul(w, poke(ff, k)));
            }
        }
    }
    v.push(qmul(poke(w, 4), f::cephes_f(a)));
    v
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut n = [0usize; 3];
    let mut h = [0usize; 3];
    let mut miss = 0usize;
    println!("NORMSDIST full last-store glue:");
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
        let t = e;
        let zf = x * INVSQRT2;
        let zx = ext_to_f64(&ext_mul(&ef(x), &ef(INVSQRT2), CW), CW);
        let mut any = false;
        let mut band = 2usize;
        for z in [zf, zx] {
            let a = z.abs();
            let b = if a < 0.5 {
                0
            } else if a < 4.0 {
                1
            } else {
                2
            };
            band = b;
            let qs = if b == 0 {
                qs_small(a)
            } else if b == 1 {
                qs_mid(a)
            } else {
                qs_tail(a)
            };
            if hit_q_list(z, t, qs) {
                any = true;
                break;
            }
        }
        n[band] += 1;
        if any {
            h[band] += 1;
        } else {
            miss += 1;
            if miss <= 8 {
                println!("  MISS x={x:.16} z={zf:.16} a={:.16}", zf.abs());
            }
        }
    }
    let nt = n[0] + n[1] + n[2];
    let ht = h[0] + h[1] + h[2];
    println!(
        "hit={ht}/{nt}  |z|<0.5 {}/{}  [0.5,4) {}/{}  |z|>=4 {}/{} miss={miss}",
        h[0], n[0], h[1], n[1], h[2], n[2]
    );
}
