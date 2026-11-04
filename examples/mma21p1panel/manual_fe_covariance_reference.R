# Development-only raw means/full QR and independently aggregated scores.
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);N<-nrow(d);K<-2L
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
D<-sapply(ids[-G],function(id)as.numeric(d$id==id)-as.numeric(d$id==ids[G]))
fullX<-cbind(1,d$lnwg,D);canonical<-lm.fit(fullX,d$lnhr);stopifnot(canonical$rank==G+1)
cdf<-N-canonical$rank;Vfull<-matrix(0,G+1,G+1)
Vfull[canonical$qr$pivot,canonical$qr$pivot]<-sum(canonical$residuals^2)/cdf*chol2inv(qr.R(canonical$qr))
rows<-list();emit<-function(model,path,value)rows[[length(rows)+1L]]<<-data.frame(model=model,path=path,value=sprintf('%.17g',value))
b<-unname(canonical$coefficients[2]);se<-sqrt(Vfull[2,2]);t<-b/se
values<-c('/coefficients/0/value'=b,'/coefficients/0/standard_error'=se,
  '/coefficients/0/statistic'=t,'/coefficients/0/p_value'=2*pt(-abs(t),cdf),
  '/observation_count'=N,'/group_count'=G,'/df_residual'=cdf,'/inference_df'=cdf)
for(k in names(values))emit('canonical',k,values[k])
d$source_row<-seq_len(N);d$mean_lnhr<-ave(d$lnhr,d$id,FUN=mean);d$mean_lnwg<-ave(d$lnwg,d$id,FUN=mean)
d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
X<-cbind(1,d$within_lnwg);fit<-lm.fit(X,d$within_lnhr);stopifnot(fit$rank==K)
b<-unname(fit$coefficients);bread<-matrix(0,K,K)
bread[fit$qr$pivot,fit$qr$pivot]<-chol2inv(qr.R(fit$qr))
scores<-X*as.numeric(fit$residuals);unit_scores<-rowsum(scores,d$id,reorder=FALSE)
for(model in c('iid','hc1','cluster')) {
  V<-switch(model,iid=sum(fit$residuals^2)/(N-K)*bread,
    hc1=N/(N-K)*bread%*%crossprod(scores)%*%bread,
    cluster=G/(G-1)*(N-1)/(N-K)*bread%*%crossprod(unit_scores)%*%bread)
  df<-if(model=='cluster')G-1 else N-K
  for(i in 1:K) {
    emit(model,paste0('/fit/coefficients/',i-1,'/value'),b[i])
    emit(model,paste0('/fit/coefficients/',i-1,'/standard_error'),sqrt(V[i,i]))
    for(j in 1:K)emit(model,paste0('/fit/covariance_matrix/',i-1,'/',j-1),V[i,j])
  }
  se<-sqrt(V[2,2]);t<-b[2]/se;p<-2*pt(-abs(t),df)
  emit(model,'/fit/coefficients/1/statistic',t);emit(model,'/fit/coefficients/1/p_value',p)
  emit(model,'/fit/observation_count',N);emit(model,'/fit/df_residual',N-K)
  values<-c(estimate=b[2],standard_error=se,statistic=t,p_value=p,
    ci_lower=b[2]-qt(.975,df)*se,ci_upper=b[2]+qt(.975,df)*se,ci_level=95,df_residual=df)
  for(k in names(values))emit(model,paste0('/lincom/',k),values[k])
  if(model=='cluster')stopifnot(max(abs(V[1,]))<=1e-12)
}
d$aux_fitted<-fit$fitted.values;d$aux_residual<-fit$residuals
cols<-c('source_row','id','year','lnhr','lnwg','kids','ageh','agesq','disab','mean_lnhr','mean_lnwg','within_lnhr','within_lnwg','aux_fitted','aux_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'predictions.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'; full dummy/auxiliary QR; fields',length(rows),'; N',N,'G',G,'; centered cluster intercept t/p deliberately excluded\n')
