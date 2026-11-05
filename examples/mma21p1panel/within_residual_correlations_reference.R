# Independent full-dummy zero-sum QR and person-by-year residual correlations.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
d<-read.table(a[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]))
ids<-sort(unique(d$id));g<-length(ids);n<-as.numeric(table(factor(d$id,levels=ids)));years<-1979:1988
stopifnot(g>=3,g<=532,all(n==10),all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),
  all(d$year%in%years),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
D<-sapply(ids[-g],function(id) as.numeric(d$id==id)-n[match(id,ids)]/n[g]*as.numeric(d$id==ids[g]))
X<-cbind(1,d$lnwg,D);fit<-lm.fit(X,d$lnhr);stopifnot(fit$rank==ncol(X))
N<-nrow(X);df<-N-fit$rank;rss<-sum(fit$residuals^2)
V<-matrix(0,ncol(X),ncol(X));V[fit$qr$pivot,fit$qr$pivot]<-(rss/df)*chol2inv(qr.R(fit$qr))
b<-unname(fit$coefficients[1:2]);V<-V[1:2,1:2];rows<-list()
emit<-function(kind,path,value)rows[[length(rows)+1L]]<<-data.frame(result_type=kind,path,value=sprintf('%.17g',unname(value)))
for(k in c('common_intercept','observation_count','group_count','df_residual','residual_sum_of_squares','root_mse'))
  emit('FixedEffectsResult',paste0('/',k),switch(k,common_intercept=b[1],observation_count=N,group_count=g,df_residual=df,residual_sum_of_squares=rss,root_mse=sqrt(rss/df)))
for(i in 1:2) {
  emit('FixedEffectsResult',paste0('/normalized_level/parameters/',i-1),b[i])
  for(j in 1:2)emit('FixedEffectsResult',paste0('/normalized_level/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
se<-sqrt(V[2,2]);t<-b[2]/se
vals<-c(value=b[2],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
for(k in names(vals))emit('FixedEffectsResult',paste0('/coefficients/0/',k),vals[k])
emit('FixedEffectsResult','/covariance_matrix/0/0',V[2,2])
vals<-c(estimate=b[2],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df),ci_lower=b[2]-qt(.975,df)*se,ci_upper=b[2]+qt(.975,df)*se,ci_level=95,df_residual=df)
for(k in names(vals))emit('LincomResult',paste0('/',k),vals[k])
d$source_row<-seq_len(N);d$fe_xb<-b[1]+b[2]*d$lnwg;d$fe_xbu<-fit$fitted.values
d$fe_u<-fit$fitted.values-d$fe_xb;d$fe_residual<-fit$residuals
M<-matrix(NA_real_,nrow=g,ncol=10)
for(j in seq_along(years)) {
  z<-d[d$year==years[j],];stopifnot(nrow(z)==g,!anyDuplicated(z$id))
  M[,j]<-z$fe_residual[match(ids,z$id)]
}
stopifnot(all(is.finite(M)),all(apply(M,2,var)>0),max(abs(rowSums(M)))<1e-9)
C<-cor(M);out<-expand.grid(year_a=years,year_b=years);out<-out[order(out$year_a,out$year_b),]
out$people<-g;out$correlation<-mapply(function(x,y) C[x-1978,y-1978],out$year_a,out$year_b)
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','fe_xb','fe_xbu','fe_u','fe_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(out,function(x)sprintf('%.17g',x))),file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'full-dummy QR fields',length(rows),'N',N,'G',g,'correlations',nrow(out),'\n')
