# Development-only probe of distinct copies versus merging repeated source IDs.
# This is a counterfactual R convention, not a claim about historical Stata.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==4L,!file.exists(args[4]),requireNamespace('gee',quietly=TRUE),
  packageVersion('gee')==package_version('4.13.30'))
tokens<-scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src<-matrix(tokens,ncol=25L,byrow=TRUE);v<-suppressWarnings(matrix(as.numeric(src),ncol=25L))
g<-nrow(v);stopifnot(g>=2L,g<=346L,!anyNA(v[,-2L]))
x<-as.vector(t(v[,11:15,drop=FALSE]));y<-as.vector(t(v[,21:25,drop=FALSE]));start<-unname(coef(glm(y~x,family=poisson())))
draws<-read.csv(args[2]);replicate<-as.integer(args[3]);stopifnot(replicate>=1,replicate<=500)
selected<-draws[draws$replicate==replicate,,drop=FALSE]
stopifnot(nrow(selected)==g,identical(as.integer(selected$position),seq_len(g)),all(selected$index0>=0),all(selected$index0<g))
ids<-selected$index0+1L
base<-data.frame(source=rep(ids,each=5L),copy=rep(seq_len(g),each=5L),
  x=as.vector(t(v[ids,11:15,drop=FALSE])),y=as.vector(t(v[ids,21:25,drop=FALSE])))
results<-matrix(NA_real_,2,5)
for(j in 1:2) {
  data<-if(j==1) base else base[order(base$source,base$copy),]
  data$id<-if(j==1) data$copy else data$source
  fit<-gee::gee(y~x,id=id,data=data,family=poisson(),corstr='exchangeable',
    scale.fix=TRUE,scale.value=1,v4.4compat=TRUE,b=start,tol=1e-12,maxiter=100,silent=TRUE)
  stopifnot(fit$error==0,fit$iterations<100L)
  results[j,]<-c(coef(fit),fit$working.correlation[1,2],fit$iterations,max(table(data$id)))
}
write.csv(data.frame(convention=c('distinct_copies','merged_source_ids'),
  intercept=results[,1],slope=results[,2],rho=results[,3],iterations=results[,4],
  maximum_cluster_rows=results[,5]),args[4],row.names=FALSE,quote=FALSE)
