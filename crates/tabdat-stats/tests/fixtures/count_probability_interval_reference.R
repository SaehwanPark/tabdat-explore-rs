# Base-R logit-Wald oracle for current-row Poisson/NB2 PMF and CDF intervals.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)

x <- c(1,-0.7,1.2)
poisson_covariance <- matrix(c(
  0.04,0.005,-0.002,
  0.005,0.01,0.001,
  -0.002,0.001,0.03),3,3)
nb2_covariance <- matrix(c(
  0.04,0.005,-0.002,0.001,
  0.005,0.01,0.001,-0.001,
  -0.002,0.001,0.03,0.002,
  0.001,-0.001,0.002,0.02),4,4)
stopifnot(min(eigen(poisson_covariance,symmetric=TRUE,only.values=TRUE)$values)>0,
  min(eigen(nb2_covariance,symmetric=TRUE,only.values=TRUE)$values)>0)

log_one_minus_exp <- function(value) {
  if (value > -log(2)) log(-expm1(value)) else log1p(-exp(value))
}
logit_probability <- function(law,mu,k,alpha,quantity) {
  if (quantity=="pmf") {
    log_probability <- if (law=="poisson") dpois(k,mu,log=TRUE) else
      dnbinom(k,size=exp(-alpha),mu=mu,log=TRUE)
    log_complement <- if (k==0) {
      if (law=="poisson") ppois(k,mu,lower.tail=FALSE,log.p=TRUE) else
        pnbinom(k,size=exp(-alpha),mu=mu,lower.tail=FALSE,log.p=TRUE)
    } else log_one_minus_exp(log_probability)
  } else {
    if (law=="poisson") {
      log_probability <- ppois(k,mu,log.p=TRUE)
      log_complement <- ppois(k,mu,lower.tail=FALSE,log.p=TRUE)
    } else {
      log_probability <- pnbinom(k,size=exp(-alpha),mu=mu,log.p=TRUE)
      log_complement <- pnbinom(k,size=exp(-alpha),mu=mu,lower.tail=FALSE,log.p=TRUE)
    }
  }
  log_probability-log_complement
}
reference <- function(law,mu,k,alpha=NA_real_,quantity) {
  covariance <- if (law=="nb2") nb2_covariance else poisson_covariance
  beta <- c(log(mu),0,0)
  log_alpha <- if (law=="nb2") log(alpha) else 0
  probability <- function(b,a) {
    fitted_mean <- exp(sum(b*x))
    if (quantity=="pmf") {
      if (law=="poisson") dpois(k,fitted_mean) else
        dnbinom(k,size=exp(-a),mu=fitted_mean)
    } else {
      if (law=="poisson") ppois(k,fitted_mean) else
        pnbinom(k,size=exp(-a),mu=fitted_mean)
    }
  }
  logit <- function(b,a) logit_probability(law,exp(sum(b*x)),k,a,quantity)
  beta_step <- 3e-7
  gradient <- vapply(seq_along(beta),function(j) {
    upper <- lower <- beta
    upper[j] <- upper[j]+beta_step
    lower[j] <- lower[j]-beta_step
    (logit(upper,log_alpha)-logit(lower,log_alpha))/(2*beta_step)
  },numeric(1))
  if (law=="nb2") {
    alpha_step <- 1e-5
    gradient <- c(gradient,
      (logit(beta,log_alpha+alpha_step)-logit(beta,log_alpha-alpha_step))/(2*alpha_step))
  }
  variance <- drop(t(gradient)%*%covariance%*%gradient)
  stopifnot(is.finite(variance),variance>=0)
  logit_estimate <- logit(beta,log_alpha)
  critical <- qnorm(0.975)
  data.frame(law=law,mean=mu,alpha=alpha,count=as.integer(k),quantity=quantity,
    estimate=probability(beta,log_alpha),
    lower=plogis(logit_estimate-critical*sqrt(variance)),
    upper=plogis(logit_estimate+critical*sqrt(variance)))
}

cases <- list(
  c("poisson",2.4,0,NA_real_),
  c("poisson",2.4,1,NA_real_),
  c("poisson",2.4,2,NA_real_),
  c("poisson",2.4,7,NA_real_),
  c("poisson",2.4,99,NA_real_),
  c("poisson",2.4,100000,NA_real_),
  c("poisson",100000,100000,NA_real_),
  c("poisson",1e-8,0,NA_real_),
  c("poisson",1e-300,0,NA_real_),
  c("nb2",2.4,0,0.75),
  c("nb2",2.4,1,0.75),
  c("nb2",2.4,2,0.75),
  c("nb2",2.4,7,0.75),
  c("nb2",2.4,99,0.75),
  c("nb2",2.4,0,0.02),
  c("nb2",2.4,7,0.02),
  c("nb2",2.4,7,exp(-11.9)),
  c("nb2",2.4,7,exp(11.9)),
  c("nb2",2.4,100000,exp(-11.9)),
  c("nb2",2.4,100000,exp(11.9)),
  c("nb2",1e-8,0,0.75),
  c("nb2",1e-300,0,0.75))
rows <- do.call(rbind,lapply(cases,function(case) {
  law <- as.character(case[[1]])
  mu <- as.numeric(case[[2]])
  k <- as.numeric(case[[3]])
  alpha <- as.numeric(case[[4]])
  rbind(reference(law,mu,k,alpha,"pmf"),reference(law,mu,k,alpha,"cdf"))
}))
write.csv(rows,args[[1]],row.names=FALSE,quote=FALSE)
