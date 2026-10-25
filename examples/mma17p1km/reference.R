# Development-only independent reference; no R dependency in TabDat.
# Rscript reference.R strikes.csv reference.csv
args <- commandArgs(TRUE)
stopifnot(length(args) == 2, !file.exists(args[2]))
library(survival)
d <- read.csv(args[1])
stopifnot(nrow(d) > 0, all(is.finite(d$dur)), all(d$dur > 0),
  !anyDuplicated(d$source_row), all(is.finite(d$source_row)),
  all(d$source_row > 0), all(d$source_row == floor(d$source_row)))
f <- survfit(Surv(dur, rep(1, nrow(d))) ~ 1, data=d,
  conf.type="log-log", conf.int=.95, stype=1, ctype=1)
# summary() gives SE(S), whereas f$std.err is SE(log S).
s <- summary(f)
se <- s$std.err
se[s$surv == 0] <- NA_real_
out <- data.frame(time=s$time, at_risk=s$n.risk, events=s$n.event,
  survival=s$surv, survival_se=se, survival_lower=s$lower,
  survival_upper=s$upper, cumhaz=s$cumhaz, cumhaz_se=s$std.chaz)
# Independent all-failure empirical-CDF identity protects sample/event convention.
stopifnot(max(abs(out$survival - vapply(out$time,
  function(t) mean(d$dur > t), numeric(1)))) < 1e-14)
write.table(out, args[2], sep=",", row.names=FALSE, quote=FALSE, na="")
cat(R.version.string, "survival", as.character(packageVersion("survival")),
  "spells", nrow(d), "distinct times", nrow(out), "\n")
