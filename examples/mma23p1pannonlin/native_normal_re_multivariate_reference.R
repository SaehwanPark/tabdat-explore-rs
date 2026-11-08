# Development-only independent multivariate/unbalanced normal-Poisson reference.
# Existing synthetic source only; direct improper integration, never native GH.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE);stopifnot(length(args)==2L,!file.exists(args[2]))
tokens<-scan(args[1],what=character(),quiet=TRUE);stopifnot(length(tokens)==40L*25L)
src<-matrix(tokens,ncol=25L,byrow=TRUE)
records<-lapply(1:40,function(id){
  time<-seq_len(1L+id%%5L)
  x<-as.numeric(src[id,10L+time]);y<-as.numeric(src[id,20L+time])
  z<-sin(id*.37)+(time-3)^2/4+.2*x*x
  data.frame(source_row=(id-1L)*5L+time-1L,id=id,year=1974L+time,y=y,x=x,z=z)
})
d<-do.call(rbind,records);X<-cbind(intercept=1,x=d$x,z=d$z);y<-d$y
stopifnot(nrow(d)==120L,all(is.finite(X)),all(y>=0),all(y==floor(y)))
groups<-split(seq_len(nrow(d)),d$id);k<-ncol(X);q<-k+1L
start<-c(unname(coef(glm(y~x+z,family=poisson(),data=d))),0)
evaluate<-function(p,tol=2e-12){
  eta<-as.vector(X%*%p[seq_len(k)]);v<-exp(p[q])
  if(any(!is.finite(p))||abs(p[q])>=8||any(abs(eta)>40))stop('parameter outside reference bounds')
  means<-exp(eta);ll<-0;score<-rep(0,q);info<-matrix(0,q,q)
  for(rows in groups){
    s<-sum(y[rows]);m<-sum(means[rows]);xx<-X[rows,,drop=FALSE]
    h<-colSums(means[rows]*xx);j<-crossprod(xx,means[rows]*xx)
    logkernel<-function(u)s*u-m*exp(u)-u*u/(2*v)
    mode<-uniroot(function(u)s-m*exp(u)-u/v,c(-100,100),tol=1e-13)$root
    scale<-1/sqrt(1/v+m*exp(mode));anchor<-logkernel(mode)
    integral<-function(moment){
      f<-function(z){
        u<-mode+scale*z;lw<-logkernel(u)-anchor;out<-numeric(length(u));ok<-is.finite(lw)&lw> -700&u<300
        out[ok]<-exp(lw[ok])*moment(u[ok]);out
      }
      integrate(f,-Inf,Inf,rel.tol=tol,abs.tol=1e-13,subdivisions=500L,stop.on.error=TRUE)$value
    }
    mass<-integral(function(u)rep(1,length(u)))
    ex<-integral(exp)/mass;u2<-integral(function(u)u*u)/mass
    vex<-integral(function(u)(exp(u)-ex)^2)/mass
    vu2<-integral(function(u)(u*u-u2)^2)/mass
    cross<-integral(function(u)(exp(u)-ex)*(u*u-u2))/mass
    ll<-ll+sum(y[rows]*eta[rows]-lgamma(y[rows]+1))+anchor+log(mass*scale)-log(2*pi*v)/2
    # Complete-data derivative expectations and centered score covariances,
    # not the native conditional-design/Stein curvature decomposition.
    score[seq_len(k)]<-score[seq_len(k)]+colSums(y[rows]*xx)-h*ex
    score[q]<-score[q]+(u2/v-1)/2
    info[seq_len(k),seq_len(k)]<-info[seq_len(k),seq_len(k)]+j*ex-tcrossprod(h)*vex
    info[seq_len(k),q]<-info[seq_len(k),q]+h*cross/(2*v)
    info[q,q]<-info[q,q]+u2/(2*v)-vu2/(4*v*v)
  }
  info[q,seq_len(k)]<-info[seq_len(k),q]
  list(ll=ll,score=score,information=info)
}
last<-NULL;last_value<-NULL
cached<-function(p){if(is.null(last)||!identical(last,p)){last<<-p;last_value<<-tryCatch(evaluate(p),error=function(e)NULL)};last_value}
objective<-function(p){e<-cached(p);if(is.null(e))1e100 else -e$ll}
gradient<-function(p){e<-cached(p);if(is.null(e))rep(0,q) else -e$score}
o<-optim(start,objective,gradient,method='BFGS',control=list(reltol=1e-13,maxit=1000L));stopifnot(o$convergence==0)
p<-o$par
for(i in 1:8){e<-evaluate(p);stopifnot(min(eigen(e$information,symmetric=TRUE)$values)>0);step<-solve(e$information,e$score);p<-p+step;if(max(abs(step)/(1+abs(p)))<1e-13)break}
e<-evaluate(p);stopifnot(max(abs(e$score))<1e-8,min(eigen(e$information,symmetric=TRUE)$values)>0)
numerical<-function(h){
  matrix<-sapply(seq_len(q),function(j){delta<-rep(0,q);delta[j]<-h;-(evaluate(p+delta)$score-evaluate(p-delta)$score)/(2*h)})
  (matrix+t(matrix))/2
}
V<-solve(e$information);h1<-numerical(1e-4);h2<-numerical(5e-5);h3<-numerical(2.5e-5)
stopifnot(max(abs(V-solve((4*h2-h1)/3)))<1e-10,max(abs(V-solve((4*h3-h2)/3)))<1e-10)
se<-sqrt(diag(V));z<-p/se;critical<-qnorm(.975);sigma<-exp(p[q]/2)
mu<-exp(as.vector(X%*%p[seq_len(k)])+exp(p[q])/2)
pooled<-sum(dpois(y,exp(as.vector(X%*%start[seq_len(k)])),log=TRUE));lr<-2*(e$ll-pooled)
wald<-as.numeric(t(p[2:k])%*%solve(V[2:k,2:k],p[2:k]))
fields<-c(observations=nrow(d),groups=length(groups),zero_groups=sum(vapply(groups,function(rows)sum(y[rows])==0,logical(1))),singletons=sum(lengths(groups)==1L),
 log_likelihood=e$ll,poisson_log_likelihood=pooled,lr_variance_zero=lr,p_lr_variance_zero=pchisq(lr,1,lower.tail=FALSE)/2,
 sigma=sigma,se_sigma=sigma*se[q]/2,lower_sigma=exp((p[q]-critical*se[q])/2),upper_sigma=exp((p[q]+critical*se[q])/2),
 wald=wald,wald_df=k-1L,wald_p=pchisq(wald,k-1L,lower.tail=FALSE),mean_mu=mean(mu),min_mu=min(mu),max_mu=max(mu),mean_residual=mean(y-mu))
