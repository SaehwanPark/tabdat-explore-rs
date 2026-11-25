# Base-R central-difference oracle for pointwise Poisson/NB2 PMF delta SEs.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)

x <- c(1,-0.7,1.2)
mu <- 2.4
k <- 4L
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

reference <- function(law) {
  alpha <- if (law=="nb2") 0.75 else NA_real_
  covariance <- if (law=="nb2") nb2_covariance else poisson_covariance
  beta <- c(log(mu),0,0)
  log_alpha <- if (law=="nb2") log(alpha) else 0
  probability <- function(b,a) {
    fitted_mean <- exp(sum(b*x))
    if (law=="poisson") dpois(k,fitted_mean) else dnbinom(k,mu=fitted_mean,size=exp(-a))
  }
  step <- 1e-5
  gradient <- vapply(seq_along(beta),function(j) {
    upper <- lower <- beta
    upper[j] <- upper[j]+step
    lower[j] <- lower[j]-step
    (probability(upper,log_alpha)-probability(lower,log_alpha))/(2*step)
  },numeric(1))
  if (law=="nb2") {
    gradient <- c(gradient,
      (probability(beta,log_alpha+step)-probability(beta,log_alpha-step))/(2*step))
  }
  variance <- drop(t(gradient)%*%covariance%*%gradient)
  stopifnot(is.finite(variance),variance>=0)
  data.frame(law=law,mean=mu,alpha=alpha,count=k,
    probability=probability(beta,log_alpha),standard_error=sqrt(variance))
}

write.csv(rbind(reference("poisson"),reference("nb2")),args[[1]],row.names=FALSE,quote=FALSE)
