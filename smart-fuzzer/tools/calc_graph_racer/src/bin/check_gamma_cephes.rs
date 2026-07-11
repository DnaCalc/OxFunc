//! W109: race Cephes gamma() against 156 live worksheet GAMMA rows.
//! Positive small path: recurrence into [2,3) + P/Q rational.
//! Large path: stirf (exp/pow based) — with Excel's x87 exp/pow chains.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const P: [f64; 7] = [
    1.60119522476751861407E-4,
    1.19135147006586384913E-3,
    1.04213797561761569935E-2,
    4.76367800457137231464E-2,
    2.07448227648435975150E-1,
    4.94214826801497100753E-1,
    9.99999999999999996796E-1,
];
const Q: [f64; 8] = [
    -2.31581873324120129819E-5,
    5.39605580493303397842E-4,
    -4.45641913851797240494E-3,
    1.18139785222060435552E-2,
    3.58236398605498653373E-2,
    -2.34591795718243348568E-1,
    7.14304917030273074085E-2,
    1.00000000000000000320E0,
];
const STIR: [f64; 5] = [
    7.87311395793093628397E-4,
    -2.29549961613378126380E-4,
    -2.68132617805781232825E-3,
    3.47222221605458667310E-3,
    8.33333333333482257126E-2,
];
const MAXSTIR: f64 = 143.01608;
const SQTPI: f64 = 2.50662827463100050242;
const CW: u16 = 0x133F;

#[derive(Clone, Copy)]
struct V(Ext80);
impl V {
    fn new(x: f64) -> V { V(rx::ext_from_f64(x)) }
    fn store(self, yes: bool) -> V {
        if yes { V::new(rx::ext_to_f64(&self.0, CW)) } else { self }
    }
    fn f(self) -> f64 { rx::ext_to_f64(&self.0, CW) }
    fn add(self, o: V) -> V { V(rx::ext_add(&self.0, &o.0, CW)) }
    fn mul(self, o: V) -> V { V(rx::ext_mul(&self.0, &o.0, CW)) }
    fn div(self, o: V) -> V { V(rx::ext_div(&self.0, &o.0, CW)) }
}

fn polevl(x: V, c: &[f64], step_store: bool) -> V {
    let mut r = V::new(c[0]);
    for k in &c[1..] {
        r = r.mul(x).add(V::new(*k)).store(step_store);
    }
    r
}

/// stirf(x) for x > 0: mask bit4 w store, bit5 y=pow/exp staging store,
/// bit6 final products stored.  Uses Excel x87 exp/pow.
fn stirf(x: f64, m: u32) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    let xv = V::new(x);
    let w = V::new(1.0).div(xv).store(true); // w = 1/x is a stored variable
    let w = V::new(1.0).add(w.mul(polevl(w, &STIR, bit(3)))).store(bit(4));
    let y = rx::excel_exp(x);
    let y = if x > MAXSTIR {
        let v = rx::excel_pow_positive(x, 0.5 * x - 0.25);
        let yy = V::new(v).div(V::new(y)).store(bit(5));
        V::new(v).mul(yy).store(bit(5)).f()
    } else {
        let p = rx::excel_pow_positive(x, x - 0.5);
        V::new(p).div(V::new(y)).store(bit(5)).f()
    };
    V::new(SQTPI).mul(V::new(y)).store(bit(6)).mul(w).f()
}

/// Cephes gamma, positive x. mask: bit0 z-loop stores, bit1 p/q polys steps,
/// bit2 z*p/q staging store, bit3 STIR poly steps, bit4..6 stirf stores,
/// bit7 z*p/q association z*(p/q) vs (z*p)/q.
fn gamma_pos(x0: f64, m: u32) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    if x0 > 33.0 {
        return stirf(x0, m);
    }
    let mut x = x0;
    let mut z = V::new(1.0);
    while x >= 3.0 {
        x -= 1.0;
        z = z.mul(V::new(x)).store(bit(0));
    }
    while x < 2.0 {
        if x < 1.0e-9 {
            // small: gamma ~ 1/(x*(1+0.5772*x))  — not exercised by corpus
            return f64::NAN;
        }
        z = z.div(V::new(x)).store(bit(0));
        x += 1.0;
    }
    if x == 2.0 {
        return z.f();
    }
    x -= 2.0;
    let xv = V::new(x);
    let p = polevl(xv, &P, bit(1));
    let q = polevl(xv, &Q, bit(1));
    if bit(7) {
        z.mul(p.div(q).store(bit(2))).f()
    } else {
        z.mul(p).store(bit(2)).div(q).f()
    }
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let mut rows: Vec<(f64, f64)> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                if x > 0.0 {
                    rows.push((x, v));
                }
            }
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows.dedup_by(|a, b| a.0.to_bits() == b.0.to_bits());
    println!("{} positive rows", rows.len());

    let mut results: Vec<(u32, u32, u64)> = Vec::new();
    for m in 0u32..256 {
        let (mut exact, mut max_ulp) = (0u32, 0u64);
        for &(x, want) in &rows {
            let v = gamma_pos(x, m);
            if v.to_bits() == want.to_bits() {
                exact += 1;
            } else {
                max_ulp = max_ulp.max(ulp_distance(v, want).unwrap_or(u64::MAX));
            }
        }
        results.push((m, exact, max_ulp));
    }
    results.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    for (m, exact, max_ulp) in results.iter().take(8) {
        println!("mask {m:08b}: {exact}/{} max_ulp {max_ulp}", rows.len());
    }
    let (m, _, _) = results[0];
    let (mut sm_ex, mut sm_n, mut st_ex, mut st_n) = (0, 0, 0, 0);
    let mut misses = 0;
    for &(x, want) in &rows {
        let v = gamma_pos(x, m);
        let (ex, n) = if x > 33.0 { (&mut st_ex, &mut st_n) } else { (&mut sm_ex, &mut sm_n) };
        *n += 1;
        if v.to_bits() == want.to_bits() {
            *ex += 1;
        } else if misses < 15 {
            misses += 1;
            println!(
                "  MISS x={x:.9e} got {:016x} want {:016x} ulp {}",
                v.to_bits(),
                want.to_bits(),
                ulp_distance(v, want).unwrap_or(u64::MAX)
            );
        }
    }
    println!("winner small(<=33): {sm_ex}/{sm_n}  stirf(>33): {st_ex}/{st_n}");
}
