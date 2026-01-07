use rand::SeedableRng;
use pyo3::prelude::*;
use rand::rngs::SmallRng;
use rand::seq::index::sample;
use rayon::prelude::*;



#[pyfunction(signature = (n1, n2, u_target, max_iter=100000))]
pub fn simrank(
    n1: usize, 
    n2: usize, 
    u_target: f64,
    max_iter: usize
) -> Option<(Vec<usize>, Vec<usize>, f64)>  {

    let total_ranks = 1..=(n1+n2);
    let r1_target = u_target + (n1 as f64) * (n1 as f64 + 1.0) / 2.0;

    for _ in 0..max_iter {
        // draw without replacement
        let mut group1_ranks = sample(&mut SmallRng::from_os_rng(), n1+n2, n1).into_vec();

        if group1_ranks.iter().sum::<usize>() as f64 == r1_target {
            group1_ranks.sort();
            let group2_ranks: Vec<usize> = total_ranks.filter(|i| !group1_ranks.contains(i)).collect();
            // find all those elements in total_ranks that do not appear in group1_ranks

            return Some(
            (group1_ranks, group2_ranks, u_target)
            )
        }
    }
    None
}

#[pyfunction(signature = (n1, n2, u_target, max_iter=100000))]
pub fn simrank_parallel(
    n1: usize, 
    n2: usize, 
    u_target: f64,
    max_iter: usize
) -> Option<(Vec<usize>, Vec<usize>, f64)>  {

    let r1_target = u_target + (n1 as f64) * (n1 as f64 + 1.0) / 2.0;

    (0..max_iter).into_par_iter().filter_map(|_| { 
        // draw without replacement
        let mut group1_ranks = sample(&mut SmallRng::from_os_rng(), n1+n2, n1).into_vec();

        if group1_ranks.iter().sum::<usize>() as f64 == r1_target {
            group1_ranks.sort();
            let group2_ranks: Vec<usize> = (1..=(n1+n2)).filter(|i| !group1_ranks.contains(i)).collect();
            // find all those elements in total_ranks that do not appear in group1_ranks
            Some((group1_ranks, group2_ranks, u_target))
        } else {
            None
        } 
    }).find_any(|_| true )
}

