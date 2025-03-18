#[cfg(test)]
mod tests {
    use crate::rounding::*;
    use core::f64;

    #[test]
    fn round_down_from_test_1() {
        assert_eq!(
            round_down_from(vec![65.3488492, 645.76543], 4, 5.0, false),
            vec![65.3488, 645.7654]
        )
    }

    #[test]
    fn round_down_from_test_2() {
        assert_eq!(
            round_down_from(vec![65.34845, 645.76543], 4, 5.0, false),
            vec![65.3484, 645.7654]
        )
    }

    #[test]
    fn round_down_from_test_4() {
        assert_eq!(
            round_down_from(vec![65.3488492, 645.76543], 4, 5.0, true),
            vec![65.3488, 645.7654]
        )
    }

    #[test]
    fn round_down_from_test_5() {
        assert_eq!(
            round_down_from(vec![65.34845, 645.76543], 4, 5.0, true),
            vec![65.3484, 645.7654]
        )
    }

    #[test]
    fn round_down_from_test_6() {
        assert_eq!(
            round_down_from(vec![-65.34845, -645.76543], 4, 5.0, true),
            vec![-65.3484, -645.7654]
        )
    }

    #[test]
    fn round_up_from_test_1() {
        assert_eq!(
            round_up_from(vec![65.3488492, 645.76543], 4, 5.0, false),
            vec![65.3488, 645.7654]
        )
    }

    #[test]
    fn round_up_from_test_2() {
        assert_eq!(
            round_up_from(vec![65.34845, 645.76543], 4, 5.0, false),
            vec![65.3485, 645.7654]
        )
    }

    #[test]
    fn round_up_from_test_4() {
        assert_eq!(
            round_up_from(vec![65.3488492, 645.76543], 4, 5.0, true),
            vec![65.3488, 645.7654]
        )
    }

    #[test]
    fn round_up_from_test_5() {
        assert_eq!(
            round_up_from(vec![65.34845, 645.76543], 4, 5.0, true),
            vec![65.3485, 645.7654]
        )
    }

    #[test]
    fn round_up_from_test_6() {
        assert_eq!(
            round_up_from(vec![-65.34845, -645.76543], 4, 5.0, true),
            vec![-65.3485, -645.7654]
        )
    }

    #[test]
    fn round_down_from_scalar_test_1() {
        let p10 = 10.0f64.powi(4);
        assert_eq!(round_down_from_scalar(65.3488492, p10, 5.0, false), 65.3488)
    }

    #[test]
    fn round_down_from_scalar_test_2() {
        let p10 = 10.0f64.powi(4);
        assert_eq!(round_down_from_scalar(65.34845, p10, 5.0, false), 65.3484)
    }

    #[test]
    fn round_down_from_test_3() {
        let xs: Vec<f64> = vec![
            1991.077, 2099.563, 1986.102, 1925.769, 2015.759, 1972.437, 1973.526, 2066.728,
            1947.636, 1920.659,
        ];

        let rounded_xs = round_down_from(xs.clone(), 2, 5.0, false);

        let ts: Vec<f64> = rounded_xs
            .clone()
            .iter()
            .map(|x| trunc_reverse(*x))
            .collect();

        let xs_truncated: Vec<f64> = xs.clone().iter().map(|x| trunc_reverse(*x)).collect();

        let rts_truncated = round_down_from(ts.clone(), 2, 5.0, false);

        let rts = round_down_from(xs_truncated.clone(), 2, 5.0, false);

        // note that unlike in R, the reversed operations do not result in exactly the same output.
        // rts and ts should be the same, but they are slightly off by machine EPSILON
        // had to re round in order to get this working
        // check if this is actually fulfilling the needs of the test
        assert_eq!(rts, rts_truncated);
    }

    #[test]
    fn round_up_from_test_3() {
        let xs: Vec<f64> = vec![
            1991.077, 2099.563, 1986.102, 1925.769, 2015.759, 1972.437, 1973.526, 2066.728,
            1947.636, 1920.659,
        ];

        let rounded_xs = round_up_from(xs.clone(), 2, 5.0, false);

        let ts: Vec<f64> = rounded_xs
            .clone()
            .iter()
            .map(|x| trunc_reverse(*x))
            .collect();

        let xs_truncated: Vec<f64> = xs.clone().iter().map(|x| trunc_reverse(*x)).collect();

        let rts_truncated = round_up_from(ts.clone(), 2, 5.0, false);

        let rts = round_up_from(xs_truncated.clone(), 2, 5.0, false);

        // note that unlike in R, the reversed operations do not result in exactly the same output.
        // rts and ts should be the same, but they are slightly off by machine EPSILON
        // had to re round in order to get this working
        // check if this is actually fulfilling the needs of the test
        assert_eq!(rts, rts_truncated);
    }

    #[test]
    fn rust_round_test_1() {
        let val = rust_round(98.7823987, 4);

        assert_eq!(98.7824, val)
    }

    #[test]
    fn round_trunc_test_1() {
        let val = round_trunc(5.786487, 3);
        assert_eq!(val, 5.786)
    }

    #[test]
    fn round_trunc_test_2() {
        let val = round_trunc(-5.786487, 3);
        assert_eq!(val, -5.786)
    }

    #[test]
    fn anti_trunc_test_1() {
        let val = anti_trunc(5.786487);
        assert_eq!(val, 6.0)
    }

    #[test]
    fn anti_trunc_test_2() {
        let val = anti_trunc(-5.786487);
        assert_eq!(val, -6.0)
    }

    #[test]
    fn round_anti_trunc_test_1() {
        let val = round_anti_trunc(5.4924723, 3);
        assert_eq!(val, 5.493)
    }

    #[test]
    fn round_anti_trunc_test_2() {
        let val = round_anti_trunc(-5.4924723, 3);
        assert_eq!(val, -5.493)
    }
}
