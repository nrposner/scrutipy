use rand::rng;
use rand::Rng;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};


#[pyclass]
#[derive(Clone,Debug)]
pub struct SimRank {
    #[pyo3(get)]
    pub n1: Vec<usize>,
    #[pyo3(get)]
    pub n2: Vec<usize>,
}

#[pymethods]
impl SimRank {
    #[new]
    fn new(n1: Vec<usize>, n2: Vec<usize>) -> Self {
        SimRank { n1, n2 }
    }
    // Since it's straightforward to recompute, no need to waste space storing the u_value with the
    // concrete data: just recalculate it when needed
    fn u_values(&self) -> (f64, f64) {
        let n1 = self.n1.len() as f64;
        let n2 = self.n2.len() as f64;
        let r1 = self.n1.iter().sum::<usize>() as f64;
        let r2 = self.n2.iter().sum::<usize>() as f64;

        let u_val_1 = (n1 * n2) + (n1*(n1+1.0))/2.0 - r1;
        let u_val_2 = (n1 * n2) + (n2*(n2+1.0))/2.0 - r2;

        (u_val_1, u_val_2)
    }
    fn __repr__(&self) -> String {
        let (u1, u2) = self.u_values();
        format!(
            "SimRank(n1={:?}, n2={:?}, U=({}, {}))",
            self.n1, self.n2, u1, u2
        )
    }
}

/// A SimRank result that includes tied ranks (half-integers).
///
/// This struct is used by `simrank_tied` to represent rank assignments where
/// at least one pair of consecutive positions share a tied rank value.
/// Unlike `SimRank` which uses integer ranks, this uses f64 to accommodate
/// half-integer ranks like 3.5 (when positions 3 and 4 are tied).
#[pyclass]
#[derive(Clone, Debug)]
pub struct SimRankTied {
    #[pyo3(get)]
    pub n1: Vec<f64>,
    #[pyo3(get)]
    pub n2: Vec<f64>,
}

#[pymethods]
impl SimRankTied {
    #[new]
    fn new(n1: Vec<f64>, n2: Vec<f64>) -> Self {
        SimRankTied { n1, n2 }
    }

    /// Computes U-values from the rank sums.
    fn u_values(&self) -> (f64, f64) {
        let n1 = self.n1.len() as f64;
        let n2 = self.n2.len() as f64;
        let r1: f64 = self.n1.iter().sum();
        let r2: f64 = self.n2.iter().sum();

        let u_val_1 = (n1 * n2) + (n1 * (n1 + 1.0)) / 2.0 - r1;
        let u_val_2 = (n1 * n2) + (n2 * (n2 + 1.0)) / 2.0 - r2;

        (u_val_1, u_val_2)
    }
    fn __repr__(&self) -> String {
        let (u1, u2) = self.u_values();
        format!(
            "SimRankTied(n1={:?}, n2={:?}, U=({}, {}))",
            self.n1, self.n2, u1, u2
        )
    }
}

// current issue: we want to allow the user to input half integers, since it is possible for a
// u-value to not be a whole number, if there was a tie among the tests. 
//
// we also want to check up-front whether the u-value provided is not within the possible range: if
// so, we should exit immediately with an informative error and pass it up to Python. Otherwise, we
// could end up spending lots of cycles on useless work.

