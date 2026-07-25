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

/// Evaluate ONE ROW of a size-3 node from provenance. This is the hot path: it lets the join
/// verify a candidate row-by-row with early exit on first mismatch, instead of materializing a
/// 234-element vector per candidate (which is what made the first EXT6 run intractable).
fn eval3_row(bank: &[Node], op: u8, a: u32, b: u32, i: usize) -> Ext80 {
    let x = &bank[a as usize].v[i];
    let y = &bank[b as usize].v[i];
    match op {
        0 => ext_add(x, y, CW), 1 => ext_sub(x, y, CW), 2 => ext_mul(x, y, CW), 3 => ext_div(x, y, CW),
        4 => ext_from_f64(d(x) + d(y)), 5 => ext_from_f64(d(x) - d(y)),
        6 => ext_from_f64(d(x) * d(y)), 7 => ext_from_f64(d(x) / d(y)),
        8 => ext_fyl2x(x, y, CW), 9 => ext_fyl2x(y, x, CW),
        10 => ext_scale(x, y, CW), 11 => ext_scale(y, x, CW),
        _ => unreachable!(),
    }
}

// re-evaluate a size-3 node op(bank[a], bank[b]) from provenance (children are size<=2 full nodes).
fn eval3(bank: &[Node], op: u8, a: u32, b: u32) -> Vec<Ext80> {
    let (av, bv) = (&bank[a as usize].v, &bank[b as usize].v);
    match op {
        0 => x87(av, bv, "add"), 1 => x87(av, bv, "sub"), 2 => x87(av, bv, "mul"), 3 => x87(av, bv, "div"),
        4 => sse(av, bv, "add"), 5 => sse(av, bv, "sub"), 6 => sse(av, bv, "mul"), 7 => sse(av, bv, "div"),
        8 => map2(av, bv, |y, x| ext_fyl2x(y, x, CW)), 9 => map2(bv, av, |y, x| ext_fyl2x(y, x, CW)),
        10 => map2(av, bv, |x, k| ext_scale(x, k, CW)), 11 => map2(bv, av, |x, k| ext_scale(x, k, CW)),
        _ => unreachable!(),
    }
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

    // ================= EXTENSION 6: size<=7 flat join via provenance-backed size-3 bank =================
    if std::env::var("SEARCH3").is_ok() {
        use std::collections::HashSet;
        use std::io::Write;
        // --- flushed progress log (survives kill; the first EXT6 run lost everything to pipe buffering) ---
        let dir = "../../work/w109/G6-solvers";
        let mut logf = std::fs::OpenOptions::new().create(true).append(true)
            .open(format!("{dir}/optree_search3.log")).unwrap();
        macro_rules! log {
            ($($t:tt)*) => {{ let s = format!($($t)*); println!("{}", s);
                let _ = writeln!(logf, "{}", s); let _ = logf.flush(); }}
        }
        log!("=== SEARCH3 start: rows={} size<=2 bank={} ===", m, bank.len());

        // --- size-3 provenance bank, checkpointed to disk (18 bytes/entry, not full vectors) ---
        let ckpt = format!("{dir}/optree_size3_bank.bin");
        let light: Vec<(u8, u32, u32, bool, f64)> = if std::path::Path::new(&ckpt).exists() {
            let raw = std::fs::read(&ckpt).unwrap();
            let n = u64::from_le_bytes(raw[0..8].try_into().unwrap()) as usize;
            let mut v = Vec::with_capacity(n);
            for i in 0..n {
                let o = 8 + i * 18;
                v.push((raw[o], u32::from_le_bytes(raw[o+1..o+5].try_into().unwrap()),
                        u32::from_le_bytes(raw[o+5..o+9].try_into().unwrap()), raw[o+9] != 0,
                        f64::from_le_bytes(raw[o+10..o+18].try_into().unwrap())));
            }
            log!("loaded size-3 bank from checkpoint: {} entries", v.len());
            v
        } else {
            let mut pairs: Vec<(u32, u32)> = Vec::new();
            for &a in &by_size[0] { for &b in &by_size[2] { pairs.push((a, b)); } }
            for &a in &by_size[1] { for &b in &by_size[1] { pairs.push((a, b)); } }
            for &a in &by_size[2] { for &b in &by_size[0] { pairs.push((a, b)); } }
            log!("building size-3 bank from {} child pairs ...", pairs.len());
            let bankr = &bank; let em_ref = &em_bits;
            let s3hit = std::sync::Mutex::new(None::<(u8, u32, u32)>);
            let raw: Vec<(u128, u8, u32, u32, bool, f64)> = pairs.par_iter().flat_map_iter(|&(a, b)| {
                let na = &bankr[a as usize]; let nb = &bankr[b as usize];
                let both = na.dbl && nb.dbl;
                let valid: [(u8, bool); 12] = [(0,true),(1,true),(2,true),(3,true),(4,both),(5,both),(6,both),(7,both),
                    (8, nb.v.iter().all(|e| d(e) > 0.0)), (9, na.v.iter().all(|e| d(e) > 0.0)), (10,true),(11,true)];
                let mut out = Vec::new();
                for (op, ok) in valid { if !ok { continue; }
                    let v = eval3(bankr, op, a, b);
                    if has_nan(&v) { continue; }
                    if bits(&v) == *em_ref { *s3hit.lock().unwrap() = Some((op, a, b)); }
                    out.push((hash80(&v), op, a, b, op >= 4 && op <= 7, d(&v[0])));
                }
                out
            }).collect();
            if let Some((op, a, b)) = *s3hit.lock().unwrap() {
                log!(">>> SIZE-3 HIT during build: op{} (#{},#{})", op, a, b);
                print!("A="); report(&bank, a as usize, m); print!("B="); report(&bank, b as usize, m);
                return;
            }
            let full_hashes = &seen;
            let mut v: Vec<(u8, u32, u32, bool, f64)> = Vec::new();
            let mut lseen: HashSet<u128> = HashSet::with_capacity(raw.len());
            for (h, op, a, b, dbl, r0) in raw {
                if full_hashes.contains(&h) { continue; }
                if lseen.insert(h) { v.push((op, a, b, dbl, r0)); }
            }
            let mut buf = Vec::with_capacity(8 + v.len() * 18);
            buf.extend_from_slice(&(v.len() as u64).to_le_bytes());
            for &(op, a, b, dbl, r0) in &v {
                buf.push(op); buf.extend_from_slice(&a.to_le_bytes()); buf.extend_from_slice(&b.to_le_bytes());
                buf.push(dbl as u8); buf.extend_from_slice(&r0.to_le_bytes());
            }
            std::fs::write(&ckpt, &buf).unwrap();
            log!("size-3 distinct = {} (checkpointed {} MB)", v.len(), buf.len() / 1_048_576);
            v
        };
        log!("bank size<=3 total = {}", bank.len() + light.len());

        // --- unified sorted join index over size<=3 ---
        #[derive(Clone, Copy)] enum JN { Full(u32), Light(u32) }
        let mut jnodes: Vec<(f64, JN)> = bank.iter().enumerate().map(|(i, n)| (d(&n.v[0]), JN::Full(i as u32))).collect();
        for (j, l) in light.iter().enumerate() { jnodes.push((l.4, JN::Light(j as u32))); }
        jnodes.retain(|x| x.0.is_finite());
        jnodes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let jkeys: Vec<f64> = jnodes.iter().map(|x| x.0).collect();
        let (bankr, lightr, jn, jk, emr) = (&bank, &light, &jnodes, &jkeys, &em_bits);
        let row_of = |n: JN, i: usize| -> Ext80 {
            match n { JN::Full(k) => bankr[k as usize].v[i],
                      JN::Light(j) => { let (op, a, b, ..) = lightr[j as usize]; eval3_row(bankr, op, a, b, i) } }
        };
        // ROW-WISE verify with early exit — the fix that makes this tractable (most candidates
        // die on row 1-2, so we never build a 234-element vector).
        let verify = |an: JN, bn: JN, op: &str, rev: bool| -> bool {
            for i in 0..m {
                let (x, y) = if rev { (row_of(bn, i), row_of(an, i)) } else { (row_of(an, i), row_of(bn, i)) };
                let r = match op { "add" => ext_add(&x, &y, CW), "sub" => ext_sub(&x, &y, CW),
                                   "mul" => ext_mul(&x, &y, CW), "div" => ext_div(&x, &y, CW), _ => unreachable!() };
                if ext_to_f64(&r, RN53).to_bits() != emr[i] { return false; }
            }
            true
        };
        let em0 = f64::from_bits(em_bits[0]);
        let win = |t: f64| { let u = 64.0 * 2f64.powi(-52) * t.abs().max(2f64.powi(-60)); (t - u, t + u) };

        // --- SHARDED, RESUMABLE join: partial coverage is recorded, so a kill loses one shard ---
        let nsh: usize = std::env::var("SHARDS").ok().and_then(|s| s.parse().ok()).unwrap_or(400);
        let start: usize = std::env::var("SHARD_START").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
        let ssz = (jnodes.len() + nsh - 1) / nsh;
        log!("join over {} nodes in {} shards of {} (resume with SHARD_START=k)", jnodes.len(), nsh, ssz);
        let t0 = std::time::Instant::now();
        for sh in start..nsh {
            let (lo_i, hi_i) = (sh * ssz, ((sh + 1) * ssz).min(jnodes.len()));
            if lo_i >= hi_i { break; }
            let found = (lo_i..hi_i).into_par_iter().find_map_any(|bi| {
                let (b0, bn) = jn[bi];
                for op in ["div", "mul", "add", "sub"] {
                    let ts: [f64; 2] = match op {
                        "div" => [em0 * b0, f64::NAN],
                        "mul" => [if b0 != 0.0 { em0 / b0 } else { f64::NAN }, f64::NAN],
                        "add" => [em0 - b0, f64::NAN],
                        _ => [em0 + b0, b0 - em0],
                    };
                    for (ti, t) in ts.iter().enumerate() {
                        if !t.is_finite() { continue; }
                        let (lo, hi) = win(*t);
                        let mut p = jk.partition_point(|&k| k < lo);
                        while p < jk.len() && jk[p] <= hi {
                            let (_, an) = jn[p];
                            if verify(an, bn, op, op == "sub" && ti == 1) { return Some((op, ti, p, bi)); }
                            p += 1;
                        }
                    }
                }
                None
            });
            if let Some((op, ti, p, bi)) = found {
                log!(">>> SIZE<=7 HIT: root {} (rev={}) jnode_a={} jnode_b={}", op, ti == 1, p, bi);
                return;
            }
            if sh % 10 == 0 || sh + 1 == nsh {
                log!("shard {}/{} cleared (outer {}..{}) elapsed {:?}", sh + 1, nsh, lo_i, hi_i, t0.elapsed());
            }
        }
        log!("EXT6: NO size<=7 arithmetic-rooted DAG over size<=3 subtrees reproduces em. ({:?})", t0.elapsed());
        return;
    }

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

    // ================= EXTENSION 3+4: INTERVAL DECOMPOSITION =================
    // Quotient locked em=spill(N/D). Fix one side to its evidenced value, search the OTHER as a
    // size<=5 joined tree (op over two size<=2 bank vectors) whose value hits the inverse-solve
    // interval, then exact-verify em. Reaches quotient trees of effective size ~11.
    // sorted index (idx/keys) reused from the root join above.
    let a_dv: Vec<Ext80> = rows.iter().map(|r| ext_from_f64(r.3 - 1.0)).collect();
    let tau_dv: Vec<Ext80> = rows.iter().map(|r| ext_from_f64(r.2)).collect();
    let lnu_dv: Vec<Ext80> = rows.iter().map(|r| ext_from_f64(r.4)).collect();
    let num_sse = sse(&a_dv, &tau_dv, "mul");   // RN53((u-1)*tau) — the evidenced numerator
    let win2 = |t: f64| { let u = 128.0 * 2f64.powi(-52) * t.abs().max(2f64.powi(-60)); (t - u, t + u) };

    // build once: for a fixed side F and role, join the other side to size<=5 and verify em.
    let decomp = |fixed: &[Ext80], num_is_fixed: bool, label: &str| -> bool {
        // target for the searched side S on row 0 (approx): if num fixed, D0 = num0/em0; else N0 = em0*D0
        let f0 = d(&fixed[0]);
        let em0 = f64::from_bits(em_bits[0]);
        let s_target0 = if num_is_fixed { if em0 != 0.0 { f0 / em0 } else { return false } } else { em0 * f0 };
        // enumerate searched side S = op(A,B), A,B in bank; prefilter S via row-0 target, verify em.
        let hit = (0..bank.len()).into_par_iter().find_map_any(|bi| {
            let nb = &bank[bi]; let b0 = d(&nb.v[0]);
            for op in ["div", "mul", "add", "sub"] {
                let a0t: Vec<f64> = match op {
                    "div" => if b0 != 0.0 { vec![s_target0 * b0] } else { vec![] }, // A/B=S -> A=S*B
                    "mul" => if b0 != 0.0 { vec![s_target0 / b0] } else { vec![] },
                    "add" => vec![s_target0 - b0], "sub" => vec![s_target0 + b0, b0 - s_target0], _ => vec![],
                };
                for (ti, t) in a0t.iter().enumerate() {
                    let (lo, hi) = win2(*t);
                    let mut p = keys.partition_point(|&k| k < lo);
                    while p < keys.len() && keys[p] <= hi {
                        let ai = idx[p].1 as usize; let na = &bank[ai];
                        let (aa, bb) = if op == "sub" && ti == 1 { (&nb.v, &na.v) } else { (&na.v, &nb.v) };
                        let s_vec = x87(aa, bb, op); // searched side value (x87)
                        let full = if num_is_fixed { spill(&x87(fixed, &s_vec, "div")) } else { spill(&x87(&s_vec, fixed, "div")) };
                        if bits(&full) == em_bits { return Some((op, ai, bi, ti)); }
                        // also try SSE searched-side op when both double
                        if na.dbl && nb.dbl {
                            let s2 = sse(aa, bb, op);
                            let full2 = if num_is_fixed { spill(&x87(fixed, &s2, "div")) } else { spill(&x87(&s2, fixed, "div")) };
                            if bits(&full2) == em_bits { return Some((op, ai, bi, ti)); }
                        }
                        p += 1;
                    }
                }
            }
            None
        });
        match hit {
            Some((op, ai, bi, _)) => {
                println!("\n>>> DECOMP HIT ({label}): searched side = {}({}, {})", op, "#", "#");
                print!("  A="); report(&bank, ai, m); print!("  B="); report(&bank, bi, m); true
            }
            None => { println!("EXT decomp ({label}): no size<=5 searched-side tree closes em.", ); false }
        }
    };

    println!("\nEXT3/4 interval decomposition (numerator=RN53((u-1)*tau) fixed / denominator=lnu_d fixed):");
    let _ = decomp(&num_sse, true, "fix numerator -> search denominator<=5")
        || decomp(&lnu_dv, false, "fix denominator=lnu_d -> search numerator<=5");

    // ================= EXTENSION 5: STREAMING SIZE-3 ANY-ROOT PASS =================
    // Extend the ANY-ROOT envelope from size 2 to size 3 (incl. transcendental/spill roots),
    // parallel + streaming (generate op over size<=2 pairs, check em, discard — no 23GB storage).
    let bankr = &bank;
    let s2 = &by_size[2]; let s1 = &by_size[1]; let s0 = &by_size[0];
    let em_ref = &em_bits;
    let check = |v: &[Ext80]| bits(v) == *em_ref;
    let apply_all = |a: &Node, b: &Node| -> bool {
        for op in ["add","sub","mul","div"] {
            if check(&spill(&x87(&a.v,&b.v,op))) || check(&x87(&a.v,&b.v,op)) { return true; }
            if a.dbl && b.dbl && check(&sse(&a.v,&b.v,op)) { return true; }
        }
        if a.v.iter().all(|e| d(e)>0.0) && check(&map2(&a.v,&b.v,|y,x|ext_fyl2x(y,x,CW))) { return true; }
        if b.v.iter().all(|e| d(e)>0.0) && check(&map2(&b.v,&a.v,|y,x|ext_fyl2x(y,x,CW))) { return true; }
        if check(&map2(&a.v,&b.v,|x,k|ext_scale(x,k,CW))) || check(&map2(&b.v,&a.v,|x,k|ext_scale(x,k,CW))) { return true; }
        false
    };
    // (size2 x size0) and (size0 x size2)
    let hit3a = s2.par_iter().any(|&ai| {
        let a = &bankr[ai as usize];
        s0.iter().any(|&bi| { let b = &bankr[bi as usize]; apply_all(a,b) })
    });
    // (size1 x size1)
    let hit3b = !hit3a && s1.par_iter().any(|&ai| {
        let a = &bankr[ai as usize];
        s1.iter().any(|&bi| { let b = &bankr[bi as usize]; apply_all(a,b) })
    });
    // unary over size2
    let hit3c = !hit3a && !hit3b && s2.par_iter().any(|&ai| {
        let a = &bankr[ai as usize];
        (!a.dbl && check(&spill(&a.v))) || (!a.dbl && check(&spill_rz(&a.v)))
            || check(&a.v.iter().map(|e|ext_sub(&ext_from_f64(0.0),e,CW)).collect::<Vec<_>>())
            || (a.v.iter().all(|e|d(e).abs()<=1.0) && check(&a.v.iter().map(|e|ext_f2xm1(e,CW)).collect::<Vec<_>>()))
    });
    println!("\nEXT5 streaming size-3 any-root pass: em reached = {}", hit3a||hit3b||hit3c);
    if !(hit3a||hit3b||hit3c) {
        println!("  NO size<=3 DAG with ANY root (arith/transcendental/spill) reproduces em.");
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
