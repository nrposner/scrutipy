# scrutiPy v0.1.0: Scientific error detection in Python

A library for scientific error checking and fraud detection, based on the R Scrutiny library by Lukas Jung. Frontend API in Python 3, backend in Rust with PyO3 bindings. 

Currently in early development. Presently available functions include:

grim_scalar(): Implements the GRIM test on single observations. 

```
from scrutipy import grim_scalar

grim_scalar("5.19", 40)
# False
```

grim_map() Implements the GRIM test on Pandas dataframes. Use the variant grim_map_pl() for Polars dataframes. Both functions require Polars, which can be enabled using `pip install scrutipy[polars]` or `pip install polars`

```
import pandas as pd
from scrutipy import grim_map 

df = pd.read_csv("data/pigs1.csv")
# it may be necessary to explicitly convert your x column to string type in order to avoid losing trailing zeros. In the event that trailing zeros may be lost, the function will throw a warning 
df["x"] = df["x"].astype(str) 
bools, errors = grim_map(df, 1, 2)
# list([True, False, False, False, False, True, False, True, False, False, True, False])
```

# Roadmap

Expand documentation

Test and document user-side GRIMMER function 

Tidy up return types as dataframes

Implicitly maintain x_col as str when appropriate

Implement DEBIT

Implement SPRITE

Implement CORVIDS or other deterministic diophantine data reconstruction algorithm. 


# Acknowledgements

Lukas Jung

Nick Brown

James Heathers

Jordan Anaya

Aurelien Allard
