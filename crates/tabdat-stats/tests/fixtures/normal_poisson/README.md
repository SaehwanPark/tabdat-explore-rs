# Normal-intercept Poisson posterior integration controls

`posterior.csv` contains six artificial `(total,mean_sum,log_variance)` cases
and eight independent posterior quantities per case: log integrated factor,
E(exp(u)),E(u²),Var(exp(u)),Cov(exp(u),u²),Var(u²),E(u),Cov(exp(u),u). The integrated factor is
E_normal[exp(total*u−mean_sum*exp(u))]; row-level y*Xbeta−log(y!) constants
are not included. The cases cover zero totals, small/large intensities,100000
counts, different variances and posterior concentration.

Generate with base R4.6.2, using adaptive improper integration, not native GH:

```sh
Rscript crates/tabdat-stats/tests/fixtures/normal_poisson/reference.R /tmp/posterior.csv
cmp /tmp/posterior.csv crates/tabdat-stats/tests/fixtures/normal_poisson/posterior.csv
```

R independently brackets the posterior mode with uniroot and integrates over
the entire standardized real line. Density is anchored at the mode and
centered products avoid raw-second-moment subtraction. Scalar gates are
1e-9+1e-10*abs(reference), not widened for numerical disagreement.

The176 fixed80/96 physicists' GH nodes in `src/normal_poisson/nodes.rs` are from
`examples/mma23p1pannonlin/normal_re_quadrature_nodes.R`, run under R4.6.2.
Eigenvalues of the symmetric Jacobi matrix give nodes; orthogonal-polynomial
recurrence gives positive weights even at tiny tails (not eigenvector first
components). Rust literals are shortest round-trip representations of the
R17-digit values. Unit checks validate positive/ordered rules and exact
Gaussian moments through degree25, rather than trusting the generated file.
The private/test-only copied-refit prototype adds128/160 rules in
`src/normal_poisson/refined_nodes.rs`, reproducible with the same generator's
explicit128160 arguments (`... /tmp/nodes.sql 128 160`). All288 new pairs were
checked bit-for-bit against fresh R output; all four rules pass the same
moment/positive-tail checks. Ordinary public normal fits still use80/96 and
the176 work factor. Refined copied fitting, raw original-normal warm starts and
its288 work factor are not a public bootstrap command or precision option.

Paired-rule differences are checked in intensity/prior-score units:
`[log_integral,M*EexpU,Eu²/v,M²*VarExpU,M*CovExpUu²/v,VarU²/v²,Eu/v,M*CovExpUu/v]`.
Integration-by-parts identities give stable count-total score Eu/v and
information M*Cov(exp(u),u)/v instead of subtracting O(total) intensity moments.
Independent direct-R values plus Richardson score differentiation check the
identity up to100000 counts; no statistical parameterization changed.
An absolute gate on tiny unscaled exp(u) moments would not adequately control
information after multiplication. The check remains
`abs(high-low)/(1+abs(high))<=1e-9`, a discrepancy check, not an error bound. High-variance t=2
R-GLM starts on the existing40-firm source actually trigger rejection.

These fixtures isolate the posterior integration component. The production
normal-RE fitter additionally has full-inference/sample controls in the
multivariate fixture and the Chapter23 native workflow; posterior controls alone
do not establish whole-estimator or historical replication. R/Python remain
development references, never production dependencies.
