//! W109 G6-01: exhaustive op-tree search (Fable-reviewed design). Bottom-up size-indexed bank
//! over the combine-leaves (transcendental outputs are leaves in both provenances), deduped by
//! the full-234 Ext80 vector, NaN-pruned (sound: NaN is absorbing, em is finite). Then a
//! goal-directed ROOT INTERVAL JOIN: em is the rounded root output, so a partner lies in a
//! per-row preimage window (loose prefilter via a sorted row-0 index + EXACT verify → sound).
//! Covers every size<=5 DAG with an arithmetic root. Reports em-reached + bank cardinalities.
use oxfunc_core::excel_numeric::research as rx;
use rayon::prelude::*;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
const CW: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;
const CAP: usize = 6_000_000; // bank cap (report if hit — no silent truncation)

fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }
fn d(x: &Ext80) -> f64 { ext_to_f64(x, RN53) }

#[derive(Clone)]
struct Node { v: Vec<Ext80>, dbl: bool, size: u8, prov: Prov }
#[derive(Clone)]
enum Prov { Leaf(&'static str), Un(&'static str, u32), Bin(&'static str, u32, u32) }

fn hash80(v: &[Ext80]) -> u128 {
    let mut h: u128 = 0xcbf29ce484222325;
    for e in v { for &byte in &e.0 { h = h.wrapping_mul(0x100000001b3).wrapping_add(byte as u128); } }
    h
}
fn has_nan(v: &[Ext80]) -> bool { v.iter().any(|e| { let f = ext_to_f64(e, RN53); f.is_nan() }) }

// elementwise ops
fn map2(a: &[Ext80], b: &[Ext80], f: impl Fn(&Ext80, &Ext80) -> Ext80) -> Vec<Ext80> {
    a.iter().zip(b).map(|(x, y)| f(x, y)).collect()
}
fn x87(a: &[Ext80], b: &[Ext80], op: &str) -> Vec<Ext80> {
    match op { "add" => map2(a,b,|x,y|ext_add(x,y,CW)), "sub" => map2(a,b,|x,y|ext_sub(x,y,CW)),
        "mul" => map2(a,b,|x,y|ext_mul(x,y,CW)), "div" => map2(a,b,|x,y|ext_div(x,y,CW)), _=>unreachable!() }
}
fn sse(a: &[Ext80], b: &[Ext80], op: &str) -> Vec<Ext80> {
    match op { "add" => map2(a,b,|x,y|ext_from_f64(d(x)+d(y))), "sub" => map2(a,b,|x,y|ext_from_f64(d(x)-d(y))),
        "mul" => map2(a,b,|x,y|ext_from_f64(d(x)*d(y))), "div" => map2(a,b,|x,y|ext_from_f64(d(x)/d(y))), _=>unreachable!() }
}
fn spill(a: &[Ext80]) -> Vec<Ext80> { a.iter().map(|x| ext_from_f64(d(x))).collect() }
fn spill_rz(a: &[Ext80]) -> Vec<Ext80> { let rz = CW_PC53_RN | 0x0C00; a.iter().map(|x| ext_from_f64(ext_to_f64(x, rz))).collect() }
fn chs(a: &[Ext80]) -> Vec<Ext80> { a.iter().map(|x| ext_sub(&ext_from_f64(0.0), x, CW)).collect() }
fn bits(v: &[Ext80]) -> Vec<u64> { v.iter().map(|x| d(x).to_bits()).collect() }

fn exp_ext(tau: &Ext80, l2e: &Ext80) -> (Ext80, Ext80) {
    let y = ext_mul(tau, l2e, CW); let k = ext_rndint(&y, CW); let f = ext_sub(&y, &k, CW);
    let w = ext_f2xm1(&f, CW); (ext_scale(&ext_add(&w, &ext_one(), CW), &k, CW), w)
}

// reals X with RN53(X) == em, widened 2 ULP for a sound (never-too-narrow) interval.
fn round_iv(em: f64) -> (f64, f64) {
    let up = |x: f64| { let b = x.to_bits(); if x >= 0.0 { f64::from_bits(b + 1) } else { f64::from_bits(b - 1) } };
    let dn = |x: f64| { let b = x.to_bits(); if x > 0.0 { f64::from_bits(b - 1) } else { f64::from_bits(b + 1) } };
    let prev = dn(em); let next = up(em);
    // midpoints (f64-approx) widened by 2 ULP each side
    (dn(dn(em - (em - prev) * 0.5)), up(up(em + (next - em) * 0.5)))
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2(); let l2e = ext_l2e(); let one = ext_one();
    let mut rows = Vec::new();
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        rows.push((f[0].parse::<i32>().unwrap(), f[1].parse::<u32>().unwrap(), b(f[2]), b(f[3]), b(f[4]), b(f[5]).to_bits()));
    }
    let m = rows.len();
    let mut em_bits: Vec<u64> = rows.iter().map(|r| r.5).collect();
    let ef = ext_from_f64;

    // SELF-TEST: replace target with a KNOWN size-3 root-join-reachable vector and confirm the
    // join finds it (guards against a prefilter false-negative). X=mul(a_d,tau_d) [size1],
    // Y=sub(lnu_d,r) [size1], target = spill(div(X,Y)) — arithmetic root over two size-1 children.
    let selftest = std::env::var("SELFTEST").is_ok();
    if selftest {
        let a_d: Vec<Ext80> = rows.iter().map(|r| ef(r.3 - 1.0)).collect();
        let tau_d: Vec<Ext80> = rows.iter().map(|r| ef(r.2)).collect();
        let lnu_d: Vec<Ext80> = rows.iter().map(|r| ef(r.4)).collect();
        let rr: Vec<Ext80> = rows.iter().map(|r| ef(2f64.powi(r.0))).collect();
        let x = x87(&a_d, &tau_d, "mul");
        let y = x87(&lnu_d, &rr, "sub");
        em_bits = bits(&spill(&x87(&x, &y, "div")));
        println!("[SELFTEST] target = spill(div(mul(a_d,tau_d), sub(lnu_d,r))) — expect HIT via join");
    }

    // ---- leaves (combine leaves: transcendental outputs in both provenances) ----
    let mut leaves: Vec<(&'static str, Vec<Ext80>, bool)> = Vec::new();
    let mut push_leaf = |nm, f: &dyn Fn(&(i32,u32,f64,f64,f64,u64)) -> Ext80, dbl, lv: &mut Vec<_>| {
        lv.push((nm, rows.iter().map(f).collect::<Vec<Ext80>>(), dbl));
    };
    push_leaf("one", &|_| one, true, &mut leaves);
    push_leaf("two", &|_| ef(2.0), true, &mut leaves);
    push_leaf("half", &|_| ef(0.5), true, &mut leaves);
    push_leaf("ln2", &|_| ln2, false, &mut leaves);
    push_leaf("l2e", &|_| l2e, false, &mut leaves);
    push_leaf("r", &|r| ef(2f64.powi(r.0)), true, &mut leaves);
    push_leaf("n", &|r| ef(r.1 as f64), true, &mut leaves);
    push_leaf("negn", &|r| ef(-(r.1 as f64)), true, &mut leaves);
    push_leaf("tau_d", &|r| ef(r.2), true, &mut leaves);
    push_leaf("u_d", &|r| ef(r.3), true, &mut leaves);
    push_leaf("lnu_d", &|r| ef(r.4), true, &mut leaves);
    push_leaf("a_d", &|r| ef(r.3 - 1.0), true, &mut leaves);
    let te = |r: &(i32,u32,f64,f64,f64,u64)| ext_mul(&ef(-(r.1 as f64)), &ext_fyl2xp1(&ln2, &ef(2f64.powi(r.0)), CW), CW);
    push_leaf("tau_e", &|r| te(r), false, &mut leaves);
    push_leaf("u_e", &|r| exp_ext(&te(r), &l2e).0, false, &mut leaves);
    push_leaf("w_e", &|r| exp_ext(&te(r), &l2e).1, false, &mut leaves);
    push_leaf("a_e", &|r| ext_sub(&exp_ext(&te(r), &l2e).0, &one, CW), false, &mut leaves);
    // lnu from the resident double-u (the useful one): FYL2X(ln2, widen(u_dbl)) kept 80-bit
    push_leaf("lnu_re", &|r| ext_fyl2x(&ln2, &ef(r.3), CW), false, &mut leaves);

    let mut bank: Vec<Node> = leaves.iter().map(|(nm, v, dbl)| Node { v: v.clone(), dbl: *dbl, size: 0, prov: Prov::Leaf(nm) }).collect();
    let mut seen: std::collections::HashSet<u128> = bank.iter().map(|n| hash80(&n.v)).collect();
    let mut by_size: Vec<Vec<u32>> = vec![(0..bank.len() as u32).collect()];

    let check_hit = |v: &[Ext80]| -> bool { bits(v) == em_bits };
    let mut hit: Option<usize> = None;

    // ---- bottom-up size-indexed bank to size S ----
    let big_s = 2u8;
    let commutative = |op: &str| matches!(op, "add" | "mul");
    for s in 1..=big_s {
        let mut fresh: Vec<Node> = Vec::new();
        let mut try_push = |v: Vec<Ext80>, dbl: bool, prov: Prov, seen: &mut std::collections::HashSet<u128>, fresh: &mut Vec<Node>, hit: &mut Option<usize>| {
            if has_nan(&v) { return; }
            let h = hash80(&v);
            if seen.insert(h) {
                if hit.is_none() && check_hit(&v) { *hit = Some(usize::MAX); }
                fresh.push(Node { v, dbl, size: s, prov });
            }
        };
        // binary: children sizes i+j = s-1
        for i in 0..=(s - 1) {
            let j = s - 1 - i;
            let (li, lj) = (by_size[i as usize].clone(), by_size[j as usize].clone());
            for &ia in &li {
                for &jb in &lj {
                    if hit.is_some() { break; }
                    let (na, nb) = (&bank[ia as usize], &bank[jb as usize]);
                    for op in ["add", "sub", "mul", "div"] {
                        if commutative(op) && i == j && ia > jb { continue; }
                        // x87 flavor
                        try_push(x87(&na.v, &nb.v, op), false, Prov::Bin(op, ia, jb), &mut seen, &mut fresh, &mut hit);
                        // sse flavor (both operands must be double)
                        if na.dbl && nb.dbl {
                            let sop = match op { "add"=>"sadd","sub"=>"ssub","mul"=>"smul","div"=>"sdiv",_=>"" };
                            try_push(sse(&na.v, &nb.v, op), true, Prov::Bin(sop, ia, jb), &mut seen, &mut fresh, &mut hit);
                        }
                    }
                    // transcendental binary ops (x87), domain-guarded (skip vector if any row invalid)
                    let all = |f: &dyn Fn(&Ext80) -> bool, v: &[Ext80]| v.iter().all(|e| f(e));
                    // FYL2X(y=na, x=nb): y*log2(x), x>0
                    if all(&|e| d(e) > 0.0, &nb.v) {
                        try_push(map2(&na.v, &nb.v, |y, x| ext_fyl2x(y, x, CW)), false, Prov::Bin("fyl2x", ia, jb), &mut seen, &mut fresh, &mut hit);
                    }
                    // FYL2XP1(y=na, x=nb): y*log2(1+x), |x|<1-sqrt2/2
                    if all(&|e| d(e).abs() < 0.2928932, &nb.v) {
                        try_push(map2(&na.v, &nb.v, |y, x| ext_fyl2xp1(y, x, CW)), false, Prov::Bin("fyl2xp1", ia, jb), &mut seen, &mut fresh, &mut hit);
                    }
                    // FSCALE(x=na, k=nb): x*2^trunc(k)
                    try_push(map2(&na.v, &nb.v, |x, k| ext_scale(x, k, CW)), false, Prov::Bin("fscale", ia, jb), &mut seen, &mut fresh, &mut hit);
                }
            }
        }
        // unary over size s-1
        for &ia in &by_size[(s - 1) as usize].clone() {
            let na = &bank[ia as usize];
            if !na.dbl { try_push(spill(&na.v), true, Prov::Un("spill", ia), &mut seen, &mut fresh, &mut hit); }
            if !na.dbl { try_push(spill_rz(&na.v), true, Prov::Un("spillrz", ia), &mut seen, &mut fresh, &mut hit); }
            try_push(chs(&na.v), na.dbl, Prov::Un("chs", ia), &mut seen, &mut fresh, &mut hit);
            // F2XM1(x): 2^x-1, |x|<=1
            if na.v.iter().all(|e| d(e).abs() <= 1.0) {
                try_push(na.v.iter().map(|e| ext_f2xm1(e, CW)).collect(), false, Prov::Un("f2xm1", ia), &mut seen, &mut fresh, &mut hit);
            }
            // FRNDINT(x): round to integer
            try_push(na.v.iter().map(|e| ext_rndint(e, CW)).collect(), false, Prov::Un("frndint", ia), &mut seen, &mut fresh, &mut hit);
        }
        let base = bank.len() as u32;
        let cnt = fresh.len();
        bank.extend(fresh);
        by_size.push((base..bank.len() as u32).collect());
        println!("size {s}: +{cnt} distinct (bank {})", bank.len());
        if let Some(_) = hit { hit = Some(bank.len() - 1); }
        if bank.len() > CAP { println!("!! bank cap {CAP} exceeded at size {s} — envelope incomplete beyond here"); break; }
    }

    if hit.is_some() { report(&bank, bank.len()-1, m); return; }

    // ---- ROOT INTERVAL JOIN: spill(A op B)==em or sse_op(A,B)==em over the whole bank ----
    // Sorted index on row-0 double value for a loose prefilter; exact-verify all rows.
    let mut idx: Vec<(f64, u32)> = bank.iter().enumerate().map(|(i, n)| (d(&n.v[0]), i as u32)).collect();
    idx.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let keys: Vec<f64> = idx.iter().map(|x| x.0).collect();
    let em0 = f64::from_bits(em_bits[0]);
    let win = |t: f64| { let u = (t.abs().to_bits() as f64) * 0.0 + 64.0 * 2f64.powi(-52) * t.abs().max(2f64.powi(-60)); (t - u, t + u) };
    let bankref = &bank;
    let found = (0..bank.len()).into_par_iter().find_map_any(|bi| {
        let nb = &bankref[bi];
        let b0 = d(&nb.v[0]);
        for op in ["div", "mul", "add", "sub"] {
            // target A_0 that makes A op B == em on row 0 (approx)
            let targets: Vec<f64> = match op {
                "div" => vec![em0 * b0],           // A/B=em -> A=em*B
                "mul" => if b0 != 0.0 { vec![em0 / b0] } else { vec![] }, // A*B=em -> A=em/B
                "add" => vec![em0 - b0],           // A+B=em
                "sub" => vec![em0 + b0, b0 - em0], // A-B=em (A=em+B) or B-A=em (A=B-em)
                _ => vec![],
            };
            for (ti, t) in targets.iter().enumerate() {
                let (lo, hi) = win(*t);
                let mut p = keys.partition_point(|&k| k < lo);
                while p < keys.len() && keys[p] <= hi {
                    let ai = idx[p].1 as usize;
                    let na = &bankref[ai];
                    // exact verify: try both x87-spill and sse flavors, in the correct operand order
                    let (aa, bb) = if op == "sub" && ti == 1 { (&nb.v, &na.v) } else { (&na.v, &nb.v) };
                    // x87 spill root
                    if bits(&spill(&x87(aa, bb, op))) == em_bits { return Some((op, ai, bi, ti, false)); }
                    // sse root (needs both double)
                    let both_dbl = if op == "sub" && ti == 1 { nb.dbl && na.dbl } else { na.dbl && nb.dbl };
                    if both_dbl && bits(&sse(aa, bb, op)) == em_bits { return Some((op, ai, bi, ti, true)); }
                    p += 1;
                }
            }
        }
        None
    });

    match found {
        Some((op, ai, bi, ti, is_sse)) => {
            println!("\n>>> HIT via ROOT JOIN: {}{} ( #{} {} #{} )", if is_sse {"sse_"} else {"x87spill_"}, op, ai, if op=="sub"&&ti==1 {"rev"} else {""}, bi);
            print!("  A = "); report(&bank, ai, m);
            print!("  B = "); report(&bank, bi, m);
        }
        None => {
            println!("\nNO size<=5 arithmetic-rooted DAG over the combine-leaves reproduces em.");
            println!("(bank size {} = all size<=2 subtrees; root join covers size<=5 div/mul/add/sub roots)", bank.len());
        }
    }

    // ================= EXTENSION 1: ONE-FREE-CONSTANT SYNTHESIS =================
    // For each subtree V in the bank and each outer op, solve for a SINGLE scalar C (same every
    // row) with spill(op(V,C)) == em. Covers the "routine with one magic constant" idiom — a
    // foreign coefficient the enumerator's fixed leaf set is otherwise blind to.
    let em_f: Vec<f64> = em_bits.iter().map(|&b| f64::from_bits(b)).collect();
    let ivs: Vec<(f64, f64)> = em_f.iter().map(|&e| round_iv(e)).collect();
    let mut synth_hits = 0;
    'vloop: for (vi, nv) in bank.iter().enumerate() {
        let vv: Vec<f64> = nv.v.iter().map(|e| d(e)).collect();
        // op forms: V*C, V/C, C/V, V+C, V-C, C-V  (C the unknown scalar)
        for form in ["mulC", "VdivC", "CdivV", "addC", "VsubC", "CsubV"] {
            let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
            let mut feasible = true;
            for i in 0..m {
                let (el, eh) = ivs[i]; let v = vv[i];
                // C-interval on row i s.t. op(v,C) in (el,eh)
                let (mut cl, mut ch) = match form {
                    "mulC" => if v > 0.0 { (el / v, eh / v) } else if v < 0.0 { (eh / v, el / v) } else { feasible = false; break; },
                    "VdivC" => { // v/C in (el,eh); C = v/y, y in (el,eh)
                        let (a, b2) = (v / el, v / eh); (a.min(b2), a.max(b2)) }
                    "CdivV" => if v != 0.0 { let (a, b2) = (el * v, eh * v); (a.min(b2), a.max(b2)) } else { feasible = false; break; },
                    "addC" => (el - v, eh - v),
                    "VsubC" => (v - eh, v - el),
                    "CsubV" => (el + v, eh + v),
                    _ => unreachable!(),
                };
                if cl > ch { std::mem::swap(&mut cl, &mut ch); }
                lo = lo.max(cl); hi = hi.min(ch);
                if lo > hi { feasible = false; break; }
            }
            if feasible && lo <= hi {
                // candidate constant: midpoint; verify exactly (try a few doubles in the interval)
                for cand in [lo, hi, (lo + hi) * 0.5, f64::from_bits(((lo.to_bits() as i128 + hi.to_bits() as i128) / 2) as u64)] {
                    if !cand.is_finite() { continue; }
                    let cvec: Vec<Ext80> = (0..m).map(|_| ext_from_f64(cand)).collect();
                    let res = match form {
                        "mulC" => spill(&x87(&nv.v, &cvec, "mul")),
                        "VdivC" => spill(&x87(&nv.v, &cvec, "div")),
                        "CdivV" => spill(&x87(&cvec, &nv.v, "div")),
                        "addC" => spill(&x87(&nv.v, &cvec, "add")),
                        "VsubC" => spill(&x87(&nv.v, &cvec, "sub")),
                        "CsubV" => spill(&x87(&cvec, &nv.v, "sub")),
                        _ => unreachable!(),
                    };
                    if bits(&res) == em_bits {
                        println!("\n>>> FREE-CONSTANT HIT: {} with C = {:.20e} (0x{:016x})", form, cand, cand.to_bits());
                        print!("  V = "); report(&bank, vi, m);
                        synth_hits += 1;
                        break 'vloop;
                    }
                }
            }
        }
    }
    if synth_hits == 0 { println!("EXT1 free-constant synthesis: no single foreign constant closes em over any size<=2 subtree + one outer op."); }

    // ================= EXTENSION 2: MATCH-MASK BRANCH MINING =================
    // Record the 234-bit em-match mask of every bank vector; if em is a 2-branch composition,
    // two masks OR-cover all rows. Report the best cover and whether it splits cleanly by |tau|.
    let taus: Vec<f64> = rows.iter().map(|r| r.2.abs()).collect();
    let masks: Vec<(u64, Vec<bool>, usize)> = bank.iter().enumerate().filter_map(|(i, nv)| {
        let bt = bits(&nv.v);
        let mask: Vec<bool> = bt.iter().zip(&em_bits).map(|(a, b)| a == b).collect();
        let cnt = mask.iter().filter(|&&x| x).count();
        if cnt >= 120 { Some((i as u64, mask, cnt)) } else { None }
    }).collect();
    println!("\nEXT2 branch mining: {} bank vectors match em on >=120 rows", masks.len());
    let mut best_cover = 0usize;
    let mut best_pair = (0u64, 0u64);
    for a in 0..masks.len() {
        for b2 in a..masks.len() {
            let cov = (0..m).filter(|&i| masks[a].1[i] || masks[b2].1[i]).count();
            if cov > best_cover { best_cover = cov; best_pair = (masks[a].0, masks[b2].0); }
        }
    }
    println!("  best 2-tree OR-cover = {}/{} rows (trees #{} , #{})", best_cover, m, best_pair.0, best_pair.1);
    if best_cover == m {
        // check if the split is a clean |tau| threshold
        let (ia, ib) = (best_pair.0 as usize, best_pair.1 as usize);
        let ma: Vec<bool> = bits(&bank[ia].v).iter().zip(&em_bits).map(|(a, b)| a == b).collect();
        let only_a: Vec<f64> = (0..m).filter(|&i| ma[i]).map(|i| taus[i]).collect();
        let only_b: Vec<f64> = (0..m).filter(|&i| !ma[i]).map(|i| taus[i]).collect();
        let clean = only_a.iter().cloned().fold(f64::MIN, f64::max) < only_b.iter().cloned().fold(f64::MAX, f64::min)
            || only_b.iter().cloned().fold(f64::MIN, f64::max) < only_a.iter().cloned().fold(f64::MAX, f64::min);
        println!("  2-tree cover EXISTS; clean |tau|-threshold split: {} (branching hypothesis {})",
                 clean, if clean { "PLAUSIBLE — investigate" } else { "unlikely (masks interleave in tau)" });
    } else {
        println!("  no 2-tree cover at size<=2 -> em is not a 2-branch composition of size<=2 subtrees.");
    }
}

fn report(bank: &[Node], i: usize, _m: usize) {
    fn s(bank: &[Node], i: usize) -> String {
        match &bank[i].prov {
            Prov::Leaf(nm) => nm.to_string(),
            Prov::Un(op, a) => format!("{}({})", op, s(bank, *a as usize)),
            Prov::Bin(op, a, b) => format!("{}({}, {})", op, s(bank, *a as usize), s(bank, *b as usize)),
        }
    }
    println!("tree: {}", s(bank, i));
}
