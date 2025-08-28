import scrutipy as s
import time

def calculate_snspn(sensitivity, specificity, sample_size, tolerance=1e-6, show_progress=True, n_pathology=None):
    """
    Estimate original confusion matrix values from sensitivity, specificity, and sample size.
    Returns a DataFrame of possible confusion matrices.
    """
    results = []
    total_iterations = (sample_size + 1) ** 4
    for tp in range(sample_size + 1):
        for tn in range(sample_size + 1):
            for fp in range(sample_size + 1):
                for fn in range(sample_size + 1):
                    if (tp + tn + fp + fn) != sample_size:
                        continue
                    if n_pathology is not None and (tp + fn) != n_pathology:
                        continue
                    calc_sens = tp / (tp + fn) if (tp + fn) else 0.0
                    calc_spec = tn / (tn + fp) if (tn + fp) else 0.0
                    sens_error = abs(sensitivity - calc_sens)
                    spec_error = abs(specificity - calc_spec)
                    total_error = sens_error + spec_error
                    results.append({
                        'TP': tp,
                        'TN': tn,
                        'FP': fp,
                        'FN': fn,
                        'Calculated_Sensitivity': calc_sens,
                        'Calculated_Specificity': calc_spec,
                        'Sensitivity_Error': sens_error,
                        'Specificity_Error': spec_error,
                        'Total_Error': total_error,
                        'Exact_Match': total_error <= tolerance
                    })

# l = s.calculate_snspn(0.8, 0.70588, 20, top_n=5)
#
# print(l)
for i in [200]:
    pt = time.time(); calculate_snspn(0.8, 0.70588, i); pe = time.time()

    rt = time.time(); s.calculate_snspn(0.8, 0.70588, i); re = time.time()

    p = pe - pt
    r = re - rt
    speedup = p/r

    print(f"N = {i}     Python: {p:.4f}, Rust: {r:.4f}, Speedup: {speedup:.2f}x")




# def main():
#     sensitivity = 0.80
#     specificity = 0.70588
#     sample_size = 200
#     print(f"Estimating for Sensitivity={sensitivity}, Specificity={specificity}, n={sample_size}")
#     calculate_snspn(sensitivity, specificity, sample_size)
#
# if __name__ == "__main__":
#     main()
