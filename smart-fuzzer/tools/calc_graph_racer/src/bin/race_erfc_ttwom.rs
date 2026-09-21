//! unmask two-mode of w×F on tail leftover DIRECT z≥4. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const P0: [f64; 9] = [
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
const Q0: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const R0: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const S0: [f64; 6] = [
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
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (qq, b) = p1evl(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, mask, 0);
        let (ss, b) = p1evl(xe, &S0, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hit(g: f64, t: f64) -> bool {
    ulp_distance(g, t).unwrap_or(99) == 0
}
fn is555(z: f64) -> bool {
    let h = format!("{:x}", z.to_bits());
    h.contains("55555555") || h.contains("aaaaaaaa")
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    let graphs: [(&str, u32, [f64; 9], [f64; 8]); 4] = [
        ("0x5005", 0x5005, P0, Q0),
        ("0x4005", 0x4005, P0, Q0),
        ("0x4e05", 0x4e05, P0, Q0),
        ("66", 0x24a5, p66, q66),
    ];
    println!("unmask two-mode on tail leftover DIRECT z>=4:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cephes(z, &P0, &Q0, 0);
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        if tmu || tmd {
            let up = hit(mul(w, fu).next_up(), t);
            let dn = hit(mul(w, fu).next_down(), t);
            let w1u = hit(mul(w.next_up(), fu), t);
            let f1u = hit(mul(w, fu.next_up()), t);
            let w1d = hit(mul(w.next_down(), fu), t);
            let f1d = hit(mul(w, fu.next_down()), t);
            println!(
                "  z={:.16} 555={} tmu={tmu} tmd={tmd} last-mul_up={up} last-mul_dn={dn} w+1={w1u} F+1={f1u} w-1={w1d} F-1={f1d}",
                z,
                is555(z)
            );
        }
    }
    for (name, mask, p, q) in &graphs {
        let mut nlo = 0usize;
        let mut nhi = 0usize;
        let mut lo_tmu = 0usize;
        let mut lo_tmu_only = 0usize;
        let mut lo_up = 0usize;
        let mut lo_w1 = 0usize;
        let mut lo_f1 = 0usize;
        let mut hi_tmd = 0usize;
        let mut hi_tmd_only = 0usize;
        let mut hi_dn = 0usize;
        let mut hi_w1 = 0usize;
        let mut hi_f1 = 0usize;
        let mut nf = 0usize;
        let mut n1 = 0usize;
        let mut tmu_b = [0usize; 4];
        let mut tmd_b = [0usize; 4];
        let mut nb = [0usize; 4];
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
            let t = f64::from_bits(r.qbits);
            let z = r.z;
            let w = f::w_rn53(z);
            let fu = cephes(z, &P0, &Q0, 0);
            let g = mul(w, cephes(z, p, q, *mask));
            let d = ulp_distance(g, t).unwrap_or(99);
            let tmu = hit(mul(w.next_up(), fu.next_up()), t);
            let tmd = hit(mul(w.next_down(), fu.next_down()), t);
            let up = hit(mul(w, fu).next_up(), t);
            let dn = hit(mul(w, fu).next_down(), t);
            let w1u = hit(mul(w.next_up(), fu), t);
            let f1u = hit(mul(w, fu.next_up()), t);
            let w1d = hit(mul(w.next_down(), fu), t);
            let f1d = hit(mul(w, fu.next_down()), t);
            let i = if d == 0 {
                nf += 1;
                0
            } else if d == 1 {
                n1 += 1;
                1
            } else if g < t {
                nlo += 1;
                if tmu {
                    lo_tmu += 1;
                }
                if up {
                    lo_up += 1;
                }
                if w1u {
                    lo_w1 += 1;
                }
                if f1u {
                    lo_f1 += 1;
                }
                if tmu && !up && !w1u && !f1u {
                    lo_tmu_only += 1;
                }
                2
            } else {
                nhi += 1;
                if tmd {
                    hi_tmd += 1;
                }
                if dn {
                    hi_dn += 1;
                }
                if w1d {
                    hi_w1 += 1;
                }
                if f1d {
                    hi_f1 += 1;
                }
                if tmd && !dn && !w1d && !f1d {
                    hi_tmd_only += 1;
                }
                3
            };
            nb[i] += 1;
            if tmu {
                tmu_b[i] += 1;
            }
            if tmd {
                tmd_b[i] += 1;
            }
        }
        println!(
            "{name} fused={nf} 1-ULP={n1} leftover-low n={nlo} tmu={lo_tmu} tmu_only={lo_tmu_only} last-mul_up={lo_up} w+1={lo_w1} F+1={lo_f1}"
        );
        println!(
            "{name} leftover-high n={nhi} tmd={hi_tmd} tmd_only={hi_tmd_only} last-mul_dn={hi_dn} w-1={hi_w1} F-1={hi_f1}"
        );
        print!("{name} tmu");
        for (lab, c) in ["fused", "1-ULP", "lo", "hi"].iter().zip(tmu_b) {
            print!(" {lab}:{c}");
        }
        print!(" tmd");
        for (lab, c) in ["fused", "1-ULP", "lo", "hi"].iter().zip(tmd_b) {
            print!(" {lab}:{c}");
        }
        println!();
    }
}
