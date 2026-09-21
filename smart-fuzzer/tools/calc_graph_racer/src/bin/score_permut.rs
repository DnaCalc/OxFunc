use oxfunc_core::functions::permut_fn::permut_kernel;

fn main() {
    let rows = [
        (10.0, 3.0, 0x4086800000000000u64),
        (25.0, 5.0, 0x4158522c00000000),
        (50.0, 10.0, 0x43608dcba2e124c0),
        (88.0, 5.0, 0x41f1834f1c000000),
        (100.0, 3.0, 0x412d9bb000000000),
        (170.0, 2.0, 0x40dc0e8000000000),
        (20.0, 20.0, 0x43c0e1b3be415a00),
    ];
    let mut e = 0;
    for &(n, k, want) in &rows {
        let got = permut_kernel(n, k).unwrap().to_bits();
        let ok = got == want;
        if ok {
            e += 1;
        }
        println!("PERMUT({n},{k}) ox=0x{got:016x} xl=0x{want:016x} {ok}");
    }
    println!("exact {e}/{}", rows.len());
}
