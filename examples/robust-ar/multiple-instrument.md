# Robust AR confidence sets with two through seventeen excluded instruments

## Implemented contract

After an HC1 or one-way CR1 2SLS fit with exactly one endogenous regressor and
two through seventeen excluded instruments, `estat ar, ci [level(95)]` inverts the
joint auxiliary-regression Wald test. Included exogenous coefficients remain
unrestricted. The reference is asymptotic chi-square(q), where q is the number
of excluded instruments. Both dimensions use the
`RobustMultipleInstrumentAndersonRubinConfidenceSet` JSON result type.

The HC1 correction is N/(N−K_aux). CR1 uses
G/(G−1)*(N−1)/(N−K_aux), with K_aux counting the intercept, included controls
and all excluded instruments. Auxiliary residuals supply row or cluster
scores. The boundary has degree at most four, six, eight, ten, twelve,
fourteen, sixteen, eighteen, twenty, twenty-two, twenty-four, twenty-six,
twenty-eight, thirty, thirty-two or thirty-four for two through seventeen instruments.
Every real boundary root is isolated through recursive derivative roots and
monotone-segment bisection; each set region and finite endpoint is checked
against the direct AR Wald test. The algorithm uses no finite beta grid. The
score covariance must stay nonsingular and numerically conditioned over the
real line; ambiguous roots, rank failure and numerical-range errors are
explicit. The current upper bound is seventeen instruments; q18 and above are rejected.


## Ten-instrument reference

[`ten_instrument_ci.td`](ten_instrument_ci.td) and
[`ten_instrument_ci_clustered.td`](ten_instrument_ci_clustered.td) extend the
same scalar workflow to ten excluded instruments. The deterministic designs have
240 rows, one included control, K_aux=12 and residual DF=228. The 95%
chi-square(10) critical value is 18.3070380532751; HC1 and CR1 corrections are
1.05263157894737 and 1.09382151029748 (24 groups).

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [0.954530263639083, 1.303078784421] |
| multiple | CR1 | [0.967341667827175, 1.2132537558033] |
| secondary | HC1 | [1.08640362505417, 1.35816576012896] |
| secondary | CR1 | [1.22845849212421, 1.35889967966114] |

The independent Base R 4.6.2 reference checks degree-twenty determinant
coefficients at 21 Chebyshev nodes and compares endpoints plus six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, corrections,
metadata, instrument rescaling, q=17 failure atomicity and JSON/text output. The
boundary solver is tested with twenty real roots and eleven accepted
components. Regenerate the data and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_ten_ci_reference.R "$work" \
  > "$work/robust_ar_ten_ci_reference.csv"
cmp examples/robust-ar/robust_ar_ten_ci_multiple_data.csv \
  "$work/robust_ar_ten_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_ten_ci_secondary_data.csv \
  "$work/robust_ar_ten_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_ten_ci_reference.csv \
  "$work/robust_ar_ten_ci_reference.csv"
```

## Eleven-instrument reference

[`eleven_instrument_ci.td`](eleven_instrument_ci.td) and
[`eleven_instrument_ci_clustered.td`](eleven_instrument_ci_clustered.td) extend
the same scalar workflow to eleven excluded instruments. The deterministic
designs have 240 rows, one included control, K_aux=13 and residual DF=227. The
95% chi-square(11) critical value is 19.6751375726825; HC1 and CR1 corrections
are 1.05726872246696 and 1.09864010725915 (24 groups).

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [1.1070615225938, 1.36180085509442] |
| multiple | CR1 | [1.23563485161083, 1.29735877469493] |
| secondary | HC1 | [1.1005376881432, 1.36072501094486] |
| secondary | CR1 | empty |

The independent Base R 4.6.2 reference checks the degree-22 determinant at 23
Chebyshev nodes and compares endpoints plus six direct Wald probes per design
and covariance. Rust checks endpoints within 3e-8, correction, metadata,
rescaling, retained state, JSON/text output and q=18 rejection. The solver
boundary test covers all 22 simple real roots and twelve accepted components.
Regenerate the data and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_eleven_ci_reference.R "$work" > "$work/robust_ar_eleven_ci_reference.csv"
cmp examples/robust-ar/robust_ar_eleven_ci_multiple_data.csv "$work/robust_ar_eleven_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_eleven_ci_secondary_data.csv "$work/robust_ar_eleven_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_eleven_ci_reference.csv "$work/robust_ar_eleven_ci_reference.csv"
```

