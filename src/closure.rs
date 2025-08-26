//! CLOSURE: complete listing of original samples of underlying raw evidence
//! 
//! Crate closure-core implements the CLOSURE technique for efficiently reconstructing
//! all possible distributions of raw data from summary statistics. It is not
//! about the Rust feature called closure.
//! 
//! The only API users are likely to need is `dfs_parallel()`. This function applies
//! the lower-level `dfs_branch()` in parallel and writes results to disk (currently
//! into a CSV file, but this may change in the future.)
//! 
//! Most of the code was written by Claude 3.5, translating Python code by Nathanael Larigaldie.
//! 
//! This version is copied from Lukas Jung and Nathanael Larigaldie's implementation for R

use num::{Float, FromPrimitive, Integer, NumCast, ToPrimitive};
use std::collections::VecDeque;
use rayon::prelude::*;
use pyo3::pyfunction;

#[pyfunction(signature = (mean, sd, n, scale_min, scale_max, rounding_error_mean = 0.05, rounding_error_sd = 0.05))]
#[cfg(not(tarpaulin_include))]
/// A Python implementation of the CLOSURE algorithm for reconstructing datasets from summary
/// statistics. 
///
/// Parameters:
///     mean (float): The target mean of the combinations.
///     sd (float): The target standard deviation of the combinations.
///     n (int): The number of values in each combination.
///     scale_min (int): The minimum scale value (inclusive).
///     scale_max (int): The maximum scale value (inclusive).
///     rounding_error_mean (float): The allowable rounding error for the mean.
///     rounding_error_sd (float): The allowable rounding error for the standard deviation.
///
/// Returns:
///     List[List[int]]: A list of lists, where each inner list represents a valid combination of integer values that meet the specified criteria.
/// Usage Example:
///     >>> from closure_core import closure
///     >>> combinations = closure(3.5, 1.2, 50, 0, 7, 0.05, 0.005)
///     >>> print(len(combinations))
///     7980
///
/// Notes:
///     - This function leverages parallel processing to efficiently explore the solution space.
///     - It is a high-level interface to the lower-level `dfs_branch()` Rust function. 
///     - Despite optimizations and parallelisms, the space of possible solutions grows explosively
///     as n, the range, and rounding error increase.
pub fn closure(
    mean: f64,
    sd: f64,
    n: i32,
    scale_min: i32,
    scale_max: i32,
    rounding_error_mean: f64,
    rounding_error_sd: f64,
) -> Vec<Vec<i32>> {
    dfs_parallel(mean, sd, n, scale_min, scale_max, rounding_error_mean, rounding_error_sd)
}

/// An iterator over a range of Rint-friendly generic integers `U`.
///
/// The `IntegerRange` struct provides an iterator that yields integers
/// starting from a specified `current` value up to, but not including,
/// an `end` value. The integers are of a generic type `U` that must
/// implement the `Integer` and `Copy` traits.
///
/// # Type Parameters
///
/// - `U`: A generic integer type that implements the `Integer` and `Copy` traits.
///
/// # Fields
///
/// - `current`: The current value of the iterator.
/// - `end`: The end value of the iterator (exclusive).
///
/// # Example
///
/// ```rust
/// use closure_core::IntegerRange;
/// use num::Integer;
///
/// let mut range = IntegerRange { current: 0, end: 5 };
/// while let Some(value) = range.next() {
///     println!("{}", value);
/// }
/// // This will print numbers 0 through 4.
/// ```
pub struct IntegerRange<U>
where
    U: Integer + Copy
{
    pub current: U,
    pub end: U,
}

impl<U> Iterator for IntegerRange<U>
where 
    U: Integer + Copy
{
    type Item = U;

    /// Returns the next integer in the range, or `None` if the end is reached.
    ///
    /// The `next` method increments the current value by one and returns it,
    /// until the current value reaches the end of the range. Once the end is
    /// reached, it returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(U)`: The next integer in the range.
    /// - `None`: If the end of the range is reached.
    ///
    /// # Example
    ///
    /// ```rust
    /// use closure_core::IntegerRange;
    /// use num::Integer;
    ///
    /// let mut range = IntegerRange { current: 0, end: 3 };
    /// assert_eq!(range.next(), Some(0));
    /// assert_eq!(range.next(), Some(1));
    /// assert_eq!(range.next(), Some(2));
    /// assert_eq!(range.next(), None);
    /// ```
    fn next(&mut self) -> Option<U> {
        if self.current < self.end {
            let next = self.current;
            self.current = self.current + U::one();
            Some(next) 
        } else {
            None
        }
    }
}

