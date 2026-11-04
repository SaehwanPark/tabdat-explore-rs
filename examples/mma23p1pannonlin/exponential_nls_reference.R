# Independent development-only original-source exponential-mean NLS reference.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2])); dir.create(args[2])
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g <- nrow(v); n <- 5L*g
x <- as.vector(t(v[,11:15,drop=FALSE])); y <- as.vector(t(v[,21:25,drop=FALSE]))
id <- rep(seq_len(g),each=5L); year <- rep(1975:1979,g)
stopifnot(g>=2L,g<=346L,all(is.finite(x)),all(abs(x)<=20),
  all(is.finite(y)),all(y>=0),all(y<=1000),all(y==floor(y)))
profile_score <- function(b) {
  z <- exp(b*x); amplitude <- sum(y*z)/sum(z*z)
  sum((y-amplitude*z)*amplitude*z*x)
}
stopifnot(profile_score(-2)>0,profile_score(2)<0)
b <- uniroot(profile_score,c(-2,2),tol=1e-14)$root
z <- exp(b*x); a <- log(sum(y*z)/sum(z*z)); mu <- exp(a+b*x); residual <- y-mu
# Secondary approximate algorithm only; the profile root above is the precision oracle.
d <- data.frame(y=y,x=x)
fit <- nls(y~exp(a+b*x),data=d,start=list(a=2.5,b=.5),
  control=nls.control(maxiter=300,tol=1e-7,warnOnly=TRUE))
stopifnot(fit$convInfo$isConv,max(abs(coef(fit)-c(a,b)))<5e-7)
j <- cbind(mu,mu*x); bread <- solve(crossprod(j)); scores <- rowsum(j*residual,id)
rss <- sum(residual^2); working <- rss/n*bread
cluster <- g/(g-1)*bread%*%crossprod(scores)%*%bread
native <- rss/(n-2)*bread
stopifnot(all(is.finite(working)),all(is.finite(cluster)),
  all(diag(working)>0),all(diag(cluster)>0))
values <- c(observations=n,firms=g,intercept=a,slope=b,rss=rss,
  working_aa=working[1,1],working_ab=working[1,2],working_bb=working[2,2],
  cluster_aa=cluster[1,1],cluster_ab=cluster[1,2],cluster_bb=cluster[2,2],
  working_se_intercept=sqrt(working[1,1]),working_se_slope=sqrt(working[2,2]),
  cluster_se_intercept=sqrt(cluster[1,1]),cluster_se_slope=sqrt(cluster[2,2]),
  working_z_intercept=a/sqrt(working[1,1]),working_z_slope=b/sqrt(working[2,2]),
  cluster_z_intercept=a/sqrt(cluster[1,1]),cluster_z_slope=b/sqrt(cluster[2,2]),
  working_lo_intercept=a-qnorm(.975)*sqrt(working[1,1]),
  working_hi_intercept=a+qnorm(.975)*sqrt(working[1,1]),
  working_lo_slope=b-qnorm(.975)*sqrt(working[2,2]),
  working_hi_slope=b+qnorm(.975)*sqrt(working[2,2]),
  cluster_lo_intercept=a-qnorm(.975)*sqrt(cluster[1,1]),
  cluster_hi_intercept=a+qnorm(.975)*sqrt(cluster[1,1]),
  cluster_lo_slope=b-qnorm(.975)*sqrt(cluster[2,2]),
  cluster_hi_slope=b+qnorm(.975)*sqrt(cluster[2,2]),
  native_se_intercept=sqrt(native[1,1]),native_se_slope=sqrt(native[2,2]),
  score_scale_a=sum(abs(mu*residual)),score_scale_b=sum(abs(mu*x*residual)))
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=id,year=year,logr=x,pat=y,nls_mu=mu,nls_residual=residual),file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=seq_len(g),score_a=scores[,1],score_b=scores[,2]),file.path(args[2],'scores.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',n,'source rows;',g,'firm scores; independent profile + approximate nls cross-check\n')
