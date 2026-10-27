# Independent R QR-based construction and 2SLS covariance calculation.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript reference.R MOMprecise.dat output-directory')
raw <- scan(args[1],quiet=TRUE)
stopifnot(length(raw)==5320*8)
d <- as.data.frame(matrix(raw,ncol=8,byrow=TRUE))
names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
stopifnot(all(d$id==rep(1:532,each=10)),all(d$year==rep(1979:1988,532)))
idx <- which(d$year>=1981)
stopifnot(all(d$id[idx]==d$id[idx-2]),all(d$year[idx-2]==d$year[idx]-2))
result <- data.frame(id=d$id[idx],year=d$year[idx],dlnhr=d$lnhr[idx]-d$lnhr[idx-1],
 dlnwg=d$lnwg[idx]-d$lnwg[idx-1],dkids=d$kids[idx]-d$kids[idx-1],
 dageh=d$ageh[idx]-d$ageh[idx-1],dagesq=d$agesq[idx]-d$agesq[idx-1],
 ddisab=d$disab[idx]-d$disab[idx-1],kidsl1=d$kids[idx-1],agehl1=d$ageh[idx-1],
 agesql1=d$agesq[idx-1],disabl1=d$disab[idx-1],agehl2=d$ageh[idx-2],
 kidsl2=d$kids[idx-2],agesql2=d$agesq[idx-2],disabl2=d$disab[idx-2],lnwgl2=d$lnwg[idx-2])
x <- as.matrix(result[,c('dlnwg','dkids','dageh','dagesq','ddisab')])
z <- as.matrix(result[,c('kidsl1','agehl1','agesql1','disabl1','agehl2','kidsl2','agesql2','disabl2','lnwgl2')])
y <- result$dlnhr
qz <- qr(z,tol=1e-10); stopifnot(qz$rank==ncol(z))
xhat <- qr.fitted(qz,x)
qx <- qr(xhat,tol=1e-10); stopifnot(qx$rank==ncol(x))
beta <- qr.coef(qx,y)
resid <- drop(y-x%*%beta)
bread <- chol2inv(qr.R(qx)); bread <- bread[order(qx$pivot),order(qx$pivot)]
n <- nrow(x); k <- ncol(x); g <- length(unique(result$id))
classical <- sum(resid^2)/(n-k)*bread
scores <- rowsum(xhat*resid,result$id,reorder=FALSE)
cluster <- bread%*%crossprod(scores)%*%bread * (g/(g-1))*((n-1)/(n-k))
result$panel_iv_fitted <- drop(x%*%beta)
result$panel_iv_residual <- resid
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2])
write.csv(result,file.path(args[2],'design.csv'),row.names=FALSE)
se_classical <- sqrt(diag(classical)); se_cluster <- sqrt(diag(cluster))
write.csv(data.frame(name=colnames(x),beta=beta,classical=se_classical,cluster=se_cluster,
 classical_p=2*pt(-abs(beta/se_classical),n-k),cluster_p=2*pt(-abs(beta/se_cluster),g-1)),
 file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(classical,file.path(args[2],'classical.csv'),row.names=FALSE)
write.csv(cluster,file.path(args[2],'cluster.csv'),row.names=FALSE)
