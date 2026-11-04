# Development-only independent R weighted-QR reference for Figure 23.1.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2L,!file.exists(a[2]))
src<-matrix(scan(a[1],what=character(),quiet=TRUE),ncol=25,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v<-suppressWarnings(matrix(as.numeric(src),ncol=25));stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
g<-nrow(v);stopifnot(g>=2L,g<=346L);n<-g*5L
d<-data.frame(id=rep(seq_len(g),each=5),year=rep(1975:1979,g),cusip=rep(v[,1],each=5),ardssic=rep(v[,2],each=5),scisect=rep(v[,3],each=5),logk=rep(v[,4],each=5),sumpat=rep(v[,5],each=5),logr=as.vector(t(v[,11:15,drop=FALSE])),pat=as.vector(t(v[,21:25,drop=FALSE])))
stopifnot(all(abs(d$logr)<=20),all(d$pat>=0),all(d$pat<=1000),all(d$pat==floor(d$pat)))
d$lpat<-log(ifelse(d$pat==0,.5,d$pat))
fit<-lm(lpat~logr,data=d);x<-model.matrix(fit);b<-coef(fit);e<-residuals(fit);bread<-solve(crossprod(x));scores<-rowsum(x*e,d$id)
vc<-g/(g-1)*(n-1)/(n-2)*bread%*%crossprod(scores)%*%bread
d$figure_linear<-as.numeric(x%*%b)
# Stable X rank, then source id/year order. R nearest-neighbour lowess is not this algorithm.
o<-order(d$logr,seq_len(n));xx<-d$logr[o];yy<-d$lpat[o];k<-floor((n*.8-.5)/2)
stopifnot(n*min(n,2*k+1)<=20000000)
fitted<-numeric(n);constant<-0L;score<-0
for(i in seq_len(n)){
  at<-seq.int(max(1L,i-k),min(n,i+k));radius<-max(abs(xx[at]-xx[i]));scale<-max(abs(yy[at]));if(scale==0)scale<-1
  z<-if(radius==0)rep(0,length(at))else(xx[at]-xx[i])/radius
  w<-(1-(abs(z)/1.0001)^3)^3;design<-if(radius==0)matrix(1,length(at),1)else cbind(1,z)
  q<-lm.wfit(design,yy[at]/scale,w,tol=1e-12)
  stopifnot(q$rank==ncol(design),all(is.finite(q$coefficients)))
  fitted[i]<-q$coefficients[1]*scale;score<-max(score,abs(crossprod(design,w*q$residuals))/length(at));constant<-constant+as.integer(radius==0)
}
stopifnot(score<1e-12);d$figure_lowess<-numeric(n);d$figure_lowess[o]<-fitted
meta<-data.frame(observations=n,firms=g,span=.8,half_window=k,constant_windows=constant,max_scaled_score=score,intercept=b[1],slope=b[2],cov00=vc[1,1],cov01=vc[1,2],cov11=vc[2,2],slope_se=sqrt(vc[2,2]),slope_t=b[2]/sqrt(vc[2,2]),slope_p=2*pt(-abs(b[2]/sqrt(vc[2,2])),g-1),inference_df=g-1,check.names=FALSE)
dir.create(a[2]);write.csv(d,file.path(a[2],'rows.csv'),row.names=FALSE,quote=FALSE,na='');write.csv(meta,file.path(a[2],'meta.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,';',n,'source/QR/linear rows;',k,'half-window;',constant,'constant windows; max QR score',score,'\n')
