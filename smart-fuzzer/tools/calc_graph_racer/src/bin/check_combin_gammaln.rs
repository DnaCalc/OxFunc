//! W109 scratch check: does COMBIN publish exp(gammaln(n+1) - gammaln(k+1)
//! - gammaln(n-k+1)) composed from Excel's PUBLISHED GAMMALN values and the
//! identified x87 EXP? Tests sub staging/order variants per witness.

use oxfunc_core::excel_numeric::research as rx;

fn dr_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}
fn dr_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}
/// Extended-continuous (g1 - g2 - g3) with one final store.
fn ext_sub3(a: f64, b: f64, c: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let t = rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), cw);
    let t = rx::ext_sub(&t, &rx::ext_from_f64(c), cw);
    rx::ext_to_f64(&t, cw)
}

fn main() {
    let g = |bits: u64| f64::from_bits(bits);
    // Published GAMMALN bits (live build 20131).
    let g4 = g(0x3ffcab0bfa2a2002);
    let g11 = g(0x402e357590954d16);
    let g14 = g(0x40368d5a9c3b32cd);
    let g24 = g(0x4049cda78b856a45);
    let g42 = g(0x405c8230869ca104);
    let g62 = g(0x406817a6467f6fba);
    let g132 = g(0x407ff020dc5fcd0b);
    let g198 = g(0x408a7ad118bda02b);
    let g201 = g(0x408af9db1c19e3c7);
    let g347 = g(0x409a42d74295b6a3);
    let g478 = g(0x40a349d10485f9cc);

    // (name, gn1, gk1, gnk1, excel_combin_bits)
    let rows = [
        ("COMBIN(23,10)", g24, g11, g14, 0x4131750200000001u64),
        ("COMBIN(200,3)", g201, g4, g198, 0x41340a77ffffffffu64),
        ("COMBIN(477,346)", g478, g347, g132, 0x58eddeca17662d6bu64),
    ];
    for (name, a, b, c, want) in rows {
        let variants = [
            ("strict (a-b)-c", (a - b) - c),
            ("strict a-(b+c)", a - (b + c)),
            ("dr (a-b)-c", dr_sub(dr_sub(a, b), c)),
            ("dr a-(b+c)", dr_sub(a, dr_add(b, c))),
            ("ext (a-b)-c", ext_sub3(a, b, c)),
        ];
        for (vn, t) in variants {
            let v = rx::excel_exp(t);
            let mark = if v.to_bits() == want { "  <-- MATCH" } else { "" };
            println!("{name} {vn:16} exp -> 0x{:016x} want 0x{want:016x}{mark}", v.to_bits());
        }
        println!();
    }
    // PERMUT(61,20) via exp(g62 - g42) for comparison with the loop result.
    let p = rx::excel_exp(g62 - g42);
    println!(
        "PERMUT(61,20) exp(g62-g42) -> 0x{:016x} want 0x470760c0a63908aa",
        p.to_bits()
    );
}
