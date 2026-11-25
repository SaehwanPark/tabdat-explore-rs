# Deterministic Base R reference for the clustered NB2 grouped GOF test.
# Rscript cluster_count_gof_reference.R input.csv reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)

set.seed(20261124)
G <- 96L
T <- 5L
N <- G*T
cluster <- rep(seq_len(G), each=T)
x <- runif(N, -1, 1)
alpha0 <- 0.6
mu0 <- exp(0.25 + 0.35*x)
effect <- rgamma(G, shape=1/alpha0, rate=1/alpha0)
y <- rpois(N, mu0*effect[cluster])
input <- data.frame(cluster=cluster, y=y, x=x)
write.csv(input, args[[1]], row.names=FALSE, quote=FALSE)

X <- cbind(1, x)
K <- 4L
Jm <- K
parameter_count <- ncol(X)+1L
loglik <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  alpha <- exp(theta[[parameter_count]])
  r <- 1/alpha
  mu <- drop(exp(X%*%beta))
  sum(dnbinom(y, size=r, mu=mu, log=TRUE))
}
score <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  alpha <- exp(theta[[parameter_count]])
  r <- 1/alpha
  mu <- drop(exp(X%*%beta))
  eta_score <- (y-mu)*r/(r+mu)
  alpha_score <- -r*(digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu))
  c(drop(crossprod(X, eta_score)), sum(alpha_score))
}
information_at <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  r <- exp(-theta[[parameter_count]])
  mu <- drop(exp(X%*%beta))
  information <- matrix(0, parameter_count, parameter_count)
  for (i in seq_len(N)) {
    q <- mu[[i]]/(r+mu[[i]])
    eta_score <- (y[[i]]-mu[[i]])*r/(r+mu[[i]])
    cross <- q*eta_score
    weight <- mu[[i]]*r/(r+mu[[i]])*(r+y[[i]])/(r+mu[[i]])
    information[seq_len(ncol(X)),seq_len(ncol(X))] <-
      information[seq_len(ncol(X)),seq_len(ncol(X))] +
      weight*tcrossprod(X[i,])
    information[seq_len(ncol(X)),parameter_count] <-
      information[seq_len(ncol(X)),parameter_count] + X[i,]*cross
    information[parameter_count,seq_len(ncol(X))] <-
      information[seq_len(ncol(X)),parameter_count]
    cell <- if (y[[i]]>0) 0:(as.integer(y[[i]])-1L) else integer(0)
    curvature <- if (length(cell))
      sum(cell/(r+cell)*r/(r+cell)) else 0
    remainder <- log1p(mu[[i]]/r)-q
    information[parameter_count,parameter_count] <-
      information[parameter_count,parameter_count] +
      r*remainder+cross-curvature
  }
  information
}
objective <- function(theta) -loglik(theta)
gradient <- function(theta) -score(theta)
initial <- c(log(mean(y)), 0, log(0.5))
fit <- optim(initial, objective, gradient, method="BFGS",
             control=list(reltol=1e-14, maxit=2000))
stopifnot(fit$convergence==0L)
theta <- fit$par
for (iteration in seq_len(8L)) {
  information <- information_at(theta)
  theta <- theta+drop(solve(information, score(theta)))
  if (max(abs(score(theta)))<1e-10) break
}
stopifnot(max(abs(score(theta)))<1e-9)
information <- information_at(theta)
information_fd <- optimHess(theta, objective, gradient)
stopifnot(max(abs(information-information_fd))<1e-4)
stopifnot(all(is.finite(information)), min(eigen(information, symmetric=TRUE,
                                                  only.values=TRUE)$values)>0)
beta <- theta[seq_len(ncol(X))]
alpha <- exp(theta[[parameter_count]])
r <- 1/alpha
mu <- drop(exp(X%*%beta))

cell_probabilities <- function(candidate) {
  candidate_beta <- candidate[seq_len(ncol(X))]
  candidate_r <- exp(-candidate[[parameter_count]])
  candidate_mu <- drop(exp(X%*%candidate_beta))
  vapply(0:(K-1L), function(cell)
    dnbinom(cell, size=candidate_r, mu=candidate_mu), numeric(N))
}
probability <- cell_probabilities(theta)
observed <- vapply(0:(K-1L), function(cell) as.numeric(y==cell), numeric(N))
moments <- observed-probability

# Analytic derivative of the first K cell moments, checked independently by
# finite differences of dnbinom probabilities at the fitted parameters.
C <- matrix(0, Jm, parameter_count)
for (cell in 0:(K-1L)) {
  cell_count <- cell
  eta_score <- (cell_count-mu)*r/(r+mu)
  alpha_score <- -r*(digamma(cell_count+r)-digamma(r)+
                      log(r/(r+mu))+1-(r+cell_count)/(r+mu))
  for (parameter in seq_len(parameter_count)) {
    row_score <- if (parameter<=ncol(X)) X[,parameter]*eta_score else alpha_score
    C[cell+1L,parameter] <- -sum(probability[,cell+1L]*row_score)
  }
}
C_fd <- matrix(0, Jm, parameter_count)
for (parameter in seq_len(parameter_count)) {
  step <- 1e-5*(1+abs(theta[[parameter]]))
  plus <- minus <- theta
  plus[[parameter]] <- plus[[parameter]]+step
  minus[[parameter]] <- minus[[parameter]]-step
  C_fd[,parameter] <- -colSums((cell_probabilities(plus)-
                                  cell_probabilities(minus))/(2*step))
}
stopifnot(max(abs(C-C_fd)) < 2e-7*(1+max(abs(C))))

eta_score <- (y-mu)*r/(r+mu)
alpha_score <- -r*(digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu))
scores <- cbind(X*eta_score, alpha_score)
projection <- C%*%solve(information)
adjusted <- moments+t(projection%*%t(scores))
cluster_moments <- rowsum(adjusted, cluster, reorder=TRUE)
correction <- G/(G-1)
V <- correction/N*crossprod(cluster_moments)
statistic <- drop(N*crossprod(colMeans(moments), solve(V, colMeans(moments))))
df <- qr(V)$rank
stopifnot(df==Jm)
p_value <- pchisq(statistic, df, lower.tail=FALSE)
observed_count <- c(colSums(observed), sum(y>=K))
fitted_bins <- c(colMeans(probability), 1-sum(colMeans(probability)))

rows <- list()
add <- function(quantity, i=0L, j=0L, value) {
  rows[[length(rows)+1L]] <<- data.frame(case="clustered", quantity,
    i=as.integer(i), j=as.integer(j), value=as.numeric(value))
}
for (i in seq_len(ncol(X))) add("coefficient", i-1L, value=beta[[i]])
add("alpha", value=alpha)
add("log_likelihood", value=loglik(theta))
add("observation_count", value=N)
add("tail_from", value=K)
add("group_count", value=G)
add("covariance_correction", value=correction)
add("statistic", value=statistic)
add("degrees_of_freedom", value=df)
add("p_value", value=p_value)
for (i in seq_len(Jm)) for (j in seq_len(Jm))
  add("moment_covariance", i-1L, j-1L, V[i,j])
for (i in seq_len(Jm)) for (j in seq_len(parameter_count))
  add("moment_derivative", i-1L, j-1L, C[i,j])
for (i in seq_len(K+1L)) {
  add("observed_count", i-1L, value=observed_count[[i]])
  add("fitted_fraction", i-1L, value=fitted_bins[[i]])
}
write.csv(do.call(rbind, rows), args[[2]], row.names=FALSE, quote=FALSE)
