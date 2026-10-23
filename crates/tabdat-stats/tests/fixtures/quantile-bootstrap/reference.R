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
