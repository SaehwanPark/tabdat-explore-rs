# Investigate, not certify, the historical Stata float-storage discrepancy.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1) stop('usage: Rscript stacked_float_probe.R MOMprecise.dat')
f32 <- function(x) readBin(writeBin(as.double(x),raw(),size=4), 'numeric',n=length(x),size=4)
a <- f32(scan(args[1],quiet=TRUE)); d <- as.data.frame(matrix(a,ncol=8,byrow=TRUE)); names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
i<-which(d$year>=1981); y<-f32(d$lnwg[i]-d$lnwg[i-1]); cols<-c('ageh','agesq','kids','disab','ageh','agesq','kids','disab','lnwg'); lag<-c(1,1,1,1,2,2,2,2,2)
z<-do.call(cbind,lapply(1:8,function(t) do.call(cbind,lapply(1:9,function(j) ifelse(d$year[i]==1980+t,d[[cols[j]]][i-lag[j]],0)))))
x<-cbind(1,z); q<-qr(x,tol=1e-10); beta<-qr.coef(q,y); u<-drop(y-x%*%beta); bread<-chol2inv(qr.R(q)); bread<-bread[order(q$pivot),order(q$pivot)]; n<-nrow(x); k<-ncol(x); g<-length(unique(d$id[i])); score<-rowsum(x*u,d$id[i]); vc<-bread%*%crossprod(score)%*%bread*g/(g-1)*(n-1)/(n-k)
vs<-sum(u*u)/(n-k)*bread
for(v in list(vc,vs)){s<-sqrt(diag(v)[-1]); f<-drop(crossprod(beta[-1]/s,solve(v[-1,-1]/outer(s,s),beta[-1]/s)))/72; cat('F',format(f,digits=15),'\n')};cat('R2',format(1-sum(u*u)/sum((y-mean(y))^2),digits=15),'\n')
