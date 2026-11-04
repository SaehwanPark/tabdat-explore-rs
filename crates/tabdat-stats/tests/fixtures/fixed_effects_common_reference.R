# Independent complete dummy-design QR with observation-weighted zero-sum effects.
args <- commandArgs(trailingOnly=TRUE);stopifnot(length(args)==1,!file.exists(args[1]))
result <- list()
emit <- function(case,path,value) { result[[length(result)+1L]] <<- data.frame(case=case,path=path,value=sprintf('%.17g',value)) }
for(case in c('balanced','unbalanced','interleaved')) {
  d <- do.call(rbind,lapply(1:6,function(id) {
    t <- 0:((if(case=='unbalanced') id else 4)-1)
    x <- .3*id+.2*t+.07*((id*t+2)%%5);z <- .13*t*t+.03*((id+2*t)%%4)
    data.frame(id=id,t=t,x=x,z=z,y=1+.4*id+.7*x-.3*z+.11*((id*t+3)%%7))
  }))
  if(case=='interleaved') d<-d[order(-d$t,-d$id),]
  ids<-sort(unique(d$id));n<-as.numeric(table(factor(d$id,levels=ids)));g<-length(ids)
  D<-sapply(ids[-g],function(id) as.numeric(d$id==id)-n[match(id,ids)]/n[g]*as.numeric(d$id==ids[g]))
  X<-cbind(1,d$x,d$z,D);fit<-lm.fit(X,d$y);stopifnot(fit$rank==ncol(X))
  df<-nrow(X)-fit$rank;sigma2<-sum(fit$residuals^2)/df
  V<-matrix(0,ncol(X),ncol(X));V[fit$qr$pivot,fit$qr$pivot]<-sigma2*chol2inv(qr.R(fit$qr))
  b<-unname(fit$coefficients[1:3]);V<-V[1:3,1:3]
  for(i in 1:3) {
    emit(case,paste0('/normalized_level/parameters/',i-1),b[i])
    for(j in 1:3) emit(case,paste0('/normalized_level/covariance/matrix/',i-1,'/',j-1),V[i,j])
  }
  for(kind in c('common','contrast')) {
    w<-if(kind=='common') c(1,0,0) else c(1,2,-.5);offset<-if(kind=='common') 0 else 1.3
    estimate<-sum(w*b)+offset;se<-sqrt(drop(t(w)%*%V%*%w));statistic<-estimate/se
    values<-c(estimate=estimate,standard_error=se,statistic=statistic,p_value=2*pt(-abs(statistic),df),
      ci_lower=estimate-qt(.975,df)*se,ci_upper=estimate+qt(.975,df)*se,confidence_level=95,degrees_of_freedom=df)
    for(k in names(values)) emit(case,paste0('/',kind,'/',k),values[k])
  }
  R<-rbind(c(1,0,0),c(0,1,-.5));delta<-drop(R%*%b)-c(0,.2)
  F<-drop(t(delta)%*%solve(R%*%V%*%t(R),delta))/2
  values<-c(f_statistic=F,df_num=2,df_denom=df,p_value=pf(F,2,df,lower.tail=FALSE))
  for(k in names(values)) emit(case,paste0('/joint/',k),values[k])
}
write.table(do.call(rbind,result),args[1],sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'; independent weighted-zero-sum full dummy QR;',length(result),'reference cells\n')
