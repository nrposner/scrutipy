use crate::utils::decimal_places_scalar;

pub fn grim_probability(x: &str, n: u32, items: u32, percent: bool) -> f64 {
    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        digits += 2
    };

    let p10 = 10.0f64.powi(digits);

    f64::max((p10 - (n as f64 * items as f64)) / p10, 0.0f64)
}

/// Find probability that a supplied mean is inconsistent with the reported sample size (allowing
/// for rare negative results, cf grim_probability())
pub fn grim_ratio(x: &str, n: u32, items: u32, percent: bool) -> f64 {
    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        digits += 2
    };

    let p10 = 10.0f64.powi(digits);

    (p10 - (n as f64 * items as f64)) / p10
}

// Find the absolute number of GRIM inconsistencies possible given the mean and sample size
pub fn grim_total(x: &str, n: u32, items: u32, percent: bool) -> i32 {
    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        digits += 2
    };

    let p10 = 10.0f64.powi(digits);

    (p10 - (n as f64 * items as f64)).floor() as i32
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn grim_probability_test_1() {
        let val = grim_probability("8.2", 6, 1, true);
        assert_eq!(val, 0.994)
    }

    #[test]
    fn grim_probability_test_2() {
        let val = grim_probability("6.7", 9, 1, false);
        assert_eq!(val, 0.1)
    }

    #[test]
    fn grim_probability_test_3() {
        let val = grim_probability("3.333", 3, 3, false);
        assert_eq!(val, 0.991)
    }

    #[test]
    fn grim_probability_test_4() {
        let val = grim_probability("60.7", 9, 7, false);
        assert_eq!(val, 0.0)
    }

    #[test]
    fn grim_ratio_test_1() {
        let val = grim_ratio("8.2", 6, 1, true);
        assert_eq!(val, 0.994)
    }

    #[test]
    fn grim_ratio_test_2() {
        let val = grim_ratio("6.7", 9, 1, false);
        assert_eq!(val, 0.1)
    }

    #[test]
    fn grim_ratio_test_3() {
        let val = grim_ratio("3.333", 3, 3, false);
        assert_eq!(val, 0.991)
    }

    #[test]
    fn grim_ratio_test_4() {
        let val = grim_ratio("60.7", 9, 7, false);
        assert_eq!(val, -5.3)
    }

    #[test]
    fn grim_total_test_1() {
        let val = grim_total("8.2", 6, 1, true);
        assert_eq!(val, 994)
    }

    #[test]
    fn grim_total_test_2() {
        let val = grim_total("6.7", 9, 1, false);
        assert_eq!(val, 1)
    }

    #[test]
    fn grim_total_test_3() {
        let val = grim_total("3.333", 3, 3, false);
        assert_eq!(val, 991)
    }

    #[test]
    fn grim_total_test_4() {
        let val = grim_total("60.7", 9, 7, false);
        assert_eq!(val, -53)
    }

    #[test]
    fn grim_probability_vector_test_1() {
        let xs = [
            "7.22", "4.74", "5.23", "2.57", "6.77", "2.68", "7.01", "7.38", "3.14", "6.89", "5.00",
            "0.24",
        ];

        let ns: [u32; 12] = [32, 25, 29, 24, 27, 28, 29, 26, 27, 31, 25, 28];

        let vals: Vec<f64> = xs
            .iter()
            .zip(ns.iter())
            .map(|(x, n)| grim_probability(x, *n, 1, false))
            .collect();

        assert_eq!(
            vals,
            vec![0.68, 0.75, 0.71, 0.76, 0.73, 0.72, 0.71, 0.74, 0.73, 0.69, 0.75, 0.72]
        )
        // x, n items=1, percent = false
    }
}
