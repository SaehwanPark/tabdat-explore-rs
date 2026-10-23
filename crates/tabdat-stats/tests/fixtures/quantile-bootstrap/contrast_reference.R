# Independent R4.6.1/quantreg6.1 common-draw zero-contrast normal inference.
# Run: Rscript contrast_reference.R fixture-directory
options(digits=17);a<-commandArgs(TRUE);d<-read.csv(file.path(a[1],'inputs.csv'))
draws<-read.csv(file.path(a[1],'draws.csv'));rows<-list()
emit<-function(case,path,value)rows[[length(rows)+1]]<<-data.frame(case,path,value=unname(value))
for(case in c('joint','noconstant','shifted','rescaled')) {
 z<-d;if(case=='shifted')z$y<-z$y+100
 if(case=='rescaled')z$x<-z$x*100
 f<-if(case=='noconstant')y~x+z-1 else y~x+z
 fit<-function(v)coef(quantreg::rq(f,v,tau=c(.25,.75),method='br'))
 original<-fit(z); ix<-draws[draws$case==case,]
 # Compare each available parameter separately, including intercept when present.
 for(j in seq_len(nrow(original))) {
  parameter<-if(rownames(original)[j]=='(Intercept)')'intercept' else rownames(original)[j]
  estimate<-original[j,2]-original[j,1]
  diffs<-vapply(ix$indices,function(v){b<-fit(z[as.integer(strsplit(v,':',fixed=TRUE)[[1]])+1,]);b[j,2]-b[j,1]},numeric(1))
  se<-sd(diffs);stat<-estimate/se
  values<-c(estimate=estimate,standard_error=se,statistic=stat,p_value=2*pnorm(-abs(stat)),ci_lower=estimate-qnorm(.975)*se,ci_upper=estimate+qnorm(.975)*se)
  for(n in names(values))emit(paste(case,parameter,sep=':'),paste0('/contrast/',n),values[[n]])
 }
}
# Large common movements but tiny contrast movement: full covariance subtraction
# loses accuracy. The variance of explicit shared differences remains resolved.
lo<-c(-2e8,-1e8,1e8,2e8);hi<-lo+1+lo*1e-10
se<-sd(hi-lo);estimate<-1;stat<-estimate/se
for(n in c('estimate','standard_error','statistic','p_value','ci_lower','ci_upper')) {
 value<-switch(n,estimate=estimate,standard_error=se,statistic=stat,p_value=2*pnorm(-abs(stat)),ci_lower=estimate-qnorm(.975)*se,ci_upper=estimate+qnorm(.975)*se)
 emit('cancellation',paste0('/contrast/',n),value)
}
write.csv(do.call(rbind,rows),file.path(a[1],'contrast_reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'quantreg',as.character(packageVersion('quantreg')),';',length(rows),'contrast quantities\n')
