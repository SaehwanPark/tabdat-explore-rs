# Base-R reference for two-instrument robust AR confidence sets.
# Rscript robust_ar_multi_ci_reference.R OUTPUT_DIRECTORY > reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)
dir.create(args[[1]],recursive=TRUE,showWarnings=FALSE)

set.seed(2)
n <- 20L
for (trial in seq_len(5L)) {
  w <- rnorm(n)
  z <- matrix(rnorm(n*2L),n,2L)
  x <- rnorm(n)
  y <- rnorm(n)
}
groups <- rep(seq_len(10L),each=2L)
multiple <- data.frame(row=seq_len(n),y,w,z1=z[,1],z2=z[,2],x,group=groups)
write.csv(multiple,file.path(args[[1]],'robust_ar_multi_ci_data.csv'),
  row.names=FALSE,quote=FALSE)

set.seed(99)
W <- cbind(intercept=1,w)
A <- cbind(W,z)
M0 <- diag(n)-A%*%solve(crossprod(A),t(A))
all_real <- data.frame(row=seq_len(n),y=drop(M0%*%rnorm(n)),w,
  z1=z[,1],z2=z[,2],x=.01*z[,1]+drop(M0%*%rnorm(n)),group=groups)
write.csv(all_real,file.path(args[[1]],'robust_ar_multi_ci_all_real_data.csv'),
  row.names=FALSE,quote=FALSE)

padd <- function(a,b,scale=1) {
  out <- numeric(max(length(a),length(b)))
  out[seq_along(a)] <- out[seq_along(a)]+a
  out[seq_along(b)] <- out[seq_along(b)]+scale*b
  out
}
pmul <- function(a,b) {
  out <- numeric(length(a)+length(b)-1L)
  for (i in seq_along(a)) for (j in seq_along(b))
    out[i+j-1L] <- out[i+j-1L]+a[i]*b[j]
  out
}

