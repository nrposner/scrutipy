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

pub fn round_down_from(x: f64, digits: i32, threshold: f64, symmetric: bool) -> f64 {
    let p10 = 10.0f64.powi(digits);
    let threshold = threshold - f64::MIN_POSITIVE.powf(0.5);

    if symmetric {
        match x < 0.0 {
            true => -(x.abs() * p10 + (1.0 - (threshold / 10.0))).ceil(), // - (floor(abs(x) * p10 + (1 - (threshold / 10))) / p10)
            false => (x * p10 + (1.0 - (threshold / 10.0))).ceil(),
        }
    } else {
        (x * p10 + (1.0 - (threshold / 10.0))).ceil()
    }
}
