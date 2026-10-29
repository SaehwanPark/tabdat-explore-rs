# Development-only legacy gee fits with explicitly relabeled copied panels.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L,!file.exists(args[3]),requireNamespace('gee',quietly=TRUE),
  packageVersion('gee')==package_version('4.13.30'))
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g <- nrow(v); n <- 5L*g; stopifnot(g>=2L,g<=346L)
x <- as.vector(t(v[,11:15,drop=FALSE])); y <- as.vector(t(v[,21:25,drop=FALSE])); id <- rep(seq_len(g),each=5L)
stopifnot(all(is.finite(x)),all(abs(x)<=20),all(is.finite(y)),all(y>=0),all(y<=1000),all(y==floor(y)))
draws <- read.csv(args[2]);stopifnot(nrow(draws)==500L*g,
  identical(as.integer(draws$replicate),rep(1:500,each=g)),
  identical(as.integer(draws$position),rep(seq_len(g),500)),
  all(draws$step>=draws$position),all(draws$index0>=0),all(draws$index0<g))
start <- unname(coef(glm(y~x,family=poisson())))
# Independent base-R moment and score iteration detects invalid copies before the
# legacy package is invoked (gee 4.13-30 can hang at a singular correlation).
base_eval <- function(beta,indices) {
  x <- v[indices,11:15,drop=FALSE];y <- v[indices,21:25,drop=FALSE]
  eta <- beta[1]+beta[2]*x
  stopifnot(all(is.finite(beta)),all(abs(beta)<=20),all(is.finite(eta)),all(abs(eta)<=40))
  mu<-exp(eta);root<-sqrt(mu);e<-(y-mu)/root;es<-rowSums(e)
  diag<-sum(e*e);scale<-sum(((abs(y)+mu)/root)^2)
  lower<-sum(es*es);upper<-5*sum((e-rowMeans(e))^2);eps<-128*.Machine$double.eps
  stopifnot(is.finite(diag),is.finite(scale),diag>eps^2*scale,
    lower>eps*4*diag,upper>eps*4*diag)
  rho<-if(lower>=diag) 1-upper/(4*diag) else -.25+lower/(4*diag)
  k<-1/(1-rho);c<-rho/(1+4*rho);h0<-rowSums(root);h1<-rowSums(root*x)
  d0<-rowSums(y-mu);d1<-rowSums(x*(y-mu))
  b00<-k*sum(rowSums(mu)-c*h0*h0);b01<-k*sum(rowSums(mu*x)-c*h0*h1)
  b11<-k*sum(rowSums(mu*x*x)-c*h1*h1);schur<-b11-b01*b01/b00
  stopifnot(is.finite(b00),b00>0,is.finite(b01),is.finite(b11),
    is.finite(schur),schur>eps*(1+abs(b11)+abs(b01*b01/b00)))
  u<-k*c(sum(d0-c*h0*es),sum(d1-c*h1*es))
  uscale<-k*c(sum(abs(d0)+abs(c*h0*es)),sum(abs(d1)+abs(c*h1*es)))
  delta<-c(u[1]/b00-(b01/b00)*(u[2]-(b01/b00)*u[1])/schur,(u[2]-(b01/b00)*u[1])/schur)
  stopifnot(all(is.finite(u)),all(is.finite(uscale)),all(is.finite(delta)))
  w00<-1/b00+(b01/b00)^2/schur;w01<-( -b01/b00)/schur;w11<-1/schur
  sensitivity<-eps*c(abs(w00)*uscale[1]+abs(w01)*uscale[2],abs(w01)*uscale[1]+abs(w11)*uscale[2])
  list(delta=delta,rho=rho,score=u,scale=uscale,sensitivity=sensitivity)
}
base_fit <- function(indices,replicate) {
  beta<-start
  for(i in 1:100) {
    e<-tryCatch(base_eval(beta,indices),error=function(err)stop(paste('invalid GEE iterate in replicate',replicate,conditionMessage(err))))
    step<-max(abs(e$delta)/(1+abs(beta))); beta<-beta+e$delta
    if(step<=1e-12) {
      final<-base_eval(beta,indices)
      stopifnot(all(abs(final$score)<=1e-10*(1+final$scale)),
        all(final$sensitivity<1e-9+1e-10*abs(beta)))
      return(c(intercept=beta[1],slope=beta[2],rho=final$rho,iterations=i))
    }
  }
  stop(paste('GEE did not converge in replicate',replicate))
}
fit <- function(indices,replicate) {
  independent <- base_fit(indices,replicate)
  # Every draw position receives a distinct ID, even if the source ID repeats.
  panel_x <- as.vector(t(v[indices,11:15,drop=FALSE]));panel_y <- as.vector(t(v[indices,21:25,drop=FALSE]))
  data <- data.frame(y=panel_y,x=panel_x,id=rep(seq_len(g),each=5L))
  capture.output(model <- gee::gee(y~x,id=id,data=data,family=poisson(),corstr='exchangeable',
    scale.fix=TRUE,scale.value=1,v4.4compat=TRUE,b=start,tol=1e-12,maxiter=100,silent=TRUE))
  if(model$error!=0 || model$iterations>=100L) stop(paste('GEE failed replicate',replicate,'error',model$error,'iterations',model$iterations))
  beta <- unname(coef(model)); eta <- beta[1]+beta[2]*panel_x;mu <- exp(eta)
  e <- (panel_y-mu)/sqrt(mu);E <- matrix(e,nrow=g,byrow=TRUE)
  rho <- sum(rowSums(E)^2-rowSums(E*E))/(4*sum(E*E))
  stopifnot(all(is.finite(beta)),all(abs(beta)<=20),all(is.finite(eta)),all(abs(eta)<=40),
    is.finite(rho),rho>-.25,rho<1,abs(rho-model$working.correlation[1,2])<1e-10,
    max(abs(beta-independent[1:2]))<1e-9,abs(rho-independent['rho'])<1e-9)
  c(intercept=beta[1],slope=beta[2],rho=rho,iterations=model$iterations)
}
base <- fit(seq_len(g),0);fits <- matrix(NA_real_,500,4)
for(rep in 1:500) fits[rep,] <- fit(draws$index0[draws$replicate==rep]+1,rep)
colnames(fits)<-c('intercept','slope','rho','iterations')
a<-fits[,1];b<-fits[,2];mu<-exp(base['intercept']+base['slope']*x)
values <- c(candidate_rows=n,firms_per_draw=g,replications=500,
  estimate_intercept=base['intercept'],estimate_slope=base['slope'],estimate_rho=base['rho'],
  original_mu_mean=mean(mu),original_mu_min=min(mu),original_mu_max=max(mu),original_residual_mean=mean(y-mu),
  mean_intercept=mean(a),mean_slope=mean(b),bias_intercept=mean(a)-base['intercept'],bias_slope=mean(b)-base['slope'],
  covariance_aa=var(a),covariance_ab=cov(a,b),covariance_bb=var(b),
  se_intercept=sd(a),se_slope=sd(b),lower_intercept=quantile(a,.025,names=FALSE,type=7),
  upper_intercept=quantile(a,.975,names=FALSE,type=7),lower_slope=quantile(b,.025,names=FALSE,type=7),
  upper_slope=quantile(b,.975,names=FALSE,type=7))
names(values)<-sub('\\.(intercept|slope|rho)$','',names(values))
dir.create(args[3])
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(replicate=1:500,fits),file.path(args[3],'fits.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=id,year=rep(1975:1979,g),logr=sprintf('%.17g',x),pat=y,gee_mu=mu,gee_residual=y-mu),file.path(args[3],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; gee',as.character(packageVersion('gee')),';',n,'rows;',g,'firms; 500 distinct-copy GEE refits\n')
