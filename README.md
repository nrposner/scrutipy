# scrutiPy v0.1.10: Scientific error detection in Python

A library for scientific error checking and fraud detection, based on the R Scrutiny library by Lukas Jung. Frontend API in Python 3, backend in Rust with PyO3 bindings. 

Currently in early development. Presently available functions include:

grim_scalar(): Implements the GRIM test on single observations. 

```
from scrutipy import grim_scalar

grim_scalar("5.19", 40)
# False
```

grim_map() Implements the GRIM test on Pandas dataframes. Use the variant grim_map_pl() for Polars dataframes. Both functions require Polars, which can be enabled using `pip install scrutipy[polars]` or `pip install polars`.

```
import pandas as pd
from scrutipy import grim_map 

df = pd.read_csv("data/pigs1.csv")
# it may be necessary to explicitly convert your x column to string type in order to avoid losing trailing zeros. In the event that trailing zeros may be lost, the function will throw a warning 
df["x"] = df["x"].astype(str) 
bools, errors = grim_map(df, 1, 2)

print(bools)
# list([True, False, False, False, False, True, False, True, False, False, True, False])

print(errors)
# None
```

grimmer() Implements the GRIMMER test on 1d iterables.

```
from scrutipy import grimmer
results = grimmer(["1.03", "52.13", "9.42375"], ["0.41", "2.26", "3.86"], [40, 30, 59], items = [1, 1, 1])

print(results)
# list(False, True, False) 

```

debit() implements the DEBIT test on 1d iterables (lists and arrays). 

```
from scrutipy import debit

results = debit(["0.36", "0.11", "0.118974"], ["0.11", "0.31", "0.6784"], [20, 40, 100])
print(results)
# list([False, True, False])
```

debit_map() implements the DEBIT test on Pandas dataframes. Use the variant debit_map_pl() for Polars dataframes. Both functions require Polars, which can be enabled using `pip install scrutipy[polars]` or `pip install polars`.

```
from scrutipy import debit_map 

df = pd.read_csv("data/debit_data.csv")
df["xs"] = df["xs"].astype(str) # ensuring that these columns are string types to silence a warning
df["sds"] = df["sds"].astype(str) # it can also be silenced with silence_numeric_warning = True.
results, errors = debit_map(df, 1, 2, 3)

print(bools)
# list([True, True, True, False, True, True, True])

print(errors)
# None
```

closure(): Implements the CLOSURE algorithm for recovering integer data from summary statistics. Any data which can be represented as integers on a bounded range, such as Likert scores, can be provably reconstructed using the mean, standard deviation, count, and range. 
This function replaces the CORVIDS algorithm, which relied on more advanced mathematics packages, with a simpler and faster algorithm. 
Note that even with CLOSURE's performance gains, the necessary time and compute to reconstruct data increases rapidly as range and count increase. 

```
# reconstruct possible datasets with a mean of 3.5, sd of 0.57, n = 100, 
# and inclusive range from 0 to 7. 
# We set the rounding error for the mean to 0.05 and for sd to 0.005

from scrutipy import closure
results = closure(3.5, 1.2, 50, 0, 7, 0.05, 0.005) 

len(results)
# 7980 
# indicates there are 7980 possible datasets with these characteristics.
```

calculate_snspn(): Calculates all possible confusion matries which could be produced from a sample size, and compares the calculated sensitivity and specificity to the input values. It returns a list of dictionaries containing the records for each possibility, as well as a total error and whether the total error is less than a certain tolerance. 
The dictionaries are ordered from least to greatest total error. For larger sample sizes, it is recommended to use a top_n argument to limit the number of returned values. The return can be trivially turned into a pandas or polars dataframe as seen below.
This is based on an application by Rod Whitely.
```
import pandas as pd
import scrutipy as s
vals = s.calculate_snspn(0.8, 0.70588, 20, top_n=5)
df = pd.DataFrame(vals)
df
   TP  TN  FP  FN  Calculated_Sensitivity  Calculated_Specificity  Sensitivity_Error  Specificity_Error  Total_Error  Exact_Match
0   8   7   3   2                0.800000                0.700000           0.000000           0.005880     0.005880        False
1   4  11   4   1                0.800000                0.733333           0.000000           0.027453     0.027453        False
2  10   5   2   3                0.769231                0.714286           0.030769           0.008406     0.039175        False
3   4  10   5   1                0.800000                0.666667           0.000000           0.039213     0.039213        False
4   5  10   4   1                0.833333                0.714286           0.033333           0.008406     0.041739        False
```

calculate_ppvnpv(): Calculates all possible confusion matries which could be produced from a sample size, and compares the calculated PPV and NPV to the input values. It returns a list of dictionaries containing the records for each possibility, as well as a total error and whether the total error is less than a certain tolerance. 
The dictionaries are ordered from least to greatest total error. For larger sample sizes, it is recommended to use a top_n argument to limit the number of returned values. The return can be trivially turned into a pandas or polars dataframe as seen below.
This is based on an application by Rod Whitely.

```
>>> import pandas as pd
>>> import scrutipy as s
>>> vals = s.calculate_ppvnpv(0.8, 0.70588, 20, top_n=5)
>>> df = pd.DataFrame(vals)
>>> df
   TP  TN  FP  FN  Calculated_PPV  Calculated_NPV  PPV_Error  NPV_Error  Total_Error  Exact_Match
0   8   7   2   3        0.800000        0.700000   0.000000   0.005880     0.005880        False
1   4  11   1   4        0.800000        0.733333   0.000000   0.027453     0.027453        False
2  10   5   3   2        0.769231        0.714286   0.030769   0.008406     0.039175        False
3   4  10   1   5        0.800000        0.666667   0.000000   0.039213     0.039213        False
4   5  10   1   4        0.833333        0.714286   0.033333   0.008406     0.041739        False
```

# Roadmap

Expand documentation

Test and document user-side GRIMMER function 

Tidy up return types as dataframes

Implicitly maintain x_col as str when appropriate

Implement SPRITE


# Acknowledgements

Lukas Jung

Nick Brown

James Heathers

Jordan Anaya

Aurelien Allard

Rod Whitely
