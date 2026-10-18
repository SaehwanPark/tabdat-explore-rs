# Independent base-R full §14.2 summaries/preparation; development only.
# Rscript reference_analysis.R Nldata.csv R_predictions.csv R_source_sample.csv R_prepared_sample.csv > R_analysis_summary.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 4L)
raw <- read.csv(args[1])
original <- c("mode", "price", "crate", "dbeach", "dpier", "dprivate",
  "dcharter", "pbeach", "ppier", "pprivate", "pcharter", "qbeach", "qpier",
  "qprivate", "qcharter", "income")
stopifnot(identical(names(raw), c("source_row", original)),
  nrow(raw) == 1182L, identical(raw$source_row, seq_len(1182L)),
  all(is.finite(as.matrix(raw))))
raw$ydiv1000 <- raw$income / 1000
write.csv(raw, args[3], row.names = FALSE, quote = FALSE)
d <- raw[raw$mode %in% c(2L, 4L), ]
stopifnot(nrow(d) == 630L, sum(d$mode == 2L) == 178L,
  sum(d$mode == 4L) == 452L, all(d$pcharter > 0), all(d$ppier > 0))
d$charter <- as.integer(d$mode == 4L)
d$pratio <- 100 * log(d$pcharter / d$ppier)
d$lnrelp <- log(d$pcharter / d$ppier)
write.csv(d, args[4], row.names = FALSE, quote = FALSE)
predictions <- read.csv(args[2])
stopifnot(identical(predictions$source_row, d$source_row),
  identical(predictions$charter, d$charter),
  isTRUE(all.equal(predictions$lnrelp, d$lnrelp, tolerance = 1e-14)))
source_variables <- c(original, "ydiv1000")
prepared_variables <- c(source_variables, "charter", "pratio", "lnrelp")
cat("block,cohort,variable,count,mean,std_dev,minimum,maximum\n")
emit <- function(block, cohort, data, variables) {
  for (variable in variables) {
    x <- data[[variable]]
    stopifnot(length(x) == nrow(data), all(is.finite(x)))
    # R sd is the sample (n-1) SD; constants are deliberately retained.
    cat(sprintf("%s,%s,%s,%d,%.17g,%.17g,%.17g,%.17g\n",
      block, cohort, variable, length(x), mean(x), sd(x), min(x), max(x)))
  }
}
emit("source", "overall", raw, source_variables)
cohorts <- list(overall = d, pier = d[d$mode == 2L, ], charter = d[d$mode == 4L, ])
for (cohort in names(cohorts)) {
  emit("full", cohort, cohorts[[cohort]], prepared_variables)
}
for (cohort in names(cohorts)) {
  emit("table14_1", cohort, cohorts[[cohort]], c("charter", "pcharter", "ppier", "lnrelp"))
}
emit("fitted", "overall", predictions,
  c("charter", "logit_classical_pr", "probit_classical_pr", "ols_classical_xb"))
