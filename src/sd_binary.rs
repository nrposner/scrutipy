// now do the sd_binary functions, originally from the sd-binary.R file not utils, but for now we
// can keep them here, they're short enough

/// Returns the standard deviation of binary value counts
///
/// Parameters:
///     zeros: count of observations in the 0-binary condition
///     ones: count of observations in the 1-binary condition
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError is zeros or ones are not usigned integers
///
/// Panics:
///     If the total number of observations is not greater than one
pub fn sd_binary_groups(zeros: u32, ones: u32) -> f64 {
    // though we take in the counts as unsigned integers, we transform them into
    // floating point values in order to perform the
    let n: f64 = zeros as f64 + ones as f64;

    // we assert that between the two groups there are at least two observations. There's not
    // much point otherwise
    assert!(n > 1.0, "Expecting at least two observations");

    (n - (n - 1.0)).sqrt() * ((zeros * ones) as f64 / n.powi(2))

    //sqrt((n / (n - 1)) * ((group_0 * group_1) / (n ^ 2)))
}

/// Returns the standard deviation of binary variables from the count of zero values and the total
///
/// Parameters:
///     zeros: count of observations in the 0-binary condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError: if zeros or n are not unsigned integers
///
/// Panics:
///     If there are more observations in the zero condition than in the total
///     If the total number of observations is not greater than one
pub fn sd_binary_0_n(zeros: u32, n: u32) -> f64 {
    let ones: f64 = n as f64 - zeros as f64;

    assert!(
        n >= zeros,
        "There cannot be more observations in one condition than in the whole system"
    );
    assert!(n > 1, "Expecting at least two observations");

    ((n - (n - 1)) as f64).sqrt() * ((zeros as f64 * ones) / (n as f64).powi(2))
}
/// Returns the standard deviation of binary variables from the count of one values and the total
///
/// Parameters:
///     ones: count of observations in the 1-binary condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError: if ones or n are not unsigned integers
///
/// Panics:
///     If there are more observations in the one condition than in the total
///     If the total number of observations is not greater than one
pub fn sd_binary_1_n(ones: u32, n: u32) -> f64 {
    let zeros: f64 = n as f64 - ones as f64;

    assert!(
        n >= ones,
        "There cannot be more observations in one condition than in the whole system"
    );
    assert!(n > 1, "Expecting at least two observations");

    ((n - (n - 1)) as f64).sqrt() * ((zeros * ones as f64) / (n as f64).powi(2))
}
/// Returns the standard deviation of binary variables from the mean and the total
///
/// Parameters:
///     mean: mean of the binary observations, namely the proportion of values in the 1-binary
///     condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary system
///
/// Raises:
///     ValueError: if mean is not a floating-point number
///     ValueError: if n is not an unsigned integer
///
/// Panics:
///     if the mean is greater than one or less than zero
pub fn sd_binary_mean_n(mean: f64, n: u32) -> f64 {
    assert!(
        mean >= 0.0,
        "The mean of binary observations cannot be less than 0"
    );
    assert!(
        mean <= 1.0,
        "The mean of binary observations cannot be greater than 1"
    );
    ((n - (n - 1)) as f64) * (mean * (1.0 - mean))
}
