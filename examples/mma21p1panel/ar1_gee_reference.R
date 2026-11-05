#!/usr/bin/env Rscript
# Independent dense AR1 covariance/score iteration, cross-checked against R gee.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript ar1_gee_reference.R MOM.dat fresh_reference_directory')
stopifnot(requireNamespace('gee',quietly=TRUE),packageVersion('gee')==package_version('4.13.30'),!file.exists(args[2]))
d <- read.table(args[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(nrow(d)>0,nrow(d)<=5320,all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),
 all(d$id==floor(d$id) & d$id>=1 & d$id<=532),all(d$year==floor(d$year) & d$year>=1979 & d$year<=1988),
 all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row <- seq_len(nrow(d));ids<-sort(unique(d$id));G<-length(ids);N<-nrow(d)
stopifnot(G>=3,G<=532,all(table(d$id)==10))
sorted<-d[order(d$id,d$year),];X<-cbind(1,sorted$lnwg);y<-sorted$lnhr
blocks<-lapply(ids,function(id) which(sorted$id==id))
start<-unname(coef(lm(lnhr~lnwg,sorted)))
evaluate<-function(beta) {
 residual<-drop(y-X%*%beta);E<-matrix(residual,ncol=10,byrow=TRUE)
 rss<-sum(E^2);phi<-rss/N;rho<-sum(E[,-1]*E[,-10])/(9*G*phi)
 stopifnot(all(is.finite(c(beta,phi,rho))),rss>1e-12,1-rho^2>1e-10)
 V<-phi*rho^abs(outer(1:10,1:10,'-'));IV<-solve(V)
 H<-matrix(0,2,2);scores<-matrix(NA_real_,G,2)
 for(i in seq_len(G)) {ix<-blocks[[i]];Z<-X[ix,,drop=FALSE];H<-H+crossprod(Z,IV%*%Z);scores[i,]<-drop(crossprod(Z,IV%*%residual[ix]))}
 W<-solve(H);influence<-scores%*%W;step<-drop(W%*%colSums(scores))
 list(phi=phi,rho=rho,rss=rss,W=W,C=G/(G-1)*crossprod(influence),scores=scores,influence=influence,step=step)
}
beta<-start;converged<-FALSE
for(i in 1:100) {
 value<-evaluate(beta);next_beta<-beta+value$step
 stopifnot(all(is.finite(next_beta)),all(abs(next_beta)<=100))
 if(max(abs(next_beta-beta)/(1+abs(beta)))<=1e-12) {beta<-next_beta;converged<-TRUE;break}
 beta<-next_beta
}
stopifnot(converged,i<100)
final<-evaluate(beta);stopifnot(max(abs(final$step)/(1+abs(beta)))<=1e-10,
 all(diag(final$C)>128*.Machine$double.eps*diag(final$W)))
# Separate package fit: failed package iterates are never treated as evidence.
capture.output(model<-gee::gee(lnhr~lnwg,id=id,data=sorted,family=gaussian(),corstr='AR-M',Mv=1,
 b=start,scale.fix=FALSE,v4.4compat=TRUE,tol=1e-12,maxiter=100,silent=TRUE))
stopifnot(model$error==0,model$iterations<100,max(abs(unname(coef(model))-beta))<1e-9,
 abs(model$scale-final$phi)<1e-9,abs(model$working.correlation[1,2]-final$rho)<1e-9,
 max(abs(model$naive.variance-final$W))<1e-9,max(abs(G/(G-1)*model$robust.variance-final$C))<1e-9)
fit<-data.frame(observations=N,people=G,years_per_person=10,intercept=beta[1],slope=beta[2],rss=final$rss,
 scale=final$phi,working_correlation=final$rho,working_covariance_aa=final$W[1,1],working_covariance_ab=final$W[1,2],
 working_covariance_bb=final$W[2,2],unit_multiplier=G/(G-1),unit_covariance_aa=final$C[1,1],
 unit_covariance_ab=final$C[1,2],unit_covariance_bb=final$C[2,2],intercept_step=final$step[1],slope_step=final$step[2])
coefs<-do.call(rbind,lapply(c('unit_sandwich_g_adjusted','working_model'),function(method) {
 V<-if(method=='working_model') final$W else final$C;se<-sqrt(diag(V));z<-beta/se
 data.frame(covariance=method,parameter=c('_cons','lnwg'),estimate=beta,standard_error=se,z=z,p_value=2*pnorm(-abs(z)),
 normal95_lower=beta-qnorm(.975)*se,normal95_upper=beta+qnorm(.975)*se,slope_wald_chi2_1=c(NA,z[2]^2))
}))
coefs<-coefs[order(coefs$covariance,coefs$parameter),]
units<-data.frame(id=ids,observations=10,score_intercept=final$scores[,1],score_slope=final$scores[,2],
 influence_intercept=final$influence[,1],influence_slope=final$influence[,2])
d$pols_mean<-start[1]+start[2]*d$lnwg;d$pols_residual<-d$lnhr-d$pols_mean
d$gee_mean<-beta[1]+beta[2]*d$lnwg;d$gee_residual<-d$lnhr-d$gee_mean
d<-d[c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','pols_mean','pols_residual','gee_mean','gee_residual')]
out<-args[2];dir.create(out)
write_precise<-function(frame,name) {
 text<-as.data.frame(lapply(frame,function(x) if(is.numeric(x)) ifelse(is.na(x),'',sprintf('%.17g',x)) else x))
 write.table(text,file.path(out,name),sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE,na='')
}
write_precise(fit,'fit.csv');write_precise(coefs,'coefficients.csv');write_precise(d,'predictions.csv');write_precise(units,'unit_scores.csv')
cat(R.version.string,'gee',as.character(packageVersion('gee')),'N',N,'G',G,'dense iterations',i,'package iterations',model$iterations,'rho',final$rho,'\n')
