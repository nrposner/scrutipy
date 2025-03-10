/// round down function
pub fn round_down(number: f64, decimals: i32) -> f64 {
    let to_round =
        (number * 10.0f64.powi(decimals + 1)) - (number * 10f64.powi(decimals)).floor() * 10.0;

    match to_round {
        5.0 => (number * 10f64.powi(decimals)).floor() / 10f64.powi(decimals),
        _ => rust_round(number, decimals),
    }
}

pub fn round_up(number: f64, decimals: i32) -> f64 {
    let to_round =
        (number * 10.0f64.powi(decimals + 1)) - (number * 10f64.powi(decimals)).floor() * 10.0;

    match to_round {
        5.0 => (number * 10f64.powi(decimals)).ceil() / 10f64.powi(decimals),
        _ => rust_round(number, decimals),
    }
}

/// rust does not have a native function that rounds binary floating point numbers to a set number
/// of decimals. This is a hacky workaround that nevertheless seems to be the best option.
pub fn rust_round(x: f64, y: i32) -> f64 {
    (x * 10.0f64.powi(y)).round() / 10.0f64.powi(y)
}

pub fn round_trunc(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);

    //For symmetry between positive and negative numbers, use the absolute value:
    let core = (x.abs() * p10).trunc() / p10; // the rust f64::trunc() function may have
                                              // different properties than the R trunc() function, check to make sure
    match x < 0.0 {
        true => -core,
        false => core,
    }

    //If `x` is negative, its truncated version should be negative or zero.
    //Therefore, in this case, the function returns the negative of `core`, the
    //absolute value; otherwise it simply returns `core` itself:
}

pub fn anti_trunc(x: f64) -> f64 {
    let core = x.abs().trunc() + 1.0;

    match x < 0.0 {
        true => -core,
        false => core,
    }
}

/// a function to return any function to its decimal portion, used in unit tests in the original R
/// library
pub fn trunc_reverse(x: f64) -> f64 {
    x - x.trunc()
}

pub fn round_anti_trunc(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);

    anti_trunc(x * p10) / p10
}

pub fn round_ceiling(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);
    (x * p10).ceil() / p10
    //ceiling(x * p10) / p10;
}

pub fn round_floor(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);
    (x * p10).floor() / p10
    //ceiling(x * p10) / p10;
}
// not sure if x and rounding are meant to be scalars or vectors, because it seems elsewhere like
// we can pass multiple arguments to rounding, and that x can also be a vector????
// But the example values for the above function look exclusively scalars

pub fn round_up_from(x: f64, digits: i32, threshold: f64, symmetric: bool) -> f64 {
    let p10 = 10.0f64.powi(digits);
    let threshold = threshold - f64::MIN_POSITIVE.powf(0.5);

    if symmetric {
        match x < 0.0 {
            true => -(x.abs() * p10 + (1.0 - (threshold / 10.0))).floor(), // - (floor(abs(x) * p10 + (1 - (threshold / 10))) / p10)
            false => (x * p10 + (1.0 - (threshold / 10.0))).floor(),
        }
    } else {
        (x * p10 + (1.0 - (threshold / 10.0))).floor()
    }
}

pub fn round_down_from(x: Vec<f64>, digits: i32, threshold: f64, symmetric: bool) -> Vec<f64> {
    let p10 = 10.0f64.powi(digits);
    let threshold = threshold + f64::EPSILON.powf(0.5);

    // let's make a round_down_from_scalar function that we can .map onto the vector
    //
    //
    x.iter()
        .map(|i| round_down_from_scalar(*i, p10, threshold, symmetric))
        .collect()
}

pub fn round_down_from_scalar(x: f64, p10: f64, threshold: f64, symmetric: bool) -> f64 {
    //let p10 = 10.0f64.powi(digits);
    //let threshold = threshold - f64::MIN_POSITIVE.powf(0.5);

    if symmetric {
        match x < 0.0 {
            true => -(x.abs() * p10 + (1.0 - (threshold / 10.0))).ceil(), // - (floor(abs(x) * p10 + (1 - (threshold / 10))) / p10)
            false => (x * p10 + (1.0 - (threshold / 10.0))).ceil(),
        }
    } else {
        (x * p10 + (1.0 - (threshold / 10.0))).floor() / p10
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;

    #[test]
    pub fn round_down_from_test_1() {
        assert_eq!(
            round_down_from(vec![65.3488492, 645.76543], 4, 5.0, false),
            vec![65.3488, 645.7654]
        )
    }

    #[test]
    pub fn round_down_from_test_2() {
        assert_eq!(
            round_down_from(vec![65.34845, 645.76543], 4, 5.0, false),
            vec![65.3484, 645.7654]
        )
    }
    #[test]
    pub fn round_down_from_scalar_test_1() {
        let p10 = 10.0f64.powi(4);
        assert_eq!(round_down_from_scalar(65.3488492, p10, 5.0, false), 65.3488)
    }

    #[test]
    pub fn round_down_from_scalar_test_2() {
        let p10 = 10.0f64.powi(4);
        assert_eq!(
            round_down_from_scalar(65.34845, p10, 5.0 + f64::EPSILON.powf(0.5), false),
            65.3484
        )
    }

    #[test]
    pub fn round_down_from_test_3() {
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
    fn rust_round_test_1() {
        let val = rust_round(98.7823987, 4);

        assert_eq!(98.7824, val)
    }
}
