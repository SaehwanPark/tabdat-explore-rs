# Bounded doubly robust difference-in-differences `drdid` syntax slice summary

## Outcome

Accepted bounded language-layer doubly robust difference-in-differences `drdid` syntax slice. PR
[#115](https://github.com/SaehwanPark/tabdat-explore-rs/pull/115) was merged to
`main` as
[`063bc3b`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/063bc3b940989f64bf74d75432616231d6d84877).

The language layer now exposes typed `DrDidMethod` enum, `DrDidCommand` struct, and `Command::DrDid` AST variant for:
- `drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]`

The parser retains the dependent variable (`outcome`), optional covariate variables (`covariates`),
treatment indicator variable (`treatment_variable`), post-treatment time period indicator variable (`post_variable`),
estimation method (`method`, defaulting to `DrDidMethod::Aipw`), robust covariance flag (`robust`),
number of bootstrap replications (`bootstrap: Option<i64>`), and random seed (`seed: Option<i64>`).
It enforces exact Python-compatible diagnostics for argument counts (< 1 argument), conditions (`if ...`),
assignment syntax (`drdid=`), delimiter guards (`drdid==`, `drdid:`), required option validation (`drdid option treat expects one variable`,
`drdid option post expects one variable`), single-use rules (`treat`, `post`, `method`, `bootstrap`, `seed`),
flag-without-value checks (`drdid option robust does not accept a value`, `drdid option method expects a value`,
`drdid option bootstrap expects an integer value`, `drdid option seed expects an integer value`),
valid enum values for method (`drdid option method must be one of: or, ipw, aipw`), numeric range limits
(`drdid option bootstrap must be at least 1`, `drdid option seed must be at least 0`), seed dependency on bootstrap
(`drdid option seed requires option bootstrap`), variable relationship constraints (`drdid treatment and post variables must be distinct`,
`drdid treatment and post variables must differ from outcome`, `drdid treatment and post variables must not appear in covariates`),
and unsupported options (alphabetically sorted).
Owned `String`, `Vec<String>`, `DrDidMethod`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "drdid" }`.

No numerical estimation, propensity score modeling, outcome regression, bootstrapping,
or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only doubly robust difference-in-differences `drdid` boundary is closed; numerical estimation,
propensity score weighting, bootstrapping, and statistical validation remain unchecked in the roadmap.
