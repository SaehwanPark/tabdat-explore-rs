# Independent QR first-stage fit and joint Wald F statistics.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript stacked_stage_reference.R stacked-reference/design.csv output-directory')
d <- read.csv(args[1],check.names=FALSE)
z <- unlist(lapply(1:8,function(i) paste0('z',1:9,'y',i)))
x <- cbind(intercept=1,as.matrix(d[,z])); y <- d$dlnwg
n <- nrow(x); k <- ncol(x); g <- length(unique(d$id))
stopifnot(n==4256,g==532)
q <- qr(x,tol=1e-10); stopifnot(q$rank==k)
beta <- qr.coef(q,y); u <- drop(y-x%*%beta)
bread <- chol2inv(qr.R(q)); bread <- bread[order(q$pivot),order(q$pivot)]
classical <- sum(u^2)/(n-k)*bread
scores <- rowsum(x*u,d$id,reorder=FALSE)
cluster <- bread%*%crossprod(scores)%*%bread*(g/(g-1))*((n-1)/(n-k))
r2 <- 1-sum(u^2)/sum((y-mean(y))^2)
d$panel_iv_fitted <- NULL; d$panel_iv_residual <- NULL
d$stage_fitted <- drop(x%*%beta); d$stage_residual <- u
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2]); write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
coefs <- data.frame(name=colnames(x),beta=beta)
tests <- data.frame(mode=character(),statistic=double(),p_value=double(),df=integer(),df_residual=integer(),r_squared=double())
for (mode in c('cluster','classical')) {
  v <- get(mode); df <- if (mode=='cluster') g-1 else n-k
  se <- sqrt(diag(v)); coefs[[mode]] <- se; coefs[[paste0(mode,'_p')]] <- 2*pt(-abs(beta/se),df)
  # Scale the covariance before solving the joint Wald system to control units.
  s <- sqrt(diag(v)[-1]); cor <- v[-1,-1]/outer(s,s)
  f <- drop(crossprod(beta[-1]/s,solve(cor,beta[-1]/s)))/(k-1)
  tests <- rbind(tests,data.frame(mode=mode,statistic=f,p_value=pf(f,k-1,df,lower.tail=FALSE),df=k-1,df_residual=df,r_squared=r2))
  write.csv(v,file.path(args[2],paste0(mode,'.csv')),row.names=FALSE)
}
write.csv(coefs,file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(tests,file.path(args[2],'tests.csv'),row.names=FALSE)
