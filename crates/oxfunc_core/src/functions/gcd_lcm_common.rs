pub fn gcd_int(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.abs()
}

pub fn lcm_int(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd_int(a, b)) * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_and_lcm_basic_lanes() {
        assert_eq!(gcd_int(24, 36), 12);
        assert_eq!(gcd_int(0, 5), 5);
        assert_eq!(lcm_int(6, 8), 24);
        assert_eq!(lcm_int(0, 5), 0);
    }
}
