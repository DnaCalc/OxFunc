//! 0x210 leftover-low F+2 misses: z, kind, ulps. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn kind(z: f64) -> &'static str {
    let m = z.to_bits() & ((1u64 << 52) - 1);
    let hex = format!("{m:013x}");
    if m.trailing_zeros() >= 20 {
        "dyad"
    } else if hex.contains("555") || hex.contains("aaa") {
        "555"
    } else {
        "other"
    }
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
fn cody_d73(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut d = D0;
    d[7] = poke(D0[7], -3);
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let mut lows: Vec<(f64, u64)> = Vec::new();
    for r in &dirs {
        let g = mul(f::w_rn53(r.z), cody(r.z));
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lows.push((r.z, r.qbits));
        }
    }
    println!("0x210 leftover-low n={}", lows.len());
    println!("F+2 misses:");
    let mut n_d = 0usize;
    let mut n_5 = 0usize;
    let mut n_o = 0usize;
    let mut misses: Vec<(f64, u64)> = Vec::new();
    for &(lz, bits) in &lows {
        let t = f64::from_bits(bits);
        let cr = mul(f::w_rn53(lz), cody(lz));
        let f2 = mul(f::w_rn53(lz), cody(lz).next_up().next_up());
        let d0 = ulp_distance(cr, t).unwrap_or(99);
        let d2 = ulp_distance(f2, t).unwrap_or(99);
        if d2 == 0 {
            continue;
        }
        misses.push((lz, bits));
        let k = kind(lz);
        match k {
            "dyad" => n_d += 1,
            "555" => n_5 += 1,
            _ => n_o += 1,
        }
        let s0 = if cr < t { "L" } else { "H" };
        let s2 = if f2 < t { "L" } else { "H" };
        println!("  z={lz:.16} kind={k} CR={d0}{s0} F+2={d2}{s2}");
    }
    println!(
        "F+2 misses n={} dyad={n_d} 555={n_5} other={n_o}",
        misses.len()
    );
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        (
            "F+3",
            Box::new(|z| mul(f::w_rn53(z), cody(z).next_up().next_up().next_up())),
        ),
        (
            "F+4",
            Box::new(|z| {
                mul(
                    f::w_rn53(z),
                    cody(z).next_up().next_up().next_up().next_up(),
                )
            }),
        ),
        ("D73", Box::new(|z| mul(f::w_rn53(z), cody_d73(z)))),
        (
            "upD73",
            Box::new(|z| mul(f::w_rn53(z), cody_d73(z).next_up())),
        ),
        ("cephes_f", Box::new(|z| mul(f::w_rn53(z), f::cephes_f(z)))),
    ];
    println!("F+2-miss graphs:");
    for &(lz, bits) in &misses {
        let t = f64::from_bits(bits);
        print!("  z={lz:.16}");
        for (name, ev) in &graphs {
            let g = ev(lz);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t {
                "L"
            } else if g > t {
                "H"
            } else {
                "="
            };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut dmid = 0usize;
        let mut hit_m = 0usize;
        for r in &dirs {
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        for &(lz, bits) in &misses {
            if ulp_distance(ev(lz), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit_m += 1;
            }
        }
        println!(
            "{name:10} F+2-miss hit={hit_m}/{} dmid={dmid}/{}",
            misses.len(),
            dirs.len()
        );
    }
}
