# Base-R central-difference oracle for current-row Poisson/NB2 CDF delta SEs.
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

reference <- function(law,mu,k,alpha=NA_real_) {
  covariance <- if (law=="nb2") nb2_covariance else poisson_covariance
  beta <- c(log(mu),0,0)
  log_alpha <- if (law=="nb2") log(alpha) else 0
  probability <- function(b,a) {
    fitted_mean <- exp(sum(b*x))
    if (law=="poisson") ppois(k,fitted_mean) else pnbinom(k,size=exp(-a),mu=fitted_mean)
  }
  survival <- function(b,a) {
    fitted_mean <- exp(sum(b*x))
    if (law=="poisson") ppois(k,fitted_mean,lower.tail=FALSE) else
      pnbinom(k,size=exp(-a),mu=fitted_mean,lower.tail=FALSE)
  }
  use_survival <- probability(beta,log_alpha)>0.5
  difference <- function(upper,lower,step) {
    if (use_survival) -(survival(upper[[1]],upper[[2]])-survival(lower[[1]],lower[[2]]))/(2*step) else
      (probability(upper[[1]],upper[[2]])-probability(lower[[1]],lower[[2]]))/(2*step)
  }
  beta_step <- 3e-7
  gradient <- vapply(seq_along(beta),function(j) {
    upper <- lower <- beta
    upper[j] <- upper[j]+beta_step
    lower[j] <- lower[j]-beta_step
    difference(list(upper,log_alpha),list(lower,log_alpha),beta_step)
  },numeric(1))
  if (law=="nb2") {
    alpha_step <- 1e-5
    gradient <- c(gradient,
      difference(list(beta,log_alpha+alpha_step),list(beta,log_alpha-alpha_step),alpha_step))
  }
  variance <- drop(t(gradient)%*%covariance%*%gradient)
  stopifnot(is.finite(variance),variance>=0)
  data.frame(law=law,mean=mu,alpha=alpha,count=as.integer(k),
    probability=probability(beta,log_alpha),standard_error=sqrt(variance))
}

rows <- rbind(
  reference("poisson",2.4,0),
  reference("poisson",2.4,1),
  reference("poisson",2.4,2),
  reference("poisson",2.4,7),
  reference("poisson",2.4,99),
  reference("poisson",100000,100000),
  reference("nb2",2.4,0,0.75),
  reference("nb2",2.4,1,0.75),
  reference("nb2",2.4,2,0.75),
  reference("nb2",2.4,7,0.75),
  reference("nb2",2.4,99,0.75),
  reference("nb2",2.4,0,0.02),
  reference("nb2",2.4,7,0.02),
  reference("nb2",2.4,7,exp(-11.9)),
  reference("nb2",2.4,7,exp(11.9)))
write.csv(rows,args[[1]],row.names=FALSE,quote=FALSE)
