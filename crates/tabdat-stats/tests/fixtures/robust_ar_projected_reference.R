# Independent Base-R reference for a two-endogenous, two-instrument robust AR
# projection. It computes each joint Wald test by auxiliary OLS, then numerically
# profiles the nuisance coefficient with a broad pilot grid and local optimize
# calls. This is deliberately independent of the production polynomial solver.
#
# Rscript robust_ar_projected_reference.R OUTPUT_DIRECTORY > robust_ar_projected_reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)
dir.create(args[[1]],recursive=TRUE,showWarnings=FALSE)

make_data <- function() {
  row <- seq_len(48L)
  phase <- 2*pi*row/48
  w <- sin(3*phase)+.15*cos(7*phase)
  z1 <- cos(phase)+.12*sin(8*phase)
  z2 <- sin(2*phase)+.17*cos(9*phase)
  group <- rep(seq_len(12L),each=4L)
  v1 <- .75*cos(4*phase)+.1*sin(11*phase)
  v2 <- .7*sin(5*phase)+.1*cos(12*phase)
  x1 <- .7*z1+.35*z2+.5*w+v1
  x2 <- -.2*z1+.9*z2-.25*w+v2
  group_error <- rep(.8*sin(seq_len(12L)*1.7)+.3*cos(seq_len(12L)*2.1),each=4L)
  error <- .35*cos(6*phase)+.2*sin(7*phase)+group_error
  y <- .8+.3*w+1.1*x1-.55*x2+error
  data.frame(row,y,w,z1,z2,x1,x2,group)
}

data <- make_data()
write.csv(data,file.path(args[[1]],'robust_ar_projected_data.csv'),
  row.names=FALSE,quote=FALSE)

n <- nrow(data)
W <- cbind(intercept=1,data$w)
Z <- as.matrix(data[,c('z1','z2')])
auxiliary_design <- cbind(W,Z)
E <- as.matrix(data[,c('x1','x2')])
y <- data$y
groups <- data$group
parameter_count <- ncol(auxiliary_design)
residual_df <- n-parameter_count
group_count <- length(unique(groups))
critical <- qchisq(.95,df=2)

joint_statistic <- function(target,nuisance,covariance) {
  adjusted <- y-drop(E%*%c(target,nuisance))
  fit <- lm.fit(auxiliary_design,adjusted)
  scores <- auxiliary_design*fit$residuals
  if (covariance=='cluster') scores <- rowsum(scores,groups)
  correction <- if (covariance=='hc1') n/residual_df else
    group_count/(group_count-1)*(n-1)/residual_df
  bread <- solve(crossprod(auxiliary_design))
  vcov <- correction*bread%*%crossprod(scores)%*%bread
  endogenous <- (ncol(W)+1L):ncol(auxiliary_design)
  coefficient <- fit$coefficients[endogenous]
  block <- vcov[endogenous,endogenous,drop=FALSE]
  as.numeric(t(coefficient)%*%solve(block,coefficient))
}

# Numerical profiling supplies an independent check on the exact production
# inversion. The pilot grid spans the nuisance line; optimize refines every
# local minimum found on it and retains the better boundary values.
profile <- function(target,covariance) {
  nuisance_grid <- seq(-30,30,length.out=601L)
  values <- vapply(nuisance_grid,joint_statistic,numeric(1),
    target=target,covariance=covariance)
  local_minima <- which(values<=c(Inf,values[-length(values)]) &
    values<=c(values[-1L],Inf))
  if (!length(local_minima)) local_minima <- which.min(values)
  best_value <- min(values[c(1L,length(values),local_minima)])
  best_nuisance <- nuisance_grid[which.min(values)]
  for (index in local_minima) if (index>1L && index<length(nuisance_grid)) {
    result <- optimize(function(value)
      joint_statistic(target,value,covariance),
      nuisance_grid[c(index-1L,index+1L)],tol=1e-10)
    if (result$objective<best_value) {
      best_value <- result$objective
      best_nuisance <- result$minimum
    }
  }
  list(statistic=best_value,nuisance=best_nuisance)
}

reference_one <- function(covariance) {
  target_grid <- seq(-10,10,length.out=201L)
  profile_values <- vapply(target_grid,function(value)
    profile(value,covariance)$statistic,numeric(1))
  included <- which(profile_values<=critical)
  stopifnot(length(included)>0L,all(diff(included)==1L),
    min(included)>1L,max(included)<length(target_grid))
  lower <- uniroot(function(value)
    profile(value,covariance)$statistic-critical,
    target_grid[c(min(included)-1L,min(included))],tol=1e-9)$root
  upper <- uniroot(function(value)
    profile(value,covariance)$statistic-critical,
    target_grid[c(max(included),max(included)+1L)],tol=1e-9)$root
  correction <- if (covariance=='hc1') n/residual_df else
    group_count/(group_count-1)*(n-1)/residual_df
  common <- data.frame(
    covariance=covariance,shape='bounded',confidence_level=95,
    observation_count=n,auxiliary_parameter_count=parameter_count,
    auxiliary_residual_df=residual_df,covariance_correction=correction,
    cluster_group_count=if (covariance=='cluster') group_count else NA_integer_,
    degrees_of_freedom=2,critical_chi_square=critical,
    record='interval',interval_index=1L,lower=lower,upper=upper,
    target=NA_real_,nuisance=NA_real_,statistic=NA_real_
  )
  probes <- c(lower-.1,lower,(lower+upper)/2,upper,upper+.1)
  profiled <- lapply(probes,function(value) profile(value,covariance))
  probe_rows <- data.frame(
    covariance=covariance,shape='bounded',confidence_level=95,
    observation_count=n,auxiliary_parameter_count=parameter_count,
    auxiliary_residual_df=residual_df,covariance_correction=correction,
    cluster_group_count=if (covariance=='cluster') group_count else NA_integer_,
    degrees_of_freedom=2,critical_chi_square=critical,
    record='probe',interval_index=NA_integer_,lower=NA_real_,upper=NA_real_,
    target=probes,
    nuisance=vapply(profiled,`[[`,numeric(1),'nuisance'),
    statistic=vapply(profiled,`[[`,numeric(1),'statistic')
  )
  rbind(common,probe_rows)
}

reference <- rbind(reference_one('hc1'),reference_one('cluster'))
write.csv(reference,stdout(),row.names=FALSE,quote=FALSE)
