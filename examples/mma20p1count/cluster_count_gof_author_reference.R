# Development-only reference for the original-person clustered NB2 GOF.
# Rscript cluster_count_gof_author_reference.R person.csv reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
input <- read.csv(args[[1]], check.names=FALSE)
predictors <- c("LC","IDP","LPI","FMDE","PHYSLIM","NDISEASE","HLTHG",
  "HLTHF","HLTHP","LINC","LFAM","EDUCDEC","AGE","FEMALE","CHILD",
  "FEMCHILD","BLACK")
stopifnot(nrow(input)==20186L, all(c("MDU","person_id","study_year",predictors)%in%names(input)),
  length(unique(input$person_id))==5908L, !anyDuplicated(input[c("person_id","study_year")]),
  identical(input$prepared_row, seq_len(20186L)),
  identical(input$original_row, setdiff(seq_len(20190L), c(9746L,10153L,10400L,10859L))),
  identical(as.integer(table(input$study_year)), c(5638L,5574L,5545L,1715L,1714L)),
  identical(as.integer(table(as.integer(table(input$person_id)))), c(265L,246L,3743L,70L,1584L)))
y <- input$MDU
X <- cbind(1, as.matrix(input[,predictors]))
storage.mode(X) <- "double"
cluster <- input$person_id
N <- length(y)
G <- length(unique(cluster))
K <- 10L
Jm <- K
parameter_count <- ncol(X)+1L
stopifnot(sum(y)==57746, sum(y==0)==6308, max(y)==77,
  all(is.finite(X)), !anyNA(y), all(y>=0 & y==floor(y)))

loglik <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  r <- exp(-theta[[parameter_count]])
  eta <- drop(X%*%beta)
  if (abs(theta[[parameter_count]])>=12 || any(!is.finite(eta)) ||
      any(eta>700) || any(eta< -700)) return(-1e100)
  mu <- exp(eta)
  sum(dnbinom(y, size=r, mu=mu, log=TRUE))
}
score <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  r <- exp(-theta[[parameter_count]])
  eta <- drop(X%*%beta)
  if (abs(theta[[parameter_count]])>=12 || any(!is.finite(eta)) ||
      any(eta>700) || any(eta< -700)) return(rep(0, parameter_count))
  mu <- exp(eta)
  eta_score <- (y-mu)*r/(r+mu)
  alpha_score <- -r*(digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu))
  c(drop(crossprod(X, eta_score)), sum(alpha_score))
}
information_at <- function(theta) {
  beta <- theta[seq_len(ncol(X))]
  r <- exp(-theta[[parameter_count]])
  mu <- drop(exp(X%*%beta))
  q <- mu/(r+mu)
  eta_score <- (y-mu)*r/(r+mu)
  cross <- q*eta_score
  weight <- mu*r/(r+mu)*(r+y)/(r+mu)
  remainder <- log1p(mu/r)-q
  curvature <- vapply(y, function(value) {
    if (value==0) return(0)
    cell <- 0:(as.integer(value)-1L)
    sum(cell/(r+cell)*r/(r+cell))
  }, numeric(1))
  information <- matrix(0, parameter_count, parameter_count)
  information[seq_len(ncol(X)),seq_len(ncol(X))] <- crossprod(X, X*weight)
  information[seq_len(ncol(X)),parameter_count] <- drop(crossprod(X, cross))
  information[parameter_count,seq_len(ncol(X))] <- information[seq_len(ncol(X)),parameter_count]
  information[parameter_count,parameter_count] <- sum(r*remainder+cross-curvature)
  information
}
cell_probabilities <- function(candidate) {
  beta <- candidate[seq_len(ncol(X))]
  r <- exp(-candidate[[parameter_count]])
  mu <- drop(exp(X%*%beta))
  vapply(0:(K-1L), function(cell)
    dnbinom(cell, size=r, mu=mu), numeric(N))
}

poisson_start <- glm.fit(X, y, family=poisson())
stopifnot(poisson_start$rank==ncol(X), all(is.finite(poisson_start$coefficients)))
initial <- c(poisson_start$coefficients, log(0.5))
objective <- function(theta) -loglik(theta)
gradient <- function(theta) -score(theta)
fit <- optim(initial, objective, gradient, method="BFGS",
             control=list(reltol=1e-13, maxit=2000))
stopifnot(fit$convergence==0L)
theta <- fit$par
for (iteration in seq_len(8L)) {
  theta <- theta+drop(solve(information_at(theta), score(theta)))
  if (max(abs(score(theta)))<1e-9) break
}
stopifnot(max(abs(score(theta)))<1e-7)
information <- information_at(theta)
information_fd <- optimHess(theta, objective, gradient)
stopifnot(max(abs(information-information_fd))<2e-5*max(abs(information)))
beta <- theta[seq_len(ncol(X))]
alpha <- exp(theta[[parameter_count]])
r <- 1/alpha
mu <- drop(exp(X%*%beta))
probability <- cell_probabilities(theta)
observed <- vapply(0:(K-1L), function(cell) as.numeric(y==cell), numeric(N))
moments <- observed-probability

# Use the NB2 probability derivative, then verify it by finite differences of
# dnbinom probabilities at the fitted parameters.
C <- matrix(0, Jm, parameter_count)
for (cell in 0:(K-1L)) {
  eta_score <- (cell-mu)*r/(r+mu)
  alpha_score <- -r*(digamma(cell+r)-digamma(r)+
                      log(r/(r+mu))+1-(r+cell)/(r+mu))
  C[cell+1L,seq_len(ncol(X))] <- -drop(crossprod(X,
    probability[,cell+1L]*eta_score))
  C[cell+1L,parameter_count] <- -sum(probability[,cell+1L]*alpha_score)
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
stopifnot(max(abs(C-C_fd))<2e-6*(1+max(abs(C))))

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
  rows[[length(rows)+1L]] <<- data.frame(case="mma20p1_person", quantity,
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
for (i in seq_len(K+1L)) {
  add("observed_count", i-1L, value=observed_count[[i]])
  add("fitted_fraction", i-1L, value=fitted_bins[[i]])
}
write.csv(do.call(rbind, rows), args[[2]], row.names=FALSE, quote=FALSE)
