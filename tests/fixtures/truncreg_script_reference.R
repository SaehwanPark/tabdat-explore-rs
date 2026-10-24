# Development-only actual-script fixture, NOT redistributed author data.
# Translate certified stats case1 by +0.6, mapping its lower-0.6 model to lower0.
# Verify the translated fit/full observed covariance against pinned truncreg;
# only normal95 intercept inference changes. No new production R dependency.
# R_LIBS_USER=... Rscript this.R truncreg_0.2-5.tar.gz
suppressPackageStartupMessages(library(truncreg))
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==1,unname(tools::md5sum(a[1]))=='6ddc73494a349da69b23f859ffe89a54',packageDescription('truncreg')$Version=='0.2-5',packageDescription('maxLik')$Version=='1.5-2.2')
tmp<-tempfile();dir.create(tmp);untar(a[1],files='truncreg/R/truncreg.R',exdir=tmp);source<-new.env(parent=baseenv());sys.source(file.path(tmp,'truncreg/R/truncreg.R'),envir=source)
for(name in c('truncreg','truncreg.fit','ml.truncreg'))stopifnot(isTRUE(all.equal(body(get(name,asNamespace('truncreg'))),body(get(name,source)),check.attributes=FALSE)))
unlink(tmp,recursive=TRUE)
path<-dirname(sub('^--file=','',grep('^--file=',commandArgs(),value=TRUE)))
stats<-file.path(path,'../../crates/tabdat-stats/tests/fixtures')
d<-read.csv(file.path(stats,'truncated_gaussian_data.csv'));d<-d[d$case==1,];r<-read.csv(file.path(stats,'truncated_gaussian_reference.csv'));r<-r[r$case==1,];r$case<-NULL
get<-function(q,i=0,j=0){v<-r$value[r$quantity==q&r$i==i&r$j==j];stopifnot(length(v)==1);unname(v)}
set<-function(q,v,i=0,j=0){match<-r$quantity==q&r$i==i&r$j==j;stopifnot(sum(match)==1);r$value[match]<<-unname(v)}
input<-data.frame(source_row=1:242,e=0,lnwage=c(d$x,-.3,.3),ystar=c(d$y+.6,0,0),ytrunc=c(d$y+.6,NA,NA),ycens=c(d$y+.6,0,0),dy=c(rep(1,240),0,0))
set('lower_limit',0);set('coefficient_value',get('coefficient_value',1)+.6,1);set('coefficient_confidence_lower',get('coefficient_confidence_lower',1)+.6,1);set('coefficient_confidence_upper',get('coefficient_confidence_upper',1)+.6,1)
set('coefficient_statistic',get('coefficient_value',1)/get('coefficient_standard_error',1),1);set('coefficient_p_value',2*pnorm(abs(get('coefficient_statistic',1)),lower.tail=FALSE),1)
beta<-c(get('coefficient_value',1),get('coefficient_value',2));sigma<-get('sigma');v<-matrix(NA_real_,3,3);for(i in 1:3)for(j in 1:3)v[i,j]<-get('joint_covariance',i,j)
f<-truncreg(ytrunc~lnwage,data=input,point=0,direction='left',scaled=FALSE,method='NR',iterlim=1000)
cat('Default package endpoint maximum coefficient/scale distance from certified affine point:',max(abs(unname(coef(f))-c(beta,sigma))),'\n')
# Default package stopping can precede our coefficient gate. Independently
# refine its actual density with observed symbolic Newton; do NOT widen gates.
part<-deriv3(~-eta-.5*((y-b0-b1*x)/exp(eta))^2-.5*log(2*pi)-log(pnorm((b0+b1*x)/exp(eta))),c('b0','b1','eta'),function.arg=c('b0','b1','eta','x','y'))
selected<-input[!is.na(input$ytrunc),]
evaluate<-function(theta){a<-part(theta[1],theta[2],theta[3],selected$lnwage,selected$ytrunc);list(ll=sum(a),score=colSums(attr(a,'gradient')),information=-apply(attr(a,'hessian'),c(2,3),sum))}
refine<-function(theta,free){
 for(iteration in 1:20){
  e<-evaluate(theta);information<-e$information[free,free,drop=FALSE];invisible(chol(information));step<-solve(information,e$score[free]);score<-max(abs(e$score[free])*pmax(1,abs(theta[free])))/240;relative_step<-max(abs(step)/pmax(1,abs(theta[free])))
  if(score<1e-11&&relative_step<1e-10)return(list(theta=theta,e=e,covariance=solve(information),score=score,step=relative_step))
  next_theta<-theta;next_theta[free]<-theta[free]+step;stopifnot(evaluate(next_theta)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)));theta<-next_theta
 };stop('root-script observed symbolic certification failed')
}
full<-refine(c(unname(coef(f)[1:2]),log(unname(coef(f)[3]))),1:3)
f0<-truncreg(ytrunc~1,data=input,point=0,direction='left',scaled=FALSE,method='NR',iterlim=1000)
null<-refine(c(unname(coef(f0)[1]),0,log(unname(coef(f0)[2]))),c(1,3))
stopifnot(max(abs(c(full$theta[1:2],exp(full$theta[3]))-c(beta,sigma)))<1e-8,abs(full$e$ll-get('log_likelihood'))<1e-8,abs(null$e$ll-get('null_log_likelihood'))<1e-8,max(abs(full$covariance-v))<1e-8)
stopifnot(abs(as.numeric(logLik(f))-full$e$ll)<1e-8)
jacobian<-diag(c(1,1,1/unname(coef(f)[3])));stopifnot(max(abs(jacobian%*%vcov(f)%*%jacobian-v)/(1+abs(v)))<1e-7)
set('scaled_score',full$score);set('scaled_step',full$step);set('null_scaled_score',null$score);set('null_scaled_step',null$step)
mu<-beta[1]+beta[2]*input$lnwage;t<-mu/sigma;p<-pnorm(t);m<-dnorm(t)/p
prediction<-cbind(input,latent=mu,retention_probability=p,conditional_mean=mu+sigma*m)
write.csv(input,file.path(path,'truncreg_script_data.csv'),row.names=FALSE,na='',quote=FALSE)
write.csv(prediction,file.path(path,'truncreg_script_predictions.csv'),row.names=FALSE,na='',quote=FALSE)
write.csv(r,file.path(path,'truncreg_script_reference.csv'),row.names=FALSE,quote=FALSE)
cat('R',as.character(getRversion()),'truncreg',packageDescription('truncreg')$Version,'certified affine control verified:240retained of242,49 quantities,9 joint covariance entries,726 current-X projections\n')
