# Development-only plm Swamy-Arora/ranef and person-by-year conditional residual cor.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);N<-nrow(d);years<-1979:1988
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%years),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);d$mean_lnhr<-ave(d$lnhr,d$id);d$mean_lnwg<-ave(d$lnwg,d$id)
d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
w<-lm.fit(matrix(d$within_lnwg,ncol=1),d$within_lnhr);stopifnot(w$rank==1)
means<-aggregate(cbind(lnhr,lnwg)~id,d,mean);between<-lm.fit(cbind(1,means$lnwg),means$lnhr);stopifnot(between$rank==2)
e<-sum(w$residuals^2)/(N-G-1);rawu<-sum(between$residuals^2)/(G-2)-e/10;u<-max(0,rawu)
theta<-1-sqrt(e/(e+10*u));stopifnot(e>0,is.finite(theta),theta>=0,theta<1)
re<-plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
stopifnot(max(abs(c(e,u)-re$ercomp$sigma2))<=1e-12,abs(theta-re$ercomp$theta)<=1e-12)
b<-unname(coef(re));V<-unname(vcov(re));se<-sqrt(diag(V));z<-b/se
rows<-list();emit<-function(path,value)rows[[length(rows)+1L]]<<-data.frame(path,value=sprintf('%.17g',unname(value)))
for(i in 1:2) {
  vals<-c(value=b[i],standard_error=se[i],statistic=z[i],p_value=2*pnorm(-abs(z[i])),ci_lower=b[i]-qnorm(.975)*se[i],ci_upper=b[i]+qnorm(.975)*se[i])
  for(k in names(vals))emit(paste0('/coefficients/',i-1,'/',k),vals[k])
  for(j in 1:2)emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
d$population_xb<-b[1]+b[2]*d$lnwg;d$population_residual<-d$lnhr-d$population_xb
d$mean_population_residual<-ave(d$population_residual,d$id)
d$sigma_e_squared<-e;d$raw_sigma_u_squared<-rawu;d$sigma_u_squared<-u;d$theta<-theta;d$shrink<-1-(1-theta)^2
ran<-plm::ranef(re);d$re_unit_effect<-as.numeric(ran[match(as.character(d$id),names(ran))])
stopifnot(all(is.finite(d$re_unit_effect)),max(abs(d$re_unit_effect-d$shrink*d$mean_population_residual))<1e-9)
d$conditional_xbu<-d$population_xb+d$re_unit_effect;d$re_e<-d$population_residual-d$re_unit_effect
q<-lm.fit(cbind(1,d$lnwg-theta*d$mean_lnwg),d$lnhr-theta*d$mean_lnhr)
vals<-c(observation_count=N,group_count=G,group_size=10,df_residual=N-2,sigma_e_squared=e,sigma_u_squared=u,raw_sigma_u_squared=rawu,theta=theta,rho=u/(e+u),transformed_rss=sum(q$residuals^2),transformed_root_mse=sqrt(sum(q$residuals^2)/(N-2)),wald_statistic=z[2]^2,wald_p_value=pchisq(z[2]^2,1,lower.tail=FALSE),wald_df=1)
for(k in names(vals))emit(paste0('/',k),vals[k])
M<-matrix(NA_real_,nrow=G,ncol=10)
for(j in seq_along(years)) {z<-d[d$year==years[j],];stopifnot(nrow(z)==G,!anyDuplicated(z$id));M[,j]<-z$re_e[match(ids,z$id)]}
stopifnot(all(is.finite(M)),all(apply(M,2,var)>0))
C<-cor(M);out<-expand.grid(year_a=years,year_b=years);out<-out[order(out$year_a,out$year_b),]
out$people<-G;out$correlation<-mapply(function(x,y) C[x-1978,y-1978],out$year_a,out$year_b)
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','population_xb','population_residual','mean_lnhr','mean_lnwg','mean_population_residual','within_lnhr','within_lnwg','sigma_e_squared','raw_sigma_u_squared','sigma_u_squared','theta','shrink','re_unit_effect','conditional_xbu','re_e')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(out,function(x)sprintf('%.17g',x))),file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'plm',as.character(packageVersion('plm')),'fields',length(rows),'N',N,'G',G,'theta',theta,'shrink',1-(1-theta)^2,'\n')
