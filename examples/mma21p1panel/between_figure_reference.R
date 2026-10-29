# Development-only independent source, person means and weighted-QR LOWESS reference.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2L,!file.exists(a[2]))
v<-scan(a[1],what=double(),na.strings=c('.','NA'),quiet=TRUE);stopifnot(length(v)%%8L==0L)
src<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(src)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year');src<-data.frame(source_row=seq_len(nrow(src)),src)
stopifnot(nrow(src)>0,all(is.finite(src$id)),all(is.finite(src$year)),all(src$id==floor(src$id)),all(src$id>=1&src$id<=532),all(src$year==floor(src$year)),all(src$year>=1979&src$year<=1988),!anyDuplicated(src[c('id','year')]))
src<-src[order(src$id,src$year),];eligible<-src[is.finite(src$lnhr)&is.finite(src$lnwg),]
p<-split(eligible,eligible$id);stopifnot(length(p)>2L)
d<-do.call(rbind,lapply(p,function(z)data.frame(id=z$id[1],model_rows=nrow(z),first_year=min(z$year),last_year=max(z$year),mean_lnhr=mean(z$lnhr),mean_lnwg=mean(z$lnwg))))
d<-d[order(d$id),];n<-nrow(d)
fit<-lm(mean_lnhr~mean_lnwg,data=d);X<-model.matrix(fit);b<-coef(fit);stopifnot(fit$rank==2,df.residual(fit)>0)
V<-vcov(fit);d$figure_linear<-as.numeric(X%*%b)
o<-order(d$mean_lnwg,seq_len(n));xx<-d$mean_lnwg[o];yy<-d$mean_lnhr[o];k<-floor((n*.8-.5)/2)
fitted<-numeric(n);constant<-0L;score<-0
for(i in seq_len(n)){
 at<-seq.int(max(1L,i-k),min(n,i+k));radius<-max(abs(xx[at]-xx[i]));scale<-max(abs(yy[at]));if(scale==0)scale<-1
 z<-if(radius==0)rep(0,length(at))else(xx[at]-xx[i])/radius;w<-(1-(abs(z)/1.0001)^3)^3
 design<-if(radius==0)matrix(1,length(at),1)else cbind(1,z);q<-lm.wfit(design,yy[at]/scale,w,tol=1e-12)
 stopifnot(q$rank==ncol(design),all(is.finite(q$coefficients)))
 fitted[i]<-q$coefficients[1]*scale;score<-max(score,abs(crossprod(design,w*q$residuals))/length(at));constant<-constant+as.integer(radius==0)
}
stopifnot(score<1e-12);d$figure_lowess<-numeric(n);d$figure_lowess[o]<-fitted
meta<-data.frame(source_rows=nrow(src),observations=n,span=.8,half_window=k,constant_windows=constant,max_scaled_score=score,intercept=b[1],slope=b[2],cov00=V[1,1],cov01=V[1,2],cov11=V[2,2],slope_se=sqrt(V[2,2]),slope_t=b[2]/sqrt(V[2,2]),slope_p=2*pt(-abs(b[2]/sqrt(V[2,2])),n-2),inference_df=n-2,check.names=FALSE)
dir.create(a[2]);write.csv(src,file.path(a[2],'source.csv'),row.names=FALSE,quote=FALSE,na='');write.csv(d,file.path(a[2],'rows.csv'),row.names=FALSE,quote=FALSE,na='');write.csv(meta,file.path(a[2],'meta.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,';',nrow(src),'source rows;',n,'people;',k,'half-window;',constant,'constant windows; max QR score',score,'\n')
