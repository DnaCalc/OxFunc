//! W109: race the fdlibm __ieee754_lgamma_r structure against all live
//! GAMMALN rows, under evaluation modes {plain double, FMA-contracted,
//! x87 statement-spill, full extended} x log implementations
//! {fdlibm log (same mode), fdlibm log plain, x87 fyl2x, platform}.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = 0x133F;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Plain,
    Fma,
    Spill,
    Ext,
}

#[derive(Clone, Copy)]
struct V {
    e: Ext80,
    mode: Mode,
}
impl V {
    fn new(x: f64, mode: Mode) -> V {
        V {
            e: rx::ext_from_f64(x),
            mode,
        }
    }
    fn f(self) -> f64 {
        rx::ext_to_f64(&self.e, CW)
    }
    /// per-op narrowing in Plain/Fma modes
    fn op(self) -> V {
        match self.mode {
            Mode::Plain | Mode::Fma => V::new(self.f(), self.mode),
            _ => self,
        }
    }
    /// per-assignment narrowing in Spill mode (and Plain/Fma trivially)
    fn st(self) -> V {
        match self.mode {
            Mode::Ext => self,
            _ => V::new(self.f(), self.mode),
        }
    }
    fn add(self, o: V) -> V {
        V {
            e: rx::ext_add(&self.e, &o.e, CW),
            mode: self.mode,
        }
        .op()
    }
    fn sub(self, o: V) -> V {
        V {
            e: rx::ext_sub(&self.e, &o.e, CW),
            mode: self.mode,
        }
        .op()
    }
    fn mul(self, o: V) -> V {
        V {
            e: rx::ext_mul(&self.e, &o.e, CW),
            mode: self.mode,
        }
        .op()
    }
    fn div(self, o: V) -> V {
        V {
            e: rx::ext_div(&self.e, &o.e, CW),
            mode: self.mode,
        }
        .op()
    }
    fn neg(self) -> V {
        V::new(0.0, self.mode).sub(self)
    }
    /// self*b + c with FMA contraction in Fma mode
    fn ma(self, b: V, c: V) -> V {
        if self.mode == Mode::Fma {
            V::new(self.f().mul_add(b.f(), c.f()), self.mode)
        } else {
            self.mul(b).add(c)
        }
    }
}

fn horner(x: V, cs: &[f64]) -> V {
    let mut r = V::new(cs[0], x.mode);
    for &c in &cs[1..] {
        r = r.ma(x, V::new(c, x.mode));
    }
    r
}

const LN2_HI: f64 = 6.93147180369123816490e-01;
const LN2_LO: f64 = 1.90821492927058770002e-10;
const LG: [f64; 7] = [
    6.666666666666735130e-01,
    3.999999999940941908e-01,
    2.857142874366239149e-01,
    2.222219843214978396e-01,
    1.818357216161805012e-01,
    1.531383769920937332e-01,
    1.479819860511658591e-01,
];

fn fdlibm_log(x0: f64, mode: Mode) -> f64 {
    let bits = x0.to_bits();
    let mut hx = (bits >> 32) as i32;
    let k0 = (hx >> 20) - 1023;
    hx &= 0x000fffff;
    let i = (hx + 0x95f64) & 0x100000;
    let xb = (((hx | (i ^ 0x3ff00000)) as u64) << 32) | (bits & 0xffffffff);
    let x = f64::from_bits(xb);
    let k = k0 + (i >> 20);
    let m = mode;
    let f = V::new(x, m).sub(V::new(1.0, m)).st();
    if (0x000fffff & (2 + hx)) < 3 {
        if f.f() == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = k as f64;
            return V::new(dk * LN2_HI, m)
                .add(V::new(dk, m).mul(V::new(LN2_LO, m)))
                .f();
        }
        let r = f
            .mul(f)
            .mul(V::new(0.5, m).sub(V::new(0.33333333333333333, m).mul(f)))
            .st();
        if k == 0 {
            return f.sub(r).f();
        }
        let dk = V::new(k as f64, m);
        return dk
            .mul(V::new(LN2_HI, m))
            .st()
            .sub(r.sub(dk.mul(V::new(LN2_LO, m))).st().sub(f).st())
            .f();
    }
    let s = f.div(V::new(2.0, m).add(f)).st();
    let dk = V::new(k as f64, m);
    let z = s.mul(s).st();
    let ii = hx - 0x6147a;
    let w = z.mul(z).st();
    let j = 0x6b851 - hx;
    let t1 = w.mul(horner(w, &[LG[5], LG[3], LG[1]])).st();
    let t2 = z.mul(horner(w, &[LG[6], LG[4], LG[2], LG[0]])).st();
    let r = t2.add(t1).st();
    if (ii | j) > 0 {
        let hfsq = V::new(0.5, m).mul(f).mul(f).st();
        if k == 0 {
            return f.sub(hfsq.sub(s.mul(hfsq.add(r))).st()).f();
        }
        dk.mul(V::new(LN2_HI, m))
            .st()
            .sub(
                hfsq.sub(s.mul(hfsq.add(r)).add(dk.mul(V::new(LN2_LO, m))))
                    .st()
                    .sub(f)
                    .st(),
            )
            .f()
    } else {
        if k == 0 {
            return f.sub(s.mul(f.sub(r))).f();
        }
        dk.mul(V::new(LN2_HI, m))
            .st()
            .sub(
                s.mul(f.sub(r))
                    .sub(dk.mul(V::new(LN2_LO, m)))
                    .st()
                    .sub(f)
                    .st(),
            )
            .f()
    }
}

