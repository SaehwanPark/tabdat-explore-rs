# Development-only plm baseline, independent raw means/QR and person scores.
options(digits=17)
library(plm)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
G<-length(unique(d$id));N<-nrow(d);K<-2L
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);d$mean_lnhr<-ave(d$lnhr,d$id,FUN=mean);d$mean_lnwg<-ave(d$lnwg,d$id,FUN=mean)
d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
w<-lm.fit(matrix(d$within_lnwg,ncol=1),d$within_lnhr);stopifnot(w$rank==1)
means<-aggregate(cbind(lnhr,lnwg)~id,d,mean);between<-lm.fit(cbind(1,means$lnwg),means$lnhr);stopifnot(between$rank==2)
e<-sum(w$residuals^2)/(N-G-1);rawu<-sum(between$residuals^2)/(G-2)-e/10;u<-max(0,rawu)
theta<-1-sqrt(e/(e+10*u));stopifnot(e>0,is.finite(theta),theta>=0,theta<1)
re<-plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
stopifnot(max(abs(c(e,u)-re$ercomp$sigma2))<=1e-12,abs(theta-re$ercomp$theta)<=1e-12)
b<-unname(coef(re));V<-unname(vcov(re));se<-sqrt(diag(V));z<-b/se
rows<-list();emit<-function(model,path,value)rows[[length(rows)+1L]]<<-data.frame(model,path,value=sprintf('%.17g',unname(value)))
for(i in 1:K) {
  values<-c(value=b[i],standard_error=se[i],statistic=z[i],p_value=2*pnorm(-abs(z[i])),ci_lower=b[i]-qnorm(.975)*se[i],ci_upper=b[i]+qnorm(.975)*se[i])
  for(k in names(values))emit('canonical',paste0('/coefficients/',i-1,'/',k),values[k])
  for(j in 1:K)emit('canonical',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
d$population_xb<-b[1]+b[2]*d$lnwg;d$population_residual<-d$lnhr-d$population_xb
d$sigma_e_squared<-e;d$raw_sigma_u_squared<-rawu;d$sigma_u_squared<-u;d$theta<-theta
d$quasi_lnhr<-d$lnhr-theta*d$mean_lnhr;d$quasi_lnwg<-d$lnwg-theta*d$mean_lnwg
X<-cbind(1,d$quasi_lnwg);fit<-lm.fit(X,d$quasi_lnhr);stopifnot(fit$rank==K)
values<-c(observation_count=N,group_count=G,group_size=10,df_residual=N-K,
  sigma_e_squared=e,sigma_u_squared=u,raw_sigma_u_squared=rawu,theta=theta,rho=u/(e+u),
  transformed_rss=sum(fit$residuals^2),transformed_root_mse=sqrt(sum(fit$residuals^2)/(N-K)),
  wald_statistic=z[2]^2,wald_p_value=pchisq(z[2]^2,1,lower.tail=FALSE),wald_df=1)
for(k in names(values))emit('canonical',paste0('/',k),values[k])
b<-unname(fit$coefficients);bread<-matrix(0,K,K);bread[fit$qr$pivot,fit$qr$pivot]<-chol2inv(qr.R(fit$qr))
stopifnot(max(abs(b-c((1-theta)*coef(re)[1],coef(re)[2])))<=1e-9)
scores<-X*as.numeric(fit$residuals);unit_scores<-rowsum(scores,d$id,reorder=FALSE)
for(model in c('iid','hc1','cluster')) {
  V<-switch(model,iid=sum(fit$residuals^2)/(N-K)*bread,
    hc1=N/(N-K)*bread%*%crossprod(scores)%*%bread,
    cluster=G/(G-1)*(N-1)/(N-K)*bread%*%crossprod(unit_scores)%*%bread)
  df<-if(model=='cluster')G-1 else N-K
  for(i in 1:K) {
    se<-sqrt(V[i,i]);t<-b[i]/se;p<-2*pt(-abs(t),df);stopifnot(se>0,is.finite(t))
    values<-c(value=b[i],standard_error=se,statistic=t,p_value=p)
    for(k in names(values))emit(model,paste0('/fit/coefficients/',i-1,'/',k),values[k])
    for(j in 1:K)emit(model,paste0('/fit/covariance_matrix/',i-1,'/',j-1),V[i,j])
    values<-c(estimate=b[i],standard_error=se,statistic=t,p_value=p,
      ci_lower=b[i]-qt(.975,df)*se,ci_upper=b[i]+qt(.975,df)*se,ci_level=95,df_residual=df)
    for(k in names(values))emit(model,paste0('/lincom',i-1,'/',k),values[k])
  }
  emit(model,'/fit/observation_count',N);emit(model,'/fit/df_residual',N-K)
}
d$quasi_fitted<-fit$fitted.values;d$quasi_residual<-fit$residuals
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','population_xb','population_residual','mean_lnhr','mean_lnwg','within_lnhr','within_lnwg','sigma_e_squared','raw_sigma_u_squared','sigma_u_squared','theta','quasi_lnhr','quasi_lnwg','quasi_fitted','quasi_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'predictions.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'; plm',as.character(packageVersion('plm')),'; fields',length(rows),'; N',N,'G',G,'raw_u',rawu,'theta',theta,'\n')
