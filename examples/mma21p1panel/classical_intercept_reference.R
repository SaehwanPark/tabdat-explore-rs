# Development-only full dummy QR; no native within moments/covariance formula reuse.
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
d<-read.table(a[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]))
ids<-sort(unique(d$id));g<-length(ids);n<-as.numeric(table(factor(d$id,levels=ids)))
stopifnot(g>=3,g<=532,all(n==10),all(d$year%in%1979:1988))
D<-sapply(ids[-g],function(id) as.numeric(d$id==id)-n[match(id,ids)]/n[g]*as.numeric(d$id==ids[g]))
X<-cbind(1,d$lnwg,D);m<-lm.fit(X,d$lnhr);stopifnot(m$rank==ncol(X))
df<-nrow(X)-m$rank;rss<-sum(m$residuals^2)
V<-matrix(0,ncol(X),ncol(X));V[m$qr$pivot,m$qr$pivot]<-(rss/df)*chol2inv(qr.R(m$qr))
b<-unname(m$coefficients[1:2]);V<-V[1:2,1:2];out<-a[2];dir.create(out)
reference<-list()
emit<-function(kind,path,value) {reference[[length(reference)+1L]]<<-data.frame(kind=kind,path=path,value=sprintf('%.17g',value))}
emit('fit','/common_intercept',b[1]);emit('fit','/observation_count',nrow(d));emit('fit','/group_count',g)
emit('fit','/df_residual',df);emit('fit','/residual_sum_of_squares',rss)
for(i in 1:2) {
 emit('fit',paste0('/normalized_level/parameters/',i-1),b[i])
 for(j in 1:2) emit('fit',paste0('/normalized_level/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
for(kind in c('common','mean')) {
 w<-if(kind=='common') c(1,0) else c(1,2.5)
 estimate<-sum(w*b);se<-sqrt(drop(t(w)%*%V%*%w));statistic<-estimate/se;p<-2*pt(-abs(statistic),df)
 values<-c(estimate=estimate,standard_error=se,statistic=statistic,p_value=p,
  ci_lower=estimate-qt(.975,df)*se,ci_upper=estimate+qt(.975,df)*se,ci_level=95,df_residual=df)
 for(k in names(values)) emit(kind,paste0('/',k),values[k])
 if(kind=='common') {
  for(k in names(values)) {
   key<-switch(k,estimate='value',ci_level='confidence_level',df_residual='degrees_of_freedom',k)
   emit('fit',paste0('/common_intercept_statistics/',key),values[k])
  }
 }
}
F<-drop(t(b)%*%solve(V,b))/2
for(k in c('statistic','p_value','df','df_residual')) emit('joint',paste0('/',k),
 switch(k,statistic=F,p_value=pf(F,2,df,lower.tail=FALSE),df=2,df_residual=df))
write.table(do.call(rbind,reference),file.path(out,'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
d$source_row<-seq_len(nrow(d));d$fe_xb<-b[1]+b[2]*d$lnwg;d$fe_xbu<-m$fitted.values
# Raw fitted unit contrasts, not a second within implementation.
d$fe_u<-m$fitted.values-d$fe_xb;d$fe_residual<-m$residuals
text<-as.data.frame(lapply(d,function(x)sprintf('%.17g',x)))
write.table(text,file.path(out,'predictions.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'; weighted-zero-sum full dummy QR;',length(reference),'native reference fields; common SE',sqrt(V[1,1]),'\n')
