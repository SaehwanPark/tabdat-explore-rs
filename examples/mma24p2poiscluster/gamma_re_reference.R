# Development-only independent shared-gamma likelihood and full information.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE)
if (!(length(a)%in%c(2,3)) || (length(a)==3 && !(a[3]%in%c('--csv','--dta')))) stop('usage: gamma_re_reference.R source output-directory [--csv | --dta]')
stopifnot(!file.exists(a[2]))
if(length(a)==3 && a[3]=='--dta') {
 d<-foreign::read.dta(a[1],convert.factors=FALSE)
 stopifnot(nrow(d)==27766,identical(which(is.na(d$PHARVIS)),27734L),all(is.na(d[27734,])))
 source<-data.frame(source_row=seq_len(nrow(d)),pharvis=d$PHARVIS,lnhhexp=d$lnhhinc,age=d$AGE,sex=d$SEX,married=d$MARRIED,educ=d$COMPED98,illness=d$ILLDUM,injury=d$INJDUM,illdays=d$ILLDAYS,actdays=d$ACTDAYS,insurance=d$HLTHINS,commune=d$commune)
} else if(length(a)==3) {source<-read.csv(a[1])} else {
 tokens<-scan(a[1],what=character(),quiet=TRUE);stopifnot(length(tokens)==27766*12)
 raw<-matrix(tokens,ncol=12,byrow=TRUE);v<-matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
 stopifnot(identical(which(rowSums(is.na(v))>0),27734L),all(is.na(v[27734,])))
 source<-data.frame(source_row=seq_len(nrow(v)),v)
 names(source)<-c('source_row','pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune')
}
pred<-c('lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ')
d<-source[complete.cases(source[,c('pharvis',pred,'commune')]),]
stopifnot(!anyDuplicated(d$source_row),all(is.finite(as.matrix(d))),all(d$pharvis>=0),all(d$pharvis==floor(d$pharvis)))
X<-cbind('_cons'=1,as.matrix(d[,pred]));y<-d$pharvis;groups<-split(seq_len(nrow(d)),d$commune)
S<-vapply(groups,function(i)sum(y[i]),0.0);K<-ncol(X);G<-length(groups)
stopifnot(G>K+1,all(S<=100000))
scales<-sqrt(colMeans(X^2));Z<-sweep(X,2,scales,'/');pooled<-glm.fit(Z,y,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
start<-c(coef(pooled),0)
evaluate<-function(p){
 r<-exp(-p[K+1]);eta<-as.vector(Z%*%p[1:K]);mu<-exp(eta);score<-rep(0,K+1);info<-matrix(0,K+1,K+1);ll<-0
 for(j in seq_along(groups)) {
  i<-groups[[j]];x<-Z[i,,drop=FALSE];m<-mu[i];M<-sum(m);s<-S[j];q<-(r+s)/(r+M);h<-as.vector(crossprod(x,m))
  ll<-ll+lgamma(r+s)-lgamma(r)+r*log(r)-(r+s)*log(r+M)+sum(y[i]*eta[i]-lgamma(y[i]+1))
  dg<-digamma(r+s)-digamma(r)+log(r)+1-log(r+M)-(r+s)/(r+M)
  dgp<-trigamma(r+s)-trigamma(r)+1/r-1/(r+M)+(s-M)/(r+M)^2
  score[1:K]<-score[1:K]+as.vector(crossprod(x,y[i]-q*m));score[K+1]<-score[K+1]-r*dg
  info[1:K,1:K]<-info[1:K,1:K]+q*crossprod(x,m*x)-q/(r+M)*tcrossprod(h)
  info[1:K,K+1]<-info[1:K,K+1]-r*(M-s)/(r+M)^2*h;info[K+1,1:K]<-info[1:K,K+1]
  info[K+1,K+1]<-info[K+1,K+1]-r*dg-r*r*dgp
 }
 list(ll=ll,score=score,info=info,mu=mu,eta=eta)
}
# Per-group objective/gradient rescaling changes no optimum and prevents the
# unnormalized initial BFGS gradient from overshooting toward zero dispersion.
f<-optim(start,function(p)-evaluate(p)$ll/G,function(p)-evaluate(p)$score/G,method='BFGS',control=list(reltol=1e-13,maxit=2000))
stopifnot(f$convergence==0);p<-f$par
for(it in 1:8){e<-evaluate(p);delta<-solve(e$info,e$score);p<-p+delta;if(max(abs(delta))<1e-12)break}
e<-evaluate(p);stopifnot(max(abs(e$score))<1e-8,min(eigen(e$info,symmetric=TRUE)$values)>0,abs(p[K+1])<12)
v<-solve(e$info)/outer(c(scales,1),c(scales,1));b<-unname(p/c(scales,1))
h<-optimHess(p,function(p)-evaluate(p)$ll,function(p)-evaluate(p)$score,control=list(ndeps=rep(1e-5,K+1)))
stopifnot(max(abs(solve(h)/outer(c(scales,1),c(scales,1))-v))<1e-8)
r<-exp(-p[K+1]);M<-vapply(groups,function(i)sum(e$mu[i]),0.0)
decomp<-sum(vapply(seq_along(groups),function(j){i<-groups[[j]];s<-S[j];dnbinom(s,size=r,mu=M[j],log=TRUE)+lgamma(s+1)-sum(lgamma(y[i]+1))+sum(y[i]*e$eta[i])-s*log(M[j])},0.0))
stopifnot(abs(decomp-e$ll)<1e-8)
se<-unname(sqrt(diag(v)));wald<-as.numeric(crossprod(b[2:K],solve(v[2:K,2:K],b[2:K])))
ll0<-sum(dpois(y,fitted(pooled),log=TRUE));lr<-2*(e$ll-ll0)
values<-c(observation_count=length(y),group_count=G,zero_total_group_count=sum(S==0),singleton_group_count=sum(lengths(groups)==1),minimum_group_size=min(lengths(groups)),maximum_group_size=max(lengths(groups)),maximum_group_count_sum=max(S),zero_count=sum(y==0),count_sum=sum(y),alpha=exp(b[K+1]),alpha_standard_error=exp(b[K+1])*se[K+1],alpha_confidence_lower=exp(b[K+1]-qnorm(.975)*se[K+1]),alpha_confidence_upper=exp(b[K+1]+qnorm(.975)*se[K+1]),log_likelihood=e$ll,poisson_log_likelihood=ll0,dispersion_likelihood_ratio=lr,dispersion_likelihood_ratio_p_value=pchisq(lr,df=1,lower.tail=FALSE)/2,wald_statistic=wald,wald_df=K-1,wald_p_value=pchisq(wald,df=K-1,lower.tail=FALSE))
for(j in seq_len(K+1)) {
 prefix<-if(j<=K)paste0('coefficients/',j-1,'/') else 'ln_alpha/'
 f<-c(value=b[j],standard_error=se[j],statistic=b[j]/se[j],p_value=2*pnorm(-abs(b[j]/se[j])),confidence_lower=b[j]-qnorm(.975)*se[j],confidence_upper=b[j]+qnorm(.975)*se[j])
 names(f)<-paste0(prefix,names(f));values<-c(values,f)
}
for(i in seq_len(K+1))for(j in seq_len(K+1))values<-c(values,setNames(v[i,j],paste0('covariance/',i-1,'/',j-1)))
dir.create(a[2]);write.csv(data.frame(path=names(values),value=unname(values)),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(source,file.path(a[2],'source.csv'),row.names=FALSE,na='')
d$re_mean<-e$mu;d$re_xb<-e$eta;d$re_residual<-y-e$mu
write.csv(d,file.path(a[2],'predictions.csv'),row.names=FALSE,na='')
cat(R.version.string,';',length(y),'rows;',G,'groups;',length(values),'quantities; LL',e$ll,'; max score',max(abs(e$score)),'; decomposition error',decomp-e$ll,'\n')
