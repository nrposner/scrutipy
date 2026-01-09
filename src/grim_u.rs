use rand::rng;
use pyo3::prelude::*;
use rayon::prelude::*;
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
) -> SimRank {
    let s = simrank(n1, n2, u_target, 1, max_iter);
    s[0].clone()
}