## Twelve-instrument reference

[`twelve_instrument_ci.td`](twelve_instrument_ci.td) and
[`twelve_instrument_ci_clustered.td`](twelve_instrument_ci_clustered.td) extend
the same scalar workflow to twelve excluded instruments. The deterministic
designs have 288 rows, one included control, K_aux=14 and residual DF=274. The
95% chi-square(12) critical value is 21.0260698174831; HC1 and CR1 corrections
are 1.05109489051095 and 1.09298635353856 (24 groups).

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [1.03010254615802, 1.28953391130388] |
| multiple | CR1 | [1.14531210503888, 1.21665138888151] |
| secondary | HC1 | empty |
| secondary | CR1 | empty |

The independent Base R 4.6.2 reference checks degree-24 determinant
coefficients at 25 Chebyshev nodes, `polyroot` endpoints, and six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, metadata,
corrections, instrument rescaling, retained fit, JSON/text output and q=14
rejection. The boundary solver is tested with 24 real roots and thirteen
accepted components. Regenerate the fixtures and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_twelve_ci_reference.R "$work" \
  > "$work/robust_ar_twelve_ci_reference.csv"
cmp examples/robust-ar/robust_ar_twelve_ci_multiple_data.csv \
  "$work/robust_ar_twelve_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_twelve_ci_secondary_data.csv \
  "$work/robust_ar_twelve_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_twelve_ci_reference.csv \
  "$work/robust_ar_twelve_ci_reference.csv"
```

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.

## Thirteen-instrument reference

[`thirteen_instrument_ci.td`](thirteen_instrument_ci.td) and
[`thirteen_instrument_ci_clustered.td`](thirteen_instrument_ci_clustered.td)
extend the scalar HC1/CR1 workflow through thirteen excluded instruments. The
312-row designs have one included control, K_aux=15 and residual DF=297. At
95%, chi-square(13)=22.3620324948269; HC1/CR1 corrections are 1.05050505050505
and 1.09266578831796 (24 groups of thirteen observations).

| Design | HC1 | CR1 |
| --- | --- | --- |
| multiple | [1.17583577145803, 1.25479744110509] | empty |
| secondary | [1.06929085556435, 1.35865752789142] | empty |

The independent Base R 4.6.2 reference checks degree-26 determinant
coefficients at 27 Chebyshev nodes, `polyroot` endpoints and six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, metadata,
corrections, rescaling, retained state, JSON/text and q=18 rejection. The solver
boundary test covers 26 simple real roots and fourteen accepted components.
Regenerate the fixtures and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_thirteen_ci_reference.R "$work" \
  > "$work/robust_ar_thirteen_ci_reference.csv"
cmp examples/robust-ar/robust_ar_thirteen_ci_multiple_data.csv \
  "$work/robust_ar_thirteen_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_thirteen_ci_secondary_data.csv \
  "$work/robust_ar_thirteen_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_thirteen_ci_reference.csv \
  "$work/robust_ar_thirteen_ci_reference.csv"
```

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.

## Fourteen-instrument reference

[`fourteen_instrument_ci.td`](fourteen_instrument_ci.td) and
[`fourteen_instrument_ci_clustered.td`](fourteen_instrument_ci_clustered.td)
extend the scalar HC1/CR1 workflow through fourteen excluded instruments. The
336-row designs have one included control, K_aux=16 and residual DF=320. At
95%, chi-square(14)=23.6847913048406; HC1/CR1 corrections are 1.05 and
1.09239130434783 (24 groups of fourteen observations).

| Design | HC1 | CR1 |
| --- | --- | --- |
| multiple | [1.0483570279782, 1.27564768652857] | [1.09853101002636, 1.26311629893277] |
| secondary | [1.12815797073209, 1.31220370829782] | empty |

