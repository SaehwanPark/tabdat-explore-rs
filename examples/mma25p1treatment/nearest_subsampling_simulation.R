# Pilot only: evaluate fixed-neighbor subsampling under a known-effect DGP.
args <- commandArgs(trailingOnly=TRUE)
outer <- if(length(args)>=1) as.integer(args[1]) else 100L
inner <- if(length(args)>=2) as.integer(args[2]) else 100L
stopifnot(outer>=10,inner>=20)
set.seed(356011)
nearest <- function(data) {
  warn <- character()
  fit <- withCallingHandlers(glm(treat~x,family=binomial(),data=data,control=glm.control(epsilon=1e-12,maxit=100)),
    warning=function(w) { warn <<- c(warn,conditionMessage(w)); invokeRestart('muffleWarning') })
  if(!fit$converged || fit$rank!=2) stop('nonconverged or rank-deficient fit')
  if(length(warn)>0) stop(paste('fit warning:',paste(warn,collapse='; ')))
  p <- unname(predict(fit,type='response')); t <- data$treat;y <- data$y
  if(any(!is.finite(p)) || any(p<=0 | p>=1) || sum(t)<2 || sum(1-t)<2) stop('invalid score or groups')
  lo <- max(min(p[t==0]),min(p[t==1]));hi <- min(max(p[t==0]),max(p[t==1]))
  it <- which(t==1 & p>=lo & p<=hi);ic <- which(t==0 & p>=lo & p<=hi)
  if(length(it)<2 || length(ic)<2 || lo>=hi) stop('empty support')
  D <- abs(outer(p[it],p[ic],'-'));m <- apply(D,1,min);ties <- D==m
  match <- as.vector(ties%*%y[ic])/rowSums(ties)
  mean(y[it]-match)
}
# An independent scalar loop checks support, tie averaging and estimator value.
brute <- function(data) {
 fit <- glm(treat~x,family=binomial(),data=data,control=glm.control(epsilon=1e-12,maxit=100))
 p <- unname(predict(fit,type='response'));t <- data$treat;y <- data$y
 lo <- max(min(p[t==0]),min(p[t==1]));hi <- min(max(p[t==0]),max(p[t==1]))
 it <- which(t==1 & p>=lo & p<=hi);ic <- which(t==0 & p>=lo & p<=hi)
 mean(vapply(it,function(i){d <- abs(p[i]-p[ic]);y[i]-mean(y[ic[d==min(d)]])},0.0))
}
test <- data.frame(x=c(-1,-.4,0,0,0,.2,.7,1,-.7,.9),
 treat=c(0,1,0,0,1,0,1,0,1,1),y=c(2,4,3,8,9,2,5,4,6,7))
stopifnot(abs(nearest(test)-brute(test))<1e-12)
# Comparing two subsample rates is a sensitivity check, not a coverage guarantee.
for(atom in c(0,0.4)) for(n in c(300L,600L)) for(exponent in c(0.65,0.8)) {
 m <- floor(n^exponent); widths <- covered <- corrected <- estimates <- numeric();outer_fail <- inner_fail <- 0L
 reasons <- character()
 for(rep in seq_len(outer)) {
  x <- runif(n,-1,1); if(atom>0) x[runif(n)<atom] <- 0
  treat <- rbinom(n,1,plogis(-.4+.9*x)); y <- 1.5*treat+2*x+.5*x*x+rnorm(n)
  dat <- data.frame(x=x,treat=treat,y=y)
  full <- tryCatch(list(value=nearest(dat),error=NULL),error=function(e) list(value=NA_real_,error=conditionMessage(e)))
  theta <- full$value
  if(!is.finite(theta)){outer_fail <- outer_fail+1L;reasons <- c(reasons,paste0('outer ',rep,': ',full$error));next}
  estimates <- c(estimates,theta)
  draws <- numeric(inner);bad <- FALSE
  for(j in seq_len(inner)) {
   sample_data <- dat[sort(sample.int(n,m,replace=FALSE)),]
   sample_fit <- tryCatch(list(value=nearest(sample_data),error=NULL),error=function(e) list(value=NA_real_,error=conditionMessage(e)))
   if(!is.finite(sample_fit$value)){inner_fail <- inner_fail+1L;reasons <- c(reasons,paste0('outer ',rep,', draw ',j,': ',sample_fit$error));bad <- TRUE;break}
   draws[j] <- sqrt(m)*(sample_fit$value-theta)
  }
  if(bad) next
  quant <- quantile(draws,c(.025,.975),type=1,names=FALSE)
  lower <- theta-quant[2]/sqrt(n);upper <- theta-quant[1]/sqrt(n)
  # This finite-population factor is exact for a sample mean; here it is only
  # an exploratory sensitivity, not a justified correction for matching.
  factor <- sqrt(1-m/n)
  lower_corrected <- theta-quant[2]/(sqrt(n)*factor);upper_corrected <- theta-quant[1]/(sqrt(n)*factor)
  widths <- c(widths,upper-lower);covered <- c(covered,lower<=1.5 && 1.5<=upper)
  corrected <- c(corrected,lower_corrected<=1.5 && 1.5<=upper_corrected)
 }
 # Do not present coverage after discarding failed full-pipeline replicates.
 complete <- length(covered)==outer
 coverage <- if(complete) mean(covered) else NA_real_
 mcse <- if(complete) sqrt(coverage*(1-coverage)/outer) else NA_real_
 correction <- if(complete) mean(corrected) else NA_real_
 width <- if(complete) mean(widths) else NA_real_
 bias <- if(complete) mean(estimates)-1.5 else NA_real_
 empirical_sd <- if(complete) sd(estimates) else NA_real_
 cat(sprintf('atom=%.1f,n=%d,m=%d,exponent=%.2f,outer=%d,inner=%d,completed=%d,outer_fail=%d,inner_fail=%d,coverage=%.4f,mcse=%.4f,exploratory_corrected_coverage=%.4f,mean_width=%.5f,bias=%.5f,empirical_sd=%.5f\n',
 atom,n,m,exponent,outer,inner,length(covered),outer_fail,inner_fail,coverage,mcse,correction,width,bias,empirical_sd))
 if(length(reasons)) cat(paste0('failures: ',paste(reasons,collapse=' | '),'\n'))
 flush.console()
}