/// Generates simulated rank tests based on rank counts and U-score.
///
/// This function takes integers n1 and n2, decimal U-value u_target (which may be an integer or
/// half-integer) and output length. It returns an array of SimRank objects containing the sampled
/// rank groups and the target U-value. Origial implementation by [David Robert Grimes](https://github.com/drg85/GRIMU), cf [*Heathers & Grimes 2026*](https://medicalevidenceproject.org/grim-u-observation-establish-impossible-p-values-ranked-tests/).
///
/// # Arguments
///
/// * `n1` - The number of tests in group 1.
/// * `n2` - The number of tests in group 2.
/// * `u_target` - The target U-value of the rank test. This may be an integer or a half-integer.
/// * `length` - How many simulated rank tests to generate.
/// * `max_iter` - The maximum number of samples the function will take before terminating, if it
/// has not already found `length` valid samples.
///
/// # Returns
///
/// Returns a `SimRank` containing two vectors of ranks `n1` and `n2` and a u-value, which should
/// in all cases be the same as the input u_target.
///
/// Returns a `PyResult` containing a vector of boolean values. Each boolean indicates whether
/// the corresponding set of inputs is consistent according to the specified parameters.
///
/// # Notes
///
/// Being a stochastic process, it is possible that the sampler will fail to find some valid
/// simrank combinations, even with an extremely high `max_iter`. It is also possible that it will
/// fail to find up to `length` elements, even if they do exist. Thus, the output vectors are not
/// guaranteed to be exactly `length` in size, and if their exact dimensions are relevant to any
/// analysis, that must be checked by the caller.
#[pyfunction(signature = (n1, n2, u_target, length=1, max_iter=100000))]
pub fn simrank(
    n1: usize, 
    n2: usize, 
    u_target: f64,
    length: usize,
    max_iter: usize
) -> Vec<SimRank> {
// ) -> Vec<(Vec<usize>, Vec<usize>, f64)> {
    let r1_target = u_target + (n1 as f64) * (n1 as f64 + 1.0) / 2.0;
    let n_total = n1 + n2;

    // use a Mutex to collect results across threads
    let results = Arc::new(Mutex::new(Vec::with_capacity(length)));
    // use an Atomic counter to stop work early
    let count = Arc::new(AtomicUsize::new(0));

    (0..max_iter).into_par_iter().for_each(|_| {
        // Early exit: if we already have enough results, stop trying
        if count.load(Ordering::Relaxed) >= length {
            return;
        }

        let mut rng = rng();
        let indices = rand::seq::index::sample(&mut rng, n_total, n1);
        let sum_1_based = indices.iter().sum::<usize>() + n1;

        if sum_1_based as f64 == r1_target {
            // Check again before expensive work
            if count.load(Ordering::Relaxed) < length {
                let mut group1_ranks = indices.into_vec();
                for val in group1_ranks.iter_mut() { *val += 1; }
                group1_ranks.sort_unstable();

                let mut group2_ranks = Vec::with_capacity(n2);
                let mut g1_iter = group1_ranks.iter().peekable();
                for i in 1..=n_total {
                    if g1_iter.peek() == Some(&&i) {
                        g1_iter.next();
                    } else {
                        group2_ranks.push(i);
                    }
                }

                let sr = SimRank {
                    n1: group1_ranks,
                    n2: group2_ranks,
                };

                let mut res_guard = results.lock().unwrap();
                // Push and increment
                if res_guard.len() < length {
                    res_guard.push(sr);
                    count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    // Unwrap the Arc and Mutex to return the inner Vec
    Arc::try_unwrap(results).expect("Arc has other owners").into_inner().unwrap()
}

#[pyfunction(signature = (n1, n2, u_target, max_iter=100000))]
pub fn simrank_single(
    n1: usize,
    n2: usize,
    u_target: f64,
    max_iter: usize
) -> Option<SimRank> {
    let s = simrank(n1, n2, u_target, 1, max_iter);
    s.into_iter().next()
}

/// Generates simulated rank tests with tied values for half-integer U-scores.
///
/// This function is the tied-value counterpart to `simrank`. While `simrank` handles
/// integer U-values with distinct ranks, `simrank_tied` handles half-integer U-values
/// (e.g., 6.5, 14.5) which can only occur when at least one pair of values are tied.
///
/// The algorithm works by:
/// 1. Randomly selecting a tie position k (meaning positions k and k+1 share rank k+0.5)
/// 2. Assigning one tied value to group 1 and one to group 2
/// 3. Sampling the remaining n1-1 positions for group 1 from untied positions
/// 4. Checking if the resulting rank sum matches the target
///
/// # Arguments
///
/// * `n1` - The number of observations in group 1.
/// * `n2` - The number of observations in group 2.
/// * `u_target` - The target U-value (must be a half-integer like 6.5, 14.5, etc.).
/// * `length` - How many simulated rank tests to generate.
/// * `max_iter` - Maximum samples before terminating if `length` results not found.
///
/// # Returns
///
/// Returns a `Vec<SimRankTied>` containing rank assignments where both groups include
/// at least one tied (half-integer) rank value.
///
/// # Notes
///
/// This function only generates configurations with exactly one tied pair split across
/// groups. For the case where both tied values are in the same group (resulting in an
/// integer U despite ties), use the standard `simrank` function.
#[pyfunction(signature = (n1, n2, u_target, length=1, max_iter=100000))]
pub fn simrank_tied(
    n1: usize,
    n2: usize,
    u_target: f64,
    length: usize,
    max_iter: usize,
) -> Vec<SimRankTied> {
    // Early return for invalid inputs
    if n1 == 0 || n2 == 0 || length == 0 {
        return Vec::new();
    }

    let r1_target = u_target + (n1 as f64) * (n1 as f64 + 1.0) / 2.0;
    let n_total = n1 + n2;

    // Need at least 2 positions to have a tie
    if n_total < 2 {
        return Vec::new();
    }

    let results = Arc::new(Mutex::new(Vec::with_capacity(length)));
    let count = Arc::new(AtomicUsize::new(0));

    (0..max_iter).into_par_iter().for_each(|_| {
        if count.load(Ordering::Relaxed) >= length {
            return;
        }

        let mut rng = rng();

        // Pick a random tie position (1 to n_total-1)
        // This means positions tie_pos and tie_pos+1 will share rank tie_pos+0.5
        let tie_pos: usize = rng.random_range(1..n_total);
        let tied_rank = tie_pos as f64 + 0.5;

        // Remaining positions (excluding the tied pair at tie_pos and tie_pos+1)
        let remaining: Vec<usize> = (1..tie_pos)
            .chain((tie_pos + 2)..=n_total)
            .collect();

        // We need n1-1 positions from remaining for group 1 (one position comes from the tie)
        let needed = n1.saturating_sub(1);
        if remaining.len() < needed {
            return;
        }

        // Sample n1-1 positions from remaining for group 1
        let sample_indices = rand::seq::index::sample(&mut rng, remaining.len(), needed);
        let other_g1: Vec<usize> = sample_indices.iter().map(|i| remaining[i]).collect();

        // Calculate rank sum for group 1: tied_rank + sum of other selected ranks
        let other_sum: usize = other_g1.iter().sum();
        let r1_actual = tied_rank + other_sum as f64;

        if r1_actual == r1_target 
        && count.load(Ordering::Relaxed) < length {
            // Build group 1 ranks: the tied rank + other selected integer ranks
            let mut g1: Vec<f64> = other_g1.iter().map(|&r| r as f64).collect();
            g1.push(tied_rank);
            g1.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Build group 2 ranks: the other tied rank + unselected remaining positions
            let g1_set: HashSet<usize> = other_g1.iter().cloned().collect();
            let mut g2: Vec<f64> = remaining
                .iter()
                .filter(|r| !g1_set.contains(r))
                .map(|&r| r as f64)
                .collect();
            g2.push(tied_rank);
            g2.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let sr = SimRankTied { n1: g1, n2: g2 };

            let mut res_guard = results.lock().unwrap();
            if res_guard.len() < length {
                res_guard.push(sr);
                count.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    Arc::try_unwrap(results)
        .expect("Arc has other owners")
        .into_inner()
        .unwrap()
}

/// Single-result convenience wrapper for `simrank_tied`.
#[pyfunction(signature = (n1, n2, u_target, max_iter=100000))]
pub fn simrank_tied_single(
    n1: usize,
    n2: usize,
    u_target: f64,
    max_iter: usize,
) -> Option<SimRankTied> {
    let s = simrank_tied(n1, n2, u_target, 1, max_iter);
    s.into_iter().next()
}

