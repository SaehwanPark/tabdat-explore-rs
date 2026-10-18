# Independent development-only Table14.1 reference; base R, no packages.
# Rscript reference_summary.R Nldata.csv R_summary_sample.csv > R_summary.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 2L)
raw <- read.csv(args[1])
stopifnot(nrow(raw) == 1182L,
  identical(raw$source_row, seq_len(1182L)))
d <- raw[raw$mode %in% c(2L, 4L), ]
stopifnot(nrow(d) == 630L, sum(d$mode == 2L) == 178L,
  sum(d$mode == 4L) == 452L,
  all(is.finite(d$pcharter) & d$pcharter > 0),
  all(is.finite(d$ppier) & d$ppier > 0))
sample <- data.frame(source_row = d$source_row, mode = d$mode,
  charter = as.integer(d$mode == 4L), pcharter = d$pcharter,
  ppier = d$ppier, lnrelp = log(d$pcharter/d$ppier))
stopifnot(all(is.finite(as.matrix(sample))))
write.csv(sample, args[2], row.names = FALSE, quote = FALSE)
cat("cohort,variable,count,mean,std_dev,minimum,maximum\n")
cohorts <- list(overall = sample, pier = sample[sample$mode == 2L, ],
  charter = sample[sample$mode == 4L, ])
for (cohort in names(cohorts)) {
  for (variable in c("charter", "pcharter", "ppier", "lnrelp")) {
    x <- cohorts[[cohort]][[variable]]
    cat(sprintf("%s,%s,%d,%.17g,%.17g,%.17g,%.17g\n",
      cohort, variable, length(x), mean(x), sd(x), min(x), max(x)))
  }
}