const A: [f64; 12] = [
    7.72156649015328655494e-02,
    3.22467033424113591611e-01,
    6.73523010531292681824e-02,
    2.05808084325167332806e-02,
    7.38555086081402883957e-03,
    2.89051383673415629091e-03,
    1.19270763183362067845e-03,
    5.10069792153511336608e-04,
    2.20862790713908385557e-04,
    1.08011567247583939954e-04,
    2.52144565451257326939e-05,
    4.48640949618915160150e-05,
];
const TC: f64 = 1.46163214496836224576e+00;
const TF: f64 = -1.21486290535849611461e-01;
const TT: f64 = -3.63867699703950536541e-18;
const T: [f64; 15] = [
    4.83836122723810047042e-01,
    -1.47587722994593911752e-01,
    6.46249402391333854778e-02,
    -3.27885410759859649565e-02,
    1.79706750811820387126e-02,
    -1.03142241298341437450e-02,
    6.10053870246291332635e-03,
    -3.68452016781138256760e-03,
    2.25964780900612472250e-03,
    -1.40346469989232843813e-03,
    8.81081882437654011382e-04,
    -5.38595305356740546715e-04,
    3.15632070903625950361e-04,
    -3.12754168375120860518e-04,
    3.35529192635519073543e-04,
];
const U: [f64; 6] = [
    -7.72156649015328655494e-02,
    6.32827064025093366517e-01,
    1.45492250137234768737e+00,
    9.77717527963372745603e-01,
    2.28963728064692451092e-01,
    1.33810918536787660377e-02,
];
const VC: [f64; 5] = [
    2.45597793713041134822e+00,
    2.12848976379893395361e+00,
    7.69285150456672783825e-01,
    1.04222645593369134254e-01,
    3.21709242282423911810e-03,
];
const S: [f64; 7] = [
    -7.72156649015328655494e-02,
    2.14982415960608852501e-01,
    3.25778796408930981787e-01,
    1.46350472652464452805e-01,
    2.66422703033638609560e-02,
    1.84028451407337715652e-03,
    3.19475326584100867617e-05,
];
const RC: [f64; 6] = [
    1.39200533467621045958e+00,
    7.21935547567138069525e-01,
    1.71933865632803078993e-01,
    1.86459191715652901344e-02,
    7.77942496381893596434e-04,
    7.32668430744625636189e-06,
];
const W: [f64; 7] = [
    4.18938533204672725052e-01,
    8.33333333333329678849e-02,
    -2.77777777728775536470e-03,
    7.93650558643019558500e-04,
    -5.95187557450339963135e-04,
    8.36339918996282139126e-04,
    -1.63092934096575273989e-03,
];

