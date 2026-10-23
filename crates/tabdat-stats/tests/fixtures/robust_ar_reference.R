# Development-only base R reference: Rscript <this file> <output directory>.
# No RNG or statistical package needed. Unrestricted OLS auxiliary residuals,
# HC1/CR1 sandwich, and direct excluded-instrument Wald chi-square.
args <- commandArgs(TRUE)
out <- if (length(args)) args[1] else tempdir()
dir.create(out, recursive=TRUE, showWarnings=FALSE)
i <- 1:72
w <- sin(i/7) + i/90
z1 <- cos(i/5) + i/70
z2 <- sin(i/11) - cos(i/13) + i/200
z3 <- cos(i/2.7)
x1 <- .3 + .4*w + .09*z1 + .02*z2 + sin(i/3) + cos(i/9)
x2 <- -.2 + .2*w - .03*z1 + .11*z2 + cos(i/4) - sin(i/6)
y <- 1.2 + .5*w + 1.7*x1 - .6*x2 + (1+i/40)*(sin(i/1.7)+cos(i/2.3))
d <- data.frame(source_row=i,y,w,x1,x2,z1,z2,z3,g=(i-1)%%12)
# Nonconsecutive sample omissions, including a missing cluster only.
d$y[2] <- NA; d$z2[8] <- NA; d$g[17] <- NA
write.table(d, file.path(out,'robust_ar_data.csv'), sep=',', quote=FALSE, row.names=FALSE, na='')
rows <- list()
for (design in c('controlled','nointercept','noexogenous')) {
  for (covariance in c('hc1','cluster')) {
    fields <- c('y','x1','x2','z1','z2','z3',if(design!='noexogenous') 'w',if(covariance=='cluster') 'g')
    keep <- complete.cases(d[,fields]); a <- d[keep,]
    X <- as.matrix(a[,c('z1','z2','z3')])
    if(design!='noexogenous') X <- cbind(w=a$w,X)
    if(design!='nointercept') X <- cbind(intercept=1,X)
    n <- nrow(X); k <- ncol(X)
    u <- a$y - 1.7*a$x1 + .6*a$x2
    fit <- lm.fit(X,u)
    score <- X * as.numeric(fit$residuals)
    g <- if(covariance=='cluster') length(unique(a$g)) else 0
    correction <- if(g) g/(g-1)*(n-1)/(n-k) else n/(n-k)
    if(g) score <- rowsum(score,a$g,reorder=FALSE)
    bread <- solve(crossprod(X)); V <- correction*bread%*%crossprod(score)%*%bread
    ix <- (k-2):k; b <- fit$coefficients[ix]
    W <- as.numeric(t(b)%*%solve(V[ix,ix],b))
    rows[[length(rows)+1]] <- data.frame(design,covariance,n,k,g,correction,statistic=W,p_value=pchisq(W,3,lower.tail=FALSE))
  }
}
write.table(do.call(rbind,rows),file.path(out,'robust_ar_reference.csv'),sep=',',quote=FALSE,row.names=FALSE)
cat(R.version.string, '\n')
