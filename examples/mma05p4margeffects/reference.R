# Independent profile exponential-density likelihood, not a Poisson fit.
args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 2) stop('usage: Rscript reference.R original.asc reference.csv')
d <- read.table(args[1], col.names=c('y', 'x'))
stopifnot(nrow(d) == 10000, all(is.finite(d$x)), all(is.finite(d$y)), all(d$y > 0))
n <- nrow(d)
score <- function(b) {
  w <- d$y * exp(b*d$x)
  mean(d$x) - sum(w*d$x)/sum(w)
}
b <- uniroot(score, c(-3, 3), tol=1e-14)$root
a <- log(n/sum(d$y*exp(b*d$x)))
rate <- exp(a+b*d$x)
mu <- 1/rate
step <- 0.0001*sd(d$x)
X <- cbind(1, d$x)
covariance <- solve(crossprod(X, X*(d$y*rate)))
stopifnot(abs(score(b)) < 1e-12)
effects <- data.frame(source_row=seq_len(n), y=d$y, x=d$x,
  mean_x=mean(d$x), step_x=step, fitted_mean=mu,
  analytical_derivative=-b*mu,
  numerical_derivative=(exp(-a-b*(d$x+step))-mu)/step,
  finite_difference=exp(-a-b*(d$x+1))-mu,
  derivative_at_mean=-b*exp(-a-b*mean(d$x)))
write.csv(effects, args[2], row.names=FALSE, quote=FALSE)
cat(sprintf('a=%.17g b=%.17g cov=%.17g,%.17g,%.17g\n', a,b,
  covariance[1,1],covariance[1,2],covariance[2,2]))
