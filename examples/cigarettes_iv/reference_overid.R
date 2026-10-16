# Development-only reference: R 4.6.1 stats::lm.fit/pchisq, no production R.
# Exact model from AER ivreg example:
# https://search.r-project.org/CRAN/refmans/AER/html/ivreg.html
# Rscript examples/cigarettes_iv/reference_overid.R /path/to/CigarettesSW.csv
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
d <- read.csv(args[[1]])
d <- d[d$year == 1995, ]
d <- d[order(d$state), ]
expected_states <- strsplit(paste("AL AR AZ CA CO CT DE FL GA IA ID IL IN KS KY LA MA MD ME MI MN MO MS MT",
  "NC ND NE NH NJ NM NV NY OH OK OR PA RI SC SD TN TX UT VA VT WA WI WV WY"), " ")[[1]]
stopifnot(nrow(d) == 48L, !anyDuplicated(d$state), identical(d$state, expected_states))
y <- log(d$packs)
w <- log(d$income / d$population / d$cpi)
x <- log(d$price / d$cpi)
z1 <- (d$taxs - d$tax) / d$cpi
z2 <- d$tax / d$cpi
stopifnot(all(is.finite(c(y, w, x, z1, z2))))
X <- cbind(intercept = 1, lrincome = w, lrprice = x)
Z <- cbind(1, w, z1, z2)
Xhat <- lm.fit(Z, X)$fitted.values
beta <- lm.fit(Xhat, y)$coefficients
u <- y - drop(X %*% beta)
aux <- lm.fit(Z, u)
# Sargan convention is N*R² = N*(u'P_Z u)/(u'u), not an N-K correction.
statistic <- length(y) * sum(aux$fitted.values^2) / sum(u^2)
print(list(sample_states = d$state, beta = beta, observations = length(y),
  sargan = statistic, df = ncol(Z) - ncol(X),
  p_value = pchisq(statistic, ncol(Z) - ncol(X), lower.tail = FALSE)), digits = 16)
# Independent chi-square tails, including extreme and higher-DF cases.
for (pair in list(c(0,1), c(0.1,1), c(3.841458820694124,1), c(100,1),
  c(10,2), c(50,3), c(100,10), c(1000,100), c(1000,1000))) {
  cat(sprintf("chisq %.16g df %d tail %.16g\n", pair[1], pair[2],
    pchisq(pair[1], pair[2], lower.tail = FALSE)))
}
