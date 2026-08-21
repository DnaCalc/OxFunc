//! W109 G6-01: exhaustive op-tree enumerator — SUBSTRATE + correctness gate.
//! Builds the [Ext80; M] value-vector machinery, the hardware ops (PC64 arith, spill, x87
//! transcendentals), and the tiered leaf set in BOTH provenances (spilled-double + 80-bit
//! resident). Gate: the known Goldberg trees must reproduce 163 (all-double) / 165 (spill-loop)
//! through this machinery, else the enumerator would be silently wrong. (Search driver added
//! after Fable's design review.)
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
const CW: u16 = CW_PC64_RN; // PC64 round-nearest = x87-resident arithmetic
const RN53: u16 = CW_PC53_RN;

fn b(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}

#[derive(Clone)]
struct Vec80(Vec<Ext80>); // one Ext80 per corpus row

impl Vec80 {
    fn map2(&self, o: &Vec80, f: impl Fn(&Ext80, &Ext80) -> Ext80) -> Vec80 {
        Vec80(self.0.iter().zip(&o.0).map(|(a, b)| f(a, b)).collect())
    }
    fn map1(&self, f: impl Fn(&Ext80) -> Ext80) -> Vec80 {
        Vec80(self.0.iter().map(f).collect())
    }
    // spill to double (RN53) and reload as Ext80 — represents an FSTP qword / reload.
    fn spill_rn(&self) -> Vec80 {
        self.map1(|x| ext_from_f64(ext_to_f64(x, RN53)))
    }
    fn to_f64_bits(&self) -> Vec<u64> {
        self.0
            .iter()
            .map(|x| ext_to_f64(x, RN53).to_bits())
            .collect()
    }
}

// x87 ops: PC64 (RN64), operands loaded to 80-bit, result 64-bit-mantissa (extended).
fn vadd(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_add(x, y, CW))
}
fn vsub(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_sub(x, y, CW))
}
fn vmul(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_mul(x, y, CW))
}
fn vdiv(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_div(x, y, CW))
}

// SSE2 ops: single RN53 rounding of the EXACT operation on doubles (the 106-bit exact product
// that x87 PC64 cannot hold — this is why the combine is a distinct number system, not x87).
// Operands must be double-kind (53-bit); extracted exactly via ext_to_f64(RN53).
fn d(x: &Ext80) -> f64 {
    ext_to_f64(x, RN53)
}
fn sse_add(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_from_f64(d(x) + d(y)))
}
fn sse_sub(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_from_f64(d(x) - d(y)))
}
fn sse_mul(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_from_f64(d(x) * d(y)))
}
fn sse_div(a: &Vec80, b: &Vec80) -> Vec80 {
    a.map2(b, |x, y| ext_from_f64(d(x) / d(y)))
}

