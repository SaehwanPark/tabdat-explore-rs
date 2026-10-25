# Development-only independent right-censoring reference.
# Rscript reference.R censored.csv reference.csv
args <- commandArgs(TRUE)
stopifnot(length(args)==2, !file.exists(args[2]))
library(survival)
d <- read.csv(args[1])
stopifnot(nrow(d)>0, all(is.finite(d$duration)), all(d$duration>0),
  all(is.finite(d$failed)), all(d$failed %in% c(0,1)), sum(d$failed)>0,
  all(is.finite(d$source_row)), all(d$source_row>0),
  all(d$source_row==floor(d$source_row)), !anyDuplicated(d$source_row))
f <- survfit(Surv(duration,failed)~1,d,stype=1,ctype=1,conf.type="log-log",conf.int=.95)
s <- summary(f,censored=TRUE)
se <- s$std.err
se[s$surv==0] <- NA_real_
z <- qnorm(.975)
hlo <- ifelse(s$cumhaz>0,s$cumhaz*exp(-z*s$std.chaz/s$cumhaz),0)
hup <- ifelse(s$cumhaz>0,s$cumhaz*exp(z*s$std.chaz/s$cumhaz),0)
out <- data.frame(time=s$time,at_risk=s$n.risk,events=s$n.event,censored=s$n.censor,
  survival=s$surv,survival_se=se,survival_lower=s$lower,survival_upper=s$upper,
  cumhaz=s$cumhaz,cumhaz_se=s$std.chaz,cumhaz_lower=hlo,cumhaz_upper=hup)
write.table(out,args[2],sep=",",row.names=FALSE,quote=FALSE,na="")
cat(R.version.string,"survival",as.character(packageVersion("survival")),
  "spells",nrow(d),"failures",sum(d$failed),"times",nrow(out),"\n")
