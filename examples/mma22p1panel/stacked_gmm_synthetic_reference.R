# Deterministic, overidentified 20-instrument control, independent of Rust.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1) stop('usage: Rscript stacked_gmm_synthetic_reference.R output-directory')
n <- 192; l <- 20; i <- seq_len(n); id <- rep(seq_len(64),each=3)
z <- sapply(seq_len(l),function(j) sin(i*(j+1)*0.137)+cos(i*(j+3)*0.071)+sin(id*(j+2)*0.119))
colnames(z) <- paste0('z',seq_len(l))
x <- cbind(x1=z[,1]+0.4*z[,2]+sin(i*0.37),x2=z[,3]-0.3*z[,4]+cos(i*0.29))
y <- drop(x%*%c(1.2,-0.7)+sin(i*0.43)+0.4*cos(id*0.17))
qz <- qr(z,tol=1e-12); stopifnot(qz$rank==l)
xhat <- qr.fitted(qz,x); qx <- qr(xhat,tol=1e-12); stopifnot(qx$rank==2)
u1 <- drop(y-x%*%qr.coef(qx,y))
moment <- function(u) crossprod(rowsum(z*u,id,reorder=FALSE))
s1 <- moment(u1); a <- crossprod(z,x); b <- crossprod(z,y)
w1a <- solve(s1,a); bread <- solve(crossprod(a,w1a))
beta <- drop(bread%*%crossprod(a,solve(s1,b))); u2 <- drop(y-x%*%beta)
s2 <- moment(u2); m <- crossprod(z,u2)
fixed <- bread%*%crossprod(w1a,s2%*%w1a)%*%bread
updated <- solve(crossprod(a,solve(s2,a)))
j1 <- drop(crossprod(m,solve(s1,m))); j2 <- drop(crossprod(m,solve(s2,m)))
if (dir.exists(args[1])) stop('output directory exists')
dir.create(args[1]); write.csv(data.frame(id,y,x,z),file.path(args[1],'data.csv'),row.names=FALSE)
write.csv(data.frame(beta1=beta[1],beta2=beta[2],fixed11=fixed[1,1],fixed12=fixed[1,2],fixed22=fixed[2,2],
 updated11=updated[1,1],updated12=updated[1,2],updated22=updated[2,2],j1=j1,j2=j2,
 p1=pchisq(j1,l-2,lower.tail=FALSE),p2=pchisq(j2,l-2,lower.tail=FALSE)),
 file.path(args[1],'reference.csv'),row.names=FALSE)
