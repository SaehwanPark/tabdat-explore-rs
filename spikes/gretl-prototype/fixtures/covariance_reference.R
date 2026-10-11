# External trusted-reference probe. R/sandwich is not a Rust runtime dependency.
args <- commandArgs(trailingOnly=TRUE)
data <- read.csv(args[1])
fit <- lm(y ~ x1 + x2, data=data)
stopifnot(nobs(fit) == 40, identical(rownames(model.frame(fit)), as.character(1:40)))
for (mode in c("classical", "hc1", "cluster")) {
  covariance <- switch(mode,
    classical=vcov(fit),
    hc1=sandwich::vcovHC(fit, type="HC1"),
    cluster=sandwich::vcovCL(fit, cluster=data$cid, type="HC1", cadjust=TRUE))
  values <- c(coef(fit), sqrt(diag(covariance)), as.vector(t(covariance)))
  cat("R,", mode, ",", paste(sprintf("%.17g", values), collapse=","), "\n", sep="")
}
cat(R.version.string, "sandwich", as.character(packageVersion("sandwich")), "\n", file=stderr())
