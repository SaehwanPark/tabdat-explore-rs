# Bounded `bayes:` prefixed command syntax slice summary

## Outcome

Accepted bounded language-layer `bayes:` prefix syntax slice. PR
[#83](https://github.com/SaehwanPark/tabdat-explore-rs/pull/83) was merged to
`main` as
[`9e2ba7c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/9e2ba7c).

The language layer now exposes typed `BayesPrefixCommand` for prefixed
`bayes [, options]: <command>` syntax, holding an inner `command: Box<Command>`
(strictly validated to be `Command::Regress` or `Command::Logit`), MCMC parameters
`draws`, `burnin` (with alias `tune`), `chains`, `thin`, `seed` (with alias `rseed`),
and ordered custom prior specifications `Vec<(String, String)>`. The parser supports
complex parenthesized expressions such as `prior(x, normal(0, 10))` and backtick-quoted
identifiers, while enforcing exact Python-compatible diagnostics for missing commands,
malformed prefix options, non-numeric option values, unsupported options, and
unsupported inner commands. Runtime execution remains an explicit unsupported-command
result.

No MCMC sampling engine (Stan, PyMC, NUTS), posterior storage, distribution
calculus, model results, post-estimation (`estat bayes`, `bayesplot`), CLI/JSON/MCP
surface, or broader Bayesian model families were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `bayes:` prefix boundary dependency is closed; Bayesian
MCMC estimation and post-estimation remain unchecked in the roadmap.
