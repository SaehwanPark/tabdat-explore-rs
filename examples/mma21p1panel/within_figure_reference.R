# Development-only independent source-scale centering, plm FE/OLS and QR audit.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)%in%c(2L,3L),!file.exists(a[2]))
if(length(a)==3L){stopifnot(a[3]=='csv');src<-read.csv(a[1],check.names=FALSE)}else{
 v<-scan(a[1],what=double(),na.strings=c('.','NA'),quiet=TRUE);stopifnot(length(v)%%8L==0L)
 src<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(src)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year');src<-data.frame(source_row=seq_len(nrow(src)),src)
}
stopifnot(identical(names(src),c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year')),nrow(src)>0,all(is.finite(src$id)),all(is.finite(src$year)),all(src$id==floor(src$id)),all(src$id>=1&src$id<=532),all(src$year==floor(src$year)),all(src$year>=1979&src$year<=1988),!anyDuplicated(src[c('id','year')]))
src<-src[order(src$id,src$year),];d<-src[is.finite(src$lnhr)&is.finite(src$lnwg),];n<-nrow(d);g<-length(unique(d$id));stopifnot(n>=3L,n<=10000L,g>=2L,all(abs(d$lnhr)<=1e6),all(abs(d$lnwg)<=1e6),all(round(d$lnhr,2)==d$lnhr),all(round(d$lnwg,2)==d$lnwg))
hundredth_mean<-function(z)(sum(round(100*z))/100)/length(z)
d$unit_lnhr<-ave(d$lnhr,d$id,FUN=hundredth_mean);d$unit_lnwg<-ave(d$lnwg,d$id,FUN=hundredth_mean)
d$overall_lnhr<-hundredth_mean(d$lnhr);d$overall_lnwg<-hundredth_mean(d$lnwg)
d$within_lnhr<-(d$lnhr-d$unit_lnhr)+d$overall_lnhr;d$within_lnwg<-(d$lnwg-d$unit_lnwg)+d$overall_lnwg
fe<-plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='within');fb<-unname(coef(fe));fv<-unname(plm::vcovHC(fe,method='arellano',type='sss',cluster='group')[1,1]);stopifnot(length(fb)==1L,df.residual(fe)==n-g-1)
fit<-lm(within_lnhr~within_lnwg,data=d);X<-model.matrix(fit);b<-coef(fit);stopifnot(fit$rank==2,df.residual(fit)>0,abs(unname(b[2])-fb)<1e-10)
e<-residuals(fit);bread<-solve(crossprod(X));scores<-rowsum(X*e,d$id);V<-g/(g-1)*(n-1)/(n-2)*bread%*%crossprod(scores)%*%bread
d$figure_linear<-as.numeric(X%*%b)
o<-order(d$within_lnwg,seq_len(n));xx<-d$within_lnwg[o];yy<-d$within_lnhr[o];k<-floor((n*.8-.5)/2);stopifnot(n*min(n,2*k+1)<=23000000)
fitted<-numeric(n);constant<-0L;score<-0
for(i in seq_len(n)){
 at<-seq.int(max(1L,i-k),min(n,i+k));radius<-max(abs(xx[at]-xx[i]));scale<-max(abs(yy[at]));if(scale==0)scale<-1
 z<-if(radius==0)rep(0,length(at))else(xx[at]-xx[i])/radius;w<-(1-(abs(z)/1.0001)^3)^3
 design<-if(radius==0)matrix(1,length(at),1)else cbind(1,z);q<-lm.wfit(design,yy[at]/scale,w,tol=1e-12)
 stopifnot(q$rank==ncol(design),all(is.finite(q$coefficients)))
 fitted[i]<-q$coefficients[1]*scale;score<-max(score,abs(crossprod(design,w*q$residuals))/length(at));constant<-constant+as.integer(radius==0)
}
stopifnot(score<1e-12);d$figure_lowess<-numeric(n);d$figure_lowess[o]<-fitted
meta<-data.frame(source_rows=nrow(src),observations=n,people=g,span=.8,half_window=k,constant_windows=constant,max_scaled_score=score,fe_slope=fb,fe_cov=fv,fe_se=sqrt(fv),fe_t=fb/sqrt(fv),fe_p=2*pt(-abs(fb/sqrt(fv)),g-1),intercept=b[1],slope=b[2],cov00=V[1,1],cov01=V[1,2],cov11=V[2,2],slope_se=sqrt(V[2,2]),slope_t=b[2]/sqrt(V[2,2]),slope_p=2*pt(-abs(b[2]/sqrt(V[2,2])),g-1),inference_df=g-1,check.names=FALSE)
dir.create(a[2]);write_precise<-function(frame,name){text<-as.data.frame(lapply(frame,function(z)ifelse(is.na(z),'',sprintf('%.17g',z))),check.names=FALSE);write.table(text,file.path(a[2],name),sep=',',row.names=FALSE,quote=FALSE,na='')}
write_precise(src,'source.csv');write_precise(d,'rows.csv');write_precise(meta,'meta.csv')
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',nrow(src),'source rows;',n,'model rows;',g,'people;',k,'half-window;',constant,'constant windows; max QR score',score,'\n')