fn exp_ext(tau: &Ext80, l2e: &Ext80) -> (Ext80, Ext80) {
    let y = ext_mul(tau, l2e, CW);
    let k = ext_rndint(&y, CW);
    let f = ext_sub(&y, &k, CW);
    let w = ext_f2xm1(&f, CW);
    (ext_scale(&ext_add(&w, &ext_one(), CW), &k, CW), w)
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();
    let l2e = ext_l2e();
    let one = ext_one();

    let mut rows: Vec<(i32, u32, f64, f64, f64, u64)> = Vec::new();
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        rows.push((
            f[0].parse().unwrap(),
            f[1].parse().unwrap(),
            b(f[2]),
            b(f[3]),
            b(f[4]),
            b(f[5]).to_bits(),
        ));
    }
    let m = rows.len();
    let em_target: Vec<u64> = rows.iter().map(|r| r.5).collect();

    // Build leaf vectors (both provenances)
    let ef = ext_from_f64;
    let mk =
        |f: &dyn Fn(&(i32, u32, f64, f64, f64, u64)) -> Ext80| Vec80(rows.iter().map(f).collect());
    let leaf_one = mk(&|_| one);
    let leaf_two = mk(&|_| ef(2.0));
    let leaf_half = mk(&|_| ef(0.5));
    let leaf_ln2 = mk(&|_| ln2);
    let leaf_l2e = mk(&|_| l2e);
    let leaf_r = mk(&|r| ef(2f64.powi(r.0)));
    let leaf_n = mk(&|r| ef(r.1 as f64));
    let leaf_negn = mk(&|r| ef(-(r.1 as f64)));
    // spilled-double captured leaves
    let leaf_tau_d = mk(&|r| ef(r.2));
    let leaf_u_d = mk(&|r| ef(r.3));
    let leaf_lnu_d = mk(&|r| ef(r.4));
    let leaf_a_d = mk(&|r| ef(r.3 - 1.0));
    // 80-bit resident recomputations
    let leaf_l1p_e = mk(&|r| ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW));
    let leaf_tau_e = mk(&|r| {
        ext_mul(
            &ef(-(r.1 as f64)),
            &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW),
            CW,
        )
    });
    let leaf_u_e = mk(&|r| {
        let tau = ext_mul(
            &ef(-(r.1 as f64)),
            &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW),
            CW,
        );
        exp_ext(&tau, &l2e).0
    });
    let leaf_w_e = mk(&|r| {
        let tau = ext_mul(
            &ef(-(r.1 as f64)),
            &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW),
            CW,
        );
        exp_ext(&tau, &l2e).1
    });
    let leaf_a_e = mk(&|r| {
        let tau = ext_mul(
            &ef(-(r.1 as f64)),
            &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW),
            CW,
        );
        ext_sub(&exp_ext(&tau, &l2e).0, &one, CW)
    });
    let leaf_lnu_e = mk(&|r| {
        let tau = ext_mul(
            &ef(-(r.1 as f64)),
            &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW),
            CW,
        );
        ext_fyl2x(&ln2, &exp_ext(&tau, &l2e).0, CW)
    });

    let leaves: Vec<(&str, &Vec80)> = vec![
        ("one", &leaf_one),
        ("two", &leaf_two),
        ("half", &leaf_half),
        ("ln2", &leaf_ln2),
        ("l2e", &leaf_l2e),
        ("r", &leaf_r),
        ("n", &leaf_n),
        ("negn", &leaf_negn),
        ("tau_d", &leaf_tau_d),
        ("u_d", &leaf_u_d),
        ("lnu_d", &leaf_lnu_d),
        ("a_d", &leaf_a_d),
        ("l1p_e", &leaf_l1p_e),
        ("tau_e", &leaf_tau_e),
        ("u_e", &leaf_u_e),
        ("w_e", &leaf_w_e),
        ("a_e", &leaf_a_e),
        ("lnu_e", &leaf_lnu_e),
    ];

    let score = |v: &Vec80| -> usize {
        v.to_f64_bits()
            .iter()
            .zip(&em_target)
            .filter(|(a, b)| a == b)
            .count()
    };

    // sanity: ext leaves spill == captured doubles
    println!("=== leaf sanity (80-bit resident spilled == captured double) ===");
    println!(
        "tau_e==tau_d: {}/{}",
        leaf_tau_e
            .spill_rn()
            .to_f64_bits()
            .iter()
            .zip(leaf_tau_d.to_f64_bits())
            .filter(|(a, b)| **a == *b)
            .count(),
        m
    );
    println!(
        "u_e==u_d:     {}/{}",
        leaf_u_e
            .spill_rn()
            .to_f64_bits()
            .iter()
            .zip(leaf_u_d.to_f64_bits())
            .filter(|(a, b)| **a == *b)
            .count(),
        m
    );
    println!(
        "lnu_e==lnu_d: {}/{}",
        leaf_lnu_e
            .spill_rn()
            .to_f64_bits()
            .iter()
            .zip(leaf_lnu_d.to_f64_bits())
            .filter(|(a, b)| **a == *b)
            .count(),
        m
    );

    // === CORRECTNESS GATE: known Goldberg trees ===
    // pure SSE2 all-double: RN53((a_d*tau_d)/lnu_d), single rounding per op
    let base_sse = sse_div(&sse_mul(&leaf_a_d, &leaf_tau_d), &leaf_lnu_d);
    // x87 spill-loop: PC64 mul -> spill, PC64 div -> spill (double-rounded numerator)
    let spill165 = vdiv(&vmul(&leaf_a_d, &leaf_tau_d).spill_rn(), &leaf_lnu_d).spill_rn();
    // fully resident (no spill) = the "too accurate" ~133
    let resid = vdiv(&vmul(&leaf_a_e, &leaf_tau_e), &leaf_lnu_e).spill_rn();

    println!("\n=== correctness gate (known trees through the substrate) ===");
    println!("pure-SSE2 all-double (expect 163): {}", score(&base_sse));
    println!("x87 spill-loop       (expect 165): {}", score(&spill165));
    println!("fully-resident       (expect 133): {}", score(&resid));
    println!("\nleaves = {}, rows = {}", leaves.len(), m);

    // === bank-growth measurement (informs the memory strategy) ===
    fn hash80(v: &Vec80) -> u128 {
        let mut h: u128 = 0xcbf29ce484222325;
        for e in &v.0 {
            for &byte in &e.0 {
                h = h.wrapping_mul(0x100000001b3).wrapping_add(byte as u128);
            }
        }
        h
    }
    #[derive(Clone)]
    struct Node {
        v: Vec80,
        dbl: bool,
    }
    // leaf kinds: constants/inputs/_d are double; ln2,l2e,_e are extended
    let dbl_leaf = |name: &str| {
        matches!(
            name,
            "one" | "two" | "half" | "r" | "n" | "negn" | "tau_d" | "u_d" | "lnu_d" | "a_d"
        )
    };
    let mut nodes: Vec<Node> = leaves
        .iter()
        .map(|(nm, v)| Node {
            v: (*v).clone(),
            dbl: dbl_leaf(nm),
        })
        .collect();
    let mut seen: std::collections::HashSet<u128> = nodes.iter().map(|nd| hash80(&nd.v)).collect();
    let target_double: Vec<u64> = em_target.clone();
    let mut found = false;

    for level in 1..=3 {
        let cur = nodes.clone();
        let mut fresh: Vec<Node> = Vec::new();
        let mut push = |v: Vec80,
                        dbl: bool,
                        seen: &mut std::collections::HashSet<u128>,
                        fresh: &mut Vec<Node>,
                        found: &mut bool| {
            let h = hash80(&v);
            if seen.insert(h) {
                if v.to_f64_bits() == target_double {
                    *found = true;
                }
                fresh.push(Node { v, dbl });
            }
        };
        // binary ops over pairs (i,j) with i,j in cur (ordered for non-commutative)
        for i in 0..cur.len() {
            for j in 0..cur.len() {
                let (a, bb) = (&cur[i], &cur[j]);
                // x87 arith (any kinds) -> extended
                if i <= j {
                    push(vadd(&a.v, &bb.v), false, &mut seen, &mut fresh, &mut found);
                }
                push(vsub(&a.v, &bb.v), false, &mut seen, &mut fresh, &mut found);
                if i <= j {
                    push(vmul(&a.v, &bb.v), false, &mut seen, &mut fresh, &mut found);
                }
                push(vdiv(&a.v, &bb.v), false, &mut seen, &mut fresh, &mut found);
                // SSE2 arith (both double) -> double
                if a.dbl && bb.dbl {
                    if i <= j {
                        push(
                            sse_add(&a.v, &bb.v),
                            true,
                            &mut seen,
                            &mut fresh,
                            &mut found,
                        );
                    }
                    push(
                        sse_sub(&a.v, &bb.v),
                        true,
                        &mut seen,
                        &mut fresh,
                        &mut found,
                    );
                    if i <= j {
                        push(
                            sse_mul(&a.v, &bb.v),
                            true,
                            &mut seen,
                            &mut fresh,
                            &mut found,
                        );
                    }
                    push(
                        sse_div(&a.v, &bb.v),
                        true,
                        &mut seen,
                        &mut fresh,
                        &mut found,
                    );
                }
            }
        }
        // unary: spill (->double)
        for a in &cur {
            if !a.dbl {
                push(a.v.spill_rn(), true, &mut seen, &mut fresh, &mut found);
            }
        }
        let n_new = fresh.len();
        nodes.extend(fresh);
        println!(
            "level {level}: +{n_new} distinct  (bank total {})  found_em={}",
            nodes.len(),
            found
        );
        if found {
            println!(">>> em vector REACHED at level {level}");
            break;
        }
    }
}
