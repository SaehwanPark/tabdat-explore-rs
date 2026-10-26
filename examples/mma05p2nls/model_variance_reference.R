# Independent profile-amplitude NLS and direct matrix sandwich, development only.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript model_variance_reference.R original.asc NEW_DIRECTORY')
if (file.exists(args[2])) stop('reference directory already exists')
d <- read.table(args[1],col.names=c('y','x'))
stopifnot(nrow(d)==10000,all(is.finite(d$x)),all(is.finite(d$y)),all(d$y>0))
profile_score <- function(slope) {
  w <- exp(slope*d$x)
  sum(d$x*d$y*w)/sum(d$y*w)-sum(d$x*w^2)/sum(w^2)
}
slope <- uniroot(profile_score,c(0,2),tol=1e-14)$root
w <- exp(slope*d$x); a <- -log(sum(d$y*w)/sum(w^2)); b <- -slope
mu <- exp(-a-b*d$x); X <- cbind(1,d$x); J <- -X*mu
B <- crossprod(J); M <- crossprod(J,J*mu^2); V <- solve(B)%*%M%*%solve(B)
stopifnot(abs(profile_score(slope))<1e-13,all(is.finite(V)),all(eigen(V,symmetric=TRUE)$values>0))
result <- data.frame(n=nrow(d),b00=B[1,1],b01=B[1,2],b11=B[2,2],
  m00=M[1,1],m01=M[1,2],m11=M[2,2],cov_aa=V[1,1],cov_ab=V[1,2],cov_bb=V[2,2],
  se_a=sqrt(V[1,1]),se_b=sqrt(V[2,2]),
  inference='variance-model plug-in, no HC correction; parameter order a,b')
pred <- data.frame(source_row=seq_len(nrow(d)),y=d$y,x=d$x,mean=mu,residual=d$y-mu)
dir.create(args[2]); options(digits=17)
write.csv(result,file.path(args[2],'covariance.csv'),row.names=FALSE,quote=TRUE)
write.csv(pred,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(sprintf('a=%.17g b=%.17g; SE=(%.17g, %.17g)\n',a,b,sqrt(V[1,1]),sqrt(V[2,2])))