/// Creates an iterator over a range of integers of type `U`.
///
/// This function generates an iterator that yields integers starting from `start`
/// up to, but not including, `end`. The integers are of a generic type `U` that
/// must implement the `Integer` and `Copy` traits.
///
/// # Parameters
///
/// - `start`: The starting value of the range (inclusive).
/// - `end`: The ending value of the range (exclusive).
///
/// # Returns
///
/// An `IntegerRange<U>` iterator that yields integers from `start` to `end - 1`.
///
/// # Example
///
/// ```rust
/// use closure_core::range_u;
/// let range = range_u(0, 5);
/// for i in range {
///     println!("{}", i);
/// }
/// // This will print numbers 0 through 4.
/// ```
pub fn range_u<U: Integer + Copy>(start: U, end: U) -> IntegerRange<U> {
    IntegerRange {current: start, end}
}

/// Represents a combination of values tracking the progress of the CLOSURE algorithm.
///
/// This struct is used to store a combination of integer values along with
/// their running sum and running second moment (M2), which are useful for
/// calculating statistical measures like the mean and standard deviation.
///
/// # Type Parameters
///
/// - `U`: A Rint-compatible generic integer type
/// - `T`: A floating-point generic type converted from U
///
/// # Fields
///
/// - `values`: A vector of integer values representing the combination.
/// - `running_sum`: The running sum of the values in the combination.
/// - `running_m2`: The running second moment (M2) of the values, used for
///   calculating variance and standard deviation.
#[derive(Clone)]
struct Combination<U, T> {
    values: Vec<U>,
    running_sum: T,
    running_m2: T,
}

/// Calculates the number of initial combinations for a given scale range.
///
/// This function computes the total number of initial combinations possible
/// within a specified range defined by `scale_min` and `scale_max`. The
/// calculation is based on the formula for the sum of the first `n` natural
/// numbers, where `n` is the size of the range.
///
/// # Parameters
///
/// - `scale_min`: The minimum value of the scale range (inclusive).
/// - `scale_max`: The maximum value of the scale range (inclusive).
///
/// # Returns
///
/// The total number of initial combinations as an `i32`.
///
/// # Example
///
/// ```rust
/// use closure_core::count_initial_combinations;
///
/// let combinations = count_initial_combinations(1, 3);
/// assert_eq!(combinations, 6);
/// ```
pub fn count_initial_combinations(scale_min: i32, scale_max: i32) -> i32 {
    let range_size = scale_max - scale_min + 1;
    (range_size * (range_size + 1)) / 2
}

