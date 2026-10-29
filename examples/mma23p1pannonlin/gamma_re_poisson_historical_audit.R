# Development-only precision/stopping probe, not a reproduction of Stata internals.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
tokens<-scan(a[1],what=character(),quiet=TRUE);stopifnot(length(tokens)>0,length(tokens)%%25==0)
src<-suppressWarnings(matrix(as.numeric(matrix(tokens,ncol=25,byrow=TRUE)),ncol=25))
Y<-src[,21:25,drop=FALSE];S<-rowSums(Y);stopifnot(all(is.finite(Y)),all(Y>=0))
float32<-function(v)readBin(writeBin(as.vector(v),raw(),size=4),what='double',size=4,n=length(v))
rows<-list()
for(kind in c('source_float64','source_float32')) {
  X<-src[,11:15,drop=FALSE]
  if(kind=='source_float32') X<-matrix(float32(X),nrow=nrow(X))
  x<-as.vector(t(X));y<-as.vector(t(Y));p<-c(unname(coef(glm(y~x,family=poisson()))),0)
  evaluate<-function(p){
    r<-exp(-p[3]);eta<-p[1]+p[2]*X;mu<-exp(eta);M<-rowSums(mu);h<-rowSums(mu*X);q<-(r+S)/(r+M)
    dg<-digamma(r+S)-digamma(r)+log(r)+1-log(r+M)-(r+S)/(r+M)
    score<-c(sum(S-q*M),sum(rowSums(Y*X)-q*h),sum(-r*dg))
    info<-matrix(0,3,3);info[1,1]<-sum(q*M-q*M*M/(r+M));info[1,2]<-info[2,1]<-sum(q*h-q*M*h/(r+M))
    info[2,2]<-sum(q*rowSums(mu*X*X)-q*h*h/(r+M));info[1,3]<-info[3,1]<- -sum(r*(M-S)*M/(r+M)^2)
    info[2,3]<-info[3,2]<- -sum(r*(M-S)*h/(r+M)^2)
    info[3,3]<- -sum(r*dg+r*r*(trigamma(r+S)-trigamma(r)+1/r-1/(r+M)+(S-M)/(r+M)^2))
    ll<-sum(lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)+rowSums(Y*eta-lgamma(Y+1)))
    list(score=score,info=info,ll=ll)
  }
  for(step in 0:6){
    e<-evaluate(p);stopifnot(all(is.finite(p)),min(eigen(e$info,symmetric=TRUE)$values)>0)
    rows[[length(rows)+1]]<-data.frame(input=kind,step=step,intercept=p[1],slope=p[2],lnalpha=p[3],log_likelihood=e$ll,maximum_score=max(abs(e$score)))
    p<-p+solve(e$info,e$score)
  }
}
write.csv(do.call(rbind,rows),a[2],row.names=FALSE,quote=FALSE)
