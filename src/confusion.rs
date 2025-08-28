use std::cmp::Ordering;

use pyo3::{pyfunction, types::{PyDict, PyDictMethods, PyList, PyListMethods}, PyObject, PyResult, Python};
use indicatif::ProgressIterator;

struct SnspnReturn {
    tp: u32,
    tn: u32,
    fp: u32,
    f_n: u32,
    calc_sens: f32,
    calc_spec: f32,
    sens_error: f32,
    spec_error: f32,
    total_error: f32,
    exact_match: bool,
}

#[pyfunction(signature = (sensitivity, specificity, sample_size, tolerance=1e-6, n_positive=None, top_n=None))]
pub fn calculate_snspn(
    py: Python,
    sensitivity: f32, 
    specificity: f32, 
    sample_size: u32, 
    tolerance: f32, 
    n_positive: Option<u32>,
    top_n: Option<u32>,
) -> PyResult<PyObject> {
    let mut results = Vec::new();

    let n_path_bool = n_positive.is_some();

    for tp in (0..sample_size+1).progress() { // because apparently std::iter::ExactSizeIterator is
        // not implemented on inclusive ranges https://github.com/rust-lang/rust/issues/36386
        for tn in 0..=(sample_size - tp) {
            for fp in 0..=(sample_size - tp - tn) {
                let f_n = sample_size - tp - tn - fp;

                if (tp + tn + fp + f_n) != sample_size {
                    continue
                }
                if n_path_bool && (tp + f_n) != n_positive.unwrap() {
                    continue
                }

                let calc_sens = if tp + f_n != 0 {
                    tp as f32 / (tp + f_n) as f32
                } else {
                    0.0
                };

                let calc_spec = if tn + fp != 0 {
                    tn as f32 / (tn + fp) as f32
                } else {
                    0.0
                };

                let sens_error = (sensitivity - calc_sens).abs();
                let spec_error = (specificity - calc_spec).abs();
                let total_error = sens_error + spec_error;

                results.push(SnspnReturn { 
                    tp,
                    tn,
                    fp, 
                    f_n, 
                    calc_sens,
                    calc_spec,
                    sens_error,
                    spec_error,
                    total_error,
                    exact_match: total_error <= tolerance,
                });
            }
        }
    }

    results.sort_by(|a, b| 
        a.total_error.partial_cmp(&b.total_error).unwrap_or(Ordering::Equal) 
    );

    // taking just the n smallest total_errors
    let results: Vec<SnspnReturn> = if let Some(n) = top_n {
        if n < sample_size {
            results.into_iter().take(n as usize).collect()
        } else { results }
    } else { results };

    let dicts = PyList::empty(py);

    for result in results {
        let dict = PyDict::new(py);
        dict.set_item("TP", result.tp)?;
        dict.set_item("TN", result.tn)?;
        dict.set_item("FP", result.fp)?;
        dict.set_item("FN", result.f_n)?;
        dict.set_item("Calculated_Sensitivity", result.calc_sens)?;
        dict.set_item("Calculated_Specificity", result.calc_spec)?;
        dict.set_item("Sensitivity_Error", result.sens_error)?;
        dict.set_item("Specificity_Error", result.spec_error)?;
        dict.set_item("Total_Error", result.total_error)?;
        dict.set_item("Exact_Match", result.exact_match)?;

        dicts.append(dict)?;
    }
    Ok(dicts.into())
}

struct PpvReturn {
    tp: u32,
    tn: u32,
    fp: u32,
    f_n: u32,
    calc_ppv: f32,
    calc_npv: f32,
    ppv_error: f32,
    npv_error: f32,
    total_error: f32,
    exact_match: bool,
}

#[pyfunction(signature = (ppv, npv, sample_size, tolerance=1e-6, n_positive=None, top_n=None))]
pub fn calculate_ppvnpv(
    py: Python,
    ppv: f32, 
    npv: f32, 
    sample_size: u32, 
    tolerance: f32, 
    n_positive: Option<u32>,
    top_n: Option<u32>,
) -> PyResult<PyObject> {
    let mut results = Vec::new();

    let n_path_bool = n_positive.is_some();

    for tp in (0..sample_size+1).progress() { // because apparently std::iter::ExactSizeIterator is
        // not implemented on inclusive ranges https://github.com/rust-lang/rust/issues/36386
        for tn in 0..=(sample_size - tp) {
            for fp in 0..=(sample_size - tp - tn) {
                let f_n = sample_size - tp - tn - fp;

                if (tp + tn + fp + f_n) != sample_size {
                    continue
                }
                if n_path_bool && (tp + f_n) != n_positive.unwrap() {
                    continue
                }

                let calc_ppv = if tp + fp != 0 {
                    tp as f32 / (tp + fp) as f32
                } else {
                    0.0
                };

                let calc_npv = if tn + f_n != 0 {
                    tn as f32 / (tn + f_n) as f32
                } else {
                    0.0
                };

                let ppv_error = (ppv - calc_ppv).abs();
                let npv_error = (npv - calc_npv).abs();
                let total_error = ppv_error + npv_error;

                results.push(PpvReturn { 
                    tp,
                    tn,
                    fp, 
                    f_n, 
                    calc_ppv,
                    calc_npv,
                    ppv_error,
                    npv_error,
                    total_error,
                    exact_match: total_error <= tolerance,
                });
            }
        }
    }
    // sort the structs by total_error
    results.sort_by(|a, b| 
        a.total_error.partial_cmp(&b.total_error).unwrap_or(Ordering::Equal) 
    );

    // taking just the n smallest total_errors
    let results: Vec<PpvReturn> = if let Some(n) = top_n {
        if n < sample_size {
            results.into_iter().take(n as usize).collect()
        } else { results }
    } else { results };

    let dicts = PyList::empty(py);

    for result in results {
        let dict = PyDict::new(py);
        dict.set_item("TP", result.tp)?;
        dict.set_item("TN", result.tn)?;
        dict.set_item("FP", result.fp)?;
        dict.set_item("FN", result.f_n)?;
        dict.set_item("Calculated_PPV", result.calc_ppv)?;
        dict.set_item("Calculated_NPV", result.calc_npv)?;
        dict.set_item("PPV_Error", result.ppv_error)?;
        dict.set_item("NPV_Error", result.npv_error)?;
        dict.set_item("Total_Error", result.total_error)?;
        dict.set_item("Exact_Match", result.exact_match)?;

        dicts.append(dict)?;
    }
    Ok(dicts.into())
}

