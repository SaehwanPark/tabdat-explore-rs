# Development-only independent reference; production TabDat does not require R.
# Usage: Rscript examples/mma04p4ivweak/reference_shea.R /path/to/DATA66.csv
# R 4.6.1, base stats::lm.fit. Input is produced by data66_to_csv.awk.
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
data <- read.csv(args[[1]])
exogenous <- c("black", "south76", "smsa76", paste0("reg", 2:9), "smsa66",
  "momdad14", "sinmom14", "nodaded", "nomomed", "daded", "momed", paste0("famed", 1:8))
endogenous <- c("grade76", "exp76", "expsq76")
instruments <- c("col4", "age76", "agesq76")
variables <- c("wage76", exogenous, endogenous, instruments)
selected <- which(!is.na(data$nlsflt) & data$nlsflt == 1)
complete <- apply(data[selected, variables], 1, function(row) all(is.finite(row)))
sample_rows <- selected[complete]
sample <- data[sample_rows, ]
stopifnot(nrow(sample) == 3010L)
w <- cbind(intercept = 1, as.matrix(sample[exogenous]))
x <- cbind(w, as.matrix(sample[endogenous]))
z <- cbind(w, as.matrix(sample[instruments]))
xhat <- lm.fit(z, x)$fitted.values
# Residualize each original/projected column on all the other columns. The
# ratio measures unique projected variation / unique original variation.
shea <- vapply(match(endogenous, colnames(x)), function(j) {
  sum(lm.fit(xhat[, -j], xhat[, j])$residuals^2) /
    sum(lm.fit(x[, -j], x[, j])$residuals^2)
}, numeric(1))
names(shea) <- endogenous
stopifnot(abs(shea[["grade76"]] - 0.00640757) < 5e-9)
print(list(observations = nrow(sample), shea_partial_r_squared = shea), digits = 14)
# Original CSV row indices make the reference sample inspectable without
# redistributing source observations.
writeLines(paste(sample_rows, collapse = ","))