The independent Base R 4.6.2 reference checks degree-28 determinant
coefficients at 29 Chebyshev nodes, `polyroot` endpoints and six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, metadata,
corrections, rescaling, retained state, JSON/text and q=18 rejection. The solver
boundary test covers all 28 simple real roots and fifteen accepted components.
Regenerate the fixtures and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_fourteen_ci_reference.R "$work" \
  > "$work/robust_ar_fourteen_ci_reference.csv"
cmp examples/robust-ar/robust_ar_fourteen_ci_multiple_data.csv \
  "$work/robust_ar_fourteen_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_fourteen_ci_secondary_data.csv \
  "$work/robust_ar_fourteen_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_fourteen_ci_reference.csv \
  "$work/robust_ar_fourteen_ci_reference.csv"
```

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.

## Fifteen-instrument reference

[`fifteen_instrument_ci.td`](fifteen_instrument_ci.td) and
[`fifteen_instrument_ci_clustered.td`](fifteen_instrument_ci_clustered.td)
extend the scalar HC1/CR1 workflow through fifteen excluded instruments. The
360-row designs have one included control, K_aux=17 and residual DF=343. At
95%, chi-square(15)=24.9957901397286; HC1/CR1 corrections are
1.04956268221574 and 1.09215363163899 (24 groups of fifteen observations).

| Design | HC1 | CR1 |
| --- | --- | --- |
| multiple | [1.09021173848078, 1.27421949335487] | [1.18150148333289, 1.27347361322705] |
| secondary | [1.10032473852914, 1.28538623900457] | [1.1515371520644, 1.2281531524211] |

The independent Base R 4.6.2 reference checks degree-30 determinant
coefficients at 31 Chebyshev nodes, `polyroot` endpoints and six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, metadata,
corrections, rescaling, retained state, JSON/text and q=18 rejection. The solver
boundary test covers all 30 simple real roots and sixteen accepted components.
Regenerate the fixtures and reference with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_fifteen_ci_reference.R "$work" \
  > "$work/robust_ar_fifteen_ci_reference.csv"
cmp examples/robust-ar/robust_ar_fifteen_ci_multiple_data.csv \
  "$work/robust_ar_fifteen_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_fifteen_ci_secondary_data.csv \
  "$work/robust_ar_fifteen_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_fifteen_ci_reference.csv \
  "$work/robust_ar_fifteen_ci_reference.csv"
```

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.

## Independent evidence

The deterministic two-instrument fixture and Base R reference are generated by
[`robust_ar_multi_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci_reference.R).
It has 20 rows, one included control and ten two-row clusters; K_aux=4,
residual DF=16 and the 95% chi-square(2) critical value is 5.99146454710798.

| Design | Covariance | Correction | Confidence set |
| --- | --- | ---: | --- |
| multiple | HC1 | 1.25 | (-inf, 0.0628690191316964] ∪ [0.715734989894617, 1.07279148560636] ∪ [2.1595204517105, +inf) |
| multiple | CR1, 10 groups | 1.31944444444444 | (-inf, 0.154198964682854] ∪ [0.573634185761531, +inf) |
| all-real | HC1 | 1.25 | (-inf, +inf) |
| all-real | CR1, 10 groups | 1.31944444444444 | (-inf, +inf) |

The three-instrument fixture and Base R 4.6.2 reference are generated by
[`robust_ar_three_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_three_ci_reference.R).
Each design has 36 rows, one control, K_aux=5 and residual DF=31. The critical
chi-square(3) value is 7.81472790325118; corrections are 1.16129032258065 for
HC1 and 1.2316715542522 for CR1 with 12 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | (-inf, -1.69715344346743] ∪ [0.611957039995684, 0.742890085413437] ∪ [14.1578677030203, +inf) |
| multiple | CR1 | (-inf, -1.99475316355064] ∪ [6.53364270848149, +inf) |
| all-real | HC1 | (-inf, +inf) |
| six roots | CR1 | (-inf, -0.740832894629453] ∪ [0.0270725896734261, 0.999474559500985] ∪ [1.79260965464543, 2.68962633247009] ∪ [25.4733950912914, +inf) |

The last design has six finite boundary roots and four accepted components.
The R reference obtains the confidence regions from polynomial roots, then
checks endpoints and probes independently with auxiliary OLS HC1/CR1 Wald
statistics. Rust tests compare every endpoint and probe, check state retention,
instrument rescaling, JSON/text reporting, rank/covariance errors and rejection
above the supported instrument range.

