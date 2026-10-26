# Independent profile exponential likelihood and central-difference effect gradients.
args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 2) stop('usage: Rscript reference_uncertainty.R original.asc reference.csv')
if (file.exists(args[2])) stop('reference output already exists')
d <- read.table(args[1], col.names=c('y', 'x'))
stopifnot(nrow(d)==10000, all(is.finite(d$x)), all(is.finite(d$y)), all(d$y>0))
n <- nrow(d)
score <- function(b) {
  w <- d$y*exp(b*d$x)
  mean(d$x)-sum(w*d$x)/sum(w)
}
b <- uniroot(score, c(-3,3), tol=1e-14)$root
a <- log(n/sum(d$y*exp(b*d$x)))
theta <- c(a,b)
X <- cbind(1,d$x)
V <- solve(crossprod(X, X*(d$y*exp(a+b*d$x))))
effects <- function(t) {
  mu <- exp(-t[1]-t[2]*d$x)
  c(-t[2]*mean(mu), mean(exp(-t[1]-t[2]*(d$x+1))-mu),
    -t[2]*exp(-t[1]-t[2]*mean(d$x)))
}
h <- 1e-5
G <- sapply(seq_along(theta), function(i) {
  delta <- rep(0,2); delta[i] <- h
  (effects(theta+delta)-effects(theta-delta))/(2*h)
})
# Step refinement verifies finite-difference precision without relying on native gradients.
G2 <- sapply(seq_along(theta), function(i) {
  delta <- rep(0,2); delta[i] <- h/2
  (effects(theta+delta)-effects(theta-delta))/h
})
stopifnot(max(abs(G-G2))<1e-9, abs(score(b))<1e-12)
se <- sqrt(diag(G2 %*% V %*% t(G2)))
estimate <- effects(theta)
out <- data.frame(n=n, effect=c('average analytical derivative',
  'average one-unit difference', 'derivative at mean x'), estimate=estimate,
  standard_error=se, ci_lower=estimate-qnorm(.975)*se,
  ci_upper=estimate+qnorm(.975)*se,
  inference='normal95, conditional x, exponential model-based')
write.csv(out,args[2],row.names=FALSE,quote=TRUE)
cat(sprintf('profile a=%.17g b=%.17g; gradient step discrepancy %.3g\n',a,b,max(abs(G-G2))))
