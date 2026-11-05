# Development-only original-theta ordinary auxiliary diagnostic, not covariance subtraction.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0)
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
d<-data.frame(source_row=seq_len(nrow(d)),d)
stopifnot(!anyDuplicated(d[c('id','year')]),all(is.finite(d$id)),all(is.finite(d$year)))
d<-d[is.finite(d$lnhr)&is.finite(d$lnwg),];sizes<-table(d$id);G<-length(sizes);N<-nrow(d);T<-unname(sizes[1]);K<-3
stopifnot(G>3,T>=2,all(sizes==T))
re<-plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar');theta<-as.numeric(re$ercomp$theta)
d$mean_lnhr<-ave(d$lnhr,d$id);d$mean_lnwg<-ave(d$lnwg,d$id)
d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
w<-lm.fit(matrix(d$within_lnwg,ncol=1),d$within_lnhr)
between<-lm(lnhr~lnwg,data=aggregate(cbind(lnhr,lnwg)~id,d,mean))
d$theta<-theta;d$quasi_lnhr<-d$lnhr-theta*d$mean_lnhr;d$quasi_lnwg<-d$lnwg-theta*d$mean_lnwg;d<-d[order(d$source_row),]
fit<-lm(quasi_lnhr~quasi_lnwg+within_lnwg,d);b<-unname(coef(fit));V<-unname(vcov(fit));df<-N-K
stopifnot(fit$rank==K,max(abs(c(b[2]-coef(between)[2],b[3]-(w$coefficients[1]-coef(between)[2]))))<=1e-9)
rows<-list();emit<-function(result_type,path,value)rows[[length(rows)+1]]<<-data.frame(result_type,path,value=unname(value))
for(i in 1:K) {
  se<-sqrt(V[i,i]);t<-b[i]/se
  values<-c(value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
  for(k in names(values))emit('RegressionResult',paste0('/coefficients/',i-1,'/',k),values[k])
  for(j in 1:K)emit('RegressionResult',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
values<-c(observation_count=N,df_residual=df,r_squared=summary(fit)$r.squared,adjusted_r_squared=summary(fit)$adj.r.squared,root_mse=sqrt(sum(resid(fit)^2)/df),log_likelihood=as.numeric(logLik(fit)))
for(k in names(values))emit('RegressionResult',paste0('/',k),values[k])
se<-sqrt(V[3,3]);t<-b[3]/se;H<-t*t
values<-c(estimate=b[3],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df),ci_lower=b[3]-qt(.975,df)*se,ci_upper=b[3]+qt(.975,df)*se,ci_level=95,df_residual=df)
for(k in names(values))emit('LincomResult',paste0('/',k),values[k])
values<-c(statistic=H,p_value=pf(H,1,df,lower.tail=FALSE),df=1,df_residual=df)
for(k in names(values))emit('TestResult',paste0('/',k),values[k])
d$iid_hausman_fitted<-fitted(fit);d$iid_hausman_residual<-resid(fit)
dir.create(a[2]);write.csv(do.call(rbind,rows),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(as.data.frame(lapply(d,function(x)sprintf('%.17g',x))),file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; plm',as.character(packageVersion('plm')),'; fields',length(rows),'N',N,'G',G,'theta',theta,'delta',b[3],'H',H,'working F p',pf(H,1,df,lower.tail=FALSE),'asymptotic chi-square p',pchisq(H,1,lower.tail=FALSE),'\n')
