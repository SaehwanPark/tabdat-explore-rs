# Development-only independent profile-score and scalar objective bootstrap audit.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L,!file.exists(args[3]))
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
eps <- 128*.Machine$double.eps
profile <- function(w,b) {
  e <- exp(b*x); u <- w*y*e; v <- w*e*e; su <- sum(u); sv <- sum(v)
  stopifnot(is.finite(su),su>0,is.finite(sv),sv>0)
  my <- sum(u*x)/su; mm <- sum(v*x)/sv
  vy <- sum(u*(x-my)^2)/su; vm <- sum(v*(x-mm)^2)/sv
  c(score=my-mm,scale=1+sum(u*abs(x))/su+sum(v*abs(x))/sv,
    curvature=2*vm-vy,curvature_scale=1+2*vm+vy,intercept=log(su/sv))
}
fit <- function(freq) {
  w <- rep(freq,each=5);lo <- profile(w,-2);hi <- profile(w,2)
  stopifnot(lo['score']>eps*lo['scale'],hi['score']< -eps*hi['scale'])
  beta <- uniroot(function(b)unname(profile(w,b)['score']),c(-2,2),tol=1e-14)$root
  p <- profile(w,beta);a <- unname(p['intercept']);eta <- a+beta*x
  stopifnot(abs(p['score'])<=1e-10*p['scale'],p['curvature']>eps*p['curvature_scale'],
    p['curvature']*(1e-9+1e-10*abs(beta))>eps*p['scale'],
    is.finite(a),abs(a)<=20,all(is.finite(eta)),all(abs(eta)<=40))
  objective <- function(b) {e<-exp(b*x);amp<-sum(w*y*e)/sum(w*e*e);sum(w*(y-amp*e)^2)}
  minimum <- optimize(objective,c(-2,2),tol=1e-12)
  # Objective minimization is only a coarse independent cross-check, not the precision oracle.
  rss <- objective(beta); rss_scale <- sum(w*(abs(y)+exp(eta))^2)
  stopifnot(abs(minimum$minimum-beta)<1e-7,
    abs(minimum$objective-rss)<1e-10*(1+rss),rss>eps^2*rss_scale)
  c(a=a,b=beta,rss=rss,curvature=unname(p['curvature']))
}
base <- fit(rep(1,g)); fits <- matrix(NA_real_,500,4)
for(rep in 1:500) fits[rep,] <- fit(tabulate(draws$index0[draws$replicate==rep]+1,g))
colnames(fits)<-c('intercept','slope','rss','curvature')
a<-fits[,1];b<-fits[,2];mu<-exp(base['a']+base['b']*x)
values <- c(candidate_rows=n,firms_per_draw=g,replications=500,
  estimate_intercept=base['a'],estimate_slope=base['b'],
  mean_intercept=mean(a),mean_slope=mean(b),bias_intercept=mean(a)-base['a'],bias_slope=mean(b)-base['b'],
  covariance_aa=var(a),covariance_ab=cov(a,b),covariance_bb=var(b),
  se_intercept=sd(a),se_slope=sd(b),lower_intercept=quantile(a,.025,names=FALSE,type=7),
  upper_intercept=quantile(a,.975,names=FALSE,type=7),lower_slope=quantile(b,.025,names=FALSE,type=7),
  upper_slope=quantile(b,.975,names=FALSE,type=7))
# Drop accidental composite names created by named scalar indexing.
names(values)<-sub('\\.(a|b)$','',names(values))
dir.create(args[3])
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(replicate=1:500,fits),file.path(args[3],'fits.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=id,year=rep(1975:1979,g),logr=sprintf('%.17g',x),pat=y,mu=mu,residual=y-mu),file.path(args[3],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',n,'rows;',g,'firms; 500 profile-score roots with independent objective cross-checks\n')
