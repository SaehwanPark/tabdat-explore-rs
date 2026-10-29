# Development-only independent R shared-gamma whole-firm bootstrap reference.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE)
stopifnot(length(args)%in%c(3L,4L),!file.exists(args[3]))
merged<-length(args)==4L
if(merged)stopifnot(args[4]=='merged_source_ids')
tokens<-scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0,length(tokens)%%25L==0)
src<-matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v<-suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g<-nrow(v);stopifnot(g>=2L,g<=346L)
X<-v[,11:15,drop=FALSE];Y<-v[,21:25,drop=FALSE]
stopifnot(all(abs(X)<=20),all(Y>=0),all(Y<=1000),all(Y==floor(Y)))
S<-rowSums(Y);x<-as.vector(t(X));y<-as.vector(t(Y))
start<-c(unname(coef(glm(y~x,family=poisson()))),0)
draws<-read.csv(args[2]);stopifnot(nrow(draws)==500L*g,
  identical(as.integer(draws$replicate),rep(1:500,each=g)),
  identical(as.integer(draws$position),rep(seq_len(g),500)),
  all(draws$step>=draws$position),all(draws$index0>=0),all(draws$index0<g))
indices<-matrix(draws$index0+1L,nrow=500L,byrow=TRUE)
# By default frequencies count distinct five-row likelihood contributions.
# The opt-in counterfactual instead merges repeated IDs into longer panels;
# it demonstrates sensitivity, not what a historical Stata version did.
evaluate<-function(p,w){
  copies<-if(merged)w else rep(1,g);w<-if(merged)as.numeric(w>0) else w;total<-S*copies
  r<-exp(-p[3]);eta<-p[1]+p[2]*X;mu<-exp(eta);M<-copies*rowSums(mu);q<-(r+total)/(r+M);h<-copies*rowSums(mu*X)
  ll<-sum(w*(lgamma(r+total)-lgamma(r)+r*log(r)-(r+total)*log(r+M)+copies*rowSums(Y*eta-lgamma(Y+1))))
  dg<-digamma(r+total)-digamma(r)+log(r)+1-log(r+M)-(r+total)/(r+M)
  score<-c(sum(w*(total-q*M)),sum(w*(copies*rowSums(Y*X)-q*h)),sum(w*(-r*dg)))
  info<-matrix(0,3,3)
  info[1,1]<-sum(w*(q*M-q*M*M/(r+M)));info[1,2]<-info[2,1]<-sum(w*(q*h-q*M*h/(r+M)))
  info[2,2]<-sum(w*(q*copies*rowSums(mu*X*X)-q*h*h/(r+M)))
  info[1,3]<-info[3,1]<- -sum(w*r*(M-total)*M/(r+M)^2)
  info[2,3]<-info[3,2]<- -sum(w*r*(M-total)*h/(r+M)^2)
  dgp<-trigamma(r+total)-trigamma(r)+1/r-1/(r+M)+(total-M)/(r+M)^2
  info[3,3]<- -sum(w*(r*dg+r*r*dgp))
  list(ll=ll,score=score,information=info)
}
fit<-function(w,replicate){
  opt<-optim(start,function(p)-evaluate(p,w)$ll,function(p)-evaluate(p,w)$score,
    method='BFGS',control=list(reltol=1e-14,maxit=2000))
  if(opt$convergence!=0 || any(!is.finite(opt$par)))stop(paste('optimizer failed replicate',replicate))
  p<-opt$par
  for(i in 1:8){
    e<-evaluate(p,w)
    if(any(!is.finite(e$information)) || min(eigen(e$information,symmetric=TRUE)$values)<=0)stop(paste('invalid information replicate',replicate))
    delta<-solve(e$information,e$score);p<-p+delta
    if(max(abs(delta)/(1+abs(p)))<1e-13)break
  }
  e<-evaluate(p,w)
  if(any(!is.finite(p)) || abs(p[1])>20 || abs(p[2])>20 || abs(p[3])>=8 ||
    max(abs(p[1]+p[2]*X[w>0,,drop=FALSE]))>40 || max(abs(e$score))>1e-8)stop(paste('invalid or nonconverged replicate',replicate))
  p
}
original<-fit(rep(1,g),0)
fits<-t(vapply(1:500,function(i)fit(tabulate(indices[i,],nbins=g),i),numeric(3)))
colnames(fits)<-c('intercept','slope','lnalpha')
V<-cov(fits);se<-sqrt(diag(V));means<-colMeans(fits);lower<-apply(fits,2,quantile,probs=.025,names=FALSE,type=7);upper<-apply(fits,2,quantile,probs=.975,names=FALSE,type=7)
mu<-exp(original[1]+original[2]*x)
fields<-c(candidate_rows=5*g,firms_per_draw=g,replications=500,
  estimate_intercept=original[1],estimate_slope=original[2],estimate_lnalpha=original[3],
  original_mu_mean=mean(mu),original_mu_min=min(mu),original_mu_max=max(mu),original_residual_mean=mean(y-mu),
  mean_intercept=means[1],mean_slope=means[2],mean_lnalpha=means[3],
  bias_intercept=means[1]-original[1],bias_slope=means[2]-original[2],bias_lnalpha=means[3]-original[3],
  covariance_aa=V[1,1],covariance_ab=V[1,2],covariance_at=V[1,3],covariance_bb=V[2,2],covariance_bt=V[2,3],covariance_tt=V[3,3],
  se_intercept=se[1],se_slope=se[2],se_lnalpha=se[3],
  lower_intercept=lower[1],upper_intercept=upper[1],lower_slope=lower[2],upper_slope=upper[2],lower_lnalpha=lower[3],upper_lnalpha=upper[3])
names(fields)<-sub('\\.(intercept|slope|lnalpha)$','',names(fields))
dir.create(args[3])
write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(replicate=1:500,fits),file.path(args[3],'fits.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=rep(seq_len(g),each=5L),year=rep(1975:1979,g),logr=x,pat=y,marginal_mu=mu,marginal_residual=y-mu),file.path(args[3],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',g,'firms; 500 independent three-parameter shared-gamma refits;',
  if(merged)'merged-source-ID counterfactual' else 'distinct firm copies','\n')
