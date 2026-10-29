# Development-only R/plm reference for the Chapter 23 log-patents panels.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE);stopifnot(length(args)==2L,!file.exists(args[2]))
src<-matrix(scan(args[1],what=character(),quiet=TRUE),ncol=25,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v<-suppressWarnings(matrix(as.numeric(src),ncol=25));stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g<-nrow(v);stopifnot(g>=2L,g<=346L);n<-g*5L
d<-data.frame(id=rep(seq_len(g),each=5),year=rep(1975:1979,g),cusip=rep(v[,1],each=5),ardssic=rep(v[,2],each=5),
  scisect=rep(v[,3],each=5),logk=rep(v[,4],each=5),sumpat=rep(v[,5],each=5),
  logr=as.vector(t(v[,11:15,drop=FALSE])),pat=as.vector(t(v[,21:25,drop=FALSE])))
stopifnot(all(abs(d$logr)<=20),all(d$pat>=0),all(d$pat<=1000),all(d$pat==floor(d$pat)),!anyDuplicated(d[c('id','year')]))
d$lpat<-log(ifelse(d$pat==0,0.5,d$pat));x<-cbind(1,d$logr);pooled<-lm(lpat~logr,data=d);pcoef<-unname(coef(pooled));pe<-as.numeric(residuals(pooled));bread<-solve(crossprod(x));scores<-rowsum(x*pe,d$id)
pvcov<-g/(g-1)*(n-1)/(n-2)*bread%*%crossprod(scores)%*%bread
within<-plm::plm(lpat~logr,data=d,index=c('id','year'),model='within');wb<-unname(coef(within));we<-as.numeric(residuals(within));wv<-unname(vcov(within));df<-n-g-1
stopifnot(length(wb)==1L,df.residual(within)==df)
re<-plm::plm(lpat~logr,data=d,index=c('id','year'),model='random',random.method='swar');rb<-unname(coef(re));rv<-unname(vcov(re))
between<-lm(lpat~logr,data=aggregate(cbind(lpat,logr)~id,d,mean));ve<-sum(we^2)/df;rawvu<-sum(resid(between)^2)/(g-2)-ve/5;vu<-max(rawvu,0);theta<-1-sqrt(ve/(ve+5*vu))
stopifnot(max(abs(c(ve,vu)-re$ercomp$sigma2))<1e-10,abs(theta-re$ercomp$theta)<1e-10)
Xstar<-cbind(1-theta,d$logr-theta*ave(d$logr,d$id));ystar<-d$lpat-theta*ave(d$lpat,d$id);tr<-lm.fit(Xstar,ystar)
stopifnot(max(abs(tr$coefficients-rb))<1e-10,max(abs(rv-sum(tr$residuals^2)/(n-2)*solve(crossprod(Xstar))))<1e-10)
alpha<-as.numeric(plm::fixef(within,type='level'));units<-as.numeric(names(plm::fixef(within,type='level')));common<-mean(d$lpat)-mean(d$logr)*wb
unit<-match(d$id,units);d$pooled_xb<-pcoef[1]+pcoef[2]*d$logr;d$pooled_residual<-d$lpat-d$pooled_xb
d$fe_xb<-common+wb*d$logr;d$fe_xbu<-alpha[unit]+wb*d$logr;d$fe_u<-alpha[unit]-common;d$fe_residual<-d$lpat-d$fe_xbu
d$re_xb<-rb[1]+rb[2]*d$logr;d$re_residual<-d$lpat-d$re_xb
stopifnot(max(abs(we-d$fe_residual))<1e-10,abs(sum(d$fe_u))<1e-9)
rows<-list();emit<-function(model,path,value)rows[[length(rows)+1L]]<<-data.frame(model,path,value=unname(value))
for(i in 1:2){se<-sqrt(pvcov[i,i]);t<-pcoef[i]/se;for(k in c('value','standard_error','statistic','p_value')) emit('pooled',paste0('/coefficients/',i-1,'/',k),switch(k,value=pcoef[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),g-1)))
 for(j in 1:2)emit('pooled',paste0('/covariance_matrix/',i-1,'/',j-1),pvcov[i,j])}
