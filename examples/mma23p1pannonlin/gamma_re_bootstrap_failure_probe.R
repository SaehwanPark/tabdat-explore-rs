# Development-only full-Newton failure probe for the pre-existing 12-firm fixture.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==3L,!file.exists(a[3]))
tokens<-scan(a[1],what=character(),quiet=TRUE);stopifnot(length(tokens)>0,length(tokens)%%25==0)
v<-suppressWarnings(matrix(as.numeric(matrix(tokens,ncol=25,byrow=TRUE)),ncol=25))
g<-nrow(v);stopifnot(g==12L,!anyNA(v[,-2L]));X<-v[,11:15];Y<-v[,21:25];S<-rowSums(Y)
draws<-read.csv(a[2]);stopifnot(nrow(draws)==500L*g,all(draws$replicate==rep(1:500,each=g)),
  all(draws$position==rep(1:g,500)),all(draws$index0>=0),all(draws$index0<g))
start<-c(unname(coef(glm(as.vector(t(Y))~as.vector(t(X)),family=poisson()))),0)
evaluate<-function(p,w){
  r<-exp(-p[3]);eta<-p[1]+p[2]*X;mu<-exp(eta);M<-rowSums(mu);h<-rowSums(mu*X);q<-(r+S)/(r+M)
  dg<-digamma(r+S)-digamma(r)+log(r)+1-log(r+M)-(r+S)/(r+M)
  score<-c(sum(w*(S-q*M)),sum(w*(rowSums(Y*X)-q*h)),sum(-w*r*dg))
  info<-matrix(0,3,3);info[1,1]<-sum(w*(q*M-q*M*M/(r+M)));info[1,2]<-info[2,1]<-sum(w*(q*h-q*M*h/(r+M)))
  info[2,2]<-sum(w*(q*rowSums(mu*X*X)-q*h*h/(r+M)));info[1,3]<-info[3,1]<- -sum(w*r*(M-S)*M/(r+M)^2)
  info[2,3]<-info[3,2]<- -sum(w*r*(M-S)*h/(r+M)^2)
  info[3,3]<- -sum(w*(r*dg+r*r*(trigamma(r+S)-trigamma(r)+1/r-1/(r+M)+(S-M)/(r+M)^2)))
  ll<-sum(w*(lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)+rowSums(Y*eta-lgamma(Y+1))))
  list(score=score,info=info,ll=ll)
}
rows<-list()
for(rep in c(0L,13L)) {
  w<-if(rep==0L)rep(1,g) else tabulate(draws$index0[draws$replicate==rep]+1L,nbins=g)
  p<-start
  for(step in 0:20){
    e<-evaluate(p,w);mineigen<-min(eigen(e$info,symmetric=TRUE)$values)
    rows[[length(rows)+1]]<-data.frame(replicate=rep,step=step,intercept=p[1],slope=p[2],lnalpha=p[3],
      log_likelihood=e$ll,minimum_information_eigenvalue=mineigen,maximum_score=max(abs(e$score)))
    if(mineigen<=0) {stopifnot(rep==13L,step==1L);break}
    delta<-solve(e$info,e$score)
    if(max(abs(delta)/(1+abs(p)))<=1e-12){stopifnot(rep==0L,max(abs(e$score))<1e-8);break}
    p<-p+delta
    if(step==20L)stop('probe did not terminate')
  }
}
write.csv(do.call(rbind,rows),a[3],row.names=FALSE,quote=FALSE)