The four-instrument fixture and Base R 4.6.2 reference are generated by
[`robust_ar_four_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_reference.R).
Each design has 48 rows, one control, K_aux=6 and residual DF=42. The critical
chi-square(4) value is 9.48772903678115; corrections are 1.14285714285714 for
HC1 and 1.22077922077922 for CR1 with 12 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [0.838181194785723, 1.84531003737431] |
| multiple | CR1 | [0.727037683486563, 1.76193191774842] |
| four roots | HC1 | [-1.71023017332182, 2.30329743770314] |
| four roots | CR1 | [-4.08201124973978, 0.25737090071055] ∪ [1.34492724853664, 2.72206249093528] |
| all-real label | HC1 and CR1 | Empty set |

The four-root CR1 case has four real boundary roots and two accepted
components. A polynomial unit test covers all eight possible roots and five
accepted components. Rust tests also compare reference probes and endpoints,
instrument rescaling, retained fit, text/JSON reporting and rank/covariance errors.

The five-instrument fixture and Base R 4.6.2 reference are generated by
[`robust_ar_five_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_five_ci_reference.R).
Each design has 100 rows, one control, K_aux=7 and residual DF=93. The critical
chi-square(5) value is 11.0704976935164; corrections are 1.0752688172043 for
HC1 and 1.12054329371817 for CR1 with 20 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [1.06421221080996, 1.39773070439002] |
| multiple | CR1 | [1.16236801651081, 1.39363336463301] |
| empty-cluster fixture | HC1 | [1.10344087914442, 1.42391918602994] |
| empty-cluster fixture | CR1 | Empty set |

All finite endpoints and direct Wald probes are compared with the independent
reference; the polynomial boundary has degree ten. A polynomial unit test
covers all ten possible roots and six accepted components. The tests also cover
rescaling, retained fit, text/JSON reporting, and rejection above the supported
instrument range.