/// Executes the CLOSURE algorithm in parallel to find all valid combinations.
///
/// This function takes summary statistics and scale parameters to compute
/// all possible combinations of integer values that match the given mean
/// and standard deviation within specified rounding errors. It leverages
/// parallel processing to efficiently explore the solution space.
///
/// # Type Parameters
///
/// - `T`: A floating-point type used for calculations.
/// - `U`: An Rint-friendly integer type representing the scale values.
///
/// # Parameters
///
/// - `mean`: The target mean of the combinations.
/// - `sd`: The target standard deviation of the combinations.
/// - `n`: The number of values in each combination.
/// - `scale_min`: The minimum scale value (inclusive).
/// - `scale_max`: The maximum scale value (inclusive).
/// - `rounding_error_mean`: The allowable rounding error for the mean.
/// - `rounding_error_sd`: The allowable rounding error for the standard deviation.
///
/// # Returns
///
/// A vector of vectors, where each inner vector represents a valid combination
/// of integer values that meet the specified criteria.
///
/// # Example
///
/// ```rust
/// use closure_core::dfs_parallel;
/// use num::FromPrimitive;
///
/// let combinations = dfs_parallel(
///     10.0, 2.0, 3, 1, 5, 0.1, 0.1
/// );
/// assert!(combinations.is_empty()); // If there are no results, the outer vector will be empty
/// ```
///
/// ```rust
/// use closure_core::dfs_parallel;
/// use num::FromPrimitive;
///
/// let combinations = dfs_parallel(
///     3.5, 0.57, 100, 0, 7, 0.05, 0.05
/// );
///
/// assert_eq!(combinations.len(), 568); // If results are found, each one will be stored as a
/// Vec<U> inside the outer vector
/// ```
pub fn dfs_parallel<T, U>(
    mean: T,
    sd: T,
    n: U,
    scale_min: U,
    scale_max: U,
    rounding_error_mean: T,
    rounding_error_sd: T,
) -> Vec<Vec<U>>
where
    T: Float + FromPrimitive + Send + Sync, // suggest renaming to F to indicate float type?
    U: Integer + NumCast + ToPrimitive + Copy + Send + Sync,
{
    // Convert integer `n` to float to enable multiplication with other floats
    let n_float = T::from(U::to_i32(&n).unwrap()).unwrap();
    
    // Target sum calculations
    let target_sum = mean * n_float;
    let rounding_error_sum = rounding_error_mean * n_float;
    
    let target_sum_upper = target_sum + rounding_error_sum;
    let target_sum_lower = target_sum - rounding_error_sum;
    let sd_upper = sd + rounding_error_sd;
    let sd_lower = sd - rounding_error_sd;

    // Convert to usize for range operations
    let n_usize = U::to_usize(&n).unwrap();
    
    // Precomputing scale sums directly on T types 
    let scale_min_sum_t: Vec<T> = (0..n_usize)
        .map(|x| T::from(scale_min).unwrap() * T::from(x).unwrap())
        .collect();
    
    let scale_max_sum_t: Vec<T> = (0..n_usize)
        .map(|x| T::from(scale_max).unwrap() * T::from(x).unwrap())
        .collect();
    
    let n_minus_1 = n - U::one();
    let scale_max_plus_1 = scale_max + U::one();

   // instead of generating the initial combinations using concrete types, we're keeping them in U
    // and T using the iterator for U 
    let combinations = range_u(scale_min, scale_max_plus_1)
    .flat_map(|i| {
        range_u(i, scale_max_plus_1).map(move |j| {
            let initial_combination = vec![i, j];

            // turn the integer type into the float type
            // again, might be good for readability to rename T to F
            let i_float = T::from(i).unwrap();
            let j_float = T::from(j).unwrap();
            let sum = i_float + j_float;
            let current_mean = sum / T::from(2).unwrap();

            let diff_i = i_float - current_mean;
            let diff_j = j_float - current_mean;
            let current_m2 = diff_i * diff_i + diff_j * diff_j;

            (initial_combination, sum, current_m2)
        })
    })
    .collect::<Vec<_>>();

    // Process combinations in parallel
    combinations.par_iter()
        .flat_map(|(combo, running_sum, running_m2)| {
            dfs_branch(
                combo.clone(),
                *running_sum,
                *running_m2,
                n_usize,
                target_sum_upper,
                target_sum_lower,
                sd_upper,
                sd_lower,
                &scale_min_sum_t,
                &scale_max_sum_t,
                n_minus_1,
                scale_max_plus_1,
            )
        })
        .collect()
}

