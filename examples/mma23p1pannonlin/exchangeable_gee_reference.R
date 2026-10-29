# Development-only independent gee package and base-R GEE/covariance audit.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]),requireNamespace('gee',quietly=TRUE))
dir.create(args[2])
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
d <- data.frame(id=id,x=x,y=y)
start <- unname(coef(glm(y~x,data=d,family=poisson())))
# The legacy moment convention is explicit: gee's modern default is different.
fit <- gee::gee(y~x,id=id,data=d,family=poisson(),corstr='exchangeable',
  scale.fix=TRUE,scale.value=1,v4.4compat=TRUE,b=start,tol=1e-12,maxiter=100,silent=TRUE)
stopifnot(fit$error==0,fit$iterations<100L)
beta <- unname(coef(fit)); eta <- beta[1]+beta[2]*x; mu <- exp(eta); residual <- y-mu
pearson <- residual/sqrt(mu); E <- matrix(pearson,nrow=g,byrow=TRUE)
rho <- sum(rowSums(E)^2-rowSums(E*E))/(4*sum(E*E))
stopifnot(is.finite(rho),rho>-.25,rho<1,abs(rho-fit$working.correlation[1,2])<1e-10)
# Independently evaluate the full matrix GEE bread and group scores at package beta.
J <- cbind(sqrt(mu),sqrt(mu)*x); unit_j <- rowsum(J,id); unit_e <- rowsum(pearson,id)
unit_je <- rowsum(J*pearson,id); k <- 1/(1-rho); c <- rho/(1+4*rho)
scores <- k*(unit_je-c*unit_j*drop(unit_e))
bread <- k*(crossprod(J)-c*crossprod(unit_j)); working <- solve(bread)
cluster <- g/(g-1)*working%*%crossprod(scores)%*%working
stopifnot(max(abs(working-fit$naive.variance))<1e-9,
  max(abs(cluster-fit$robust.variance*g/(g-1)))<1e-9,
  max(abs(colSums(scores)))<1e-5)
a <- beta[1]; b <- beta[2]; sw <- sqrt(diag(working)); sc <- sqrt(diag(cluster)); z <- qnorm(.975)
values <- c(observations=n,firms=g,intercept=a,slope=b,rho=rho,
  working_aa=working[1,1],working_ab=working[1,2],working_bb=working[2,2],
  cluster_aa=cluster[1,1],cluster_ab=cluster[1,2],cluster_bb=cluster[2,2],
  working_se_intercept=sw[1],working_se_slope=sw[2],cluster_se_intercept=sc[1],cluster_se_slope=sc[2],
  working_z_intercept=a/sw[1],working_z_slope=b/sw[2],cluster_z_intercept=a/sc[1],cluster_z_slope=b/sc[2],
  working_lo_intercept=a-z*sw[1],working_hi_intercept=a+z*sw[1],
  working_lo_slope=b-z*sw[2],working_hi_slope=b+z*sw[2],
  cluster_lo_intercept=a-z*sc[1],cluster_hi_intercept=a+z*sc[1],
  cluster_lo_slope=b-z*sc[2],cluster_hi_slope=b+z*sc[2])
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=id,year=year,logr=x,pat=y,gee_mu=mu,gee_residual=residual),file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=seq_len(g),score_a=scores[,1],score_b=scores[,2]),file.path(args[2],'scores.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; gee',as.character(packageVersion('gee')),';',n,'source rows;',g,'firm scores; fixed scale1 legacy-moment exchangeable GEE\n')
