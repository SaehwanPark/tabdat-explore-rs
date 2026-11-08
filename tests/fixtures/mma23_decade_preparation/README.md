# Full-decade preparation control

Artificial40-firm source is shared with
`../mma23_gamma_re_bootstrap/source.asc`; no publisher data is committed.
`full.csv` has400 keyed1970–79 rows and21 fields; `selected.csv` has200 rows
selected only AFTER all five lags are formed. `summaries.csv` has42 variable-wise
cohort/variable records with N, mean, sample SD (N-1), minimum and maximum.

Regenerate using development-only base R4.6.2:

```sh
Rscript examples/mma23p1pannonlin/decade_preparation_reference.R \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc NEW_OUTPUT_DIRECTORY
```

The reference reads all25 raw tokens per firm and indexes the original ten-year
history directly, not SQL windows or native derived exports. Seventeen-digit
numeric CSV preserves binary64 values; blank cells mean missing, including the
first j years of lag j. Source numeric fields/keys/missingness compare exactly;
derived values/summaries use1e-10+1e-12*abs(reference). Native labels and explicit
DOUBLE ARDSSIC are semantic adaptations, not historical Stata storage parity.
The numerical Cargo tests consume these frozen files without an R runtime; the
separate hostile-artifact test invokes Python for the independent checker.

The workflow and bounded validation are described in
[preparation notes](../../../examples/mma23p1pannonlin/decade_preparation.md).
