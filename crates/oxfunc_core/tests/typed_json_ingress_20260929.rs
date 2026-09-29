//! Protect decimal-encoded live evidence used by Rust regression replays.
#[test]
fn decimal_json_evidence_keeps_its_original_source_bits() {
    for (decimal, bits) in [
        ("8.76062371711073e-303", 0x013807eb4fb3ba95_u64),
        ("1.9737081258726104e+244", 0x72a71fed759c9875),
        ("6.585554626534559e+304", 0x7f38020efc84768a),
        ("1.8574852041111743e+171", 0x637ec2f30e2f451f),
        ("1.6873896392686601e+195", 0x68771d7f8b44ee96),
    ] {
        let parsed: serde_json::Value = serde_json::from_str(decimal).unwrap();
        assert_eq!(parsed.as_f64().unwrap().to_bits(), bits, "{decimal}");
    }
}
