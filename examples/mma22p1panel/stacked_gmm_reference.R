# Independent R reference for the year-stacked Table 22.2 clustered GMM design.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript stacked_gmm_reference.R stacked-reference/design.csv output-directory')
d <- read.csv(args[1],check.names=FALSE)
xnames <- c('dlnwg','dkids','dageh','dagesq','ddisab')
znames <- unlist(lapply(1:8,function(i) paste0('z',1:9,'y',i)))
x <- as.matrix(d[,xnames]); z <- as.matrix(d[,znames]); y <- d$dlnhr
groups <- d$id; n <- nrow(x); k <- ncol(x); l <- ncol(z); g <- length(unique(groups))
stopifnot(n==4256,k==5,l==72,g==532)
sx <- sqrt(colSums(x*x)); sz <- sqrt(colSums(z*z))
xs <- sweep(x,2,sx,'/'); zs <- sweep(z,2,sz,'/')
qz <- qr(zs,tol=1e-12); stopifnot(qz$rank==l)
xhat <- qr.fitted(qz,xs); qx <- qr(xhat,tol=1e-12); stopifnot(qx$rank==k)
b1 <- qr.coef(qx,y); u1 <- drop(y-xs%*%b1)
moment <- function(u) crossprod(rowsum(zs*u,groups,reorder=FALSE))
s1 <- moment(u1); a <- crossprod(zs,xs); b <- crossprod(zs,y)
w1a <- solve(s1,a); w1b <- solve(s1,b)
bread <- solve(crossprod(a,w1a)); b2 <- drop(bread%*%crossprod(a,w1b))
u2 <- drop(y-xs%*%b2); s2 <- moment(u2); m <- crossprod(zs,u2)
fixed <- bread%*%crossprod(w1a,s2%*%w1a)%*%bread
updated <- solve(crossprod(a,solve(s2,a)))
fixed <- fixed/outer(sx,sx); updated <- updated/outer(sx,sx); beta <- b2/sx
j_fixed <- drop(crossprod(m,solve(s1,m))); j_updated <- drop(crossprod(m,solve(s2,m)))
d$panel_iv_fitted <- NULL; d$panel_iv_residual <- NULL
d$gmm_fitted <- drop(x%*%beta); d$gmm_residual <- u2
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2]); write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
se <- sqrt(diag(fixed))
write.csv(data.frame(name=xnames,beta=beta,fixed_se=se,updated_se=sqrt(diag(updated)),
 p_value=2*pnorm(-abs(beta/se)),confidence_lower=beta-qnorm(.975)*se,
 confidence_upper=beta+qnorm(.975)*se),file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(fixed,file.path(args[2],'fixed_covariance.csv'),row.names=FALSE)
write.csv(updated,file.path(args[2],'updated_covariance.csv'),row.names=FALSE)
write.csv(data.frame(n=n,groups=g,parameters=k,instruments=l,fixed_j=j_fixed,updated_j=j_updated,
 fixed_p=pchisq(j_fixed,l-k,lower.tail=FALSE),updated_p=pchisq(j_updated,l-k,lower.tail=FALSE),
 pilot_moment_condition=kappa(cov2cor(s1),exact=TRUE),updated_moment_condition=kappa(cov2cor(s2),exact=TRUE)),
 file.path(args[2],'diagnostics.csv'),row.names=FALSE)
