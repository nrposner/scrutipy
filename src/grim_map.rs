use crate::grim::grim_rust;
use crate::grim_stats::grim_probability;
//use crate::utils::decimal_places_scalar;
use serde::Deserialize;
use std::error::Error;

// TODO 3/12/35
// add capacity for grim_map to take in either a csv file directly, loading it internally, or a
// pandas/polars dataframe. Look into how PyO3 encodes that. Probably will need to include keyword
// arguments on the python side to select the right columns in cases where there are more than two,
// custom errors if the columns don't include the right kinds of objects, checks and coercions up
// front to make this flexible and easy to use. It'll be a good challenge

/// for the moment, implementing without show_rec
/// we want to take in a dataframe, be able to take in some arguments for which column contains
/// means (as strings) and which one contains counts, and if we don't get one or the other, we default to the
/// first and second columns respectively

// in the actual csv, the index is an empty string, and the mean and count columns are "x" and "n"
// we're renaming them here for the sake of readability
// we also addd a _ in front of the field names in order to silence unused field warnings, since so
// far this is only used in the testing section
// unless we end up using this struct more in the transition between rust and python dataframes,
// move it to the testing section in order to not have something so ugly
#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    #[serde(rename = "")]
    _index: usize,
    #[serde(rename = "x")]
    _mean: String,
    #[serde(rename = "n")]
    _count: u32,
}

fn _load_csv(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    Ok(records)
}

pub fn grim_map(
    xs: Vec<String>,
    ns: Vec<u32>,
    bool_params: Vec<bool>,
    items: Vec<u32>,
    rounding: Vec<&str>,
    threshold: f64,
    tolerance: f64,
) -> (Vec<bool>, Vec<f64>) {
    let xs: Vec<&str> = xs.iter().map(|s| &**s).collect(); // idiomatic way to turn
                                                           // Vec<String> into Vec<&str>

    let consistencies: Vec<bool> = grim_rust(
        xs.clone(),
        ns.clone(),
        bool_params,
        items,
        rounding,
        threshold,
        tolerance,
    );

    let probs: Vec<f64> = xs
        .clone()
        .iter()
        .zip(ns.clone().iter())
        .map(|(x, n)| grim_probability(x, *n, 1, false))
        .collect();

    (consistencies, probs)
}

#[cfg(test)]
pub mod tests {
    use core::f64;

    use super::*;

    #[test]
    fn grim_map_pigs1_test_1() {
        let records = _load_csv("data/pigs1.csv").unwrap();

        let xs: Vec<String> = records.iter().map(|rec| rec._mean.clone()).collect();

        let ns: Vec<u32> = records.iter().map(|rec| rec._count).collect();

        let vals = grim_map(
            xs,
            ns,
            vec![false, false, false],
            vec![1u32; 12],
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );

        assert_eq!(
            vals.0,
            vec![true, false, false, false, false, true, false, true, false, false, true, false]
        );

        assert_eq!(
            vals.1,
            vec![0.68, 0.75, 0.71, 0.76, 0.73, 0.72, 0.71, 0.74, 0.73, 0.69, 0.75, 0.72]
        );
    }
}
