
#[cfg(test)]
mod tests {
    use crate::utils::*;
    use crate::sd_binary::SdBinaryError;
    use crate::rounding::rust_round;
    const FUZZ_VALUE: f64 = 1e-12;

    #[test]
    fn check_rounding_singular_test_1() {
        let val = check_rounding_singular(vec!["up_or_down"], "ceiling_or_floor", "up", "down");
        assert_eq!(Ok(()), val)
    }

    #[test]
    fn check_rounding_singular_test_2() {
        let val =
            check_rounding_singular(vec!["ceiling_or_floor"], "ceiling_or_floor", "up", "down");

        let b = match val {
            Ok(()) => true,
            Err(_) => false,
        };

        assert!(!b)
    }

    #[test]
    fn decimal_places_test_1() {
        assert_eq!(decimal_places_scalar(Some("9.846"), "."), Some(3));
    }

    #[test]
    fn decimal_places_test_2() {
        assert_eq!(decimal_places_scalar(Some(".9678"), "."), Some(4));
    }

    #[test]
    fn decimal_places_test_3() {
        assert_eq!(decimal_places_scalar(Some("1."), "."), None);
    }

    #[test]
    fn decimal_places_test_4() {
        assert_eq!(decimal_places_scalar(Some("0"), "."), None);
    }

    #[test]
    fn decimal_places_test_5() {
        assert_eq!(decimal_places_scalar(Some("1.52.0"), "."), Some(2));
    }

    #[test]
    fn decimal_places_test_6() {
        assert_eq!(decimal_places_scalar(Some("Not a Number"), "."), None);
    }

    #[test]
    fn decimal_places_test_7() {
        assert_eq!(decimal_places_scalar(None, "."), None);
    }

    #[test]
    fn decimal_places_test_8() {
        assert_eq!(decimal_places_scalar(Some("52.13"), "."), Some(2));
    }


    // testing reconstruct_sd_scalar
    #[test]
    fn reconstruct_sd_scalar_test_1() {
        let sd_rec_scalar = reconstruct_sd_scalar("mean_n", "0.3", 30, 12, 15);
        assert_eq!(0.4660916, rust_round(sd_rec_scalar.unwrap(), 7)); // rounding to 7th decimal
                                                                      // place to match R output
    }

    #[test]
    fn reconstruct_sd_scalar_test_2() {
        let sd_rec_scalar = reconstruct_sd_scalar("groups", "0.3", 30, 12, 15);
        assert_eq!(0.5063697, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_sd_scalar_test_3() {
        let sd_rec_scalar = reconstruct_sd_scalar("0_n", "0.3", 30, 12, 15);
        assert_eq!(0.4982729, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_sd_scalar_test_4() {
        let sd_rec_scalar = reconstruct_sd_scalar("1_n", "0.3", 30, 12, 15);
        assert_eq!(0.5085476, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_sd_scalar_test_5() {
        let sd_rec_scalar = reconstruct_sd_scalar("1_n", "Not. a number", 30, 12, 15);

        let res = match sd_rec_scalar {
            Ok(num) => Some(num),
            Err(ReconstructSdError::NotANumber(_)) => Some(-1.0),
            Err(ReconstructSdError::NotAFormula(_)) => None,
            Err(ReconstructSdError::SdBinaryError(_, _)) => None,
        };
        assert_eq!(Some(-1.0), res)
    }

    #[test]
    fn reconstruct_sd_scalar_test_6() {
        let sd_rec_scalar = reconstruct_sd_scalar("1__n", "0.3", 30, 12, 15);

        let res = match sd_rec_scalar {
            Ok(num) => Some(num),
            Err(ReconstructSdError::NotAFormula(_)) => Some(-1.0),
            Err(ReconstructSdError::NotANumber(_)) => None,
            Err(ReconstructSdError::SdBinaryError(_, _)) => None,
        };
        assert_eq!(Some(-1.0), res)
    }

    #[test]
    fn reconstruct_sd_scalar_test_7() {
        let sd_rec_scalar = reconstruct_sd_scalar("0_n", "0.8", 2, 3, 1);

        let _s = "0_n".to_string();
        let res = match sd_rec_scalar {
            Ok(_num) => None,
            Err(ReconstructSdError::NotAFormula(_)) => None,
            Err(ReconstructSdError::NotANumber(_)) => None,
            Err(ReconstructSdError::SdBinaryError(
                _s,
                SdBinaryError::ObservationCountError(3, 2),
            )) => Some(1),
            _ => None,
        };
        assert_eq!(res, Some(1))
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_1() {
        let res = reconstruct_rounded_numbers_scalar(2.9876, 3, "up_or_down", 5.0, false);
        assert_eq!(res, vec![2.988, 2.988])
    }

    #[test]
    #[should_panic]
    fn reconstruct_rounded_numbers_scalar_test_2() {
        let _res =
            reconstruct_rounded_numbers_scalar(2.9876, 3, "up_from_or_down_from", 5.0, false);
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_3() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "up_from_or_down_from", 6.0, false);

        assert_eq!(res, vec![2.986, 2.986])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_4() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "down", 5.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    #[should_panic]
    fn reconstruct_rounded_numbers_scalar_test_5() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "up_from", 5.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_6() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "up_from", 6.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    #[should_panic]
    fn reconstruct_rounded_numbers_scalar_test_7() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "down_from", 5.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_8() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "down_from", 6.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_9() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "ceiling", 5.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_10() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "floor", 5.0, false);

        assert_eq!(res, vec![2.985])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_11() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "trunc", 5.0, false);

        assert_eq!(res, vec![2.985])
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_12() {
        let res = reconstruct_rounded_numbers_scalar(2.9856, 3, "anti_trunc", 5.0, false);

        assert_eq!(res, vec![2.986])
    }

    #[test]
    #[should_panic]
    fn reconstruct_rounded_numbers_scalar_test_13() {
        let _res = reconstruct_rounded_numbers_scalar(2.9856, 3, "wrong", 5.0, false);
    }
    #[test]
    fn check_threshold_specified_test_1() {
        check_threshold_specified(7.0); // this just needs to not panic
    }

    #[test]
    #[should_panic]
    fn check_threshold_specified_test_2() {
        check_threshold_specified(5.0);
    }

    #[test]
    fn dustify_test_1() {
        let val = dustify(4.897034);
        assert_eq!(val, vec![4.897034 - FUZZ_VALUE, 4.897034 + FUZZ_VALUE])
    }

    #[test]
    fn reround_test_1() {
        let val = reround(
            vec![2.9876, 8.78964, 6.98767],
            3,
            "up_or_down",
            5.0,
            false,
        );

        assert_eq!(val, vec![2.988, 2.988, 8.790, 8.790, 6.988, 6.988])
    }
}
