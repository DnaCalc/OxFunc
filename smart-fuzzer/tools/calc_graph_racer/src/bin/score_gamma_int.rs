use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::gamma_kernel;

fn factorial_gamma(n: u32) -> f64 {
    let mut a = 1.0;
    for k in 2..n {
        a *= k as f64;
    }
    a
}

fn main() {
    let excel: &[(f64, u64)] = &[
        (1.0, 0x3ff0000000000000),
        (2.0, 0x3ff0000000000000),
        (3.0, 0x4000000000000000),
        (4.0, 0x4018000000000000),
        (5.0, 0x4038000000000000),
        (6.0, 0x405e000000000000),
        (7.0, 0x4086800000000000),
        (8.0, 0x40b3b00000000000),
        (9.0, 0x40e3b00000000000),
        (10.0, 0x4116260000000000),
        (11.0, 0x414baf8000000000),
        (12.0, 0x418308a800000000),
        (13.0, 0x41bc8cfc00000000),
        (14.0, 0x41f7328cc0000000),
        (15.0, 0x42344c3b28000000),
        (16.0, 0x4273077775800000),
        (17.0, 0x42b3077775800000),
        (18.0, 0x42f437eeecd80000),
        (19.0, 0x4336beecca730000),
        (20.0, 0x437b02b930689000),
        (21.0, 0x43c0e1b3be415a00),
        (22.0, 0x4406283be9b5c620),
        (0.5, 0x3ffc5bf891b4ef6b),
        (1.5, 0x3fec5bf891b4ef6b),
        (8.5, 0x40cb693422315f91),
    ];
    let mut exact = 0;
    for &(x, want) in excel {
        let got = gamma_kernel(x).unwrap().to_bits();
        let ok = got == want;
        if ok {
            exact += 1;
        }
        println!(
            "GAMMA({x}) oxfunc=0x{got:016x} excel=0x{want:016x} exact={ok}"
        );
    }
    println!("exact {exact}/{}", excel.len());
    println!("\n== sequential factorial vs Excel");
    let mut fexact = 0;
    let mut n = 0;
    for &(x, want) in excel {
        if x < 1.0 || x != x.trunc() {
            continue;
        }
        n += 1;
        let got = factorial_gamma(x as u32).to_bits();
        let ok = got == want;
        if ok {
            fexact += 1;
        }
        println!("  n={x} fact=0x{got:016x} excel=0x{want:016x} exact={ok}");
    }
    println!("factorial exact {fexact}/{n}");
    let spi = std::f64::consts::PI.sqrt();
    println!(
        "sqrt(PI)={spi} bits=0x{:016x} excel GAMMA(0.5)=0x3ffc5bf891b4ef6b",
        spi.to_bits()
    );
    let excel_large: &[(u32, u64)] = &[
        (23, 0x444e77526159f06c),
        (24, 0x4495e5c335f8a4ce),
        (25, 0x44e06c52687a7b9a),
        (26, 0x4529a940c33f6120),
        (30, 0x465be6518687a784),
        (40, 0x498c95619f1a8e65),
        (50, 0x4cf7a88e4484be3f),
        (100, 0x605166c698cf183e),
        (140, 0x718d88957d1c3025),
        (170, 0x7f2f2054eb4d96f5),
        (171, 0x7fa4ab786441863a),
    ];
    fn x87_fact(n: u32, store: bool) -> f64 {
        let mut a = rx::ext_from_f64(1.0);
        for k in 2..n {
            a = rx::ext_mul(&a, &rx::ext_from_f64(k as f64), rx::CW_PC64_RN);
            if store {
                a = rx::ext_from_f64(rx::ext_to_f64(&a, rx::CW_PC64_RN));
            }
        }
        rx::ext_to_f64(&a, rx::CW_PC64_RN)
    }
    println!("\n== large-n factorial vs Excel");
    for &(n, want) in excel_large {
        let nativ = factorial_gamma(n).to_bits();
        let cont = x87_fact(n, false).to_bits();
        let st = x87_fact(n, true).to_bits();
        println!(
            "  n={n} native={} x87cont={} x87store={} excel=0x{want:016x}",
            nativ == want,
            cont == want,
            st == want
        );
        if nativ != want && cont != want && st != want {
            println!(
                "    native=0x{nativ:016x} cont=0x{cont:016x} store=0x{st:016x}"
            );
        }
    }
    println!("\n== reverse / pairwise native");
    for &(n, want) in excel_large {
        let mut rev = 1.0;
        for k in (2..n).rev() {
            rev *= k as f64;
        }
        let mut xs: Vec<f64> = (2..n).map(|k| k as f64).collect();
        fn pair(v: &[f64]) -> f64 {
            match v.len() {
                0 => 1.0,
                1 => v[0],
                2 => v[0] * v[1],
                m => pair(&v[..m / 2]) * pair(&v[m / 2..]),
            }
        }
        let p = pair(&xs);
        println!(
            "  n={n} reverse={} pairwise={}",
            rev.to_bits() == want,
            p.to_bits() == want
        );
    }
    let g25 = f64::from_bits(0x44e06c52687a7b9a);
    let rec26 = (g25 * 25.0).to_bits();
    println!(
        "GAMMA(25)*25 = 0x{rec26:016x} excel26=0x4529a940c33f6120 match={}",
        rec26 == 0x4529a940c33f6120
    );
    println!("\n== reverse factorial bits");
    for n in [
        26u32, 27, 28, 29, 30, 40, 50, 55, 60, 65, 70, 75, 80, 85, 86, 87, 88, 89, 90, 95, 99, 100, 110, 120,
        130, 140, 150, 160, 170, 171,
    ] {
        let mut rev = 1.0;
        for k in (2..n).rev() {
            rev *= k as f64;
        }
        println!("  n={n} reverse=0x{:016x}", rev.to_bits());
    }
}
