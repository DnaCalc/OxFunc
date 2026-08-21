//! W109 G6-01: attack the fv!=0/ty!=0 combine wall (fvty 43%). Keep the proven
//! em (excel_expm1_internal), vary ONLY the fv/ty numerator+denominator assembly.
//! >2ulp misses => wrong op-graph space (positive-power pv*P+fv vs pv+fv*v).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn em_of(r: f64, n: f64) -> (f64, f64) {
    let nl = -(n * rx::excel_log1p(r));
    (rx::excel_expm1_internal(nl), nl)
}

// 1 landed
fn c1(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}
// 2 v = exp(nl)
fn c2(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, nl) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = rx::excel_exp(nl);
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}
// 3 positive-power numerator, P=1/v: pmt = -(pv*P+fv)*r/((P-1)*tf)
fn c3(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let p = 1.0 / v;
    let tf = 1.0 + r * ty;
    -(pv * p + fv) * r / ((p - 1.0) * tf)
}
// 4 positive-power, P=exp(-nl) via excel_exp directly
fn c4(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (_, nl) = em_of(r, n);
    let p = rx::excel_exp(-nl); // (1+r)^n
    let tf = 1.0 + r * ty;
    -(pv * p + fv) * r / ((p - 1.0) * tf)
}
// 5 mult r into num first: (pv+fv*v)*r/(em*tf)
fn c5(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    (num * r) / (em * tf)
}
// 6 tf as reciprocal-multiply
fn c6(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) * (1.0 / tf)) * r
}
// 7 positive-power num but em-denominator: pmt = -(pv*P+fv)*r / (em' ...)
//   use P=1/v for num, but keep denom = em*tf (reciprocal space) — mix
fn c7(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let p = 1.0 / v;
    let tf = 1.0 + r * ty;
    // -(pv*P+fv) then / (P-1) but P-1 via -em/v (=-em*p)
    let num = -(pv * p + fv);
    let denom = (-em * p) * tf;
    num * r / denom
}
// 8 ty applied as division of the whole by tf, done LAST after *r
fn c8(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let (em, _) = em_of(r, n);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    (num / em * r) / tf
}

fn main() {
    let corpora = ["fvty", "fv1sweep", "heldout", "combsweep", "po2"];
    type F = fn(f64, f64, f64, f64, f64) -> f64;
    let cands: [(&str, F); 8] = [
        ("1 landed", c1),
        ("2 v=exp", c2),
        ("3 posP 1/v", c3),
        ("4 posP exp", c4),
        ("5 r-in-num", c5),
        ("6 tf-recip", c6),
        ("7 posPnum/emden", c7),
        ("8 tf-last", c8),
    ];
    print!("{:12}{:>7}", "corpus", "N");
    for (nm, _) in &cands {
        print!(" {:>12}", nm);
    }
    println!();
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc = [0u32; 8];
        let mut tot = 0u32;
        for w in &ws.witnesses {
            let a: Vec<f64> = w
                .args
                .iter()
                .filter_map(|x| match x {
                    WitnessArg::Scalar(s) => parse_bits_hex(s),
                    _ => None,
                })
                .collect();
            if a.len() != 5 {
                continue;
            }
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1;
            for (i, (_, f)) in cands.iter().enumerate() {
                if f(a[0], a[1], a[2], a[3], a[4]).to_bits() == want {
                    sc[i] += 1;
                }
            }
        }
        print!("{:12}{:>7}", cn, tot);
        for i in 0..8 {
            print!(" {:>11.1}%", 100.0 * sc[i] as f64 / tot as f64);
        }
        println!();
    }
}
