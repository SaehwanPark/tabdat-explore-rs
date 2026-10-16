# Development-only R 4.6.1 reference; no packages or production R dependency.
# Same AER ivreg specification/sample as reference_overid.R. Classical Wu-Hausman
# uses nested homoskedastic OLS RSS, not the HC0 diagnostic from sandwich output.
# Rscript examples/cigarettes_iv/reference_endogenous.R /path/to/CigarettesSW.csv
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
d <- read.csv(args[[1]])
d <- d[d$year == 1995, ]; d <- d[order(d$state), ]
expected_states <- strsplit(paste("AL AR AZ CA CO CT DE FL GA IA ID IL IN KS KY LA MA MD ME MI MN MO MS MT",
  "NC ND NE NH NJ NM NV NY OH OK OR PA RI SC SD TN TX UT VA VT WA WI WV WY"), " ")[[1]]
stopifnot(nrow(d) == 48L, identical(d$state, expected_states))
y <- log(d$packs); w <- log(d$income/d$population/d$cpi); x <- log(d$price/d$cpi)
Z <- cbind(1,w,(d$taxs-d$tax)/d$cpi,d$tax/d$cpi)
stopifnot(all(is.finite(c(y,w,x,Z))))
v <- lm.fit(Z,x)$residuals
X <- cbind(1,w,x); A <- cbind(X,v)
restricted <- lm.fit(X,y); unrestricted <- lm.fit(A,y)
rss_r <- sum(restricted$residuals^2); rss_u <- sum(unrestricted$residuals^2)
df <- length(y)-ncol(A)
F <- (rss_r-rss_u)/(rss_u/df)
print(list(states=d$state, N=length(y), restricted_rss=rss_r, augmented_rss=rss_u,
  numerator_df=1L, denominator_df=df, wu_hausman_F=F,
  p_value=pf(F,1,df,lower.tail=FALSE)),digits=16)
# Contrast only, NOT the implemented classical statistic: HC0 auxiliary Wald.
bread <- solve(crossprod(A))
vcov_hc0 <- bread %*% crossprod(A,A*drop(unrestricted$residuals^2)) %*% bread
hc0_F <- unrestricted$coefficients[4]^2/vcov_hc0[4,4]
cat(sprintf("HC0 contrast only: F(1,%d)=%.16g p=%.16g\n",df,hc0_F,
  pf(hc0_F,1,df,lower.tail=FALSE)))