/// Collects all valid combinations from a starting point using a depth-first search approach.
///
/// This function is a lower-level component of the CLOSURE algorithm, designed to explore
/// all possible combinations of integer values that meet specified statistical criteria.
/// It uses a stack-based depth-first search to efficiently traverse the solution space.
///
/// # Type Parameters
///
/// - `T`: A floating-point type used for calculations.
/// - `U`: An integer type representing the scale values.
///
/// # Parameters
///
/// - `start_combination`: A vector of integers representing the initial combination to start from.
/// - `running_sum_init`: The initial running sum of the values in the starting combination.
/// - `running_m2_init`: The initial running second moment (M2) of the values, used for variance calculation.
/// - `n`: The total number of values in each combination.
/// - `target_sum_upper`: The upper bound for the target sum of the combination.
/// - `target_sum_lower`: The lower bound for the target sum of the combination.
/// - `sd_upper`: The upper bound for the standard deviation of the combination.
/// - `sd_lower`: The lower bound for the standard deviation of the combination.
/// - `scale_min_sum_t`: A precomputed vector of minimum scale sums for each position.
/// - `scale_max_sum_t`: A precomputed vector of maximum scale sums for each position.
/// - `_n_minus_1`: The total number of values minus one, used for internal calculations.
/// - `scale_max_plus_1`: The maximum scale value plus one, used for range operations.
///
/// # Returns
///
/// A vector of vectors, where each inner vector represents a valid combination of integer
/// values that meet the specified criteria.
#[inline]
#[allow(clippy::too_many_arguments)]
fn dfs_branch<T, U>(
    start_combination: Vec<U>,
    running_sum_init: T,
    running_m2_init: T,
    n: usize,  // Use usize for the length
    target_sum_upper: T,
    target_sum_lower: T,
    sd_upper: T,
    sd_lower: T,
    scale_min_sum_t: &[T],
    scale_max_sum_t: &[T],
    _n_minus_1: U,
    scale_max_plus_1: U,
) -> Vec<Vec<U>>
where
    T: Float + FromPrimitive + Send + Sync,
    U: Integer + NumCast + ToPrimitive + Copy + Send + Sync,
{
    let mut stack = VecDeque::with_capacity(n * 2); // Preallocate with reasonable capacity
    let mut results = Vec::new();
    
    stack.push_back(Combination {
        values: start_combination.clone(),
        running_sum: running_sum_init,
        running_m2: running_m2_init,
    });
    
    while let Some(current) = stack.pop_back() {
        if current.values.len() >= n {
            let n_minus_1_float = T::from(n - 1).unwrap();
            let current_std = (current.running_m2 / n_minus_1_float).sqrt();
            if current_std >= sd_lower {
                results.push(current.values);
            }
            continue;
        }

        // Calculate remaining items to add
        let current_len = current.values.len();
        let n_left = n - current_len - 1; // How many more items after the next one
        let next_n = current_len + 1;

        // Get current mean
        let current_mean = current.running_sum / T::from(current_len).unwrap();

        // Get the last value        
        let last_value = current.values[current_len - 1];

        for next_value in range_u(last_value, scale_max_plus_1) {
            let next_value_as_t = T::from(next_value).unwrap();
            let next_sum = current.running_sum + next_value_as_t;
            
            // Safe indexing with bounds check (using usize for indexing)
            if n_left < scale_min_sum_t.len() {
                let minmean = next_sum + scale_min_sum_t[n_left];
                if minmean > target_sum_upper {
                    break; // Early termination - better than take_while!
                }
                
                // Safe indexing with bounds check (using usize for indexing)
                if n_left < scale_max_sum_t.len() {
                    let maxmean = next_sum + scale_max_sum_t[n_left];
                    if maxmean < target_sum_lower {
                        continue;
                    }
                    
                    let next_mean = next_sum / T::from(next_n).unwrap();
                    let delta = next_value_as_t - current_mean;
                    let delta2 = next_value_as_t - next_mean;
                    let next_m2 = current.running_m2 + delta * delta2;
                    
                    let min_sd = (next_m2 / T::from(n - 1).unwrap()).sqrt();
                    if min_sd <= sd_upper {
                        let mut new_values = current.values.clone();
                        new_values.push(next_value);
                        stack.push_back(Combination {
                            values: new_values,
                            running_sum: next_sum,
                            running_m2: next_m2,
                        });
                    }
                }
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_initial_combinations() {
        assert_eq!(count_initial_combinations(1, 3), 6);
        assert_eq!(count_initial_combinations(1, 4), 10);
    }

    #[test]
    fn test_7980() {
        assert_eq!(dfs_parallel(3.5, 1.2, 50, 0, 7, 0.05, 0.005).len(), 7980);
    }
    #[test]
    fn test_empty() {
        assert!(dfs_parallel(10.0, 2.0, 3, 1, 5, 0.1, 0.1).is_empty());
    }
}