fit_reference <- function(data,fixture,covariance) {
  n <- nrow(data)
  X <- cbind(intercept=1,data$w,data$z1,data$z2)
  W <- cbind(intercept=1,data$w)
  Z <- cbind(data$z1,data$z2)
  y <- data$y
  x <- data$x
  groups <- data$group
  Q <- qr.resid(qr(W),Z)
  y_w <- qr.resid(qr(W),y)
  x_w <- qr.resid(qr(W),x)
  y_residual <- qr.resid(qr(X),y)
  x_residual <- qr.resid(qr(X),x)
  g0 <- drop(crossprod(Q,y_w))
  gx <- drop(crossprod(Q,x_w))

  if (covariance=='cluster') {
    score_y <- rowsum(Q*y_residual,groups)
    score_x <- rowsum(Q*x_residual,groups)
    cluster_count <- length(unique(groups))
    correction <- cluster_count/(cluster_count-1)*(n-1)/(n-ncol(X))
  } else {
    score_y <- Q*y_residual
    score_x <- Q*x_residual
    cluster_count <- NA_integer_
    correction <- n/(n-ncol(X))
  }

  s11 <- c(sum(score_y[,1]^2),-2*sum(score_y[,1]*score_x[,1]),
    sum(score_x[,1]^2))
  s12 <- c(sum(score_y[,1]*score_y[,2]),
    -sum(score_y[,1]*score_x[,2]+score_x[,1]*score_y[,2]),
    sum(score_x[,1]*score_x[,2]))
  s22 <- c(sum(score_y[,2]^2),-2*sum(score_y[,2]*score_x[,2]),
    sum(score_x[,2]^2))
  det_s <- padd(pmul(s11,s22),pmul(s12,s12),-1)
  t1 <- c(g0[1],-gx[1])
  t2 <- c(g0[2],-gx[2])
  critical <- qchisq(.95,df=2)
  boundary <- padd(
    padd(pmul(s22,pmul(t1,t1)),pmul(s11,pmul(t2,t2))),
    pmul(s12,pmul(t1,t2)),-2)
  boundary <- padd(boundary,det_s,-critical*correction)
  boundary_roots <- polyroot(boundary)
  real_roots <- sort(Re(boundary_roots[
    abs(Im(boundary_roots))<1e-7 & is.finite(Re(boundary_roots))]))
  det_roots <- polyroot(det_s)
  stopifnot(all(abs(Im(det_roots))>=1e-7))

  regions <- c(-Inf,real_roots,Inf)
  root_bound <- if (length(real_roots)==0L) 10 else max(10,10*max(abs(real_roots)))
  accepted <- logical(length(regions)-1L)
  for (i in seq_along(accepted)) {
    point <- if (is.infinite(regions[i])) {
      -root_bound
    } else if (is.infinite(regions[i+1L])) {
      root_bound
    } else {
      regions[i]+(regions[i+1L]-regions[i])/2
    }
    value <- sum(boundary*point^(seq_along(boundary)-1L))
    accepted[i] <- value<=0
  }
  intervals <- list()
  for (i in seq_along(accepted)) {
    if (accepted[i]) {
      lower <- if (i==1L) -Inf else real_roots[i-1L]
      upper <- if (i>length(real_roots)) Inf else real_roots[i]
      intervals[[length(intervals)+1L]] <- c(lower,upper)
    }
  }
  if (length(real_roots)==0L && accepted[1L])
    intervals <- list(c(-Inf,Inf))
  if (length(intervals)==0L) {
    shape <- 'empty'
  } else if (length(intervals)==1L && all(is.infinite(intervals[[1L]]))) {
    shape <- 'all_real'
  } else if (length(intervals)>2L) {
    shape <- 'multiple'
  } else if (length(intervals)==2L) {
    shape <- 'disjoint'
  } else if (is.infinite(intervals[[1L]][1L])) {
    shape <- 'lower_ray'
  } else if (is.infinite(intervals[[1L]][2L])) {
    shape <- 'upper_ray'
  } else {
    shape <- 'bounded'
  }

  bread <- solve(crossprod(X))
  direct_statistic <- function(beta) {
    auxiliary <- lm.fit(X,y-beta*x)
    scores <- X*auxiliary$residuals
    if (covariance=='cluster') scores <- rowsum(scores,groups)
    covariance <- correction*bread%*%crossprod(scores)%*%bread
    gamma <- auxiliary$coefficients[3:4]
    as.numeric(t(gamma)%*%solve(covariance[3:4,3:4],gamma))
  }
  base <- data.frame(
    fixture=fixture,covariance=covariance,shape=shape,
    confidence_level=95,observation_count=n,
    auxiliary_parameter_count=ncol(X),auxiliary_residual_df=n-ncol(X),
    covariance_correction=correction,cluster_group_count=cluster_count,
    degrees_of_freedom=2,critical_chi_square=critical,
    record='interval',interval_index=seq_along(intervals),
    lower=vapply(intervals,function(x)x[1],numeric(1)),
    upper=vapply(intervals,function(x)x[2],numeric(1)),
    beta=NA_real_,statistic=NA_real_
  )
  probes <- switch(fixture,
    multiple=if (covariance=='hc1') c(-10,.4,.9,1.5,3) else c(-1,.3,.7,10),
    all_real=c(-10,0,10))
  probe_rows <- data.frame(
    fixture=fixture,covariance=covariance,shape=shape,
    confidence_level=95,observation_count=n,
    auxiliary_parameter_count=ncol(X),auxiliary_residual_df=n-ncol(X),
    covariance_correction=correction,cluster_group_count=cluster_count,
    degrees_of_freedom=2,critical_chi_square=critical,
    record='probe',interval_index=NA_integer_,lower=NA_real_,upper=NA_real_,
    beta=probes,statistic=vapply(probes,direct_statistic,numeric(1))
  )
  rbind(base,probe_rows)
}

reference <- rbind(
  fit_reference(multiple,'multiple','hc1'),
  fit_reference(multiple,'multiple','cluster'),
  fit_reference(all_real,'all_real','hc1'),
  fit_reference(all_real,'all_real','cluster')
)
write.csv(reference,stdout(),row.names=FALSE,quote=FALSE)
