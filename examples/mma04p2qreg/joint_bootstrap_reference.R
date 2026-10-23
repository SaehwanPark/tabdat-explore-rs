# R4.6.1, quantreg6.1; original ASCII sample and external common index oracle.
# Usage: Rscript joint_bootstrap_reference.R raw.asc draws.csv output-directory
options(digits=17)
a<-commandArgs(TRUE);dir.create(a[3],recursive=TRUE,showWarnings=FALSE)
d<-as.data.frame(scan(a[1],what=list(0,0,0,'','',0,0,0,0),na.strings='.',quote='"',quiet=TRUE),stringsAsFactors=FALSE)
names(d)<-c('sex','age','educyr98','farm','urban98','hhsize','lhhexp1','lhhex12m','lnrlfood')
stopifnot(nrow(d)==5999);d$source_row<-seq_len(nrow(d))
d<-d[is.finite(d$lhhexp1)&is.finite(d$lhhex12m),];stopifnot(nrow(d)==5006)
d$lntotal<-d$lhhexp1;d$lnmed<-d$lhhex12m
qs<-(1:19)/20;fit<-function(z)as.vector(coef(quantreg::rq(lnmed~lntotal,z,tau=qs,method='br')))
original<-fit(d);draws<-read.csv(a[2]);stopifnot(nrow(draws)==200,identical(draws$replication,1:200))
estimates<-t(vapply(draws$indices,function(v){
 ix<-as.integer(strsplit(v,':',fixed=TRUE)[[1]])+1
 stopifnot(length(ix)==5006,all(ix>=1&ix<=5006));fit(d[ix,])
},numeric(38)))
V<-cov(estimates);means<-colMeans(estimates);rows<-list()
emit<-function(path,value)rows[[length(rows)+1]]<<-data.frame(path,value=unname(value))
for(j in 1:38){
 v<-c(estimate=original[j],bootstrap_mean=means[j],bias=means[j]-original[j],standard_error=sqrt(V[j,j]),ci_lower=quantile(estimates[,j],.025,type=7,names=FALSE),ci_upper=quantile(estimates[,j],.975,type=7,names=FALSE))
 for(n in names(v))emit(paste0('/coefficients/',j-1,'/',n),v[[n]])
 for(k in 1:38)emit(paste0('/covariance_matrix/',j-1,'/',k-1),V[j,k])
}
write.csv(do.call(rbind,rows),file.path(a[3],'reference.csv'),row.names=FALSE,quote=FALSE)
out<-d[,c('source_row','lntotal','lnmed')]
out$median<-original[19]+original[20]*d$lntotal;out$median_residual<-d$lnmed-out$median
write.csv(out,file.path(a[3],'cohort.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'quantreg',as.character(packageVersion('quantreg')),';',length(rows),'quantities\n')