fn lgamma_pos(x: f64, mode: Mode, log: &dyn Fn(f64) -> f64) -> f64 {
    let m = mode;
    let bits = x.to_bits();
    let ix = ((bits >> 32) & 0x7fffffff) as i64;
    let lx = bits & 0xffffffff;
    if ix < 0x3b900000 {
        return -log(x);
    }
    if ((ix - 0x3ff00000) | lx as i64) == 0 || ((ix - 0x40000000) | lx as i64) == 0 {
        return 0.0;
    }
    if ix < 0x40000000 {
        let (mut r, y, i) = if ix <= 0x3feccccc {
            let r = V::new(-log(x), m);
            if ix >= 0x3FE76944 {
                (r, V::new(1.0, m).sub(V::new(x, m)).st(), 0)
            } else if ix >= 0x3FCDA661 {
                (r, V::new(x, m).sub(V::new(TC - 1.0, m)).st(), 1)
            } else {
                (r, V::new(x, m), 2)
            }
        } else {
            let r = V::new(0.0, m);
            if ix >= 0x3FFBB4C3 {
                (r, V::new(2.0, m).sub(V::new(x, m)).st(), 0)
            } else if ix >= 0x3FF3B4C4 {
                (r, V::new(x, m).sub(V::new(TC, m)).st(), 1)
            } else {
                (r, V::new(x, m).sub(V::new(1.0, m)).st(), 2)
            }
        };
        match i {
            0 => {
                let z = y.mul(y).st();
                let p1 = horner(z, &[A[10], A[8], A[6], A[4], A[2], A[0]]).st();
                let p2 = z
                    .mul(horner(z, &[A[11], A[9], A[7], A[5], A[3], A[1]]))
                    .st();
                let p = y.ma(p1, p2).st();
                r = r.add(p.sub(V::new(0.5, m).mul(y))).st();
            }
            1 => {
                let z = y.mul(y).st();
                let w = z.mul(y).st();
                let p1 = horner(w, &[T[12], T[9], T[6], T[3], T[0]]).st();
                let p2 = horner(w, &[T[13], T[10], T[7], T[4], T[1]]).st();
                let p3 = horner(w, &[T[14], T[11], T[8], T[5], T[2]]).st();
                let p = z.ma(p1, V::new(TT, m).sub(w.mul(y.ma(p3, p2))).neg()).st();
                r = r.add(V::new(TF, m).add(p)).st();
            }
            _ => {
                let p1 = y.mul(horner(y, &[U[5], U[4], U[3], U[2], U[1], U[0]])).st();
                let p2 = horner(y, &[VC[4], VC[3], VC[2], VC[1], VC[0], 1.0]).st();
                r = r.add(V::new(-0.5, m).mul(y).add(p1.div(p2))).st();
            }
        }
        return r.f();
    }
    if ix < 0x40200000 {
        let i = x as i32;
        let y = V::new(x - i as f64, m);
        let p = y
            .mul(horner(y, &[S[6], S[5], S[4], S[3], S[2], S[1], S[0]]))
            .st();
        let q = horner(y, &[RC[5], RC[4], RC[3], RC[2], RC[1], RC[0], 1.0]).st();
        let mut r = V::new(0.5, m).mul(y).add(p.div(q)).st();
        if i >= 3 {
            let mut z = V::new(1.0, m);
            for k in (2..i).rev() {
                z = z.mul(y.add(V::new(k as f64, m))).st();
            }
            r = r.add(V::new(log(z.f()), m)).st();
        }
        return r.f();
    }
    if ix < 0x43900000 {
        let t = V::new(log(x), m);
        let z = V::new(1.0, m).div(V::new(x, m)).st();
        let y = z.mul(z).st();
        let w = V::new(W[0], m)
            .add(z.mul(horner(y, &[W[6], W[5], W[4], W[3], W[2], W[1]])))
            .st();
        return V::new(x, m)
            .sub(V::new(0.5, m))
            .mul(t.sub(V::new(1.0, m)))
            .add(w)
            .f();
    }
    V::new(x, m).mul(V::new(log(x) - 1.0, m)).f()
}

fn region(x: f64) -> &'static str {
    let ix = ((x.to_bits() >> 32) & 0x7fffffff) as i64;
    if ix < 0x3b900000 {
        return "tiny";
    }
    if x == 1.0 || x == 2.0 {
        return "exact12";
    }
    if ix < 0x40000000 {
        if ix <= 0x3feccccc {
            if ix >= 0x3FE76944 {
                "lo-i0"
            } else if ix >= 0x3FCDA661 {
                "lo-i1"
            } else {
                "lo-i2"
            }
        } else if ix >= 0x3FFBB4C3 {
            "hi-i0"
        } else if ix >= 0x3FF3B4C4 {
            "hi-i1"
        } else {
            "hi-i2"
        }
    } else if ix < 0x40200000 {
        "mid"
    } else {
        "asym"
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
                rows.push((x, v));
            }
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows.dedup_by(|a, b| a.0.to_bits() == b.0.to_bits());
    println!("{} rows", rows.len());

    let modes = [
        (Mode::Plain, "plain"),
        (Mode::Fma, "fma"),
        (Mode::Spill, "spill"),
        (Mode::Ext, "ext"),
    ];
    for (mode, mname) in modes {
        let logs: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
            ("fdlog-same", Box::new(move |x| fdlibm_log(x, mode))),
            ("fdlog-plain", Box::new(|x| fdlibm_log(x, Mode::Plain))),
            ("fyl2x", Box::new(rx::excel_ln)),
            ("platform", Box::new(f64::ln)),
        ];
        for (lname, log) in &logs {
            let (mut exact, mut max_ulp) = (0u32, 0u64);
            let mut regmiss: std::collections::BTreeMap<&str, u32> = Default::default();
            for &(x, want) in &rows {
                let v = lgamma_pos(x, mode, log);
                if v.to_bits() == want.to_bits() {
                    exact += 1;
                } else {
                    *regmiss.entry(region(x)).or_default() += 1;
                    max_ulp = max_ulp.max(ulp_distance(v, want).unwrap_or(u64::MAX));
                }
            }
            println!(
                "{mname:6}+{lname:11} {exact}/{} max {max_ulp}  miss {regmiss:?}",
                rows.len()
            );
        }
    }
}
