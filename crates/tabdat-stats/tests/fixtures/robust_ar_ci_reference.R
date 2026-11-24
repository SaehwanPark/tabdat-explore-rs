# Development-only base R reference: Rscript <this file> <fixture directory>.
# No RNG or statistical package needed. The reference independently fits the
# auxiliary regressions and forms HC1/CR1 covariance from their observation or
# cluster scores, then inverts the one-restriction Wald statistic.
options(digits=17)
args <- commandArgs(TRUE)
out <- if (length(args)) args[1] else tempdir()
dir.create(out, recursive=TRUE, showWarnings=FALSE)
i <- 1:80
w <- sin(i/7) + i/90
z <- cos(i/5) + i/70
x <- 0.35*w + 1.25*z + sin(i/3) + cos(i/9)
error <- (0.5+i/40)*(sin(i/1.7)+cos(i/2.3))
y <- 1.2 + 0.5*w + 1.7*x + error
d <- data.frame(source_row=i,y,w,x,z,g=(i-1)%%10+1)
# Check exact complete-case retention, including a cluster-only exclusion.
d$y[2] <- NA; d$z[8] <- NA; d$g[17] <- NA
write.table(d, file.path(out,'robust_ar_ci_data.csv'), sep=',', quote=FALSE,
            row.names=FALSE, na='', col.names=TRUE)

rows <- list()
for (covariance in c('hc1','cluster')) {
  fields <- c('y','w','x','z',if(covariance=='cluster') 'g')
  keep <- complete.cases(d[,fields])
  a <- d[keep,,drop=FALSE]
  n <- nrow(a)
  C <- cbind(intercept=1,w=a$w)
  X <- cbind(C,z=a$z)
  k <- ncol(X)
  df <- n-k
  residual_z <- lm.fit(C,a$z)$residuals
  residual_y <- lm.fit(C,a$y)$residuals
  residual_x <- lm.fit(C,a$x)$residuals
  residual_y_full <- lm.fit(X,a$y)$residuals
  residual_x_full <- lm.fit(X,a$x)$residuals
  reduced_form_y <- sum(residual_z*residual_y)
  reduced_form_x <- sum(residual_z*residual_x)
  center <- reduced_form_y/reduced_form_x
  score_y <- residual_z*residual_y_full
  score_x <- residual_z*residual_x_full
  G <- if(covariance=='cluster') length(unique(a$g)) else 0
  correction <- if(G) G/(G-1)*(n-1)/df else n/df
  if(G) {
    score_y <- as.numeric(rowsum(score_y,a$g,reorder=FALSE))
    score_x <- as.numeric(rowsum(score_x,a$g,reorder=FALSE))
  }
  centered_moment <- sum(residual_z*(residual_y-center*residual_x))
  centered_score_y <- score_y-center*score_x

  statistic <- function(beta) {
    fit <- lm.fit(X,a$y-beta*a$x)
    scores <- X*as.numeric(fit$residuals)
    if(G) scores <- rowsum(scores,a$g,reorder=FALSE)
    bread <- solve(crossprod(X))
    V <- correction*bread%*%crossprod(scores)%*%bread
    as.numeric(fit$coefficients[3]^2/V[3,3])
  }
  for (level in c(90,95)) {
    Q <- qchisq(level/100,1)*correction
    qa <- reduced_form_x^2-Q*sum(score_x^2)
    qb <- -2*centered_moment*reduced_form_x+2*Q*sum(centered_score_y*score_x)
    qc <- centered_moment^2-Q*sum(centered_score_y^2)
    roots <- polyroot(c(qc,qb,qa))
    roots <- sort(Re(roots[abs(Im(roots))<1e-8]))+center
    if(length(roots)!=2) stop(paste('expected two finite roots for',covariance,level))
    rows[[length(rows)+1]] <- data.frame(
      covariance,level,n,k,G,correction,critical=qchisq(level/100,1),center,
      lower=roots[1],upper=roots[2],
      statistic_low=statistic(roots[1]),statistic_center=statistic(center),
      statistic_high=statistic(roots[2]),
      statistic_below=statistic(roots[1]-0.5),
      statistic_above=statistic(roots[2]+0.5)
    )
  }
}
write.table(do.call(rbind,rows),file.path(out,'robust_ar_ci_reference.csv'),
            sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE)
cat(R.version.string,'\n')
