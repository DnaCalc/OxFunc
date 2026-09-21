//! Score production libm erf and a few public small-z graphs against the
//! live 12h small-z ERF.PRECISE capture. No Excel.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::{erf_precise_kernel, erfc_precise_kernel};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Deserialize)]
struct Row {
    z: f64,
    erf_bits: String,
    erfc_bits: String,
}

fn parse_hex(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn sulp(got: u64, want: u64) -> i64 {
    fn ord(v: u64) -> u64 {
        if v >> 63 == 0 {
            v | (1u64 << 63)
        } else {
            !v
        }
    }
    (i128::from(ord(got)) - i128::from(ord(want))) as i64
}

fn hist(name: &str, ds: &[i64]) {
    let mut c: BTreeMap<i64, usize> = BTreeMap::new();
    let mut exact = 0;
    let mut max = 0u64;
    for &d in ds {
        if d == 0 {
            exact += 1;
        }
        max = max.max(d.unsigned_abs());
        *c.entry(d.clamp(-8, 8)).or_default() += 1;
    }
    println!("{name:<42} {exact}/{n} max={max} hist={c:?}", n = ds.len());
}

fn native_nswc(x: f64) -> f64 {
    const A: [f64; 5] = [
        0.771058495001320e-04,
        -0.133733772997339e-02,
        0.323076579225834e-01,
        0.479137145607681e-01,
        0.128379167095513e+00,
    ];
    const B: [f64; 3] = [
        0.301048631703895e-02,
        0.538971687740286e-01,
        0.375795757275549e+00,
    ];
    let t = x * x;
    let mut top = 0.0;
    for &c in &A {
        top = top * t + c;
    }
    top += 1.0;
    let mut bot = 0.0;
    for &c in &B {
        bot = bot * t + c;
    }
    bot = bot * t + 1.0;
    x * (top / bot)
}

fn x87_nswc(x: f64) -> f64 {
    use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, CW_PC64_RN};
    const A: [f64; 5] = [
        0.771058495001320e-04,
        -0.133733772997339e-02,
        0.323076579225834e-01,
        0.479137145607681e-01,
        0.128379167095513e+00,
    ];
    const B: [f64; 3] = [
        0.301048631703895e-02,
        0.538971687740286e-01,
        0.375795757275549e+00,
    ];
    let t = ext_mul(&ext_from_f64(x), &ext_from_f64(x), CW_PC64_RN);
    let mut top = ext_from_f64(0.0);
    for &c in &A {
        top = ext_add(&ext_mul(&top, &t, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
    }
    top = ext_add(&top, &ext_from_f64(1.0), CW_PC64_RN);
    let mut bot = ext_from_f64(0.0);
    for &c in &B {
        bot = ext_add(&ext_mul(&bot, &t, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
    }
    bot = ext_add(&ext_mul(&bot, &t, CW_PC64_RN), &ext_from_f64(1.0), CW_PC64_RN);
    let r = ext_div(&top, &bot, CW_PC64_RN);
    ext_to_f64(&ext_mul(&ext_from_f64(x), &r, CW_PC64_RN), CW_PC64_RN)
}

const FDLIBM_P: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_Q: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];

fn native_fdlibm(x: f64) -> f64 {
    let z = x * x;
    let mut r = 0.0;
    for &c in FDLIBM_P.iter().rev() {
        r = r * z + c;
    }
    let mut q = 0.0;
    for &c in FDLIBM_Q.iter().rev() {
        q = q * z + c;
    }
    let s = 1.0 + z * q;
    x + x * (r / s)
}

fn native_fdlibm_x_times(x: f64) -> f64 {
    let z = x * x;
    let mut r = 0.0;
    for &c in FDLIBM_P.iter().rev() {
        r = r * z + c;
    }
    let mut q = 0.0;
    for &c in FDLIBM_Q.iter().rev() {
        q = q * z + c;
    }
    let s = 1.0 + z * q;
    x * (1.0 + r / s)
}

fn x87_fdlibm(x: f64, spill: bool) -> f64 {
    use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, CW_PC64_RN};
    let st = |v: rx::Ext80| {
        if spill {
            ext_from_f64(ext_to_f64(&v, CW_PC64_RN))
        } else {
            v
        }
    };
    let z = st(ext_mul(&ext_from_f64(x), &ext_from_f64(x), CW_PC64_RN));
    let mut r = ext_from_f64(0.0);
    for &c in FDLIBM_P.iter().rev() {
        r = st(ext_add(
            &ext_mul(&r, &z, CW_PC64_RN),
            &ext_from_f64(c),
            CW_PC64_RN,
        ));
    }
    let mut q = ext_from_f64(0.0);
    for &c in FDLIBM_Q.iter().rev() {
        q = st(ext_add(
            &ext_mul(&q, &z, CW_PC64_RN),
            &ext_from_f64(c),
            CW_PC64_RN,
        ));
    }
    let s = st(ext_add(
        &ext_from_f64(1.0),
        &ext_mul(&z, &q, CW_PC64_RN),
        CW_PC64_RN,
    ));
    let ratio = st(ext_div(&r, &s, CW_PC64_RN));
    let xr = st(ext_mul(&ext_from_f64(x), &ratio, CW_PC64_RN));
    ext_to_f64(&st(ext_add(&ext_from_f64(x), &xr, CW_PC64_RN)), CW_PC64_RN)
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/erf-smallz/capture.jsonl".into()
    });
    let text = fs::read_to_string(&path).expect("capture");
    let rows: Vec<Row> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("row"))
        .filter(|r: &Row| r.z <= 0.5)
        .collect();
    println!("rows {}", rows.len());
    let want: Vec<u64> = rows.iter().map(|r| parse_hex(&r.erf_bits)).collect();
    let zs: Vec<f64> = rows.iter().map(|r| r.z).collect();

    let score = |name: &str, eval: fn(f64) -> f64| {
        let ds: Vec<i64> = zs
            .iter()
            .zip(want.iter())
            .map(|(&z, &w)| sulp(eval(z).to_bits(), w))
            .collect();
        hist(name, &ds);
    };

    score("production erf_precise_kernel", |z| {
        erf_precise_kernel(z).unwrap()
    });
    score("libm::erf", libm::erf);
    score("native nswc x*(top/bot)", native_nswc);
    score("x87-continuous nswc", x87_nswc);
    score("native fdlibm x+x*(r/s)", native_fdlibm);
    score("native fdlibm x*(1+r/s)", native_fdlibm_x_times);
    score("x87-continuous fdlibm", |z| x87_fdlibm(z, false));
    score("x87-spill fdlibm", |z| x87_fdlibm(z, true));
    score("nextafter +inf libm.erf", |z| libm::erf(z).next_up());
    score("nextafter -inf libm.erf", |z| libm::erf(z).next_down());

    let want_q: Vec<u64> = rows.iter().map(|r| parse_hex(&r.erfc_bits)).collect();
    let ds_q: Vec<i64> = zs
        .iter()
        .zip(want_q.iter())
        .map(|(&z, &w)| sulp(erfc_precise_kernel(z).unwrap().to_bits(), w))
        .collect();
    hist("production erfc vs Excel ERFC", &ds_q);
    let ds_1m: Vec<i64> = zs
        .iter()
        .zip(want_q.iter())
        .map(|(&z, &w)| {
            let p = erf_precise_kernel(z).unwrap();
            sulp((1.0 - p).to_bits(), w)
        })
        .collect();
    hist("1-prod_erf vs Excel ERFC", &ds_1m);

    // Mixed fdlibm: x87 only on t=x*x, or PC53 Horner.
    let fd_t_x87 = |x: f64| {
        let t = rx::ext_to_f64(
            &rx::ext_mul(&rx::ext_from_f64(x), &rx::ext_from_f64(x), rx::CW_PC64_RN),
            rx::CW_PC64_RN,
        );
        let mut r = 0.0;
        for &c in FDLIBM_P.iter().rev() {
            r = r * t + c;
        }
        let mut q = 0.0;
        for &c in FDLIBM_Q.iter().rev() {
            q = q * t + c;
        }
        x + x * (r / (1.0 + t * q))
    };
    let fd_pc53 = |x: f64| {
        let t = rx::ext_to_f64(
            &rx::ext_mul(&rx::ext_from_f64(x), &rx::ext_from_f64(x), rx::CW_PC53_RN),
            rx::CW_PC53_RN,
        );
        let mut r = rx::ext_from_f64(0.0);
        for &c in FDLIBM_P.iter().rev() {
            r = rx::ext_add(
                &rx::ext_mul(&r, &rx::ext_from_f64(t), rx::CW_PC53_RN),
                &rx::ext_from_f64(c),
                rx::CW_PC53_RN,
            );
        }
        let mut q = rx::ext_from_f64(0.0);
        for &c in FDLIBM_Q.iter().rev() {
            q = rx::ext_add(
                &rx::ext_mul(&q, &rx::ext_from_f64(t), rx::CW_PC53_RN),
                &rx::ext_from_f64(c),
                rx::CW_PC53_RN,
            );
        }
        let s = rx::ext_add(
            &rx::ext_from_f64(1.0),
            &rx::ext_mul(&rx::ext_from_f64(t), &q, rx::CW_PC53_RN),
            rx::CW_PC53_RN,
        );
        let ratio = rx::ext_div(&r, &s, rx::CW_PC53_RN);
        let xr = rx::ext_mul(&rx::ext_from_f64(x), &ratio, rx::CW_PC53_RN);
        rx::ext_to_f64(
            &rx::ext_add(&rx::ext_from_f64(x), &xr, rx::CW_PC53_RN),
            rx::CW_PC53_RN,
        )
    };
    score("fdlibm t=x87 x*x rest native", fd_t_x87);
    score("fdlibm PC53 horner+final", fd_pc53);
}
