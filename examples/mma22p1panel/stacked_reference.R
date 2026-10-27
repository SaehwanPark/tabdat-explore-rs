# Independent R QR calculation on the independently constructed base design.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript stacked_reference.R base-reference/design.csv output-directory')
d <- read.csv(args[1]); stopifnot(nrow(d)==4256,all(d$year %in% 1981:1988))
base <- c('agehl1','agesql1','kidsl1','disabl1','agehl2','agesql2','kidsl2','disabl2','lnwgl2')
for (i in 1:8) for (j in 1:9) d[[paste0('z',j,'y',i)]] <- ifelse(d$year==1980+i,d[[base[j]]],0)
x <- as.matrix(d[,c('dlnwg','dkids','dageh','dagesq','ddisab')])
names_z <- unlist(lapply(1:8,function(i) paste0('z',1:9,'y',i)))
z <- as.matrix(d[,names_z]); y <- d$dlnhr
qz <- qr(z,tol=1e-10); stopifnot(qz$rank==ncol(z))
xhat <- qr.fitted(qz,x); qx <- qr(xhat,tol=1e-10); stopifnot(qx$rank==ncol(x))
beta <- qr.coef(qx,y); resid <- drop(y-x%*%beta)
bread <- chol2inv(qr.R(qx)); bread <- bread[order(qx$pivot),order(qx$pivot)]
n <- nrow(x); k <- ncol(x); g <- length(unique(d$id))
classical <- sum(resid^2)/(n-k)*bread
scores <- rowsum(xhat*resid,d$id,reorder=FALSE)
cluster <- bread%*%crossprod(scores)%*%bread*(g/(g-1))*((n-1)/(n-k))
d$panel_iv_fitted <- drop(x%*%beta); d$panel_iv_residual <- resid
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2])
write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
se_classical <- sqrt(diag(classical)); se_cluster <- sqrt(diag(cluster))
write.csv(data.frame(name=colnames(x),beta=beta,classical=se_classical,cluster=se_cluster,
 classical_p=2*pt(-abs(beta/se_classical),n-k),cluster_p=2*pt(-abs(beta/se_cluster),g-1)),
 file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(classical,file.path(args[2],'classical.csv'),row.names=FALSE)
write.csv(cluster,file.path(args[2],'cluster.csv'),row.names=FALSE)
