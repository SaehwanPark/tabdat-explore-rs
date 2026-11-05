# Development-only QR, full-dummy QR and plm::ranef joint residual reference.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);N<-nrow(d);years<-1979:1988
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%years),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);rows<-list()
emit<-function(kind,path,value)rows[[length(rows)+1L]]<<-data.frame(result_type=kind,path,value=sprintf('%.17g',unname(value)))
# Ordinary pooled QR on the original sample.
pool<-lm(lnhr~lnwg,d);stopifnot(pool$rank==2);b<-unname(coef(pool));V<-unname(vcov(pool));df<-N-2
for(i in 1:2) {
  se<-sqrt(V[i,i]);t<-b[i]/se;vals<-c(value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
  for(k in names(vals))emit('RegressionResult',paste0('/coefficients/',i-1,'/',k),vals[k])
  for(j in 1:2)emit('RegressionResult',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
for(k in c('observation_count','df_residual','r_squared','adjusted_r_squared','root_mse')) {
  value<-switch(k,observation_count=N,df_residual=df,r_squared=summary(pool)$r.squared,adjusted_r_squared=summary(pool)$adj.r.squared,root_mse=sqrt(sum(resid(pool)^2)/df))
  emit('RegressionResult',paste0('/',k),value)
}
d$pooled_xb<-fitted(pool);d$pooled_residual<-resid(pool)
# Full individual dummy QR with observation-weighted zero-mean normalization.
D<-sapply(ids[-G],function(id) as.numeric(d$id==id)-as.numeric(d$id==ids[G]));X<-cbind(1,d$lnwg,D)
fe<-lm.fit(X,d$lnhr);stopifnot(fe$rank==ncol(X));df<-N-fe$rank;rss<-sum(fe$residuals^2)
V<-matrix(0,ncol(X),ncol(X));V[fe$qr$pivot,fe$qr$pivot]<-(rss/df)*chol2inv(qr.R(fe$qr));b<-unname(fe$coefficients[1:2]);V<-V[1:2,1:2]
for(k in c('common_intercept','observation_count','group_count','df_residual','residual_sum_of_squares','root_mse'))
  emit('FixedEffectsResult',paste0('/',k),switch(k,common_intercept=b[1],observation_count=N,group_count=G,df_residual=df,residual_sum_of_squares=rss,root_mse=sqrt(rss/df)))
for(i in 1:2) {
  emit('FixedEffectsResult',paste0('/normalized_level/parameters/',i-1),b[i])
  for(j in 1:2)emit('FixedEffectsResult',paste0('/normalized_level/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
se<-sqrt(V[2,2]);t<-b[2]/se;vals<-c(value=b[2],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
for(k in names(vals))emit('FixedEffectsResult',paste0('/coefficients/0/',k),vals[k])
emit('FixedEffectsResult','/covariance_matrix/0/0',V[2,2])
d$fe_xb<-b[1]+b[2]*d$lnwg;d$fe_xbu<-fe$fitted.values;d$fe_u<-fe$fitted.values-d$fe_xb;d$fe_residual<-fe$residuals
# Balanced Swamy–Arora moments plus independent plm coefficients and ranef.
d$mean_lnhr<-ave(d$lnhr,d$id);d$mean_lnwg<-ave(d$lnwg,d$id);d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
w<-lm.fit(matrix(d$within_lnwg,ncol=1),d$within_lnhr);stopifnot(w$rank==1)
means<-aggregate(cbind(lnhr,lnwg)~id,d,mean);between<-lm.fit(cbind(1,means$lnwg),means$lnhr);stopifnot(between$rank==2)
e<-sum(w$residuals^2)/(N-G-1);rawu<-sum(between$residuals^2)/(G-2)-e/10;u<-max(0,rawu);theta<-1-sqrt(e/(e+10*u));stopifnot(e>0,is.finite(theta),theta>=0,theta<1)
re<-plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar');stopifnot(max(abs(c(e,u)-re$ercomp$sigma2))<=1e-12,abs(theta-re$ercomp$theta)<=1e-12)
b<-unname(coef(re));V<-unname(vcov(re));se<-sqrt(diag(V));z<-b/se
for(i in 1:2) {
  vals<-c(value=b[i],standard_error=se[i],statistic=z[i],p_value=2*pnorm(-abs(z[i])),ci_lower=b[i]-qnorm(.975)*se[i],ci_upper=b[i]+qnorm(.975)*se[i])
  for(k in names(vals))emit('RandomEffectsResult',paste0('/coefficients/',i-1,'/',k),vals[k])
  for(j in 1:2)emit('RandomEffectsResult',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
d$population_xb<-b[1]+b[2]*d$lnwg;d$population_residual<-d$lnhr-d$population_xb;d$mean_population_residual<-ave(d$population_residual,d$id)
d$sigma_e_squared<-e;d$raw_sigma_u_squared<-rawu;d$sigma_u_squared<-u;d$theta<-theta;d$shrink<-1-(1-theta)^2
ran<-plm::ranef(re);d$re_unit_effect<-as.numeric(ran[match(as.character(d$id),names(ran))]);stopifnot(all(is.finite(d$re_unit_effect)),max(abs(d$re_unit_effect-d$shrink*d$mean_population_residual))<1e-9)
d$conditional_xbu<-d$population_xb+d$re_unit_effect;d$re_e<-d$population_residual-d$re_unit_effect
q<-lm.fit(cbind(1,d$lnwg-theta*d$mean_lnwg),d$lnhr-theta*d$mean_lnhr)
vals<-c(observation_count=N,group_count=G,group_size=10,df_residual=N-2,sigma_e_squared=e,sigma_u_squared=u,raw_sigma_u_squared=rawu,theta=theta,rho=u/(e+u),transformed_rss=sum(q$residuals^2),transformed_root_mse=sqrt(sum(q$residuals^2)/(N-2)),wald_statistic=z[2]^2,wald_p_value=pchisq(z[2]^2,1,lower.tail=FALSE),wald_df=1)
for(k in names(vals))emit('RandomEffectsResult',paste0('/',k),vals[k])
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','pooled_xb','pooled_residual','fe_xb','fe_xbu','fe_u','fe_residual','population_xb','population_residual','mean_lnhr','mean_lnwg','mean_population_residual','within_lnhr','within_lnwg','sigma_e_squared','raw_sigma_u_squared','sigma_u_squared','theta','shrink','re_unit_effect','conditional_xbu','re_e')
variables<-c('fe_residual','lnhr','lnwg','pooled_residual','re_e');summaries<-list()
for(scope in c('overall','year')) for(year in if(scope=='overall') NA_integer_ else years) for(name in variables) {
  z<-if(is.na(year)) d[[name]] else d[d$year==year,name]
  summaries[[length(summaries)+1L]]<-data.frame(scope=scope,year=year,variable=name,n=length(z),mean=mean(z),std_dev=sd(z),minimum=min(z),maximum=max(z))
}
out<-do.call(rbind,summaries);for(k in c('n','mean','std_dev','minimum','maximum'))out[[k]]<-sprintf('%.17g',out[[k]])
# Separately model the plausible historical float32 input/prediction storage path.
f32<-function(x)readBin(writeBin(as.double(x),raw(),size=4),what='numeric',n=length(x),size=4)
h<-d;h$lnhr<-f32(d$lnhr);h$lnwg<-f32(d$lnwg)
h$pooled_residual<-f32(resid(lm(lnhr~lnwg,h)))
hfe<-lm.fit(cbind(1,h$lnwg,D),h$lnhr);stopifnot(hfe$rank==ncol(X));h$fe_residual<-f32(hfe$residuals)
hre<-plm::plm(lnhr~lnwg,data=h,index=c('id','year'),model='random',random.method='swar');hr<-plm::ranef(hre)
h$re_e<-f32(h$lnhr-unname(coef(hre)[1])-unname(coef(hre)[2])*h$lnwg-as.numeric(hr[match(as.character(h$id),names(hr))]))
historical<-list()
for(scope in c('overall','year')) for(year in if(scope=='overall') NA_integer_ else years) for(name in variables) {
  z<-if(is.na(year)) h[[name]] else h[h$year==year,name]
  historical[[length(historical)+1L]]<-data.frame(scope=scope,year=year,variable=name,n=length(z),mean=mean(z),std_dev=sd(z),minimum=min(z),maximum=max(z))
}
historical<-do.call(rbind,historical);for(k in c('n','mean','std_dev','minimum','maximum'))historical[[k]]<-sprintf('%.17g',historical[[k]])
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(historical,file.path(a[2],'historical_float32.csv'),sep=',',row.names=FALSE,quote=FALSE,na='')
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(out,file.path(a[2],'summary.csv'),sep=',',row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'plm',as.character(packageVersion('plm')),'fields',length(rows),'N',N,'G',G,'theta',theta,'summary rows',nrow(out),'\n')
