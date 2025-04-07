// now do the sd_binary functions, originally from the sd-binary.R file not utils, but for now we
// can keep them here, they're short enough
// 66, 70, 94, 98, 122, 126
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
pub fn sd_binary_groups(zeros: u32, ones: u32) -> Result<f64, SdBinaryError> {
    // though we take in the counts as unsigned integers, we transform them into
    // floating point values in order to perform the
    let n: f64 = f64::from(zeros) + f64::from(ones);

    if n < 2.0 {
        return Err(SdBinaryError::InsufficientObservationsError);
    }
    // sqrt((n / (n - 1)) * ((group_0 * group_1) / (n ^ 2)))
    Ok((n / (n - 1.0) * (f64::from(zeros * ones) / n.powi(2))).sqrt())

    //sqrt((n / (n - 1)) * ((group_0 * group_1) / (n ^ 2)))
}

use thiserror::Error;
#[derive(Debug, Error, PartialEq)]
pub enum SdBinaryError {
    #[error("There cannot be more observations {0} in one condition than in the whole system {1}")]
    ObservationCountError(u32, u32),
    #[error("There must be at least two observations")]
    InsufficientObservationsError,
    #[error("The mean of binary observations cannot be less than 0.0")]
    NegativeBinaryMeanError,
    #[error("The mean of binary observations cannot be greater than 1.0")]
    InvalidBinaryError,
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
pub fn sd_binary_0_n(zeros: u32, n: u32) -> Result<f64, SdBinaryError> {
    let ones: f64 = f64::from(n) - f64::from(zeros);

    if n < zeros {
        return Err(SdBinaryError::ObservationCountError(zeros, n));
    }

    if n < 2 {
        return Err(SdBinaryError::InsufficientObservationsError);
    }

    Ok(((f64::from(n) / f64::from(n - 1) ) * ((f64::from(zeros) * ones) / (f64::from(n) ).powi(2))).sqrt())
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
pub fn sd_binary_1_n(ones: u32, n: u32) -> Result<f64, SdBinaryError> {
    let zeros: f64 = f64::from(n) - f64::from(ones);

    if n < ones {
        return Err(SdBinaryError::ObservationCountError(ones, n));
    }

    if n < 2 {
        return Err(SdBinaryError::InsufficientObservationsError);
    }

    Ok(((f64::from(n) / f64::from(n - 1) ) * ((zeros * f64::from(ones) ) / (f64::from(n) ).powi(2))).sqrt())
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
pub fn sd_binary_mean_n(mean: f64, n: u32) -> Result<f64, SdBinaryError> {
    if mean < 0.0 {
        return Err(SdBinaryError::NegativeBinaryMeanError);
    }

    if mean > 1.0 {
        return Err(SdBinaryError::InvalidBinaryError);
    }

    Ok(((f64::from(n) / f64::from(n - 1) ) * (mean * (1.0 - mean))).sqrt())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::rounding::rust_round;

    #[test]
    fn sd_binary_mean_test_1() {
        let res = sd_binary_mean_n(0.3, 30);
        assert_eq!(rust_round(res.unwrap(), 7), 0.4660916) // rounding to the 7th decimal place to match R
                                                           // output
    }

    #[test]
    fn sd_binary_mean_test_2() {
        let res = sd_binary_mean_n(-0.04, 78);

        assert_eq!(SdBinaryError::NegativeBinaryMeanError, res.unwrap_err())
    }

    #[test]
    fn sd_binary_mean_test_3() {
        let res = sd_binary_mean_n(1.04, 78);

        assert_eq!(SdBinaryError::InvalidBinaryError, res.unwrap_err())
    }

    #[test]
    fn sd_binary_groups_test_1() {
        let res = sd_binary_groups(0, 1);
        assert_eq!(
            SdBinaryError::InsufficientObservationsError,
            res.unwrap_err()
        )
    }

    #[test]
    fn sd_binary_0_n_test_1() {
        let res = sd_binary_0_n(5, 4);
        assert_eq!(SdBinaryError::ObservationCountError(5, 4), res.unwrap_err())
    }

    #[test]
    fn sd_binary_0_n_test_2() {
        let res = sd_binary_0_n(1, 1);
        assert_eq!(
            SdBinaryError::InsufficientObservationsError,
            res.unwrap_err()
        )
    }

    #[test]
    fn sd_binary_1_n_test_1() {
        let res = sd_binary_1_n(5, 4);
        assert_eq!(SdBinaryError::ObservationCountError(5, 4), res.unwrap_err())
    }

    #[test]
    fn sd_binary_1_n_test_2() {
        let res = sd_binary_1_n(1, 1);
        assert_eq!(
            SdBinaryError::InsufficientObservationsError,
            res.unwrap_err()
        )
    }
}
