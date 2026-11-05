# Development-only raw MOM, explicit annual pairs, QR OLS and independent correlations.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);years<-1980:1988
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(nrow(d));d<-d[order(d$id,d$year),];p<-d[d$year>1979,];z<-d[d$year<1988,]
stopifnot(all(p$id==z$id),all(p$year==z$year+1));p$previous_source_row<-z$source_row;p$previous_year<-z$year;p$previous_lnhr<-z$lnhr;p$previous_lnwg<-z$lnwg
p$dlnhr<-p$lnhr-p$previous_lnhr;p$dlnwg<-p$lnwg-p$previous_lnwg;N<-nrow(p);stopifnot(N==9*G)
fit<-lm(dlnhr~dlnwg,p);stopifnot(fit$rank==2);b<-unname(coef(fit));V<-unname(vcov(fit));df<-N-2
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
# Historical .dta precision is unknown; check the plausible float32 storage path
# separately from the canonical raw-double/R fit rather than widening tolerance.
f32<-function(x)readBin(writeBin(as.double(x),raw(),size=4),what='numeric',n=length(x),size=4)
h<-data.frame(dlnhr=f32(f32(p$lnhr)-f32(p$previous_lnhr)),dlnwg=f32(f32(p$lnwg)-f32(p$previous_lnwg)))
fh<-lm(dlnhr~dlnwg,h);hc<-summary(fh)$coefficients;hi<-confint(fh)
historical<-data.frame(name=rep(c('intercept','dlnwg'),each=6),field=rep(c('value','standard_error','statistic','p_value','ci_lower','ci_upper'),2),value=sprintf('%.17g',c(hc[1,],hi[1,],hc[2,],hi[2,])))
p$fd_fitted<-fitted(fit);p$fd_residual<-resid(fit);output<-list()
for(name in c('dlnhr','dlnwg','fd_residual')) {
  M<-matrix(NA_real_,nrow=G,ncol=9)
  for(j in seq_along(years)) {z<-p[p$year==years[j],];stopifnot(nrow(z)==G,!anyDuplicated(z$id));M[,j]<-z[[name]][match(ids,z$id)]}
  stopifnot(all(is.finite(M)),all(apply(M,2,var)>0));C<-cor(M)
  out<-expand.grid(year_a=years,year_b=years);out<-out[order(out$year_a,out$year_b),];out$variable<-name;out$people<-G
  out$correlation<-mapply(function(x,y) C[x-1979,y-1979],out$year_a,out$year_b);output[[name]]<-out[c('variable','year_a','year_b','people','correlation')]
}
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','previous_source_row','previous_year','previous_lnhr','previous_lnwg','dlnhr','dlnwg','fd_fitted','fd_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(historical,file.path(a[2],'historical_float32.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(p[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
out<-do.call(rbind,output);out[-1]<-lapply(out[-1],function(x)sprintf('%.17g',x));write.table(out,file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'fields',length(rows),'N',N,'G',G,'correlations',nrow(out),'\n')