labels<-c('intercept','x','z','lnvariance')
for(i in seq_len(q)){
 for(prefix in c('estimate','se','z','p','lower','upper','score')){
  value<-switch(prefix,estimate=p[i],se=se[i],z=z[i],p=2*pnorm(abs(z[i]),lower.tail=FALSE),lower=p[i]-critical*se[i],upper=p[i]+critical*se[i],score=e$score[i])
  fields[paste(prefix,labels[i],sep='_')]<-value
 }
 for(j in i:q)fields[paste('covariance',labels[i],labels[j],sep='_')]<-V[i,j]
}
# Preserve the exact fitted design in the native input, including constructed z.
# write.csv otherwise prints numeric data at fewer than17 significant digits.
exact_csv<-function(data,path){
  encoded<-as.data.frame(lapply(data,function(column)if(is.numeric(column))sprintf('%.17g',column) else column),check.names=FALSE)
  write.csv(encoded,path,row.names=FALSE,quote=FALSE)
}
dir.create(args[2]);exact_csv(d,file.path(args[2],'input.csv'))
write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
exact_csv(data.frame(source_row=d$source_row,id=d$id,year=d$year,y=y,x=d$x,z=d$z,marginal_mu=mu,marginal_residual=y-mu),file.path(args[2],'predictions.csv'))
cat(R.version.string,';120 rows/40 groups/two slopes/unbalanced/singletons; direct integration; covariance Richardson difference',max(abs(V-solve((4*h2-h1)/3))),'\n')
