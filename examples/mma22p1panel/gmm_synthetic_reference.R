# Deterministic compact overidentified and exactly identified clustered controls.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1) stop('usage: Rscript gmm_synthetic_reference.R output-directory')
if (dir.exists(args[1])) stop('output directory exists')
dir.create(args[1])
i <- rep(1:16,each=3); t <- rep(1:3,16)
z1 <- sin(i*.71+t*.3)+i*.03; z2 <- cos(i*.41-t*.62)+t*.07
x <- .9*z1+.45*z2+.17*sin(i*t*.83)
y <- 1.7*x+.3*cos(i*.57+t*.81)+.12*sin(i*t*.51)
d <- data.frame(id=i,y=y,x=x,z1=z1,z2=z2)
write.csv(d,file.path(args[1],'data.csv'),row.names=FALSE)
for (case in c('overidentified','exact')) {
  z <- if (case=='exact') matrix(z1,ncol=1) else cbind(z1,z2)
  X <- matrix(x,ncol=1); q <- qr(z); xhat <- qr.fitted(q,X)
  b1 <- qr.coef(qr(xhat),y); u1 <- y-drop(X%*%b1)
  S <- function(u) crossprod(rowsum(z*u,i,reorder=FALSE))
  s1 <- S(u1); a <- crossprod(z,X); b <- crossprod(z,y)
  w1a <- solve(s1,a); w1b <- solve(s1,b); bread <- solve(crossprod(a,w1a))
  b2 <- drop(bread%*%crossprod(a,w1b)); u2 <- y-drop(X*b2); s2 <- S(u2)
  fixed <- drop(bread%*%crossprod(w1a,s2%*%w1a)%*%bread)
  updated <- drop(solve(crossprod(a,solve(s2,a)))); m <- crossprod(z,u2)
  j1 <- drop(crossprod(m,solve(s1,m))); j2 <- drop(crossprod(m,solve(s2,m)))
  write.csv(data.frame(beta=b2,fixed_variance=fixed,updated_variance=updated,fixed_j=j1,updated_j=j2),file.path(args[1],paste0(case,'.csv')),row.names=FALSE)
}
