# Independent normal-RE bootstrap control

Artificial200-row/40-firm/five-year source shared with
`../mma23_gamma_re_bootstrap/source.asc`, SHA256
`24d60c4cde69fb4967f97d27a7237383921362f72bd272a925bc181200b45bca`.
Eight zero-total groups remain. No publisher data is committed here.

`fits.csv`:500 independent joint intercept/logr/log-variance refits, marginal
and same-copy full-X pooled log likelihoods, and three signed raw scores.
`reference.csv`:31 original-point/population-mean and empirical mean/bias/full
B-1 covariance/SE/type7 percentile95 quantities. Log variance is not alpha or a
natural-sigma bootstrap interval, and testing log variance0 is not testing
no heterogeneity.

Reproduce from the repo root with R4.6.2 (development oracle only):

```sh
python3 examples/mma23p1pannonlin/native_normal_re_bootstrap_reference.py \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc DRAWS_CSV NEW_OUTPUT_DIR \
  --control --jobs 4
```

DRAWS_CSV is the shared native500-replicate/40-draw seed10001 stream (also used
by the native gamma bootstrap); the driver independently verifies every index,
position, rejection step and ordinal firm label before invoking R. Each repeated
firm is a distinct Gaussian effect, not merged with its original ID. Each R fit
starts from its own GLM/t0, never native coefficients; adaptive improper
integration/complete-data scores/information and BFGS/Newton are independent of
native GH/Stein/whitened Newton. Full information must be PD during Newton/final
and raw max score<1e-8; unused BFGS information and per-copy Wald/Richardson
exports are omitted. Base-R cov/quantile provide independent empirical summaries.
Actual reference run60.361s. Frozen model SHA256
`c822000ed71943c3efa0419dd4094ad0877b872747c8b5582b1236c1131bf835`, summary
`ea3687bf7f3f4221a62513da76643b5a3c940a99907e0feb4d56fa496bc84862`.

Native copied fitting now uses fixed176/208 GH nodes (384 work factor), because
128/160 and160/192 disagree beyond1e-9 before copy105 reaches its independently
verified t2.0836168535458266 optimum. No gate relaxation/redraw. All500 native
joint triples and empirical covariance/bias/SE/type7 match fixed scalar gates
1e-9+1e-10*abs(reference); separate development empirical checks also use scale-
relative covariance identities. These frozen fixtures have no R/Python runtime dependency
in Cargo tests. The separate
[terminal/script workflow](../../../examples/mma23p1pannonlin/native_normal_re_bootstrap.md)
now validates read-only state/file/report behavior and every copied-fit raw score;
20 bootstrap forgeries include coherent wrong native/R artifacts.
