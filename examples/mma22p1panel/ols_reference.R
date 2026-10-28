# Independent QR OLS and covariance on the independently reconstructed source design.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript ols_reference.R reference/design.csv output-directory')
d <- read.csv(args[1],check.names=FALSE)
x <- as.matrix(d[,c('dlnwg','dkids','dageh','dagesq','ddisab')])
y <- d$dlnhr
n <- nrow(x); k <- ncol(x); g <- length(unique(d$id))
stopifnot(n==4256, g==532)
q <- qr(x,tol=1e-10); stopifnot(q$rank==k)
beta <- qr.coef(q,y)
u <- drop(y-x%*%beta)
bread <- chol2inv(qr.R(q)); bread <- bread[order(q$pivot),order(q$pivot)]
classical <- sum(u^2)/(n-k)*bread
hc1 <- bread%*%crossprod(x*u)%*%bread*n/(n-k)
scores <- rowsum(x*u,d$id,reorder=FALSE)
cluster <- bread%*%crossprod(scores)%*%bread*(g/(g-1))*((n-1)/(n-k))
d$panel_iv_fitted <- NULL; d$panel_iv_residual <- NULL
d$fd_ols_fitted <- drop(x%*%beta); d$fd_ols_residual <- u
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2])
write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
coefs <- data.frame(name=colnames(x),beta=beta)
for (mode in c('classical','hc1','cluster')) {
  v <- get(mode); se <- sqrt(diag(v)); df <- if (mode=='cluster') g-1 else n-k
  coefs[[mode]] <- se
  coefs[[paste0(mode,'_p')]] <- 2*pt(-abs(beta/se),df)
  write.csv(v,file.path(args[2],paste0(mode,'.csv')),row.names=FALSE)
}
write.csv(coefs,file.path(args[2],'coefficients.csv'),row.names=FALSE)
