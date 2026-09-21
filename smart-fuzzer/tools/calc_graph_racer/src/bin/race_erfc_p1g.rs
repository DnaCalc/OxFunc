//! Q z<0.5 as 1−P of A21 last-store glue. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
const CW_RD: u16 = CW_PC64_RN | 0x0400;
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
fn one_w(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn glue_p(z: f64) -> Vec<f64> {
    let ja = joint_a();
    let mut v = Vec::new();
    let mut push = |p: f64| {
        if p.is_finite() {
            v.push(p);
        }
    };
    push(a21(z, &ja));
    push(a21(z, &AS0));
    let owj = one_w(z, &ja);
    let owjf = ext_to_f64(&owj, CW);
    let owc = one_w(z, &AS0);
    let owcf = ext_to_f64(&owc, CW);
    for k in [-2i32, -1, 1, 2] {
        push(ext_to_f64(&ext_mul(&ef(poke(z, k)), &owj, CW), CW));
        push(ext_to_f64(&ext_mul(&ef(z), &ef(poke(owjf, k)), CW), CW));
        push(ext_to_f64(&ext_mul(&ef(poke(z, k)), &owc, CW), CW));
        push(ext_to_f64(&ext_mul(&ef(z), &ef(poke(owcf, k)), CW), CW));
    }
    push(ext_to_f64(&ext_mul(&ef(z.next_up()), &owj, CW_RU), CW_RU));
    push(ext_to_f64(
        &ext_mul(&ef(z.next_up()), &ef(owjf.next_up()), CW),
        CW,
    ));
    push(ext_to_f64(&ext_mul(&ef(z.next_down()), &owj, CW_RD), CW_RD));
    push(ext_to_f64(
        &ext_mul(&ef(z.next_down()), &ef(owjf.next_down()), CW),
        CW,
    ));
    push(ext_to_f64(&ext_mul(&ef(z.next_up()), &owc, CW), CW));
    v
}
fn q1(p: f64) -> f64 {
    1.0 - p
}
fn q1x(p: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(p), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let ja = joint_a();
    let mut n = 0usize;
    let mut nd = 0usize;
    let mut h_j = 0usize;
    let mut hd_j = 0usize;
    let mut h_g = 0usize;
    let mut hd_g = 0usize;
    let mut h_x = 0usize;
    let mut hd_x = 0usize;
    let mut miss_d = 0usize;
    println!("Q z<0.5 1−P of A21 last-store glue:");
    for r in rows.iter().filter(|rr| rr.z > 0.0 && rr.z < 0.5) {
        n += 1;
        if r.direct {
            nd += 1;
        }
        let t = f64::from_bits(r.qbits);
        let pj = a21(r.z, &ja);
        let ej = ulp_distance(q1(pj), t).unwrap_or(99) == 0;
        let eg = glue_p(r.z)
            .into_iter()
            .any(|p| ulp_distance(q1(p), t).unwrap_or(99) == 0);
        let ex = glue_p(r.z)
            .into_iter()
            .any(|p| ulp_distance(q1x(p), t).unwrap_or(99) == 0);
        if ej {
            h_j += 1;
            if r.direct {
                hd_j += 1;
            }
        }
        if eg {
            h_g += 1;
            if r.direct {
                hd_g += 1;
            }
        }
        if ex {
            h_x += 1;
            if r.direct {
                hd_x += 1;
            }
        }
        if r.direct && !eg && !ex {
            miss_d += 1;
            if miss_d <= 12 {
                let d = ulp_distance(q1(pj), t).unwrap_or(99);
                println!(
                    "  DIRECT-MISS z={:.16} 1-joint={}{}",
                    r.z,
                    d,
                    if q1(pj) < t { "L" } else { "H" }
                );
            }
        }
    }
    println!(
        "Q n={n} d={nd} 1-joint={h_j} d={hd_j} 1-glue={h_g} d={hd_g} x87-1-glue={h_x} d={hd_x} DIRECT-miss={miss_d} (bar 1-joint 1520/435)"
    );
}
