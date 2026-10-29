# Development-only R reference: shared gamma panel likelihood and derivatives.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]))
tokens<-scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0,length(tokens)%%25L==0)
src<-matrix(tokens,ncol=25,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2))
v<-suppressWarnings(matrix(as.numeric(src),ncol=25))
stopifnot(!anyNA(v[,-2]),all(is.finite(v[,-2])))
g<-nrow(v);stopifnot(g>=2,g<=346)
X<-v[,11:15,drop=FALSE];Y<-v[,21:25,drop=FALSE]
stopifnot(all(abs(X)<=20),all(Y>=0),all(Y<=1000),all(Y==floor(Y)))
S<-rowSums(Y);x<-as.vector(t(X));y<-as.vector(t(Y))
start<-c(unname(coef(glm(y~x,family=poisson()))),0)
evaluate<-function(p){
  r<-exp(-p[3]);eta<-p[1]+p[2]*X;mu<-exp(eta);M<-rowSums(mu);q<-(r+S)/(r+M)
  h<-rowSums(mu*X)
  ll<-sum(lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)+rowSums(Y*eta-lgamma(Y+1)))
  dg<-digamma(r+S)-digamma(r)+log(r)+1-log(r+M)-(r+S)/(r+M)
  score<-c(sum(S-q*M),sum(rowSums(Y*X)-q*h),sum(-r*dg))
  info<-matrix(0,3,3)
  info[1,1]<-sum(q*M-q*M*M/(r+M))
  info[1,2]<-info[2,1]<-sum(q*h-q*M*h/(r+M))
  info[2,2]<-sum(q*rowSums(mu*X*X)-q*h*h/(r+M))
  info[1,3]<-info[3,1]<- -sum(r*(M-S)*M/(r+M)^2)
  info[2,3]<-info[3,2]<- -sum(r*(M-S)*h/(r+M)^2)
  dgp<-trigamma(r+S)-trigamma(r)+1/r-1/(r+M)+(S-M)/(r+M)^2
  info[3,3]<- -sum(r*dg+r*r*dgp)
  list(ll=ll,score=score,information=info,mu=mu)
}
fit<-optim(start,function(p)-evaluate(p)$ll,function(p)-evaluate(p)$score,
  method='BFGS',control=list(reltol=1e-14,maxit=2000))
stopifnot(fit$convergence==0,all(is.finite(fit$par)))
p<-fit$par
# Refine the BFGS solution using observed information; do not accept its
# objective-only stopping rule as a fully converged score.
for(i in 1:5){e<-evaluate(p);stopifnot(min(eigen(e$information,symmetric=TRUE)$values)>0)
  delta<-solve(e$information,e$score);p<-p+delta;if(max(abs(delta)/(1+abs(p)))<1e-13)break}
e<-evaluate(p);stopifnot(max(abs(e$score))<1e-8)
cov<-solve(e$information);se<-sqrt(diag(cov))
# A central numerical derivative of the independent analytic score is a
# separate check of all Hessian entries and covariance, not the source of it.
h<-optimHess(p,function(p)-evaluate(p)$ll,function(p)-evaluate(p)$score,control=list(ndeps=rep(1e-5,3)))
stopifnot(max(abs(solve(h)-cov))<1e-8)
r<-exp(-p[3]);M<-rowSums(e$mu)
ll_decomposed<-sum(dnbinom(S,size=r,mu=M,log=TRUE)+lgamma(S+1)-rowSums(lgamma(Y+1))+
  rowSums(Y*(p[1]+p[2]*X))-S*log(M))
stopifnot(abs(ll_decomposed-e$ll)<1e-8)
pooled<-glm(y~x,family=poisson());ll0<-as.numeric(logLik(pooled));lr<-2*(e$ll-ll0)
values<-c(candidate_rows=length(y),firms=g,zero_total_firms=sum(S==0),
  estimate_intercept=p[1],estimate_slope=p[2],estimate_lnalpha=p[3],alpha=exp(p[3]),
  se_intercept=se[1],se_slope=se[2],se_lnalpha=se[3],se_alpha=exp(p[3])*se[3],
  lower_alpha=exp(p[3]-qnorm(.975)*se[3]),upper_alpha=exp(p[3]+qnorm(.975)*se[3]),
  z_intercept=p[1]/se[1],z_slope=p[2]/se[2],z_lnalpha=p[3]/se[3],
  p_intercept=2*pnorm(-abs(p[1]/se[1])),p_slope=2*pnorm(-abs(p[2]/se[2])),p_lnalpha=2*pnorm(-abs(p[3]/se[3])),
  lower_intercept=p[1]-qnorm(.975)*se[1],upper_intercept=p[1]+qnorm(.975)*se[1],
  lower_slope=p[2]-qnorm(.975)*se[2],upper_slope=p[2]+qnorm(.975)*se[2],
  lower_lnalpha=p[3]-qnorm(.975)*se[3],upper_lnalpha=p[3]+qnorm(.975)*se[3],
  covariance_aa=cov[1,1],covariance_ab=cov[1,2],covariance_at=cov[1,3],
  covariance_bb=cov[2,2],covariance_bt=cov[2,3],covariance_tt=cov[3,3],
  log_likelihood=e$ll,poisson_log_likelihood=ll0,lr_alpha_zero=lr,p_lr_alpha_zero=pchisq(lr,df=1,lower.tail=FALSE)/2,
  mean_prediction=mean(e$mu),mean_residual=mean(Y-e$mu))
dir.create(args[2])
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=rep(seq_len(g),each=5),year=rep(1975:1979,g),logr=x,pat=y,
  marginal_mu=as.vector(t(e$mu)),marginal_residual=y-as.vector(t(e$mu))),file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',g,'firms;',length(y),'rows; logLik',e$ll,'; max score',max(abs(e$score)),
  '; decomposition difference',abs(ll_decomposed-e$ll),'\n')
