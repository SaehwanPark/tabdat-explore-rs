# Development-only raw MOM pooled QR and independently reshaped year correlations.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
N<-nrow(d);ids<-sort(unique(d$id));G<-length(ids);years<-1979:1988
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%years),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);fit<-lm(lnhr~lnwg,d);stopifnot(fit$rank==2)
b<-unname(coef(fit));V<-unname(vcov(fit));df<-N-2
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
se<-sqrt(V[2,2]);t<-b[2]/se
vals<-c(estimate=b[2],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df),ci_lower=b[2]-qt(.975,df)*se,ci_upper=b[2]+qt(.975,df)*se,ci_level=95,df_residual=df)
for(k in names(vals))emit('LincomResult',paste0('/',k),vals[k])
d$pooled_fitted<-fitted(fit);d$pooled_residual<-resid(fit)
M<-matrix(NA_real_,nrow=G,ncol=10)
for(j in seq_along(years)) {
  z<-d[d$year==years[j],];stopifnot(nrow(z)==G,!anyDuplicated(z$id))
  M[,j]<-z$pooled_residual[match(ids,z$id)]
}
stopifnot(all(is.finite(M)),all(apply(M,2,var)>0))
C<-cor(M);out<-expand.grid(year_a=years,year_b=years);out<-out[order(out$year_a,out$year_b),]
out$people<-G;out$correlation<-mapply(function(x,y) C[x-1978,y-1978],out$year_a,out$year_b)
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','pooled_fitted','pooled_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(out,function(x)sprintf('%.17g',x))),file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'fields',length(rows),'N',N,'G',G,'correlations',nrow(out),'\n')
