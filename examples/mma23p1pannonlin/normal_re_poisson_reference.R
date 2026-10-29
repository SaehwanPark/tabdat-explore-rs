# Development-only independent base-R adaptive-integration normal-RE reference.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2L,!file.exists(a[2]))
src<-matrix(scan(a[1],what=character(),quiet=TRUE),ncol=25,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v<-suppressWarnings(matrix(as.numeric(src),ncol=25));stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g<-nrow(v);stopifnot(g>=2,g<=346)
X<-v[,11:15,drop=FALSE];Y<-v[,21:25,drop=FALSE]
stopifnot(all(abs(X)<=20),all(Y>=0),all(Y<=1000),all(Y==floor(Y)))
S<-rowSums(Y);YX<-rowSums(Y*X);F<-rowSums(lgamma(Y+1));x<-as.vector(t(X));y<-as.vector(t(Y))
start<-c(unname(coef(glm(y~x,family=poisson()))),0)
# Direct improper integration after posterior-mode centering, not Gauss-Hermite.
evaluate<-function(p,tol=2e-12){
  if(any(!is.finite(p))||max(abs(p[1:2]))>20||abs(p[3])>=8||max(abs(p[1]+p[2]*X))>40)stop('parameter outside independent reference bounds')
  v<-exp(p[3]);means<-exp(p[1]+p[2]*X);M<-rowSums(means);H<-rowSums(means*X)
  rows<-matrix(0,g,4)
  for(i in seq_len(g)){
    logkernel<-function(u) S[i]*u-M[i]*exp(u)-u*u/(2*v)
    mode<-uniroot(function(u)S[i]-M[i]*exp(u)-u/v,c(-100,100),tol=1e-13)$root
    scale<-sqrt(1/(1/v+M[i]*exp(mode)));base<-logkernel(mode)
    integrate_moment<-function(k){
      fn<-switch(k,
        function(z)exp(logkernel(mode+scale*z)-base),
        function(z){u<-mode+scale*z;exp(logkernel(u)-base+u)},
        function(z){u<-mode+scale*z;u*u*exp(logkernel(u)-base)})
      integrate(fn,-Inf,Inf,rel.tol=tol,abs.tol=1e-13,subdivisions=300,stop.on.error=TRUE)$value
    }
    den<-integrate_moment(1);ex<-integrate_moment(2)/den;u2<-integrate_moment(3)/den
    rows[i,]<-c(S[i]*p[1]+YX[i]*p[2]-F[i]+base+log(den*scale)-.5*log(2*pi*v),S[i]-M[i]*ex,YX[i]-H[i]*ex,(u2/v-1)/2)
  }
  list(ll=sum(rows[,1]),score=colSums(rows[,2:4,drop=FALSE]),rows=rows)
}
# Cache function/gradient at the same BFGS parameter vector.
last<-NULL;last_eval<-NULL
cached<-function(p){if(is.null(last)||!identical(last,p)){last<<-p;last_eval<<-tryCatch(evaluate(p),error=function(e)NULL)};last_eval}
objective<-function(p){z<-cached(p);if(is.null(z))1e100 else -z$ll}
gradient<-function(p){z<-cached(p);if(is.null(z))rep(0,3) else -z$score}
o<-optim(start,objective,gradient,method='BFGS',control=list(reltol=1e-13,maxit=500))
stopifnot(o$convergence==0,all(is.finite(o$par)))
p<-o$par
numerical_info<-function(p,eps){
  h<-sapply(seq_along(p),function(j){d<-rep(0,3);d[j]<-eps;(evaluate(p+d)$score-evaluate(p-d)$score)/(2*eps)})
  -(h+t(h))/2
}
for(i in 1:5){e<-evaluate(p);h<-numerical_info(p,1e-4);stopifnot(min(eigen(h,symmetric=TRUE)$values)>0);step<-solve(h,e$score);p<-p+step;if(max(abs(step)/(1+abs(p)))<1e-12)break}
e<-evaluate(p);h<-numerical_info(p,1e-4);h2<-numerical_info(p,5e-5);h3<-numerical_info(p,2.5e-5)
h_richardson<-(4*h2-h)/3;h_check<-(4*h3-h2)/3
stopifnot(max(abs(e$score))<1e-8,min(eigen(h_check,symmetric=TRUE)$values)>0)
vcov<-solve(h_check);stopifnot(max(abs(solve(h_richardson)-vcov))<1e-10);se<-sqrt(diag(vcov));sigma<-exp(p[3]/2);mu<-exp(p[1]+p[2]*x+exp(p[3])/2)
fields<-c(candidate_rows=5*g,firms=g,estimate_intercept=p[1],estimate_slope=p[2],estimate_lnvariance=p[3],
  sigma=sigma,log_likelihood=e$ll,score_intercept=e$score[1],score_slope=e$score[2],score_lnvariance=e$score[3],
  covariance_aa=vcov[1,1],covariance_ab=vcov[1,2],covariance_at=vcov[1,3],covariance_bb=vcov[2,2],covariance_bt=vcov[2,3],covariance_tt=vcov[3,3],
  se_intercept=se[1],se_slope=se[2],se_lnvariance=se[3],mean_mu=mean(mu),min_mu=min(mu),max_mu=max(mu),mean_residual=mean(y-mu))
# Names inherited from glm coefficients are not part of the report contract.
names(fields)<-sub('\\.(intercept|x)$','',names(fields))
dir.create(a[2]);write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=rep(seq_len(g),each=5),year=rep(1975:1979,g),logr=x,pat=y,marginal_mu=mu,marginal_residual=y-mu),file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=seq_len(g),log_likelihood=e$rows[,1],score_a=e$rows[,2],score_b=e$rows[,3],score_t=e$rows[,4]),file.path(a[2],'firms.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',g,'firms; direct adaptive improper integration; max score',max(abs(e$score)),'; Richardson-refined covariance maximum difference',max(abs(solve(h_richardson)-vcov)),'\n')
