#[cfg(test)]
pub mod tests {
    use crate::grim::*;
    use core::f64;

    #[test]
    fn grim_scalar_rust_test_1() {
        let val = grim_scalar_rust(
            "5.19",
            40,
            vec![false, false, false],
            1,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, false)
    }

    #[test]
    fn grim_scalar_rust_test_2() {
        let val = grim_scalar_rust(
            "5.18",
            40,
            vec![false, false, false],
            1,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, true);
    }

    #[test]
    fn grim_scalar_rust_test_3() {
        let val = grim_scalar_rust(
            "5.19",
            40,
            vec![false, false, false],
            2,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, true);
    }

    #[test]
    fn grim_scalar_rust_test_4() {
        let val = grim_scalar_rust(
            "5.19",
            20,
            vec![false, true, false],
            1,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, false);
    }

    #[test]
    fn grim_scalar_rust_test_5() {
        let val = grim_scalar_rust(
            "5.19",
            20,
            vec![false, true, false],
            1,
            "up",
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, false);
    }

    #[test]
    fn grim_rust_test_1() {
        let xs = vec![
            "7.22", "4.74", "5.23", "2.57", "6.77", "2.68", "7.01", "7.38", "3.14", "6.89", "5.00",
            "0.24",
        ];

        let ns = vec![32, 25, 29, 24, 27, 28, 29, 26, 27, 31, 25, 28];

        let items = vec![1; 12]; //presumably all 1s?

        let bools = grim_rust(
            xs,
            ns,
            vec![false, false, false],
            items,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );

        assert_eq!(
            bools,
            vec![true, false, false, false, false, true, false, true, false, false, true, false]
        );
    }

    #[test]
    fn grim_rust_test_2() {
        let xs = vec![
            "7.22", "4.74", "5.23", "2.57", "6.77", "2.68", "7.01", "7.38", "3.14", "6.89", "5.00",
            "0.24",
        ];

        let ns = vec![32, 25, 29, 24, 27, 28, 29, 26, 27, 31, 25, 28];

        let items = vec![1; 12]; //presumably all 1s?

        let bools = grim_rust(
            xs,
            ns,
            vec![true, false, false],
            items,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );

        assert_eq!(
            bools,
            vec![
                false, false, false, false, false, false, false, false, false, false, false, false
            ]
        );
    }

    #[test]
    fn grim_scalar_test_1() {
        let input = GRIMInput::Str("5.19".to_string());
        let val = grim_scalar(
            input,
            40,
            "up_or_down".to_string(),
            1,
            false,
            false,
            5.0,
            false,
            f64::EPSILON.powf(0.5),
        );
        assert!(!val);
    }

    #[test]
    fn grim_scalar_test_2() {
        let input = GRIMInput::Num(5.19);
        let val = grim_scalar(
            input,
            40,
            "up_or_down".to_string(),
            1,
            false,
            false,
            5.0,
            false,
            f64::EPSILON.powf(0.5),
        );
        assert!(!val);
    }

    #[test]
    #[should_panic]
    fn grim_scalar_test_3() {
        let input = GRIMInput::Str("not a number".to_string());
        let _ = grim_scalar(
            input,
            40,
            "up_or_down".to_string(),
            1,
            false,
            false,
            5.0,
            false,
            f64::EPSILON.powf(0.5),
        );
    }

    #[test]
    #[should_panic]
    fn grim_tester_test_1() {
        let val = grim_scalar_rust(
            "not a number",
            50,
            vec![false, false, false],
            1,
            "up_or_down",
            5.0,
            f64::EPSILON.powf(0.5),
        );

        grim_tester(val, false)
    }
}
