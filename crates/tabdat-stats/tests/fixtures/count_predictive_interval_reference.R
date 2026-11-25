# Base-R equal-tail future-count intervals and attained coverage.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)

levels <- c(50,80,90,95,99)
near_100 <- 100 - 2^-46
cases <- rbind(
  data.frame(law="poisson",mean=rep(c(0,1e-8,0.3,2.4,10000),each=length(levels)),
    alpha=NA_real_,level=rep(levels,5)),
  do.call(rbind,lapply(c(exp(-11.9),0.02,0.75,exp(11.9)),function(alpha) {
    data.frame(law="nb2",mean=rep(c(0.3,2.4,100),each=length(levels)),alpha=alpha,
      level=rep(levels,3))
  })),
  data.frame(law=c("poisson","nb2"),mean=2.4,alpha=c(NA_real_,0.75),
    level=near_100)
)

reference <- function(law,mean,alpha,level) {
  tail <- (100-level)/200
  if (law=="poisson") {
    lower <- qpois(tail,mean)
    upper <- qpois(tail,mean,lower.tail=FALSE)
    lower_previous_cdf <- if (lower==0) 0 else ppois(lower-1,mean)
    lower_cdf <- ppois(lower,mean)
    upper_previous_survival <- if (upper==0) 1 else
      ppois(upper-1,mean,lower.tail=FALSE)
    upper_survival <- ppois(upper,mean,lower.tail=FALSE)
  } else {
    size <- 1/alpha
    lower <- qnbinom(tail,size=size,mu=mean)
    upper <- qnbinom(tail,size=size,mu=mean,lower.tail=FALSE)
    lower_previous_cdf <- if (lower==0) 0 else
      pnbinom(lower-1,size=size,mu=mean)
    lower_cdf <- pnbinom(lower,size=size,mu=mean)
    upper_previous_survival <- if (upper==0) 1 else
      pnbinom(upper-1,size=size,mu=mean,lower.tail=FALSE)
    upper_survival <- pnbinom(upper,size=size,mu=mean,lower.tail=FALSE)
  }
  data.frame(law=law,mean=mean,alpha=alpha,level=level,
    lower=as.integer(lower),upper=as.integer(upper),
    lower_previous_cdf=lower_previous_cdf,lower_cdf=lower_cdf,
    upper_previous_survival=upper_previous_survival,
    upper_survival=upper_survival,
    coverage=1-upper_survival-lower_previous_cdf)
}
rows <- do.call(rbind,lapply(seq_len(nrow(cases)),function(i) {
  reference(cases$law[i],cases$mean[i],cases$alpha[i],cases$level[i])
}))
stopifnot(all(is.finite(rows$coverage)),all(rows$lower<=rows$upper),
  all(rows$upper<=100000))
rows$level <- sprintf("%.17g",rows$level)
write.csv(rows,args[[1]],row.names=FALSE,quote=FALSE)
