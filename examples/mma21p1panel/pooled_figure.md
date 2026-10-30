# WIP: Chapter 21 Figure 21.1 pooled source-data adaptation

## Contract before implementation

Deliver the author-program Figure 21.1 (§21.3 p.712) as an end-to-end native
load/guard → pooled OLS → ordered-row nonparametric fit → report/CSV/SVG
workflow. Use all original 5,320 MOM person-years (532 men × 1979–88), log
hourly wage X and log annual hours Y, no extra regressors, native clustered
pooled OLS with person inference, and explicitly specified nonrobust centred-
rank span-0.8 LOWESS. Native sorting fixes source `(id,year)` within-X tie
order; historical Stata sort/storage and WMF/pixel parity are not established.
This is a reproducible source-data adaptation, not complete chapter coverage.

The original sample requires k=2,127 and work bound
`5320*min(5320,2*2127+1)=22,636,600`, exceeding the installed 20,000,000
conservative bound. The smallest bounded public change that enables this and
related required Chapter 21 source figures is a **23,000,000** cap; the
unchanged N<=10,000/finite/span/atomic-state rules still apply. Verify actual
runtime/resource use, accept this original case and reject over-limit cases;
update help, admission tests, reference/checker work bounds and current status.
Do not change the smoothing method or interpret the cap as general unlimited
work. If measured cost is impractical, revisit the bound or stop rather than
relax safety/verification.

Independent R per-window weighted QR and Python 2×2 normal equations must
check exact source/sample, rank/tie order and every fitted value, clustered OLS
coefficient/covariance/inference, all actual 5,320 SVG markers and 10,640
connected curve vertices. Fixed tolerance `1e-11+1e-10*abs(reference)` and
relative `1e-9` for nonzero p-values; certificates `<1e-12`. Use no production
R/Python. Fixture/control cases: tied X, shuffled raw input, text/JSON equality,
missing model/nonmodel fields and excluded NULLs where appropriate; reject
invalid/duplicate keys, collisions, singular design, too much work and existing
outputs without clobber, and test retained state/forgery. Document native tie
ordering, assumptions and nontransactional failure semantics. Change broad
LOWESS limit only with its focused stats/runtime/reporting tests and relevant
reverse consumers plus original-source independent verification.
