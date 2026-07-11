//! W109 Phase-3 scratch check: do the raw x87 FSIN/FPTAN instructions
//! reproduce Excel's large-argument trig bits (the 719/5664-ULP recon
//! witnesses)? Hardware FSIN/FCOS/FPTAN reduce internally with the 66-bit π.

use oxfunc_core::excel_numeric::research as rx;

fn f(bits: u64) -> f64 {
    f64::from_bits(bits)
}

fn fsin(x: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_sin(&rx::ext_from_f64(x), rx::CW_PC64_RN), rx::CW_PC64_RN)
}
fn fcos(x: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_cos(&rx::ext_from_f64(x), rx::CW_PC64_RN), rx::CW_PC64_RN)
}
fn fptan(x: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_tan(&rx::ext_from_f64(x), rx::CW_PC64_RN), rx::CW_PC64_RN)
}

/// FPREM1 pre-reduction against 2π (extended FLDPI doubled exactly), then the
/// trig instruction on the small residue — the legacy CRT chain shape.
fn prem1_then(x: f64, op: fn(&rx::Ext80, u16) -> rx::Ext80) -> f64 {
    let cw = rx::CW_PC64_RN;
    let two_pi = rx::ext_add(&rx::ext_pi(), &rx::ext_pi(), cw); // exact doubling
    let r = rx::ext_prem1(&rx::ext_from_f64(x), &two_pi, cw);
    rx::ext_to_f64(&op(&r, cw), cw)
}
/// Same with truncating FPREM.
fn prem_then(x: f64, op: fn(&rx::Ext80, u16) -> rx::Ext80) -> f64 {
    let cw = rx::CW_PC64_RN;
    let two_pi = rx::ext_add(&rx::ext_pi(), &rx::ext_pi(), cw);
    let r = rx::ext_prem(&rx::ext_from_f64(x), &two_pi, cw);
    rx::ext_to_f64(&op(&r, cw), cw)
}

/// π-reduction with quotient-parity sign fixup: sin(x) = (-1)^Q sin(r).
fn sin_pi_parity(x: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let (r, q) = rx::ext_prem1_quo(&rx::ext_from_f64(x), &rx::ext_pi(), cw);
    let mut s = rx::ext_sin(&r, cw);
    if q & 1 == 1 {
        s = rx::ext_chs(&s, cw);
    }
    rx::ext_to_f64(&s, cw)
}
/// cos(x) = (-1)^Q cos(r) after π-reduction.
fn cos_pi_parity(x: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let (r, q) = rx::ext_prem1_quo(&rx::ext_from_f64(x), &rx::ext_pi(), cw);
    let mut c = rx::ext_cos(&r, cw);
    if q & 1 == 1 {
        c = rx::ext_chs(&c, cw);
    }
    rx::ext_to_f64(&c, cw)
}
/// tan has period π: no fixup needed.
fn tan_pi(x: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let (r, _q) = rx::ext_prem1_quo(&rx::ext_from_f64(x), &rx::ext_pi(), cw);
    rx::ext_to_f64(&rx::ext_tan(&r, cw), cw)
}

fn main() {
    let tan_x = 797601.58f64;
    let sin_x = 134217727.0f64;
    println!(
        "TAN prem1+fptan 0x{:016x}  prem+fptan 0x{:016x}  tan_pi 0x{:016x}  excel 0x4023ebd3414768f1",
        prem1_then(tan_x, rx::ext_tan).to_bits(),
        prem_then(tan_x, rx::ext_tan).to_bits(),
        tan_pi(tan_x).to_bits()
    );
    println!(
        "SIN prem1+fsin  0x{:016x}  prem+fsin  0x{:016x}  sin_pi 0x{:016x}  excel 0xbfee977f5248babf",
        prem1_then(sin_x, rx::ext_sin).to_bits(),
        prem_then(sin_x, rx::ext_sin).to_bits(),
        sin_pi_parity(sin_x).to_bits()
    );
    // The residues-near-±π rows that killed the 2π reduction.
    for (x, excel) in [
        (40360944.19700387f64, 0u64), // placeholder — replaced below by live checks
    ] {
        let _ = (x, excel);
    }
    println!("sin_pi(1.5) 0x{:016x} vs platform 0x{:016x}", sin_pi_parity(1.5).to_bits(), (1.5f64).sin().to_bits());
    println!("cos_pi(149.214601836) 0x{:016x}", cos_pi_parity(149.214601836f64).to_bits());

    // Debug the cos quadrant dispatch on the failing near-π/2 rows.
    debug_rows();
}

fn debug_rows() {
    let cw = rx::CW_PC64_RN;
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    // (label, x_bits, excel_cos_bits) — from answers-r1 for COS.
    let rows: &[(&str, u64, u64)] = &[
        ("nearpi2-009", 0x41700a1462204a8a, 0xbf4ee1aec61c9a91),
        ("nearpi2-015", 0x41736be01096754b, 0x3f404417a3e3b499),
        ("nearpi2-043", 0x4190fc7b5add76ae, 0x3f4095446483fdc8),
        ("band-030", 0x4012d97c7f3321d2, 0xbcaa7c0000000000),
        ("bandneg-030", 0xc012d97c7f3321d2, 0xbcaa7c0000000000),
    ];
    for &(label, xb, want) in rows {
        let x = f64::from_bits(xb);
        let (r, q) = rx::ext_prem1_quo(&rx::ext_from_f64(x), &pi_half, cw);
        let rf = rx::ext_to_f64(&r, cw);
        let s = rx::ext_to_f64(&rx::ext_sin(&r, cw), cw);
        let c = rx::ext_to_f64(&rx::ext_cos(&r, cw), cw);
        println!(
            "{label}: q={q} q&3={} r={rf:+.9e} sin=0x{:016x} -sin=0x{:016x} cos=0x{:016x} want 0x{want:016x}",
            q & 3,
            s.to_bits(),
            (-s).to_bits(),
            c.to_bits()
        );
    }
}