The six-instrument fixture and Base R 4.6.2 reference are generated by
[`robust_ar_six_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_six_ci_reference.R).
Each design has 150 rows, one control, K_aux=8 and residual DF=142. The 95%
chi-square(6) critical value is 12.591587243744; corrections are 1.05633802816901
for HC1 and 1.09301643192488 for CR1 with 25 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [0.915364151803677, 1.53507912187761] |
| multiple | CR1 | [0.951605392208714, 1.45797127952922] |
| secondary | HC1 | [0.744654120556704, 1.63797231436378] |
| secondary | CR1 | [0.848220516944529, 1.52707474910469] |

Regenerate the inputs and reference from the repository root with Base R 4.6.2:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_six_ci_reference.R "$work" \
  > "$work/robust_ar_six_ci_reference.csv"
cmp examples/robust-ar/robust_ar_six_ci_multiple_data.csv \
  "$work/robust_ar_six_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_six_ci_secondary_data.csv \
  "$work/robust_ar_six_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_six_ci_reference.csv \
  "$work/robust_ar_six_ci_reference.csv"
```

The independent R reference builds the degree-twelve boundary and obtains its
real roots with `polyroot`; endpoints and fixed beta probes are checked with
direct auxiliary-OLS HC1/CR1 Wald statistics. Rust tests compare every reference
endpoint and probe within 3e-8, check metadata and rescaling, and exercise
retained-fit behavior. A polynomial unit test covers all twelve possible roots
and seven accepted components. The supported scalar result covers exactly one
endogenous regressor and at most seventeen excluded instruments; robust projected
inference remains separately bounded to two endogenous regressors and two
excluded instruments.

The seven-instrument fixtures and Base R 4.6.2 reference are generated by
[robust_ar_seven_ci_reference.R](../../crates/tabdat-stats/tests/fixtures/robust_ar_seven_ci_reference.R).
Each design has 168 rows, one control, K_aux=9 and residual DF=159. The 95%
chi-square(7) critical value is 14.0671404493402; corrections are
1.05660377358491 for HC1 and 1.09598031173093 for CR1 with 24 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [0.87058126469739, 1.50944638151829] |
| multiple | CR1 | [1.03741500489376, 1.47495371597571] |
| secondary | HC1 | [1.11163705575731, 1.51047538844265] |
| secondary | CR1 | [1.17198974024994, 1.53810157793966] |

The reference checks endpoints and six direct Wald probes for every design and
covariance. Rust tests compare those values, metadata, rescaling and retained
fit state; a polynomial unit test covers fourteen roots and eight accepted
components. The generator writes both data CSVs and the reference table to its
output directory. The eight-instrument extension is documented below.

The eight-instrument fixtures and Base R 4.6.2 reference are generated by
[`robust_ar_eight_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_eight_ci_reference.R).
Each design has 192 rows, one control, K_aux=10 and residual DF=182. The 95%
chi-square(8) critical value is 15.5073130558655; corrections are
1.05494505494505 for HC1 and 1.09507883420927 for CR1 with 24 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [1.0887754541176, 1.40883145836894] |
| multiple | CR1 | [1.22474047234525, 1.32397269272157] |
| secondary | HC1 | [1.11427595691973, 1.40051392176535] |
| secondary | CR1 | [1.01318139921973, 1.37571882762637] |

The reference checks every endpoint and six direct Wald probes for all four
design/covariance combinations. Rust tests also check correction and retained
fit metadata, rescaling, JSON/text output, and q18 rejection. The degree-sixteen
method supports sixteen real roots and nine accepted components; a polynomial
unit test covers all sixteen roots. Regenerate the inputs and reference from
the repository root with Base R 4.6.2:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_eight_ci_reference.R "$work" \
  > "$work/robust_ar_eight_ci_reference.csv"
cmp examples/robust-ar/robust_ar_eight_ci_multiple_data.csv \
  "$work/robust_ar_eight_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_eight_ci_secondary_data.csv \
  "$work/robust_ar_eight_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_eight_ci_reference.csv \
  "$work/robust_ar_eight_ci_reference.csv"
```

The nine-instrument fixtures and Base R 4.6.2 reference are generated by
[`robust_ar_nine_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_nine_ci_reference.R).
Each design has 216 rows, one included control, K_aux=11 and residual DF=205.
The 95% chi-square(9) critical value is 16.9189776046204; corrections are
1.05365853658537 for HC1 and 1.09437963944857 for CR1 with 24 groups.

| Design | Covariance | Confidence set |
| --- | --- | --- |
| multiple | HC1 | [0.911362825704433, 1.28291406681336] |
| multiple | CR1 | [0.945792747726182, 1.20035149024702] |
| secondary | HC1 | [1.072618576308, 1.2584091894859] |
| secondary | CR1 | Empty set |

The Base R reference uses `lm.fit`, explicit HC1/CR1 score sandwiches and
`polyroot`. It cross-checks the degree-eighteen determinant coefficients with
numeric `det()` evaluations at 19 Chebyshev nodes, then checks endpoints and six
direct Wald probes in every design/covariance combination. Rust compares
endpoints within 3e-8, checks metadata, covariance correction, instrument
rescaling, retained state and text/JSON output. The degree-eighteen solver
supports eighteen real roots and ten accepted components; a polynomial unit test
covers that full boundary. Regenerate the
inputs and reference from the repository root with Base R 4.6.2:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_nine_ci_reference.R "$work" \
  > "$work/robust_ar_nine_ci_reference.csv"
cmp examples/robust-ar/robust_ar_nine_ci_multiple_data.csv \
  "$work/robust_ar_nine_ci_multiple_data.csv"
cmp examples/robust-ar/robust_ar_nine_ci_secondary_data.csv \
  "$work/robust_ar_nine_ci_secondary_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_nine_ci_reference.csv \
  "$work/robust_ar_nine_ci_reference.csv"
```

Eighteen or more excluded instruments remain unsupported.

The all-real fixtures retain nonzero first-stage relevance. These are synthetic
method checks, not a Cameron–Trivedi illustration, coverage simulation or
evidence that instruments are valid. HC1 assumes independent rows. CR1 assumes
independent groups and many-group asymptotics, with no finite-sample or
few-cluster guarantee. This scalar result type covers exactly one endogenous
regressor and at most seventeen excluded instruments. Robust projected inference is
separate and bounded to exactly two endogenous regressors and two excluded
instruments; larger dimensions and HAC/survey covariance remain unsupported.

## Sixteen-instrument reference

[sixteen_instrument_ci.td](sixteen_instrument_ci.td) and
[sixteen_instrument_ci_clustered.td](sixteen_instrument_ci_clustered.td)
extend the scalar HC1/CR1 workflow through sixteen excluded instruments. The
384-row designs have one included control, K_aux=18 and residual DF=366. At
95%, chi-square(16)=26.2962276048642; HC1/CR1 corrections are
1.04918032786885 and 1.09194583036351 (24 groups of sixteen observations).

| Design | HC1 | CR1 |
| --- | --- | --- |
| multiple | [1.11266606306348, 1.25891703334138] | [1.19071994538493, 1.2294493675657] |
| secondary | [1.10152242485367, 1.29356316438751] | [1.16084602287508, 1.24209372746074] |

The independent Base R 4.6.2 reference checks degree-32 determinant
coefficients at 33 Chebyshev nodes, polyroot endpoints and six direct Wald
probes per design/covariance. Rust checks endpoints within 3e-8, metadata,
corrections, rescaling, retained state, JSON/text and q=17 rejection. The solver
boundary test covers all 32 simple real roots and seventeen accepted components.
Regenerate the fixtures and reference by running from the repository root:

    work=$(mktemp -d)
    Rscript crates/tabdat-stats/tests/fixtures/robust_ar_sixteen_ci_reference.R "$work" > "$work/robust_ar_sixteen_ci_reference.csv"
    cmp examples/robust-ar/robust_ar_sixteen_ci_multiple_data.csv "$work/robust_ar_sixteen_ci_multiple_data.csv"
    cmp examples/robust-ar/robust_ar_sixteen_ci_secondary_data.csv "$work/robust_ar_sixteen_ci_secondary_data.csv"
    cmp crates/tabdat-stats/tests/fixtures/robust_ar_sixteen_ci_reference.csv "$work/robust_ar_sixteen_ci_reference.csv"

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.

## Seventeen-instrument reference

[seventeen_instrument_ci.td](seventeen_instrument_ci.td) and
[seventeen_instrument_ci_clustered.td](seventeen_instrument_ci_clustered.td)
extend the scalar HC1/CR1 workflow through seventeen excluded instruments. The
408-row designs have one included control, K_aux=19 and residual DF=389. At
95%, chi-square(17)=27.5871116382753; HC1/CR1 corrections are
1.04884318766067 and 1.09176260198949 (24 groups of seventeen observations).

| Design | HC1 | CR1 |
| --- | --- | --- |
| multiple | [1.11927005953282, 1.24911390109925] | Empty set |
| secondary | [1.13863792577277, 1.28154941634978] | Empty set |

The independent Base R 4.6.2 reference checks degree-34 determinant
coefficients at 35 Chebyshev nodes, polyroot endpoints and six direct Wald
probes per design/covariance. Rust checks HC1 endpoints within 3e-8, verifies
the empty CR1 sets and checks metadata, corrections, rescaling, retained state,
JSON/text and q=18 rejection. The solver boundary test covers all 34 simple real
roots and eighteen accepted components.
Regenerate the fixtures and reference from the repository root:

    work=$(mktemp -d)
    Rscript crates/tabdat-stats/tests/fixtures/robust_ar_seventeen_ci_reference.R "$work" > "$work/robust_ar_seventeen_ci_reference.csv"
    cmp examples/robust-ar/robust_ar_seventeen_ci_multiple_data.csv "$work/robust_ar_seventeen_ci_multiple_data.csv"
    cmp examples/robust-ar/robust_ar_seventeen_ci_secondary_data.csv "$work/robust_ar_seventeen_ci_secondary_data.csv"
    cmp crates/tabdat-stats/tests/fixtures/robust_ar_seventeen_ci_reference.csv "$work/robust_ar_seventeen_ci_reference.csv"

Eighteen or more excluded instruments, robust projection outside the bounded
two-endogenous/two-instrument slice, larger endogenous systems, HAC/survey
covariance and few-cluster inference remain unsupported.
