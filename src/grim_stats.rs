#[allow(unused_imports)]
use crate::utils::{decimal_places_scalar, dustify, reround};

pub fn grim_probability(x: &str, n: u32, items: u32, percent: bool) -> f64 {
    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        digits += 2
    };

    let p10 = 10.0f64.powi(digits);

    f64::max((p10 - (n as f64 * items as f64)) / p10, 0.0f64)
}

pub fn grim_ratio(x: &str, n: u32, items: u32, percent: bool) -> f64 {
    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        digits += 2
    };

    let p10 = 10.0f64.powi(digits);

    (p10 - (n as f64 * items as f64)) / p10
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn grim_probability_test_1() {
        let val = grim_probability("8.2", 6, 1, true);
        assert_eq!(val, 0.994)
    }

    #[test]
    pub fn grim_probability_test_2() {
        let val = grim_probability("6.7", 9, 1, false);
        assert_eq!(val, 0.1)
    }

    #[test]
    pub fn grim_probability_test_3() {
        let val = grim_probability("3.333", 3, 3, false);
        assert_eq!(val, 0.991)
    }

    #[test]
    pub fn grim_probability_test_4() {
        let val = grim_probability("60.7", 9, 7, false);
        assert_eq!(val, 0.0)
    }

    #[test]
    pub fn grim_ratio_test_1() {
        let val = grim_ratio("8.2", 6, 1, true);
        assert_eq!(val, 0.994)
    }

    #[test]
    pub fn grim_ratio_test_2() {
        let val = grim_ratio("6.7", 9, 1, false);
        assert_eq!(val, 0.1)
    }

    #[test]
    pub fn grim_ratio_test_3() {
        let val = grim_ratio("3.333", 3, 3, false);
        assert_eq!(val, 0.991)
    }

    #[test]
    pub fn grim_ratio_test_4() {
        let val = grim_ratio("60.7", 9, 7, false);
        assert_eq!(val, -5.3)
    }
}
