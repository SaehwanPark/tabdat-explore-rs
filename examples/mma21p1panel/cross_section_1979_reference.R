# Development-only raw MOM, 1979 independent QR fit and float32 historical control.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(nrow(d));p<-d[d$year==1979,];stopifnot(nrow(p)==G,!anyDuplicated(p$id));N<-nrow(p)
fit<-lm(lnhr~lnwg,p);stopifnot(fit$rank==2);b<-unname(coef(fit));V<-unname(vcov(fit));df<-N-2
rows<-list();emit<-function(kind,path,value)rows[[length(rows)+1]]<<-data.frame(result_type=kind,path,value=sprintf('%.17g',unname(value)))
for(i in 1:2) {
  se<-sqrt(V[i,i]);t<-b[i]/se;vals<-c(value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
  for(k in names(vals))emit('RegressionResult',paste0('/coefficients/',i-1,'/',k),vals[k])
  for(j in 1:2)emit('RegressionResult',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
for(k in c('observation_count','df_residual','r_squared','adjusted_r_squared','root_mse')) {
  value<-switch(k,observation_count=N,df_residual=df,r_squared=summary(fit)$r.squared,adjusted_r_squared=summary(fit)$adj.r.squared,root_mse=sqrt(sum(resid(fit)^2)/df))
  emit('RegressionResult',paste0('/',k),value)
}
se<-sqrt(V[2,2]);t<-b[2]/se;vals<-c(estimate=b[2],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df),ci_lower=b[2]-qt(.975,df)*se,ci_upper=b[2]+qt(.975,df)*se,ci_level=95,df_residual=df)
for(k in names(vals))emit('LincomResult',paste0('/',k),vals[k])
for(k in c('df','df_residual','statistic','p_value'))emit('TestResult',paste0('/',k),switch(k,df=1,df_residual=df,statistic=t*t,p_value=pf(t*t,1,df,lower.tail=FALSE)))
# A float32 source-storage hypothesis is separate from the canonical raw-double fit.
f32<-function(x)readBin(writeBin(as.double(x),raw(),size=4),what='numeric',n=length(x),size=4)
h<-data.frame(lnhr=f32(p$lnhr),lnwg=f32(p$lnwg));fh<-lm(lnhr~lnwg,h);hc<-summary(fh)$coefficients;hi<-confint(fh)
historical<-data.frame(name=rep(c('intercept','lnwg'),each=6),field=rep(c('value','standard_error','statistic','p_value','ci_lower','ci_upper'),2),value=sprintf('%.17g',c(hc[1,],hi[1,],hc[2,],hi[2,])))
p$cross_fitted<-fitted(fit);p$cross_residual<-resid(fit);cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','cross_fitted','cross_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(historical,file.path(a[2],'historical_float32.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(p[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'fields',length(rows),'N',N,'slope',sprintf('%.17g',b[2]),'\n')