for(k in c('observation_count','df_residual','root_mse','r_squared','adjusted_r_squared')) emit('pooled',paste0('/',k),switch(k,observation_count=n,df_residual=n-2,root_mse=sqrt(sum(pe^2)/(n-2)),r_squared=summary(pooled)$r.squared,adjusted_r_squared=summary(pooled)$adj.r.squared))
fse<-sqrt(wv[1,1]);ft<-wb/fse;wtotal<-sum((d$lpat-ave(d$lpat,d$id))^2);rss<-sum(we^2)
fields<-c(observation_count=n,group_count=g,absorbed_intercepts=g,df_residual=df,inference_df=df,common_intercept=common,within_r_squared=1-rss/wtotal,conditional_r_squared=1-rss/sum((d$lpat-mean(d$lpat))^2),root_mse=sqrt(rss/df),residual_sum_of_squares=rss,within_total_sum_of_squares=wtotal)
for(k in names(fields))emit('fe',paste0('/',k),fields[k])
for(k in c('value','standard_error','statistic','p_value'))emit('fe',paste0('/coefficients/0/',k),switch(k,value=wb,standard_error=fse,statistic=ft,p_value=2*pt(-abs(ft),df)))
emit('fe','/covariance_matrix/0/0',wv[1,1]);poolplm<-plm::plm(lpat~logr,data=d,index=c('id','year'),model='pooling');f<-plm::pFtest(within,poolplm)
for(k in c('f_statistic','df_num','df_denom','p_value'))emit('fe',paste0('/unit_effects_f_test/',k),switch(k,f_statistic=unname(f$statistic),df_num=unname(f$parameter[1]),df_denom=unname(f$parameter[2]),p_value=unname(f$p.value)))
for(i in 1:2){se<-sqrt(rv[i,i]);z<-rb[i]/se;for(k in c('value','standard_error','statistic','p_value','ci_lower','ci_upper'))emit('re',paste0('/coefficients/',i-1,'/',k),switch(k,value=rb[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),ci_lower=rb[i]-qnorm(.975)*se,ci_upper=rb[i]+qnorm(.975)*se))
 for(j in 1:2)emit('re',paste0('/covariance_matrix/',i-1,'/',j-1),rv[i,j])}
fields<-c(observation_count=n,group_count=g,group_size=5,df_residual=n-2,sigma_e_squared=ve,sigma_u_squared=vu,raw_sigma_u_squared=rawvu,theta=theta,rho=vu/(ve+vu),transformed_root_mse=sqrt(sum(tr$residuals^2)/(n-2)),transformed_rss=sum(tr$residuals^2),wald_statistic=(rb[2]/sqrt(rv[2,2]))^2,wald_p_value=pchisq((rb[2]/sqrt(rv[2,2]))^2,1,lower.tail=FALSE),wald_df=1)
for(k in names(fields))emit('re',paste0('/',k),fields[k])
for(model in c('pooled','fe')){
  estimate<-if(model=='pooled')pcoef[2] else wb;se<-if(model=='pooled')sqrt(pvcov[2,2]) else fse;dof<-if(model=='pooled')g-1 else df;t<-estimate/se
  vals<-c(estimate=estimate,standard_error=se,statistic=t,p_value=2*pt(-abs(t),dof),ci_lower=estimate-qt(.975,dof)*se,ci_upper=estimate+qt(.975,dof)*se,ci_level=95,df_residual=dof)
  for(k in names(vals))emit(paste0(model,'_lincom'),paste0('/',k),vals[k])
  vals<-c(statistic=t*t,p_value=pf(t*t,1,dof,lower.tail=FALSE),df=1,df_residual=dof)
  for(k in names(vals))emit(paste0(model,'_test'),paste0('/',k),vals[k])
}
dir.create(args[2]);write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE);write.csv(d,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',g,'firms;',length(rows),'reference quantities;',n,'source/prediction rows\n')
