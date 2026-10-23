# R4.6.1 / quantreg6.1; independent br LP and common external row draws.
# Run: Rscript .../reference.R fixture-directory
options(digits=17)
a<-commandArgs(TRUE); d<-read.csv(file.path(a[1],'inputs.csv'))
draws<-read.csv(file.path(a[1],'draws.csv'));rows<-list()
emit<-function(case,path,value) rows[[length(rows)+1]]<<-data.frame(case,path,value=unname(value))
for(case in c('joint','noconstant','shifted','rescaled')) {
 z<-d;if(case=='shifted')z$y<-z$y+100
 if(case=='rescaled')z$x<-z$x*100
 f<-if(case=='noconstant')y~x+z-1 else y~x+z
 qs<-c(.2,.5,.8);coefq<-function(v) as.vector(coef(quantreg::rq(f,v,tau=qs,method='br')))
 original<-coefq(z); ix<-draws[draws$case==case,]
 estimates<-t(vapply(ix$indices,function(v)coefq(z[as.integer(strsplit(v,':',fixed=TRUE)[[1]])+1,]),numeric(length(original))))
 V<-cov(estimates);means<-colMeans(estimates)
 for(j in seq_along(original)) {
  values<-c(estimate=original[j],bootstrap_mean=means[j],bias=means[j]-original[j],standard_error=sqrt(V[j,j]),ci_lower=quantile(estimates[,j],.025,type=7,names=FALSE),ci_upper=quantile(estimates[,j],.975,type=7,names=FALSE))
  for(n in names(values))emit(case,paste0('/coefficients/',j-1,'/',n),values[[n]])
  for(k in seq_along(original))emit(case,paste0('/covariance_matrix/',j-1,'/',k-1),V[j,k])
 }
}
write.csv(do.call(rbind,rows),file.path(a[1],'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'quantreg',as.character(packageVersion('quantreg')),';',length(rows),'quantities\n')
# A near-zero residual is not an exact active constraint. The first native LP
# precision misclassified this row; tighter numerical refinement preserves the
# unchanged quantreg optimum and the same strict polishing certificates.
z<-d;b<-coef(quantreg::rq(y~x+z,z,tau=.35,method='br'))
z$y[111]<-sum(c(1,z$x[111],z$z[111])*b)+1e-8
f<-quantreg::rq(y~x+z,z,tau=.35,method='br');s<-summary(f,se='iid',covariance=TRUE,hs=TRUE)
b<-unname(coef(f));V<-s$cov
for(j in 1:3){
 se<-sqrt(V[j,j]);stat<-b[j]/se
 values<-c(value=b[j],standard_error=se,statistic=stat,p_value=2*pnorm(-abs(stat)),ci_lower=b[j]-qnorm(.975)*se,ci_upper=b[j]+qnorm(.975)*se)
 for(n in names(values))emit('near_active',paste0('/coefficients/',j-1,'/',n),values[[n]])
 for(k in 1:3)emit('near_active',paste0('/covariance_matrix/',j-1,'/',k-1),V[j,k])
}
write.csv(do.call(rbind,rows),file.path(a[1],'reference.csv'),row.names=FALSE,quote=FALSE)
cat('Including near-active regression:',length(rows),'quantities\n')
