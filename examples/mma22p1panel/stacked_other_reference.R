# Independent QR fits and clustered Wald diagnostics for the other four regressors.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript stacked_other_reference.R stacked-reference/design.csv output-directory')
d <- read.csv(args[1],check.names=FALSE)
z <- unlist(lapply(1:8,function(i) paste0('z',1:9,'y',i)))
x <- cbind(intercept=1,as.matrix(d[,z])); n <- nrow(x); k <- ncol(x); g <- length(unique(d$id))
stopifnot(n==4256,g==532)
q <- qr(x,tol=1e-10); stopifnot(q$rank==k)
bread <- chol2inv(qr.R(q)); bread <- bread[order(q$pivot),order(q$pivot)]
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2]); d$panel_iv_fitted <- NULL; d$panel_iv_residual <- NULL
results <- data.frame()
for (yname in c('dkids','dageh','dagesq','ddisab')) {
  y <- d[[yname]]; beta <- qr.coef(q,y); u <- drop(y-x%*%beta)
  scores <- rowsum(x*u,d$id,reorder=FALSE)
  v <- bread%*%crossprod(scores)%*%bread*(g/(g-1))*((n-1)/(n-k))
  s <- sqrt(diag(v)[-1]); cor <- v[-1,-1]/outer(s,s)
  f <- drop(crossprod(beta[-1]/s,solve(cor,beta[-1]/s)))/(k-1)
  se <- sqrt(diag(v))
  results <- rbind(results,data.frame(name=yname,statistic=f,p_value=pf(f,k-1,g-1,lower.tail=FALSE),
    r_squared=1-sum(u^2)/sum((y-mean(y))^2),condition=kappa(cor,exact=TRUE)))
  write.csv(data.frame(name=colnames(x),beta=beta,se=se,p_value=2*pt(-abs(beta/se),g-1)),file.path(args[2],paste0(yname,'-coefficients.csv')),row.names=FALSE)
  write.csv(v,file.path(args[2],paste0(yname,'-covariance.csv')),row.names=FALSE)
  d[[paste0('fitted_',yname)]] <- drop(x%*%beta)
  d[[paste0('residual_',yname)]] <- u
}
write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
write.csv(results,file.path(args[2],'tests.csv'),row.names=FALSE)
