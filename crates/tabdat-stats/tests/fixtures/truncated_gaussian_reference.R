# Development-only R4.6.1/truncreg0.2-5/maxLik1.5-2.2 controls.
# Usage: R_LIBS_USER=... Rscript this.R pinned_truncreg_0.2-5.tar.gz
suppressPackageStartupMessages(library(truncreg))
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==1,unname(tools::md5sum(a[1]))=='6ddc73494a349da69b23f859ffe89a54',packageDescription('truncreg')$Version=='0.2-5',packageDescription('maxLik')$Version=='1.5-2.2')
tmp<-tempfile();dir.create(tmp);untar(a[1],files='truncreg/R/truncreg.R',exdir=tmp);source<-new.env(parent=baseenv());sys.source(file.path(tmp,'truncreg/R/truncreg.R'),envir=source)
for(name in c('truncreg','truncreg.fit','ml.truncreg'))stopifnot(isTRUE(all.equal(body(get(name,asNamespace('truncreg'))),body(get(name,source)),check.attributes=FALSE)))
unlink(tmp,recursive=TRUE)
likelihood<-deriv3(~-eta-.5*((y-b0*x0-b1*x1)/exp(eta))^2-.5*log(2*pi)-log(pnorm((b0*x0+b1*x1-lower)/exp(eta))),c('b0','b1','eta'),function.arg=c('b0','b1','eta','x0','x1','y','lower'))
quantities<-list();inputs<-list();predictions<-list()
for(case in 1:3) {
 n<-240;i<-1:n;x<-qnorm((i-.5)/n);z<-cos(i*sqrt(2));u<-((i*73)%%241+.5)/241;constant<-case!=2
 lower<-if(case==1)-.6 else if(case==2).8 else -1e100
 mu<-if(constant).4+.7*x else 1.3*x-.4*z;sigma<-.9
 if(case==2)sigma<-.7
 y<-mu+sigma*qnorm((1-u)*pnorm((mu-lower)/sigma),lower.tail=FALSE)
 d<-data.frame(y=y,x=x,z=z);design<-if(constant)cbind(1,x)else cbind(x,z)
 stopifnot(all(is.finite(y)),all(y>lower))
 f<-truncreg(if(constant)y~x else y~x+z-1,data=d,point=lower,direction='left',scaled=FALSE,method='NR',iterlim=1000)
 evaluate<-function(theta) {
  part<-likelihood(theta[1],theta[2],theta[3],design[,1],design[,2],y,lower)
  list(ll=sum(part),score=colSums(attr(part,'gradient')),information=-apply(attr(part,'hessian'),c(2,3),sum))
 }
 refine<-function(theta,free) {
  for(iteration in 1:20) {
   e<-evaluate(theta);information<-e$information[free,free,drop=FALSE];invisible(chol(information));step<-solve(information,e$score[free])
   score_bound<-max(abs(e$score[free])*pmax(1,abs(theta[free])))/n;step_bound<-max(abs(step)/pmax(1,abs(theta[free])))
   if(score_bound<1e-11&&step_bound<1e-10)return(list(theta=theta,e=e,covariance=solve(information),score_bound=score_bound,step_bound=step_bound))
   candidate<-theta;candidate[free]<-theta[free]+step;stopifnot(evaluate(candidate)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)));theta<-candidate
  };stop('symbolic observed-Newton certification failed')
 }
 fit<-refine(c(unname(coef(f)[1:2]),log(unname(coef(f)[3]))),1:3)
 transform<-diag(c(1,1,1/unname(coef(f)[3])))
 stopifnot(abs(fit$e$ll-as.numeric(logLik(f)))<1e-8,max(abs(fit$covariance-transform%*%vcov(f)%*%transform)/(1+abs(fit$covariance)))<1e-7)
 if(constant) {
  f0<-truncreg(y~1,data=d,point=lower,direction='left',scaled=FALSE,method='NR',iterlim=1000)
  null<-refine(c(unname(coef(f0)[1]),0,log(unname(coef(f0)[2]))),c(1,3));stopifnot(abs(null$e$ll-as.numeric(logLik(f0)))<1e-8)
 } else {
  # The package's 1:ncol(X) does not support zero columns. Independently solve
  # the actual fixed-zero-location one-scale null, never insert a fake intercept.
  start<-optimize(function(eta)-evaluate(c(0,0,eta))$ll,c(-8,8),tol=1e-12)$minimum
  null<-refine(c(0,0,start),3)
 }
 b<-fit$theta[1:2];eta<-fit$theta[3];s<-exp(eta);v<-fit$covariance;latent<-drop(design%*%b);t<- (latent-lower)/s;p<-pnorm(t);m<-exp(dnorm(t,log=TRUE)-pnorm(t,log.p=TRUE));delta<-m*(m+t);h<-1-delta;hp<-delta*(2*m+t)-m
 mean_h<-mean(h);mean_gradient<-c(colMeans(hp*design/s),mean(-t*hp));stopifnot(all(is.finite(mean_gradient)),all(h>0))
 emit<-function(quantity,value,i=0,j=0)quantities[[length(quantities)+1L]]<<-data.frame(case=case,quantity=quantity,i=i,j=j,value=as.numeric(value))
 inference<-function(prefix,value,se,i) {
  stat<-value/se;p<-2*pnorm(abs(stat),lower.tail=FALSE);stopifnot(is.finite(se),se>0,p>0)
  fields<-c(value=value,standard_error=se,statistic=stat,p_value=p,confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  for(name in names(fields))emit(paste0(prefix,'_',name),fields[name],i)
 }
 for(j in 1:2)inference('coefficient',b[j],sqrt(v[j,j]),j)
 for(j in 1:3)for(k in 1:3)emit('joint_covariance',v[j,k],j,k)
 for(j in 1:3)emit('mean_conditional_derivative_gradient',mean_gradient[j],j)
 for(j in if(constant)2 else 1:2) {
  gradient<-b[j]*mean_gradient;gradient[j]<-gradient[j]+mean_h;variance<-drop(t(gradient)%*%v%*%gradient);stopifnot(variance>0)
  inference('ame',b[j]*mean_h,sqrt(variance),if(constant)1 else j)
 }
 lr<-2*(fit$e$ll-null$e$ll);df<-if(constant)1 else 2
 summaries<-c(n=n,intercept=as.numeric(constant),lower_limit=lower,log_likelihood=fit$e$ll,null_log_likelihood=null$e$ll,likelihood_ratio=lr,likelihood_ratio_df=df,likelihood_ratio_p_value=pchisq(lr,df,lower.tail=FALSE),sigma=s,log_sigma=eta,log_sigma_standard_error=sqrt(v[3,3]),sigma_standard_error=s*sqrt(v[3,3]),sigma_confidence_lower=exp(eta-qnorm(.975)*sqrt(v[3,3])),sigma_confidence_upper=exp(eta+qnorm(.975)*sqrt(v[3,3])),mean_conditional_derivative=mean_h,scaled_score=fit$score_bound,scaled_step=fit$step_bound,null_scaled_score=null$score_bound,null_scaled_step=null$step_bound)
 for(name in names(summaries))emit(name,summaries[name])
 inputs[[case]]<-data.frame(case=case,row=i,y=y,x=x,z=z)
 predictions[[case]]<-data.frame(case=case,row=i,latent=latent,retention_probability=p,conditional_mean=latent+s*m)
}
path<-dirname(sub('^--file=','',grep('^--file=',commandArgs(),value=TRUE)))
write.csv(do.call(rbind,inputs),file.path(path,'truncated_gaussian_data.csv'),row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,predictions),file.path(path,'truncated_gaussian_predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,quantities),file.path(path,'truncated_gaussian_reference.csv'),row.names=FALSE,quote=FALSE)
cat('R',as.character(getRversion()),'truncreg',packageDescription('truncreg')$Version,'maxLik',packageDescription('maxLik')$Version,'three240-row controls, observed symbolic certificates and full joint inference\n')
