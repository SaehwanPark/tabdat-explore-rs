# Development-only R4.6.1/plm oracle and independent unbalanced moment equations.
# Rscript this.R reference.csv inputs.csv [original MOM.dat for sensitivity]
options(digits=17);library(plm)
a<-commandArgs(TRUE);stopifnot(length(a)%in%c(2,3));rows<-list();inputs<-list()
emit<-function(case,path,value)rows[[length(rows)+1L]]<<-data.frame(case,path,value=unname(value))
for(case in if(length(a)==3)'mom_sensitivity' else c('unequal','shifted_interleaved','zero_component')) {
 sizes<-c(1,2,3,4,5,3,2,4)
 d<-do.call(rbind,lapply(1:8,function(id)data.frame(id,time=seq_len(sizes[id])-1)))
 d$x<-.3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
 d$z<-.13*d$time^2+.03*((d$id+2*d$time)%%4)
 d$y<-1+.7*d$x-.3*d$z+.3*((d$id^2+1)%%5)+.11*((d$id*d$time+3)%%7)
 f<-y~x+z
 if(case=='shifted_interleaved'){d$y<-d$y+100;d<-d[order(-d$time,-d$id),]}
 if(case=='mom_sensitivity'){
  d<-read.table(a[3],col.names=c('y','x','kids','ageh','agesq','disab','id','time'))
  stopifnot(nrow(d)==5320);d$source_row<-seq_len(nrow(d))
  d<-d[!((d$id%%3==0 & d$time%in%c(1980,1985)) | (d$id%%7==0 & d$time>=1981 & d$time<=1983)),]
  stopifnot(nrow(d)==4738,length(unique(d$id))==532);f<-y~x
 }
 X<-model.matrix(f,d);N<-nrow(X);K<-ncol(X);G<-length(unique(d$id))
 Xm<-apply(X,2,function(v)ave(v,d$id));Xw<-X-Xm;Ti<-ave(d$id,d$id,FUN=length)
 if(case=='zero_component'){
  e<-sin(1.3*seq_len(N));e<-e-ave(e,d$id)
  e<-lm.fit(Xw[,-1,drop=FALSE],e)$residuals
  d$y<-drop(X%*%c(1,.7,-.3))+e
 }
 ym<-ave(d$y,d$id);yw<-d$y-ym;W<-lm.fit(Xw[,-1,drop=FALSE],yw)
 ve<-sum(W$residuals^2)/(N-G-K+1)
 first<-!duplicated(d$id);Z<-Xm[first,,drop=FALSE];Tg<-Ti[first]
 B<-lm.wfit(Z,ym[first],w=Tg)
 # Weighted between projection: E[u'B_Mu]=(G-K)*ve+trace-corrected*vu.
 invA<-solve(crossprod(Z*sqrt(Tg)));C<-crossprod(Z*Tg)
 denominator<-N-sum(invA*t(C))
 stopifnot(denominator>0)
 rawvu<-(sum(Tg*B$residuals^2)-(G-K)*ve)/denominator
 vu<-max(rawvu,0);theta<-1-sqrt(ve/(ve+Ti*vu))
 fit<-plm(f,d,index=c('id','time'),model='random',random.method='swar')
 stopifnot(max(abs(c(ve,vu)-fit$ercomp$sigma2))<1e-9)
 Q<-X-Xm*theta;yq<-d$y-theta*ym;direct<-lm.fit(Q,yq)
 V<-sum(direct$residuals^2)/(N-K)*solve(crossprod(Q));b<-unname(coef(fit))
 stopifnot(max(abs(direct$coefficients-b))<1e-9,max(abs(V-vcov(fit)))<1e-9)
 for(i in seq_len(K)){
  se<-sqrt(V[i,i]);stat<-b[i]/se
  vals<-c(value=b[i],standard_error=se,p_value=2*pnorm(-abs(stat)))
  for(nm in names(vals))emit(case,paste0('/gls/coefficients/',i-1,'/',nm),vals[[nm]])
  emit(case,paste0('/confidence_intervals/',i-1,'/0'),b[i]-qnorm(.975)*se)
  emit(case,paste0('/confidence_intervals/',i-1,'/1'),b[i]+qnorm(.975)*se)
  for(j in seq_len(K))emit(case,paste0('/gls/covariance/matrix/',i-1,'/',j-1),V[i,j])
 }
 Vcl<-unname(vcovHC(fit,method='arellano',type='sss',cluster='group'))
 for(i in seq_len(K))for(j in seq_len(K))
  emit(case,paste0('/cluster_covariance/',i-1,'/',j-1),Vcl[i,j])
 for(i in seq_len(N)){
  emit(case,paste0('/gls/fitted_values/',i-1),sum(Q[i,]*b))
  emit(case,paste0('/gls/residuals/',i-1),direct$residuals[i])
 }
 wald<-drop(t(b[-1])%*%solve(V[-1,-1,drop=FALSE],b[-1]))
 vals<-c(group_count=G,group_size=0,sigma_e_squared=ve,sigma_u_squared=vu,raw_sigma_u_squared=rawvu,theta=min(theta),rho=vu/(ve+vu),wald_statistic=wald,wald_p_value=pchisq(wald,K-1,lower.tail=FALSE))
 for(nm in names(vals))emit(case,paste0('/',nm),vals[[nm]])
 inputs[[length(inputs)+1]]<-data.frame(case,id=d$id,time=d$time,y=d$y,x=d$x,z=if('z'%in%names(d))d$z else 0)
}
write.csv(do.call(rbind,rows),a[1],row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,inputs),a[2],row.names=FALSE,quote=FALSE)
cat(R.version.string,';plm',as.character(packageVersion('plm')),';',length(rows),'scalar references;native implementation pending\n')
