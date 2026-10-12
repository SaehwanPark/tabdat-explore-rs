# Independent MASS NB2 fit with full observed beta/alpha covariance.
d <- read.csv(commandArgs(trailingOnly=TRUE)[1])
f <- MASS::glm.nb(y ~ x1 + x2, data=d, control=glm.control(epsilon=1e-12, maxit=100))
stopifnot(f$converged, nobs(f)==320, identical(rownames(model.frame(f)),as.character(1:320)))
x <- model.matrix(f); beta <- coef(f); alpha <- 1/f$theta
mu <- as.vector(exp(x %*% beta)); r <- 1/alpha; y <- d$y
bb <- crossprod(x, (mu*(1+alpha*y)/(1+alpha*mu)^2)*x)
ba <- as.vector(crossprod(x, (y-mu)*mu/(1+alpha*mu)^2))
sr <- digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu)
drr <- trigamma(y+r)-trigamma(r)+1/r-1/(r+mu)+(y-mu)/(r+mu)^2
aa <- -sum(r^4*drr+2*r^3*sr)
information <- rbind(cbind(bb, ba), c(ba, aa))
parameter <- c(beta, alpha)
objective <- function(p) -sum(dnbinom(y, size=1/p[4], mu=exp(x %*% p[1:3]), log=TRUE))
stopifnot(max(abs(information-optimHess(parameter,objective))) < 1e-3)
covariance <- solve(information)
values <- c(parameter,sqrt(diag(covariance)),as.vector(t(covariance)),as.numeric(logLik(f)))
cat("R,nb2,",paste(sprintf("%.17g",values),collapse=","),"\n",sep="")
cat(R.version.string," MASS ",as.character(packageVersion("MASS")),"\n",sep="",file=stderr())
