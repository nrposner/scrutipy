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

struct PlrReturn {
    tp: u32,
    tn: u32,
    fp: u32,
    f_n: u32,
    calc_plr: f64,
    calc_nlr: f64,
    plr_error: f64,
    nlr_error: f64,
    total_error: f64,
    exact_match: bool,
}

#[pyfunction(signature = (plr, nlr, sample_size, tolerance=1e-6, n_positive=None, top_n=None))]
pub fn calculate_likelihoodratios(
    py: Python,
    plr: f64, 
    nlr: f64, 
    sample_size: u32, 
    tolerance: f64, 
    n_positive: Option<u32>, 
    top_n: Option<u32>
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

                let sens = if tp + f_n != 0 {
                    tp as f64 / (tp + f_n) as f64
                } else {0.0};

                let spec = if tn + fp != 0 {
                    tn as f64 / (tn + fp) as f64
                } else {0.0};

                let calc_plr = if 1.0 - spec != 0.0 {
                    sens / (1.0 / spec)
                } else {f64::INFINITY};

                let calc_nlr = if spec != 0.0 {
                    (1.0 - sens) / spec
                } else {f64::INFINITY};

                let plr_error = (plr - calc_plr).abs();
                let nlr_error = (nlr - calc_nlr).abs();

                let total_error = plr_error + nlr_error;

                results.push(PlrReturn {
                    tp, 
                    tn, 
                    fp, 
                    f_n, 
                    calc_plr, 
                    calc_nlr, 
                    plr_error, 
                    nlr_error, 
                    total_error, 
                    exact_match: total_error <= tolerance
                })
            }
        }
    }

    // sort the structs by total_error
    results.sort_by(|a, b| 
        a.total_error.partial_cmp(&b.total_error).unwrap_or(Ordering::Equal) 
    );

    // taking just the n smallest total_errors
    let results: Vec<PlrReturn> = if let Some(n) = top_n {
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
        dict.set_item("Calculated_PLR", result.calc_plr)?;
        dict.set_item("Calculated_NLR", result.calc_nlr)?;
        dict.set_item("PLR_Error", result.plr_error)?;
        dict.set_item("NLR_Error", result.nlr_error)?;
        dict.set_item("Total_Error", result.total_error)?;
        dict.set_item("Exact_Match", result.exact_match)?;

        dicts.append(dict)?;
    }
    Ok(dicts.into())
}

#[pyfunction]
pub fn calculate_metrics_from_counts(
    py: Python,
    tp: u32, 
    tn: u32, 
    fp: u32, 
    f_n: u32
) -> PyResult<PyObject> {

    let sensitivity = if tp+f_n != 0 {
        tp as f64 / (tp + f_n) as f64
    } else { 0.0 };
    let specificity = if tn+fp != 0 {
        tn as f64 / (tn + fp) as f64
    } else { 0.0 };
    let ppv = if tp + fp != 0 {
        tp as f64/ (tp + fp) as f64
    } else { 0.0 };
    let npv = if tn + f_n != 0 {
        tn as f64 / (tn + f_n) as f64
    } else { 0.0 };

    let plr = if (1.0 - specificity) != 0.0 {
        sensitivity / (1.0 - specificity)
    } else {f64::INFINITY};

    let nlr = if specificity != 0.0 {
        (1.0 - sensitivity) / specificity
    } else {f64::INFINITY};

    let dict = PyDict::new(py);

    dict.set_item("Sensitivity", sensitivity)?;
    dict.set_item("Specificity", specificity)?;
    dict.set_item("PPV", ppv)?;
    dict.set_item("NPV", npv)?;
    dict.set_item("+LR", plr)?;
    dict.set_item("-LR", nlr)?;

    Ok(dict.into())
}

